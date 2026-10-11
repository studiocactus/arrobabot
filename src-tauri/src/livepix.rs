use crate::{engine::Runtime, secrets};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub const MODULE: &str = "livepix";
const DATA: &str = "livepix_data";

// ------- Structures -------

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GoalDef {
    pub title: String,
    pub target: i64,
    pub font_family: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub enabled: bool,
    pub client_id: String,
    pub active_goal: String,
    pub goals: HashMap<String, GoalDef>,
    pub start_timestamp: i64,
}

// ------- Helpers -------

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ------- Config / Data -------

pub fn config(rt: &Runtime, p: &str) -> Config {
    serde_json::from_value(rt.db.module(p, MODULE)).unwrap_or_default()
}

pub fn data(rt: &Runtime, p: &str) -> Value {
    let d = rt.db.module(p, DATA);
    if d.is_object() { d } else { json!({}) }
}

fn save_data(rt: &Runtime, p: &str, d: &Value) {
    let _ = rt.db.set_module(p, DATA, d);
}

// ---- Token ----

async fn fetch_token(rt: &Arc<Runtime>, profile_id: &str) -> Result<String, String> {
    let cfg = config(rt, profile_id);
    let client_id = cfg.client_id.trim();
    if client_id.is_empty() {
        return Err("Client ID da LivePix não configurado".into());
    }
    let secret = secrets::get(profile_id, "client_secret")
        .map_err(|_| "Client Secret não salvo no cofre. Use Salvar segredo.")?;
    let body = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}",
        percent_encode(client_id),
        percent_encode(&secret)
    );
    let resp = rt
        .http
        .post("https://oauth.livepix.gg/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Erro de rede ao obter token: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Token rejeitado ({status}) {text}"));
    }
    let v: Value = resp.json().await.map_err(|e| format!("Resposta inválida: {e}"))?;
    v.get("data")
        .and_then(|d| d.get("accessToken"))
        .and_then(|t| t.as_str())
        .or_else(|| v.get("access_token").and_then(|t| t.as_str()))
        .map(str::to_owned)
        .ok_or_else(|| "access_token não retornado".into())
}

// ---- Tick ----

pub async fn tick(rt: &Arc<Runtime>) {
    for profile in rt.db.profiles().unwrap_or_default() {
        let cfg = config(rt, &profile.id);
        if !cfg.enabled || cfg.active_goal.is_empty() { continue; }
        let goal = match cfg.goals.get(&cfg.active_goal) { Some(g) => g.clone(), None => continue };
        let token = match fetch_token(rt, &profile.id).await {
            Ok(t) => t,
            Err(e) => { rt.log(&profile.id, "livepix", &format!("token: {e}"), "warn"); continue; }
        };
        let url = format!("https://api.livepix.gg/v2/payments?limit=100&currency=BRL");
        let resp = rt.http.get(&url).bearer_auth(&token).timeout(Duration::from_secs(20)).send().await;
        let resp = match resp { Ok(r) => r, Err(e) => { rt.log(&profile.id, "livepix", &e.to_string(), "error"); continue; } };
        let body: Value = match resp.json().await { Ok(b) => b, Err(_) => continue };
        let arr = body.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default();
        let start_ts = cfg.start_timestamp;
        let total: i64 = arr.iter().filter_map(|p| {
            let amount = p.get("amount").and_then(|a| a.as_i64())?;
            let created = p.get("createdAt").and_then(|c| c.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|d| d.timestamp());
            if start_ts == 0 || created.map_or(false, |c| c >= start_ts) { Some(amount) } else { None }
        }).sum();
        let mut d = data(rt, &profile.id);
        let old = d["current"].as_i64().unwrap_or(0);
        let new = old.max(total);
        d["current"] = json!(new);
        d["updated_at"] = json!(now_seconds());
        d["active_goal"] = json!(&cfg.active_goal);
        if d["start_timestamp"].is_null() || d["start_timestamp"].as_i64().unwrap_or(0) == 0 {
            d["start_timestamp"] = json!(now_seconds());
        }
        save_data(rt, &profile.id, &d);
        let pct = if goal.target > 0 { ((new * 100) / goal.target).clamp(0, 100) } else { 0 };
        let _ = rt.broadcast.send(json!({
            "type": "goal",
            "payload": {
                "profileId": profile.id,
                "title": goal.title,
                "current": new,
                "target": goal.target,
                "font_family": goal.font_family,
                "percent": pct,
            }
        }));
    }
}

// ---- RPC -----

pub async fn operation(rt: &Arc<Runtime>, p: &str, op: &str, args: &Value) -> Result<Value, String> {
    rt.db.profile(p)?;
    match op {
        "livepix.save_credentials" => {
            let cid = args["clientId"].as_str().unwrap_or("").trim().to_owned();
            if !cid.is_empty() {
                let mut cfg = config(rt, p);
                cfg.client_id = cid;
                rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            }
            if let Some(sec) = args["clientSecret"].as_str().filter(|s| !s.trim().is_empty()) {
                secrets::set(p, "client_secret", sec.trim())?;
            }
            rt.log(p, "livepix", "Credenciais da LivePix atualizadas.", "info");
            Ok(Value::Null)
        }
        "livepix.enable" => {
            let goals: Vec<Value> = args["goals"].as_array().cloned().unwrap_or_default();
            if goals.is_empty() { return Err("Adicione ao menos uma meta".into()); }
            let mut cfg = config(rt, p);
            cfg.enabled = true;
            cfg.goals.clear();
            for (i, g) in goals.iter().enumerate() {
                let title = g["title"].as_str().unwrap_or("Meta").to_owned();
                let target = g["target"].as_i64().unwrap_or(0);
                let font = g["font_family"].as_str().unwrap_or("").to_owned();
                cfg.goals.insert(format!("goal_{i}"), GoalDef { title, target, font_family: font });
            }
            cfg.active_goal = cfg.goals.keys().next().cloned().unwrap_or_default();
            cfg.start_timestamp = now_seconds();
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            let d = json!({"current":0,"updated_at":now_seconds(),"start_timestamp":cfg.start_timestamp,"active_goal":cfg.active_goal});
            save_data(rt, p, &d);
            rt.log(p, "livepix", "Integração LivePix ativada.", "info");
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        "livepix.disable" => {
            let mut cfg = config(rt, p);
            cfg.enabled = false;
            cfg.active_goal = String::new();
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            save_data(rt, p, &json!({}));
            rt.log(p, "livepix", "Integração LivePix desativada.", "info");
            Ok(Value::Null)
        }
        "livepix.set_active_goal" => {
            let gid = args["goalId"].as_str().ok_or("Meta ausente")?;
            let mut cfg = config(rt, p);
            if !cfg.goals.contains_key(gid) { return Err("Meta inválida".into()); }
            cfg.active_goal = gid.to_owned();
            cfg.start_timestamp = now_seconds();
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            let mut d = data(rt, p);
            d["start_timestamp"] = json!(now_seconds());
            d["current"] = json!(0);
            save_data(rt, p, &d);
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        "livepix.remove_goal" => {
            let gid = args["goalId"].as_str().ok_or("Meta ausente")?;
            let mut cfg = config(rt, p);
            if cfg.goals.remove(gid).is_none() { return Err("Meta inválida".into()); }
            if cfg.active_goal == gid { cfg.active_goal = String::new(); }
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        _ => Err("Operação desconhecida".into()),
    }
}
