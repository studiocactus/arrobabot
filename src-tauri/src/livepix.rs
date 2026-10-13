use crate::{engine::Runtime, model::Profile, secrets};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant, SystemTime},
};

pub const MODULE: &str = "livepix";
const DATA: &str = "livepix_data";
const DESIGN: &str = "livepix_design";

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
    /// Chave estável do overlay: fica na URL do OBS e sobrevive a reaberturas.
    pub overlay_token: String,
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
    rt.db.module_cfg(p, MODULE)
}

pub fn data(rt: &Runtime, p: &str) -> Value {
    let d = rt.db.module(p, DATA);
    if d.is_object() { d } else { json!({}) }
}

fn save_data(rt: &Runtime, p: &str, d: &Value) {
    let _ = rt.db.set_module(p, DATA, d);
}

// ---- Design do overlay (Estúdio) ----
//
// O design vem do painel e vira CSS no navegador do OBS: por isso cada campo é
// reconstruído aqui, um a um. Chaves desconhecidas caem fora, faixas são aparadas
// e cores/textos seguem permitido estrito — nada entra no overlay sem passar por aqui.

fn hex_color(v: &Value, def: &str) -> String {
    let s = v.as_str().unwrap_or("");
    let b = s.as_bytes();
    let ok = b.first() == Some(&b'#')
        && matches!(b.len(), 4 | 5 | 7 | 9)
        && b[1..].iter().all(|c| c.is_ascii_hexdigit());
    if ok { s.to_owned() } else { def.to_owned() }
}

fn num_in(v: &Value, def: f64, min: f64, max: f64) -> f64 {
    let n = v.as_f64().unwrap_or(def);
    if n.is_finite() { n.clamp(min, max) } else { def }
}

fn int_in(v: &Value, def: i64, min: i64, max: i64) -> i64 {
    num_in(v, def as f64, min as f64, max as f64).round() as i64
}

fn choice(v: &Value, def: &str, allowed: &[&str]) -> String {
    let s = v.as_str().unwrap_or("");
    if allowed.contains(&s) { s.to_owned() } else { def.to_owned() }
}

fn flag(v: &Value, def: bool) -> bool {
    v.as_bool().unwrap_or(def)
}

/// Família de fonte / rótulo livre: letras, dígitos e ' -,.&+_ (mesmo permitido do TS).
fn free_text(v: &str, def: &str, max: usize) -> String {
    let ok = !v.trim().is_empty()
        && v.chars().count() <= max
        && v.chars().all(|c| {
            c.is_alphanumeric() || matches!(c, ' ' | '\'' | ',' | '-' | '.' | '_' | '&' | '+')
        });
    if ok { v.to_owned() } else { def.to_owned() }
}

/// Id de arquivo enviado na tela Sons de Live (UUID); vazio = sem som.
fn asset_id(v: &Value) -> String {
    let s = v.as_str().unwrap_or("");
    if s.chars().count() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        s.to_owned()
    } else {
        String::new()
    }
}

