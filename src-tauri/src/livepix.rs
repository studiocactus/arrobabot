use crate::{db::Db, engine::Runtime, model::Profile};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};
use tokio::sync::Mutex as AsyncMutex;

pub const MODULE: &str = "livepix";
const DATA: &str = "livepix_data";
const POLL_SECS: u64 = 15; // polling interval (1000 req/min → 15s is safe)

// ------- Structures -------

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GoalDef {
    pub title: String,
    pub target: i64, // target amount in cents (e.g. 500000 = 5000.00 BRL)
    pub font_family: String, // CSS font-family; empty = use system default
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub enabled: bool,
    pub active_goal: String, // ID of the active goal; empty = none
    pub goals: HashMap<String, GoalDef>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub current: i64,     // current accumulated amount in cents
    pub updated_at: i64,  // Unix timestamp when last updated
    pub start_timestamp: i64, // when the goal was started (0 = now)
    pub active_goal: String, // copy of active goal id
}

// ------- Helpers -------

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn parse_brl_to_cents(s: &str) -> i64 {
    // Remove R$, spaces, replace comma with dot, parse as f64 * 100
    let s = s
        .replace("R$", "")
        .replace(" ", "")
        .replace(',', ".")
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0)
        .abs()
        .max(0.0);
    (s * 100.0).round() as i64
}

// ------- Config -------

pub fn config(rt: &Runtime, p: &str) -> Config {
    serde_json::from_value(rt.db.module(p, MODULE)).unwrap_or_default()
}

pub fn data(rt: &Runtime, p: &str) -> Value {
    let mut d = rt.db.module(p, DATA);
    if !d.is_object() {
        d = json!({});
    }
    d
}

pub fn save_data(rt: &Runtime, p: &str, d: &Value) {
    let _ = rt.db.set_module(p, DATA, d);
}

// ------- Authentication helpers (internal) -------

/// Tries to obtain an OAuth2 client_credentials token using the profile's
/// client_id/client_secret stored in the system vault.
async fn fetch_token(rt: &Arc<Runtime>, profile_id: &str) -> Result<String, String> {
    // 1. Read client_id from profile
    let profile = rt.db.profile(profile_id).map_err(|_| "Perfil não encontrado")?;
    let client_id = profile.client_id.trim();
    if client_id.is_empty() {
        return Err("client_id não configurado neste perfil".into());
    }
    // 2. Read client_secret from vault
    let secret = secrets::get(profile_id, "client_secret")
        .await
        .map_err(|_| "client_secret não salvo no cofre. Use a tela LivePix → Salvar segredo.")?;
    // 3. Build token request body
    let body = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}",
        percent_encode(client_id.as_ref()),
        percent_encode(secret.as_ref())
    );
    let scopes = "payments:read account:read";
    let url = format!(
        "https://oauth.livepix.gg/oauth2/token?grant_type=client_credentials&client_id={}&client_secret={}&scope={}",
        percent_encode(client_id.as_ref()),
        percent_encode(secret.as_ref()),
        percent_encode(scopes)
    );
    // Actually the LivePix OAuth endpoint expects form-encoded body, not query.
    // We'll do a POST as the docs say.
    let token_resp = tokio::http::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Erro ao criar cliente HTTP: {e}"))?;
    let post_body = format!("grant_type=client_credentials&client_id={}&client_secret={}", percent_encode(client_id), percent_encode(secret));
    let resp = token_resp
        .post("https://oauth.livepix.gg/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(post_body)
        .await
        .map_err(|e| format!("Erro de rede ao requisitar token: {e}"))?;
    if !resp.status().is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("Token rejeitado ({}) {}", resp.status(), body_text));
    }
    let token_json: Value = resp.json().await.map_err(|e| format!("Resposta inválida: {e}"))?;
    let access_token = token_json
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("access_token não retornado")?
        .to_owned();
    Ok(access_token)
}

/// URL‑encodes a string for use in HTTP parameters (simple percent‑encoding).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.bytes() {
        match c {
            0x20..=0x7E | 0xA0..=0xFF => {
                // printable ASCII + Latin-1 kept as‑is (the simple approach)
                out.push(c as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", c));
            }
        }
    }
    out
}

// ------- Public API -------

