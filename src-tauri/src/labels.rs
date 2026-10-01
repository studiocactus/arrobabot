//! Rótulos estilo StreamLabels: arquivos .txt para o OBS com totais do canal,
//! eventos recentes, espectadores e horas assistidas. Tudo vem da Twitch
//! (totais e chatters) ou do que o bot já recebe (follows, subs, cheers, raids).
use crate::{db::Db, engine::Runtime, model::Profile, oauth};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, path::Path, sync::Arc};
pub const MODULE: &str = "labels";
const DATA: &str = "labels_data";
const MAX_TEMPLATE: usize = 200;
const REFRESH_SECS: i64 = 300;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LabelCfg {
 pub enabled: bool,
 pub template: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
 pub enabled: bool,
 pub folder: String,
 pub labels: HashMap<String, LabelCfg>,
}
pub struct Def {
 pub key: &'static str,
 pub file: &'static str,
 pub name: &'static str,
 pub template: &'static str,
 pub tokens: &'static str,
}
pub const DEFS: &[Def] = &[
 Def { key: "follower_total", file: "follower_total.txt", name: "Total de seguidores", template: "{total}", tokens: "{total}" },
 Def { key: "recent_follower", file: "recent_follower.txt", name: "Seguidor recente", template: "{name}", tokens: "{name}" },
 Def { key: "sub_total", file: "sub_total.txt", name: "Total de subs", template: "{total}", tokens: "{total}" },
 Def { key: "recent_sub", file: "recent_sub.txt", name: "Sub recente", template: "{name}", tokens: "{name}" },
 Def { key: "recent_resub", file: "recent_resub.txt", name: "Resub recente", template: "{name} ({months} meses)", tokens: "{name} {months}" },
 Def { key: "gifts_session", file: "gifts_session.txt", name: "Subs de presente na sessão", template: "{count}", tokens: "{count}" },
 Def { key: "cheer_session", file: "cheer_session.txt", name: "Bits na sessão", template: "{total}", tokens: "{total}" },
 Def { key: "recent_cheer", file: "recent_cheer.txt", name: "Cheer recente", template: "{name} ({amount})", tokens: "{name} {amount}" },
 Def { key: "recent_raid", file: "recent_raid.txt", name: "Raid recente", template: "{name} ({viewers})", tokens: "{name} {viewers}" },
 Def { key: "viewers_now", file: "viewers_now.txt", name: "Espectadores no chat", template: "{count}", tokens: "{count}" },
 Def { key: "watch_total", file: "watch_total.txt", name: "Horas assistidas (todos)", template: "{hours}", tokens: "{hours}" },
 Def { key: "top_watcher", file: "top_watcher.txt", name: "Quem mais assistiu", template: "{name} ({hours})", tokens: "{name} {hours}" },
 Def { key: "uptime", file: "uptime.txt", name: "Tempo de live", template: "{uptime}", tokens: "{uptime}" },
];
fn template_of(c: &Config, key: &str) -> String {
 c.labels.get(key).map(|l| l.template.clone()).filter(|t| !t.trim().is_empty())
  .unwrap_or_else(|| DEFS.iter().find(|d| d.key == key).map(|d| d.template.to_owned()).unwrap_or_default())
}
fn enabled(c: &Config, key: &str) -> bool {
 c.labels.get(key).map(|l| l.enabled).unwrap_or(true)
}
pub fn config(rt: &Runtime, p: &str) -> Config {
 serde_json::from_value(rt.db.module(p, MODULE)).unwrap_or_default()
}
fn data(rt: &Runtime, p: &str) -> Value {
 let mut d = rt.db.module(p, DATA);
 if !d.is_object() { d = json!({}); }
 d
}
fn save_data(rt: &Runtime, p: &str, d: &Value) {
 let _ = rt.db.set_module(p, DATA, d);
}
/// Troca os tokens `{nome}` pelos valores. Token fora da lista sai como está.
pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
 let mut out = String::with_capacity(template.len());
 let mut rest = template;
 while let Some(s) = rest.find('{') {
  out.push_str(&rest[..s]);
  match rest[s + 1..].find('}') {
   Some(e) => {
    let name = &rest[s + 1..s + 1 + e];
    out.push_str(&vars.get(name).map(String::as_str).unwrap_or_else(|| &rest[s..s + 1 + e + 1]));
    rest = &rest[s + 1 + e + 1..];
   }
   None => { out.push_str(&rest[s..]); rest = ""; break; }
  }
 }
 out.push_str(rest);
 out
}
pub fn hours_text(seconds: i64) -> String {
 format!("{:.1}", seconds as f64 / 3600.0).replace('.', ",")
}
fn uptime_text(started_at: &str) -> Option<String> {
 let start = chrono::DateTime::parse_from_rfc3339(started_at).ok()?.timestamp();
 let el = chrono::Utc::now().timestamp().saturating_sub(start).max(0);
 Some(format!("{}:{:02}:{:02}", el / 3600, el % 3600 / 60, el % 60))
}
fn values(d: &Value) -> HashMap<String, HashMap<String, String>> {
 let str_ = |v: &Value| match v {
  Value::String(s) => s.clone(),
  Value::Number(_) => v.to_string(),
  _ => String::new(),
 };
 let get = |k: &str| str_(&d[k]);
 let mut m: HashMap<String, HashMap<String, String>> = HashMap::new();
 let mut one = HashMap::new();
 one.insert("total".into(), get("followers"));
 m.insert("follower_total".into(), one);
 let mut one = HashMap::new();
 one.insert("name".into(), get("recent_follower"));
 m.insert("recent_follower".into(), one);
 let mut one = HashMap::new();
 one.insert("total".into(), get("subs"));
 m.insert("sub_total".into(), one);
 let mut one = HashMap::new();
 one.insert("name".into(), get("recent_sub"));
 m.insert("recent_sub".into(), one);
 let mut one = HashMap::new();
 one.insert("name".into(), get("recent_resub"));
 one.insert("months".into(), get("recent_resub_months"));
 m.insert("recent_resub".into(), one);
 let mut one = HashMap::new();
 one.insert("count".into(), get("session_gifts"));
 m.insert("gifts_session".into(), one);
 let mut one = HashMap::new();
 one.insert("total".into(), get("session_bits"));
 m.insert("cheer_session".into(), one);
 let mut one = HashMap::new();
 one.insert("name".into(), get("recent_cheer"));
 one.insert("amount".into(), get("recent_cheer_bits"));
 m.insert("recent_cheer".into(), one);
 let mut one = HashMap::new();
 one.insert("name".into(), get("recent_raid"));
 one.insert("viewers".into(), get("recent_raid_viewers"));
 m.insert("recent_raid".into(), one);
 let mut one = HashMap::new();
 one.insert("count".into(), get("viewers_total"));
 m.insert("viewers_now".into(), one);
 let mut one = HashMap::new();
 one.insert("hours".into(), get("watch_hours"));
 m.insert("watch_total".into(), one);
 let mut one = HashMap::new();
 one.insert("name".into(), get("top_watcher"));
 one.insert("hours".into(), get("top_hours"));
 m.insert("top_watcher".into(), one);
 let mut one = HashMap::new();
 one.insert("uptime".into(), get("uptime"));
 m.insert("uptime".into(), one);
 m
}
fn folder(c: &Config) -> Result<std::path::PathBuf, String> {
 let f = c.folder.trim();
 if f.is_empty() { return Err("Escolha a pasta dos arquivos.".into()); }
 let path = Path::new(f);
 std::fs::create_dir_all(path).map_err(|_| "Não foi possível criar a pasta. Confira o caminho.".to_string())?;
 if !path.is_dir() { return Err("O caminho não é uma pasta.".into()); }
 Ok(path.to_path_buf())
}
fn write_all(rt: &Runtime, p: &Profile, c: &Config, d: &Value) -> Result<(), String> {
 let dir = folder(c)?;
 let vals = values(d);
 for def in DEFS {
  if !enabled(c, def.key) { continue; }
  let text = render(&template_of(c, def.key), &vals.get(def.key).cloned().unwrap_or_default());
  std::fs::write(dir.join(def.file), text).map_err(|_| format!("Não foi possível gravar {}.", def.file))?;
 }
 let _ = rt;
 Ok(())
}
/// Soma segundos assistidos por login (minúsculo). Uma transação por chamada.
pub fn add_watch(db: &Db, profile: &str, logins: &[String], secs: i64) -> Result<(), String> {
 if logins.is_empty() || secs <= 0 { return Ok(()); }
 let mut c = db.0.lock().unwrap();
 let tx = c.transaction().map_err(|e| e.to_string())?;
 for login in logins {
  let login = login.trim().to_ascii_lowercase();
  if login.is_empty() || login.len() > 25 { continue; }
  tx.execute("INSERT INTO watch_time VALUES(?,?,?) ON CONFLICT(profile_id,user_login) DO UPDATE SET seconds=seconds+excluded.seconds", rusqlite::params![profile, login, secs]).map_err(|e| e.to_string())?;
 }
 tx.execute("DELETE FROM watch_time WHERE profile_id=?1 AND user_login NOT IN (SELECT user_login FROM watch_time WHERE profile_id=?1 ORDER BY seconds DESC LIMIT 20000)", rusqlite::params![profile]).map_err(|e| e.to_string())?;
 tx.commit().map_err(|e| e.to_string())?;
 Ok(())
}
pub fn watch_summary(db: &Db, profile: &str) -> (i64, String, i64) {
 let c = db.0.lock().unwrap();
 let total: i64 = c.query_row("SELECT COALESCE(SUM(seconds),0) FROM watch_time WHERE profile_id=?", rusqlite::params![profile], |r| r.get(0)).unwrap_or(0);
 let top: Option<(String, i64)> = c.query_row("SELECT user_login,seconds FROM watch_time WHERE profile_id=? ORDER BY seconds DESC LIMIT 1", rusqlite::params![profile], |r| Ok((r.get(0)?, r.get(1)?))).ok();
 match top {
  Some((name, secs)) => (total, name, secs),
  None => (total, String::new(), 0),
 }
}
/// Lista quem está no chat agora (exige o bot moderador com leitura de chatters).
pub async fn chatters(rt: &Arc<Runtime>, p: &Profile) -> Result<(i64, Vec<String>), String> {
 if p.platform != "twitch" { return Err("Espectadores ao vivo só na Twitch.".into()); }
 let token = oauth::token(&rt.http, p, "bot").await?;
 let v: Value = rt.http.get("https://api.twitch.tv/helix/chat/chatters")
  .query(&[("broadcaster_id", p.channel_id.as_str()), ("moderator_id", p.bot_id.as_str()), ("first", "1000")])
  .header("Client-Id", &p.client_id).bearer_auth(&token)
  .send().await.map_err(|_| "Falha ao consultar espectadores.".to_string())?
  .json().await.map_err(|_| "Resposta inválida de espectadores.".to_string())?;
 if v["status"].as_u64().is_some_and(|s| s == 401 || s == 403) {
  return Err("Sem permissão de chatters: autorize novamente a conta do bot no perfil.".into());
 }
 let total = v["total"].as_i64().unwrap_or(0);
 let users: Vec<String> = v["data"].as_array().cloned().unwrap_or_default().into_iter()
  .filter_map(|u| u["user_login"].as_str().map(|s| s.to_owned())).take(1000).collect();
 Ok((total, users))
}
async fn totals(rt: &Arc<Runtime>, p: &Profile) -> (Option<i64>, Option<i64>, String) {
 let (followers, subs) = crate::twitch_ops::channel_counts(rt, p).await;
 let mut uptime = String::new();
 if let Ok(token) = oauth::token(&rt.http, p, "channel").await {
  if let Ok(r) = rt.http.get("https://api.twitch.tv/helix/streams").query(&[("user_id", p.channel_id.as_str())])
   .header("Client-Id", &p.client_id).bearer_auth(&token).send().await {
   if let Ok(v) = r.json::<Value>().await {
    if let Some(s) = v["data"][0]["started_at"].as_str() {
     uptime = uptime_text(s).unwrap_or_default();
    }
   }
  }
 }
 (followers, subs, uptime)
}
fn stale(d: &Value, now: i64) -> bool {
 d["refreshed"].as_i64().unwrap_or(0) + REFRESH_SECS <= now
}
/// Atualiza totais + uptime + horas e regrava os arquivos. Nunca falha para fora.
pub async fn refresh(rt: &Arc<Runtime>, p: &Profile) {
 let c = config(rt, &p.id);
 if !c.enabled || p.platform != "twitch" { return; }
 let now = chrono::Utc::now().timestamp();
 let (followers, subs, uptime) = totals(rt, p).await;
 let (watch_secs, top_name, top_secs) = watch_summary(&rt.db, &p.id);
 let mut d = data(rt, &p.id);
 if let Some(f) = followers { d["followers"] = json!(f); }
 if let Some(s) = subs { d["subs"] = json!(s); }
 if !uptime.is_empty() { d["uptime"] = json!(uptime); }
 d["watch_hours"] = json!(hours_text(watch_secs));
 d["top_watcher"] = json!(top_name);
 d["top_hours"] = json!(hours_text(top_secs));
 d["refreshed"] = json!(now);
 save_data(rt, &p.id, &d);
 if let Err(err) = write_all(rt, p, &c, &d) { rt.log(&p.id, "labels", &err, "error"); }
}
/// Eventos de follow/sub/cheer/raid atualizam recentes + sessão na hora, sem rede.
pub async fn on_event(rt: &Arc<Runtime>, p: &Profile, kind: &str, user: &str, amount: i64, months: i64) {
 let c = config(rt, &p.id);
 if !c.enabled || p.platform != "twitch" { return; }
 let mut d = data(rt, &p.id);
 match kind {
  "follow" => { d["recent_follower"] = json!(user); }
  "subscription" => { d["recent_sub"] = json!(user); }
  "resub" => { d["recent_resub"] = json!(user); d["recent_resub_months"] = json!(months); }
  "gift" => { d["session_gifts"] = json!(d["session_gifts"].as_i64().unwrap_or(0) + amount.max(1)); }
  "cheer" => {
   d["recent_cheer"] = json!(user);
   d["recent_cheer_bits"] = json!(amount);
   d["session_bits"] = json!(d["session_bits"].as_i64().unwrap_or(0) + amount);
  }
  "raid" => { d["recent_raid"] = json!(user); d["recent_raid_viewers"] = json!(amount); }
  _ => return,
 }
 save_data(rt, &p.id, &d);
 if let Err(err) = write_all(rt, p, &c, &d) { rt.log(&p.id, "labels", &err, "error"); }
 let now = chrono::Utc::now().timestamp();
 if matches!(kind, "follow" | "subscription" | "gift") && stale(&d, now) {
  let rt2 = rt.clone();
  let p2 = p.clone();
  tokio::spawn(async move { refresh(&rt2, &p2).await; });
 }
}
/// Zera a sessão ao conectar; recentes e horas acumuladas continuam.
pub fn session_reset(db: &Db, p: &Profile) {
 let mut d: Value = db.module(&p.id, DATA);
 if !d.is_object() { d = json!({}); }
 d["session_bits"] = json!(0);
 d["session_gifts"] = json!(0);
 d["session_start"] = json!(chrono::Utc::now().timestamp());
 d["refreshed"] = json!(0);
 let _ = db.set_module(&p.id, DATA, &d);
}
/// A cada 30s: quem está no chat soma tempo assistido; arquivos do momento regravados.
pub async fn tick(rt: &Arc<Runtime>) {
 let now = chrono::Utc::now().timestamp();
 for p in rt.db.profiles().unwrap_or_default() {
  if p.platform != "twitch" { continue; }
  if rt.statuses.lock().unwrap().get(&p.id).map(String::as_str) != Some("online") { continue; }
  let c = config(rt, &p.id);
  if !c.enabled { continue; }
  let mut d = data(rt, &p.id);
  let delta = (now - d["ticked"].as_i64().unwrap_or(now - 30)).clamp(5, 120);
  d["ticked"] = json!(now);
  match chatters(rt, &p).await {
   Ok((total, users)) => {
    d["fail_streak"] = json!(0);
    let bot = bot_login(rt, &p).await;
    let present: Vec<String> = users.into_iter().filter(|u| Some(u) != bot.as_ref()).collect();
    if let Err(err) = add_watch(&rt.db, &p.id, &present, delta) { rt.log(&p.id, "labels", &err, "error"); }
    d["viewers_total"] = json!(total);
    d["viewers"] = json!(present);
    let (watch_secs, top_name, top_secs) = watch_summary(&rt.db, &p.id);
    d["watch_hours"] = json!(hours_text(watch_secs));
    d["top_watcher"] = json!(top_name);
    d["top_hours"] = json!(hours_text(top_secs));
    save_data(rt, &p.id, &d);
    if let Err(err) = write_all(rt, &p, &c, &d) { rt.log(&p.id, "labels", &err, "error"); }
    if stale(&d, now) { refresh(rt, &p).await; }
   }
   Err(err) => {
    let streak = d["fail_streak"].as_i64().unwrap_or(0) + 1;
    d["fail_streak"] = json!(streak);
    save_data(rt, &p.id, &d);
    if streak == 1 || streak % 10 == 0 { rt.log(&p.id, "labels", &err, "error"); }
   }
  }
 }
}
async fn bot_login(rt: &Arc<Runtime>, p: &Profile) -> Option<String> {
 let token = oauth::token(&rt.http, p, "bot").await.ok()?;
 rt.http.get("https://id.twitch.tv/oauth2/validate").header("Authorization", format!("OAuth {token}"))
  .send().await.ok()?.json::<Value>().await.ok()
  .and_then(|v| v["login"].as_str().map(|s| s.to_owned()))
}
fn validate_save(v: &Value) -> Result<Config, String> {
 let enabled = v["enabled"] == true;
 let folder = v["folder"].as_str().unwrap_or("").trim().to_owned();
 if enabled {
  if folder.is_empty() { return Err("Escolha a pasta dos arquivos.".into()); }
  std::fs::create_dir_all(&folder).map_err(|_| "Não foi possível criar a pasta. Confira o caminho.".to_string())?;
  if !Path::new(&folder).is_dir() { return Err("O caminho não é uma pasta.".into()); }
 }
 let mut labels = HashMap::new();
 let items = v["labels"].as_object().cloned().unwrap_or_default();
 for def in DEFS {
  let item = items.get(def.key);
  let template = item.and_then(|l| l["template"].as_str()).unwrap_or(def.template);
  if template.chars().count() > MAX_TEMPLATE { return Err(format!("Modelo de {} passa de 200 caracteres.", def.name)); }
  labels.insert(def.key.to_owned(), LabelCfg { enabled: item.and_then(|l| l["enabled"].as_bool()).unwrap_or(true), template: template.into() });
 }
 Ok(Config { enabled, folder, labels })
}
pub async fn operation(rt: &Arc<Runtime>, p: &str, op: &str, args: &Value) -> Result<Value, String> {
 rt.db.profile(p)?;
 let profile = rt.db.profile(p)?;
 if profile.platform != "twitch" { return Err("Rótulos só na Twitch.".into()); }
 match op {
  "labels.get" => {
   let c = config(rt, p);
   let d = data(rt, p);
   let vals = values(&d);
   let defs: Vec<Value> = DEFS.iter().map(|f| {
    json!({"key": f.key, "file": f.file, "name": f.name, "tokens": f.tokens,
     "enabled": enabled(&c, f.key), "template": template_of(&c, f.key),
     "value": render(&template_of(&c, f.key), &vals.get(f.key).cloned().unwrap_or_default())})
   }).collect();
   Ok(json!({"config": {"enabled": c.enabled, "folder": c.folder},
    "defs": defs,
    "viewers": d["viewers"], "viewersTotal": d["viewers_total"].as_i64().unwrap_or(0)}))
  }
  "labels.save" => {
   let c = validate_save(&args["config"])?;
   rt.db.set_module(p, MODULE, &serde_json::to_value(&c).map_err(|e| e.to_string())?)?;
   let d = data(rt, p);
   if c.enabled { write_all(rt, &profile, &c, &d)?; }
   rt.log(&p, "labels", "Rótulos salvos.", "success");
   Ok(Value::Null)
  }
  "labels.folder" => {
   let folder = args["path"].as_str().ok_or("Escolha uma pasta.")?.trim();
   if folder.is_empty() { return Err("Escolha uma pasta.".into()); }
   std::fs::create_dir_all(folder).map_err(|_| "Não foi possível criar a pasta.".to_string())?;
   let mut c = config(rt, p);
   c.folder = folder.into();
   rt.db.set_module(p, MODULE, &serde_json::to_value(&c).map_err(|e| e.to_string())?)?;
   Ok(json!(c.folder))
  }
  "labels.viewers" => {
   let (total, users) = chatters(rt, &profile).await?;
   Ok(json!({"total": total, "users": users}))
  }
  "labels.refresh" => {
   refresh(rt, &profile).await;
   Ok(Value::Null)
  }
  _ => Err("Operação desconhecida".into()),
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn template_tokens_swap_values_and_keep_unknowns() {
  let mut vars = HashMap::new();
  vars.insert("name".into(), "Ana".into());
  vars.insert("months".into(), "5".into());
  assert_eq!(render("{name} ({months} meses)", &vars), "Ana (5 meses)");
  assert_eq!(render("{name} {oops}", &vars), "Ana {oops}");
  assert_eq!(render("sem token", &vars), "sem token");
  assert_eq!(render("{aberto", &vars), "{aberto");
  assert_eq!(render("Olá {name}, café!", &vars), "Olá Ana, café!");
 }
 #[test]
 fn hours_and_uptime_format_for_overlay_files() {
  assert_eq!(hours_text(0), "0,0");
  assert_eq!(hours_text(5400), "1,5");
  assert_eq!(uptime_text("2026-09-30T06:00:00Z").map(|s| s.len() > 0), Some(true));
  assert_eq!(uptime_text("lixo"), None);
 }
 #[test]
 fn watch_time_accumulates_per_login_and_reports_top() {
  let dir = tempfile::tempdir().unwrap();
  let db = Db::open(&dir.path().join("t.sqlite")).unwrap();
  let p: Profile = serde_json::from_value(json!({"id": "00000000-0000-4000-8000-000000000001", "name": "P", "platform": "twitch", "channel": "canal"})).unwrap();
  db.save_profile(&p).unwrap();
  add_watch(&db, &p.id, &["Ana".into(), "Bia".into()], 30).unwrap();
  add_watch(&db, &p.id, &["ana".into()], 30).unwrap();
  assert!(add_watch(&db, &p.id, &[], 30).is_ok());
  assert!(add_watch(&db, &p.id, &["Cid".into()], 0).is_ok());
  let (total, top, secs) = watch_summary(&db, &p.id);
  assert_eq!(total, 90);
  assert_eq!((top.as_str(), secs), ("ana", 60));
 }
 #[test]
 fn save_rejects_bad_folder_and_long_templates() {
  assert!(validate_save(&json!({"enabled": true, "folder": "", "labels": {}})).is_err());
  let long = "x".repeat(201);
  assert!(validate_save(&json!({"enabled": true, "folder": ".", "labels": {"follower_total": {"enabled": true, "template": long}}})).is_err());
  let c = validate_save(&json!({"enabled": false, "folder": "", "labels": {}})).unwrap();
  assert!(!c.enabled);
  assert_eq!(template_of(&c, "follower_total"), "{total}");
 }
}