/// Lê o design do painel e devolve somente os campos conhecidos, dentro das faixas.
pub fn sanitize_design(v: &Value) -> Value {
    let map = v.as_object();
    let null = Value::Null;
    let g = |k: &str| map.and_then(|m| m.get(k)).unwrap_or(&null);
    let border_width = num_in(g("borderWidth"), 1.0, 0.0, 6.0) * 2.0;
    let weight = num_in(g("fontWeight"), 700.0, 100.0, 900.0).round();
    let weight = if [300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0].contains(&weight) { weight } else { 700.0 };
    json!({
        "template": free_text(g("template").as_str().unwrap_or(""), "clean", 24),
        "layout": choice(g("layout"), "bar", &["bar", "row", "panel", "ring"]),
        "align": choice(g("align"), "center", &["left", "center", "right"]),
        "valign": choice(g("valign"), "bottom", &["top", "center", "bottom"]),
        "width": (num_in(g("width"), 80.0, 20.0, 100.0) * 10.0).round() / 10.0,
        "fillStart": hex_color(g("fillStart"), "#b7ef72"),
        "fillEnd": hex_color(g("fillEnd"), "#5bc46b"),
        "track": hex_color(g("track"), "#1b2634"),
        "text": hex_color(g("text"), "#e8edf4"),
        "muted": hex_color(g("muted"), "#9fb0c3"),
        "panel": hex_color(g("panel"), "#101721dd"),
        "border": hex_color(g("border"), "#2a3646"),
        "borderWidth": border_width.round() / 2.0,
        "radius": int_in(g("radius"), 10, 0, 48),
        "barHeight": int_in(g("barHeight"), 26, 4, 64),
        "font": free_text(g("font").as_str().unwrap_or(""), "Segoe UI", 40),
        "fontSize": int_in(g("fontSize"), 18, 10, 48),
        "fontWeight": weight,
        "uppercase": flag(g("uppercase"), false),
        "tracking": (num_in(g("tracking"), 0.0, -2.0, 10.0) * 10.0).round() / 10.0,
        "glow": int_in(g("glow"), 25, 0, 100),
        "shadow": int_in(g("shadow"), 45, 0, 100),
        "animate": flag(g("animate"), false),
        "pulse": flag(g("pulse"), true),
        "celebrate": flag(g("celebrate"), true),
        "soundDonate": asset_id(g("soundDonate")),
        "soundCelebrate": asset_id(g("soundCelebrate")),
    })
}

/// Design do perfil: o do banco (higienizado) ou o padrão igual ao Estúdio.
pub fn design(rt: &Runtime, p: &str) -> Value {
    let d = rt.db.module(p, DESIGN);
    if d.is_object() { sanitize_design(&d) } else { default_design() }
}

pub fn default_design() -> Value {
    sanitize_design(&json!({}))
}

/// Estado da meta para o overlay: `hidden` quando a integração está desligada.
pub fn goal_message(rt: &Runtime, p: &str) -> Value {
    let cfg = config(rt, p);
    let goal = if cfg.enabled { cfg.goals.get(&cfg.active_goal) } else { None };
    let Some(goal) = goal else {
        return json!({"type": "goal", "payload": {"profileId": p, "hidden": true}});
    };
    let current = data(rt, p)["current"].as_i64().unwrap_or(0);
    let percent = if goal.target > 0 { ((current * 100) / goal.target).clamp(0, 100) } else { 0 };
    json!({"type": "goal", "payload": {
        "profileId": p,
        "title": goal.title,
        "current": current,
        "target": goal.target,
        "font_family": goal.font_family,
        "percent": percent,
        "celebrate": false,
        "hidden": false,
    }})
}

fn publish_goal(rt: &Runtime, p: &str) {
    let _ = rt.broadcast.send(goal_message(rt, p));
}

/// Sons escolhidos no Estúdio da Meta: saem pela linha de áudio do app
/// (evento viewer-sound, volume cheio), nunca pela fonte do OBS. `key` é a
/// chave do design — vazia (sem escolha) ou arquivo removido não toca nada.
fn goal_sound(rt: &Runtime, profile: &Profile, key: &str) {
    if key.is_empty() { return; }
    let asset = asset_id(&design(rt, &profile.id)[key]);
    if asset.is_empty() || crate::chat_extras::asset(rt, &profile.id, &asset, "sound").is_err() { return; }
    crate::chat_extras::play_line(rt, profile, &asset, None, 1.0);
}

/// Doação nova toca o som de doação; ao cruzar o alvo a celebração substitui
/// (pagamento que completa a meta não toca os dois sons de uma vez).
fn goal_sound_key(celebrate: bool, gained: bool) -> &'static str {
    if celebrate { "soundCelebrate" } else if gained { "soundDonate" } else { "" }
}

// ---- Chave estável do overlay ----

/// Qual perfil aceita esta chave de overlay (None = não é uma chave de overlay).
pub fn overlay_profile(rt: &Runtime, token: &str) -> Option<String> {
    if token.is_empty() { return None; }
    for p in rt.db.profiles().unwrap_or_default() {
        let c = config(rt, &p.id);
        if !c.overlay_token.is_empty() && c.overlay_token == token { return Some(p.id); }
    }
    None
}