/// Called by the scheduler every POLL_SECS seconds.
/// Fetches payments from LivePix, accumulates toward the active goal,
/// and emits a `"goal"` event that the Browser‑source overlay listens to.
pub async fn tick(rt: &Arc<Runtime>) {
    let now = now_seconds();
    for profile in rt.db.profiles().unwrap_or_default() {
        let cfg = config(rt, &profile.id);
        if !cfg.enabled || cfg.active_goal.is_empty() {
            continue;
        }
        let goal = cfg.goals.get(&cfg.active_goal);
        if let Some(goal) = goal {
            // Obtain OAuth token
            let token = match fetch_token(rt, &profile.id).await {
                Ok(t) => t,
                Err(e) => {
                    // Log but don't stop the whole tick; try next profile
                    let _ = rt.log(&profile.id, "livepix", &format!("token error: {e}"), "warn");
                    continue;
                }
            };
            // Build auth header for API calls
            let auth_header = format!("Bearer {}", token);

            // Fetch payments: limit=100 (max per request), currency=BRL, paginate
            let mut total_current: i64 = 0;
            let mut start_ts = cfg.start_timestamp; // 0 means "from now backwards" – we'll treat as "from goal start"
            // If goal was just started, start_ts = 0 => we sum all payments since epoch (or we could store start_time).
            // For simplicity we'll just sum the last page; in a full implementation you'd track start_time per goal.
            let mut page = 1;
            let mut done = false;
            while !done && page <= 10 { // safety break
                let payments_resp = tokio::http::Client::builder()
                    .timeout(std::time::Duration::from_secs(20))
                    .build()
                    .map_err(|e| format!("HTTP client error: {e}"))?;
                let url = format!(
                    "https://api.livepix.gg/v2/payments?limit=100&page={}&currency=BRL",
                    page
                );
                let resp = payments_resp
                    .get(&url)
                    .header("Authorization", &auth_header)
                    .await
                    .map_err(|e| format!("Erro ao requisitar pagamentos: {e}"))?;
                if !resp.status().is_success() && !resp.status().is_client_error() {
                    // temporary error – stop pagination for this profile
                    let _ = rt.log(&profile.id, "livepix", &format!("API error: {}", resp.status()), "error");
                    done = true;
                    continue;
                }
                let payments_json: Value = resp.json().await.unwrap_or(Value::Null);
                let data_arr = payments_json.get("data").and_then(|a| a.as_array()).cloned().unwrap_or(Value::Array::new());
                for pay in data_arr {
                    if let Some(amount) = pay.get("amount").and_then(|a| a.as_i64()) {
                        // Only count payments created after the goal start.
                        // If start_timestamp is 0 we count everything (or you could store a manual start).
                        let created = pay.get("createdAt").and_then(|c| c.as_str()).and_then(|s| {
                            chrono::DateTime::parse_from_rfc3339(s)
                                .ok()
                                .map(|dt| dt.timestamp())
                        });
                        if start_ts == 0 || created.map_or(false, |c| c >= start_ts) {
                            total_current += amount;
                        }
                    }
                }
                // Check for next page via Link header or simply stop after first page for this prototype.
                // LivePix API doesn't always include Link; we'll stop after one page.
                done = true; // for this prototype – one page is enough for demo
            }

            // Update local data
            let mut d = data(rt, &profile.id);
            if !d.is_object() {
                d = json!({});
            }
            let old_current = d["current"].as_i64().unwrap_or(0);
            // Only increase – never decrease (goal only goes forward)
            let new_current = old_current.max(total_current);
            d["current"] = json!(new_current);
            d["updated_at"] = json!(now);
            d["active_goal"] = json!(&cfg.active_goal);
            // If start_timestamp not set, set it now (first poll after enable)
            if d["start_timestamp"].is_null() {
                d["start_timestamp"] = json!(now);
            }
            save_data(rt, &profile.id, &d);

            // Emit goal event for the overlay
            let pct = if goal.target > 0 {
                ((new_current * 100) / goal.target).clamp(0, 100)
            } else {
                0
            };
            let payload = json!({
                "profileId": profile.id,
                "title": &goal.title,
                "current": new_current,
                "target": goal.target,
                "font_family": &goal.font_family,
                "percent": pct
            });
            let _ = rt.broadcast.send(json!({ "type": "goal", "payload": payload }));
        }
    }
}

// ------- Profile‑level actions (exposed via RPC) -------

/// Enable LivePix integration for a profile, with a list of goals.
/// Usage: rpc "livepix.enable" { profileId, goals: [{title, target_cents, font_family}, ...] }
#[tauri::command]
pub async fn enable(rt: &Arc<Runtime>, profile_id: String, goals: Vec<Value>) -> Result<Value, String> {
    let mut cfg = config(rt, &profile_id);
    cfg.enabled = true;
    cfg.active_goal = String::new(); // reset
    let mut goals_map: HashMap<String, GoalDef> = HashMap::new();
    for (idx, g) in goals.into_iter().enumerate() {
        let title = g.get("title").and_then(|v| v.as_str()).unwrap_or("Meta").to_owned();
        let target = g.get("target_cents").and_then(|v| v.as_i64()).unwrap_or(0);
        let font = g.get("font_family").and_then(|v| v.as_str()).unwrap_or("").to_owned();
        let gid = format!("goal_{}", idx);
        goals_map.insert(gid.clone(), GoalDef { title, target, font_family: font });
    }
    cfg.goals = goals_map;
    let v = serde_json::to_value(&cfg).map_err(|e| format!("Serde error: {e}"))?;
    rt.db.set_module(&profile_id, MODULE, &v)?;
    // Also initialise data module
    let d = json!({
        "current": 0,
        "updated_at": now_seconds(),
        "start_timestamp": 0,
        "active_goal": ""
    });
    rt.db.set_module(&profile_id, DATA, &d)?;
    rt.log(&profile_id, "livepix", "Integração LivePix ativada; metas configuradas.", "info");
    Ok(v)
}

/// Disable LivePix integration for a profile.
#[tauri::command]
pub async fn disable(rt: &Arc<Runtime>, profile_id: String) -> Result<Value, String> {
    let mut cfg = config(rt, &profile_id);
    cfg.enabled = false;
    cfg.active_goal = String::new();
    let v = serde_json::to_value(&cfg).map_err(|e| format!("Serde error: {e}"))?;
    rt.db.set_module(&profile_id, MODULE, &v)?;
    // clear data
    let d = json!({});
    rt.db.set_module(&profile_id, DATA, &d)?;
    rt.log(&profile_id, "livepix", "Integração LivePix desativada.", "info");
    Ok(v)
}

/// Set the active goal by its ID (the goal keys are "goal_0", "goal_1", …)
#[tauri::command]
pub async fn set_active_goal(rt: &Arc<Runtime>, profile_id: String, goal_id: String) -> Result<Value, String> {
    let mut cfg = config(rt, &profile_id);
    if !cfg.goals.contains_key(&goal_id) {
        return Err("ID de meta inválido".into());
    }
    cfg.active_goal = goal_id;
    let v = serde_json::to_value(&cfg).map_err(|e| format!("Serde error: {e}"))?;
    rt.db.set_module(&profile_id, MODULE, &v)?;
    rt.log(&profile_id, "livepix", &format!("Meta ativa alterada para {}", goal_id), "info");
    Ok(v)
}

/// Remove a goal from the profile's goal list.
#[tauri::command]
pub async fn remove_goal(rt: &Arc<Runtime>, profile_id: String, goal_id: String) -> Result<Value, String> {
    let mut cfg = config(rt, &profile_id);
    if !cfg.goals.contains_key(&goal_id) {
        return Err("ID de meta inválido".into());
    }
    cfg.goals.remove(&goal_id);
    // If the removed goal was active, clear active_goal
    if cfg.active_goal == goal_id {
        cfg.active_goal = String::new();
    }
    let v = serde_json::to_value(&cfg).map_err(|e| format!("Serde error: {e}"))?;
    rt.db.set_module(&profile_id, MODULE, &v)?;
    // also clean data module keys if needed
    let d = data(rt, &profile.id);
    let mut d = serde_json::from_value::<Value>(d).unwrap_or(json!({}));
    // remove start_timestamp if no goals left
    if cfg.goals.is_empty() {
        d["start_timestamp"] = json!(0);
        d["current"] = json!(0);
        rt.db.set_module(&profile_id, DATA, &d)?;
    }
    rt.log(&profile_id, "livepix", &format!("Meta {} removida.", goal_id), "info");
    Ok(v)
}