/// Gera (uma única vez) e devolve a chave estável + o endereço do overlay no OBS.
pub fn overlay_info(rt: &Runtime, p: &str) -> Result<Value, String> {
    let mut cfg = config(rt, p);
    if cfg.overlay_token.is_empty() {
        cfg.overlay_token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
    }
    let port = rt.db.get("apiPort").as_u64().unwrap_or(9876).clamp(1024, 65535);
    Ok(json!({
        "token": cfg.overlay_token,
        "port": port,
        "url": format!("http://127.0.0.1:{port}/overlay#token={}&profile={}", cfg.overlay_token, p),
    }))
}

// ---- Token ----

/// Cache em memória do access token (uma reabertura não refaz o OAuth2 e o tick
/// de 15 segundos não gera um pedido novo a cada volta). Nunca vai para o banco.
fn token_cache() -> &'static Mutex<HashMap<String, (String, Instant)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (String, Instant)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn forget_token(profile_id: &str) {
    token_cache().lock().unwrap().remove(profile_id);
}

async fn fetch_token(rt: &Arc<Runtime>, profile_id: &str) -> Result<String, String> {
    let cfg = config(rt, profile_id);
    let client_id = cfg.client_id.trim();
    if client_id.is_empty() {
        return Err("Client ID da LivePix não configurado".into());
    }
    if let Some((token, until)) = token_cache().lock().unwrap().get(profile_id) {
        if *until > Instant::now() { return Ok(token.clone()); }
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
    let token = v.get("data")
        .and_then(|d| d.get("accessToken"))
        .and_then(|t| t.as_str())
        .or_else(|| v.get("access_token").and_then(|t| t.as_str()))
        .map(str::to_owned)
        .ok_or_else(|| "access_token não retornado".to_owned())?;
    let ttl = v.get("expires_in").and_then(|e| e.as_u64()).unwrap_or(600).saturating_sub(60).clamp(30, 3600);
    token_cache().lock().unwrap().insert(profile_id.to_owned(), (token.clone(), Instant::now() + Duration::from_secs(ttl)));
    Ok(token)
}

// ---- Tick ----

/// Guarda o último problema na própria meta (chave `last_error`) para a tela
/// mostrar o aviso na hora certa — token recusado não fica só no log.
fn note_error(rt: &Runtime, p: &str, msg: &str) {
    let mut d = data(rt, p);
    d["last_error"] = json!(msg);
    d["last_error_at"] = json!(now_seconds());
    save_data(rt, p, &d);
}

pub async fn tick(rt: &Arc<Runtime>) {
    for profile in rt.db.profiles().unwrap_or_default() {
        let cfg = config(rt, &profile.id);
        if !cfg.enabled || cfg.active_goal.is_empty() { continue; }
        let goal = match cfg.goals.get(&cfg.active_goal) { Some(g) => g.clone(), None => continue };
        let token = match fetch_token(rt, &profile.id).await {
            Ok(t) => t,
            Err(e) => { rt.log(&profile.id, "livepix", &format!("token: {e}"), "warn"); note_error(rt, &profile.id, &format!("Não foi possível renovar o acesso da LivePix ({e}). Confira as credenciais na aba Conta.")); continue; }
        };
        let url = format!("https://api.livepix.gg/v2/payments?limit=100&currency=BRL");
        let resp = rt.http.get(&url).bearer_auth(&token).timeout(Duration::from_secs(20)).send().await;
        let resp = match resp { Ok(r) => r, Err(e) => { rt.log(&profile.id, "livepix", &e.to_string(), "error"); note_error(rt, &profile.id, &format!("A API da LivePix não respondeu ({e}).")); continue; } };
        if resp.status().as_u16() == 401 {
            forget_token(&profile.id);
            rt.log(&profile.id, "livepix", "Token da LivePix recusado; será renovado na próxima consulta.", "warn");
            note_error(rt, &profile.id, "O acesso da LivePix foi recusado (token expirado). Salve as credenciais novamente para renovar — o contador para enquanto isso.");
            continue;
        }
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
        // Pagamento chegou: o aviso antigo de acesso perde o sentido.
        d.as_object_mut().map(|o| { o.remove("last_error"); o.remove("last_error_at"); });
        d["current"] = json!(new);
        d["updated_at"] = json!(now_seconds());
        d["active_goal"] = json!(&cfg.active_goal);
        if d["start_timestamp"].is_null() || d["start_timestamp"].as_i64().unwrap_or(0) == 0 {
            d["start_timestamp"] = json!(now_seconds());
        }
        save_data(rt, &profile.id, &d);
        let pct = if goal.target > 0 { ((new * 100) / goal.target).clamp(0, 100) } else { 0 };
        let celebrate = goal.target > 0 && old > 0 && old < goal.target && new >= goal.target;
        let _ = rt.broadcast.send(json!({
            "type": "goal",
            "payload": {
                "profileId": profile.id,
                "title": goal.title,
                "current": new,
                "target": goal.target,
                "font_family": goal.font_family,
                "percent": pct,
                "celebrate": celebrate,
            }
        }));
        // Sons da Meta: o broadcast acima é só visual (overlay/estúdio).
        goal_sound(rt, &profile, goal_sound_key(celebrate, new > old));
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
            let previous = cfg.active_goal.clone();
            cfg.goals.clear();
            for (i, g) in goals.iter().enumerate() {
                let title = g["title"].as_str().unwrap_or("Meta").to_owned();
                let target = g["target"].as_i64().unwrap_or(0);
                let font = g["font_family"].as_str().unwrap_or("").to_owned();
                cfg.goals.insert(format!("goal_{i}"), GoalDef { title, target, font_family: font });
            }
            // Reativar mantém a meta e o progresso: o contador só muda com
            // dinheiro entrando ou com um "Reiniciar contador" pedido.
            cfg.active_goal = if previous.is_empty() { String::new() } else { previous };
            if !cfg.goals.contains_key(&cfg.active_goal) {
                cfg.active_goal = cfg.goals.keys().next().cloned().unwrap_or_default();
            }
            let mut d = data(rt, p);
            let stamp = d["start_timestamp"].as_i64().filter(|v| *v > 0).unwrap_or_else(now_seconds);
            d["start_timestamp"] = json!(stamp);
            if d["current"].is_null() { d["current"] = json!(0); }
            d["active_goal"] = json!(cfg.active_goal);
            cfg.start_timestamp = stamp;
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            save_data(rt, p, &d);
            publish_goal(rt, p);
            rt.log(p, "livepix", "Integração LivePix ativada.", "info");
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        "livepix.disable" => {
            let mut cfg = config(rt, p);
            cfg.enabled = false;
            cfg.active_goal = String::new();
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            // O progresso fica guardado: desligar a integração esconde a meta
            // no overlay sem apagar quanto já entrou.
            publish_goal(rt, p);
            rt.log(p, "livepix", "Integração LivePix desativada.", "info");
            Ok(Value::Null)
        }
        "livepix.update_goal" => {
            let gid = args["goalId"].as_str().ok_or("Meta ausente")?;
            let title = args["title"].as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or("Informe o título da meta")?.to_owned();
            let target = args["target"].as_i64().unwrap_or(0);
            if target <= 0 { return Err("Informe um valor-alvo maior que zero".into()); }
            let mut cfg = config(rt, p);
            let Some(goal) = cfg.goals.get_mut(gid) else { return Err("Meta inválida".into()); };
            goal.title = title;
            goal.target = target;
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            publish_goal(rt, p);
            rt.log(p, "livepix", "Meta atualizada no estúdio.", "info");
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        "livepix.reset_counter" => {
            let mut cfg = config(rt, p);
            if cfg.active_goal.is_empty() { return Err("Ative uma meta antes de reiniciar a contagem".into()); }
            cfg.start_timestamp = now_seconds();
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            let mut d = data(rt, p);
            d["current"] = json!(0);
            d["start_timestamp"] = json!(cfg.start_timestamp);
            save_data(rt, p, &d);
            publish_goal(rt, p);
            rt.log(p, "livepix", "Contador da meta reiniciado.", "info");
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
            publish_goal(rt, p);
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        "livepix.remove_goal" => {
            let gid = args["goalId"].as_str().ok_or("Meta ausente")?;
            let mut cfg = config(rt, p);
            let Some(gone) = cfg.goals.remove(gid) else { return Err("Meta inválida".into()); };
            if cfg.active_goal == gid { cfg.active_goal = String::new(); }
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            rt.log(p, "livepix", &format!("Meta \"{}\" removida do estúdio.", gone.title), "info");
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        // Adiciona meta sem tocar na meta ativa nem zerar o contador (diferente
        // de enable, que reinicia a contagem por definição).
        "livepix.add_goal" => {
            let title = args["title"].as_str().map(str::trim).filter(|s| !s.is_empty()).ok_or("Informe o título da meta")?.to_owned();
            let target = args["target"].as_i64().unwrap_or(0);
            if target <= 0 { return Err("Informe um valor-alvo maior que zero".into()); }
            let mut cfg = config(rt, p);
            let mut n = cfg.goals.len();
            let mut id = format!("goal_{n}");
            while cfg.goals.contains_key(&id) { n += 1; id = format!("goal_{n}"); }
            cfg.goals.insert(id.clone(), GoalDef { title, target, font_family: String::new() });
            if cfg.active_goal.is_empty() { cfg.active_goal = id; }
            rt.db.set_module(p, MODULE, &serde_json::to_value(&cfg).map_err(|e| e.to_string())?)?;
            rt.log(p, "livepix", "Meta adicionada ao estúdio.", "info");
            Ok(serde_json::to_value(&cfg).map_err(|e| e.to_string())?)
        }
        "livepix.design_get" => Ok(design(rt, p)),
        "livepix.design_save" => {
            let raw = args.get("design").ok_or("Design ausente")?;
            if !raw.is_object() { return Err("Design inválido".into()); }
            let d = sanitize_design(raw);
            rt.db.set_module(p, DESIGN, &d)?;
            // O overlay recebe na hora: editar no Estúdio redesenha o OBS sem recarregar a fonte.
            let _ = rt.broadcast.send(json!({"type": "design", "payload": {"profileId": p, "design": d}}));
            Ok(d)
        }
        "livepix.overlay_info" => overlay_info(rt, p),
        _ => Err("Operação desconhecida".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(name: &str) -> crate::model::Profile {
        crate::model::Profile {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            platform: "twitch".into(),
            channel: name.into(),
            channel_id: "1".into(),
            bot_id: String::new(),
            client_id: String::new(),
            blocklist: vec![],
            topics: vec![],
            editors: vec![],
            ai: Default::default(),
            modules: json!({}),
        }
    }

    /// As chaves do design vivem em src/overlayDesign.keys.json, lidas pelos dois lados:
    /// mudar o campo em um idioma sem mudar no outro quebra este teste.
    #[test]
    fn as_chaves_do_design_fortem_o_mesmo_conjunto_do_estudio() {
        let mut shared: Vec<String> =
            serde_json::from_str(include_str!("../../src/overlayDesign.keys.json")).expect("lista compartilhada de chaves");
        shared.sort();
        let mut got: Vec<String> = default_design().as_object().unwrap().keys().cloned().collect();
        got.sort();
        assert_eq!(got, shared, "Rust e Estúdio (TS) precisam higienizar as mesmas chaves do design");
    }

    #[test]
    fn design_padrao_e_o_mesmo_do_estudio() {
        let d = default_design();
        assert_eq!(d["template"], "clean");
        assert_eq!(d["layout"], "bar");
        assert_eq!(d["align"], "center");
        assert_eq!(d["valign"], "bottom");
        assert_eq!(d["width"].as_f64(), Some(80.0));
        assert_eq!(d["font"], "Segoe UI");
        assert_eq!(d["fontSize"].as_i64(), Some(18));
        assert_eq!(d["animate"].as_bool(), Some(false));
        assert_eq!(d["celebrate"].as_bool(), Some(true));
        assert_eq!(d["soundDonate"], "", "sem som escolhido por padrão");
        assert_eq!(d["soundCelebrate"], "");
    }

    #[tokio::test]
    async fn sons_da_meta_saim_pela_linha_somente_com_escolha_valida() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        let p = profile("P");
        rt.db.save_profile(&p).unwrap();
        rt.db.set_module(&p.id, "audio", &json!({"device": "cable-3"})).unwrap();
        let mut rx = rt.broadcast.subscribe();
        // Sem escolha no design não toca nada.
        goal_sound(&rt, &p, "soundDonate");
        goal_sound(&rt, &p, "soundCelebrate");
        assert!(!std::iter::from_fn(|| rx.try_recv().ok()).any(|v| v["type"] == "viewer-sound"), "nada sem escolha");
        // Com escolha, toca na linha do app com o aparelho da tela Áudio.
        let file = dir.path().join("jingle.wav");
        std::fs::write(&file, b"RIFF0000WAVEfmt ").unwrap();
        let id = crate::chat_extras::operation(&rt, &p.id, "chatExtras.import", &json!({"path": file, "kind": "sound"}))
            .unwrap()["id"].as_str().unwrap().to_string();
        let mut d = default_design();
        d["soundDonate"] = json!(&id);
        d["soundCelebrate"] = json!(&id);
        rt.db.set_module(&p.id, DESIGN, &d).unwrap();
        goal_sound(&rt, &p, "soundDonate");
        goal_sound(&rt, &p, "soundCelebrate");
        let seen: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).filter(|v| v["type"] == "viewer-sound").collect();
        assert_eq!(seen.len(), 2, "doação e celebração tocam");
        assert_eq!(seen[0]["payload"]["deviceId"], "cable-3", "aparelho vem da aba Áudio");
        assert_eq!(seen[0]["payload"]["asset"], json!(id));
        assert_eq!(seen[0]["payload"]["volume"], 1.0, "volume cheio na escala 0–1");
        // Arquivo removido depois da escolha: silencia em vez de notificar erro.
        rt.db.set_module(&p.id, DESIGN, &default_design()).unwrap();
        goal_sound(&rt, &p, "soundDonate");
        assert!(std::iter::from_fn(|| rx.try_recv().ok()).next().is_none(), "asset removido não toca");
        // Higienização: id fora do formato vira vazio (sem som).
        assert_eq!(asset_id(&json!("x; drop table")), "");
        assert_eq!(asset_id(&json!(id)), id, "UUID passa");
    }

    #[test]
    fn celebracao_substitui_o_som_da_doacao() {
        assert_eq!(goal_sound_key(true, true), "soundCelebrate");
        assert_eq!(goal_sound_key(false, true), "soundDonate");
        assert_eq!(goal_sound_key(false, false), "");
    }

    #[test]
    fn design_apara_faixas_remove_chaves_e_rejeita_texto_de_css() {
        let d = sanitize_design(&json!({
            "layout": "ring", "align": "left", "valign": "top", "width": 900.0,
            "glow": -20.0, "shadow": 500.0, "barHeight": 1.0, "fontSize": 900.0,
            "fontWeight": 555.0, "borderWidth": 3.7,
            "fillStart": "#ff0000", "panel": "javascript:alert(1)",
            "font": "x;}body{background:url(evil",
            "unknownField": true,
        }));
        assert_eq!(d["layout"], "ring");
        assert_eq!(d["width"].as_f64(), Some(100.0));
        assert_eq!(d["glow"].as_i64(), Some(0));
        assert_eq!(d["shadow"].as_i64(), Some(100));
        assert_eq!(d["barHeight"].as_i64(), Some(4));
        assert_eq!(d["fontSize"].as_i64(), Some(48));
        assert_eq!(d["fontWeight"].as_f64(), Some(700.0));
        assert_eq!(d["borderWidth"].as_f64(), Some(3.5));
        assert_eq!(d["fillStart"], "#ff0000");
        assert_eq!(d["panel"], "#101721dd");
        assert_eq!(d["font"], "Segoe UI");
        assert!(d.get("unknownField").is_none());
    }

    #[test]
    fn design_aceita_fonte_legitima_com_espaco_e_apostrofo() {
        assert_eq!(sanitize_design(&json!({"font": "Bebas Neue", "template": "aurora"}))["font"], "Bebas Neue");
        assert_eq!(sanitize_design(&json!({"font": "Fonte d'água"}))["font"], "Fonte d'água");
        assert_eq!(sanitize_design(&json!({"font": "Press Start 2P"}))["font"], "Press Start 2P");
    }

    #[tokio::test]
    async fn goal_message_esconde_quando_desligada_e_mostra_percentual_quando_ligada() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        let p = profile("P");
        rt.db.save_profile(&p).unwrap();
        assert_eq!(goal_message(&rt, &p.id)["payload"]["hidden"], json!(true));

        let mut cfg = config(&rt, &p.id);
        cfg.enabled = true;
        cfg.active_goal = "goal_0".into();
        cfg.goals.insert("goal_0".into(), GoalDef { title: "Meta".into(), target: 50000, font_family: String::new() });
        rt.db.set_module(&p.id, MODULE, &serde_json::to_value(&cfg).unwrap()).unwrap();
        rt.db.set_module(&p.id, DATA, &json!({"current":12500})).unwrap();

        let msg = goal_message(&rt, &p.id);
        assert_eq!(msg["payload"]["hidden"], json!(false));
        assert_eq!(msg["payload"]["percent"].as_i64(), Some(25));
        assert_eq!(msg["payload"]["current"].as_i64(), Some(12500));
        assert_eq!(msg["payload"]["title"], json!("Meta"));
    }

    #[tokio::test]
    async fn chave_do_overlay_e_estavel_e_so_le_estado() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        let p = profile("P");
        rt.db.save_profile(&p).unwrap();
        assert!(overlay_profile(&rt, "qualquer").is_none());
        let info = overlay_info(&rt, &p.id).unwrap();
        let token = info["token"].as_str().unwrap().to_owned();
        assert!(info["url"].as_str().unwrap().contains("/overlay#token="));
        // a mesma chave em outra chamada: o endereço do OBS nunca muda
        assert_eq!(overlay_info(&rt, &p.id).unwrap()["token"], json!(token));
        assert_eq!(overlay_profile(&rt, &token).as_deref(), Some(p.id.as_str()));
        // chave da sessão do aplicativo não é aceita como chave de overlay
        assert!(overlay_profile(&rt, &rt.api_token).is_none());
    }

    #[tokio::test]
    async fn adicionar_meta_preserva_a_ativa_e_o_contador_e_evita_colisao_de_id() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Arc::new(Runtime::new(dir.path().into()).unwrap());
        let p = profile("P");
        rt.db.save_profile(&p).unwrap();

        // validação básica
        assert!(operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "   ", "target": 500})).await.is_err());
        assert!(operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "Meta", "target": 0})).await.is_err());

        // a primeira meta criada vira a ativa
        let cfg: Value = operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "Meta A", "target": 50000})).await.unwrap();
        let first = cfg["activeGoal"].as_str().unwrap().to_owned();
        assert_eq!(cfg["goals"][&first]["title"], json!("Meta A"));
        rt.db.set_module(&p.id, DATA, &json!({"current": 12500})).unwrap();

        // acrescentar meta com a integração rodando não troca a ativa nem zera o contador
        let cfg: Value = operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "Meta B", "target": 80000})).await.unwrap();
        assert_eq!(cfg["activeGoal"], json!(first));
        assert_eq!(cfg["goals"].as_object().unwrap().len(), 2);
        assert_eq!(data(&rt, &p.id)["current"].as_i64(), Some(12500));

        // remover a ativa libera a vaga e a próxima meta não reusa id existente
        operation(&rt, &p.id, "livepix.remove_goal", &json!({"goalId": first})).await.unwrap();
        let cfg: Value = operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "Meta C", "target": 30000})).await.unwrap();
        let keys: Vec<String> = cfg["goals"].as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, vec!["goal_1".to_string(), "goal_2".to_string()]);
        assert_eq!(cfg["activeGoal"], json!("goal_2"));
        assert_eq!(data(&rt, &p.id)["current"].as_i64(), Some(12500));
    }

    #[tokio::test]
    async fn ligar_de_novo_nao_zera_a_contagem_e_pegar_apaga() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Arc::new(Runtime::new(dir.path().into()).unwrap());
        let p = profile("P");
        rt.db.save_profile(&p).unwrap();

        operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "Meta A", "target": 50000})).await.unwrap();
        rt.db.set_module(&p.id, DATA, &json!({"current": 12500, "start_timestamp": 1728000000i64})).unwrap();

        // Desligar guarda o progresso e só esconde a meta no overlay.
        operation(&rt, &p.id, "livepix.disable", &json!({})).await.unwrap();
        assert_eq!(data(&rt, &p.id)["current"].as_i64(), Some(12500));
        assert_eq!(goal_message(&rt, &p.id)["payload"]["hidden"], json!(true));

        // Ligar de novo mantém o contador e o marco de início (o dinheiro
        // desde o começo da meta continua contando).
        let cfg: Value = operation(&rt, &p.id, "livepix.enable", &json!({"goals": [{"title": "Meta A", "target": 50000}]})).await.unwrap();
        assert_eq!(cfg["activeGoal"], json!("goal_0"));
        assert_eq!(data(&rt, &p.id)["current"].as_i64(), Some(12500));
        assert_eq!(data(&rt, &p.id)["start_timestamp"].as_i64(), Some(1728000000));
        assert_eq!(goal_message(&rt, &p.id)["payload"]["current"].as_i64(), Some(12500));

        // Reiniciar contador é um passo à parte, pedido de propósito.
        operation(&rt, &p.id, "livepix.reset_counter", &json!({})).await.unwrap();
        assert_eq!(data(&rt, &p.id)["current"].as_i64(), Some(0));
    }

    #[tokio::test]
    async fn editar_meta_muda_titulo_e_alvo_e_publica_na_hora() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Arc::new(Runtime::new(dir.path().into()).unwrap());
        let p = profile("P");
        rt.db.save_profile(&p).unwrap();

        assert!(operation(&rt, &p.id, "livepix.update_goal", &json!({"goalId": "goal_0", "title": "Nova", "target": 10})).await.is_err());
        operation(&rt, &p.id, "livepix.add_goal", &json!({"title": "Meta A", "target": 50000})).await.unwrap();
        assert!(operation(&rt, &p.id, "livepix.update_goal", &json!({"goalId": "goal_0", "title": "  ", "target": 10})).await.is_err());
        assert!(operation(&rt, &p.id, "livepix.update_goal", &json!({"goalId": "goal_0", "title": "Nova", "target": 0})).await.is_err());

        // Ligar a integração com a meta pronta e a contagem andando.
        rt.db.set_module(&p.id, DATA, &json!({"current": 25000})).unwrap();
        operation(&rt, &p.id, "livepix.enable", &json!({"goals": [{"title": "Meta A", "target": 50000}]})).await.unwrap();

        let cfg: Value = operation(&rt, &p.id, "livepix.update_goal", &json!({"goalId": "goal_0", "title": "Meta Mensal", "target": 100000})).await.unwrap();
        assert_eq!(cfg["goals"]["goal_0"]["title"], json!("Meta Mensal"));
        assert_eq!(cfg["goals"]["goal_0"]["target"].as_i64(), Some(100000));
        let msg = goal_message(&rt, &p.id);
        assert_eq!(msg["payload"]["title"], json!("Meta Mensal"));
        assert_eq!(msg["payload"]["percent"].as_i64(), Some(25));
    }
}

