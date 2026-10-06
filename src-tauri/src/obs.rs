//! OBS Studio local via OBS WebSocket 5.x. O app é desktop e o OBS está na
//! mesma máquina: conexão direta em ws://localhost, sem nuvem, sem túnel,
//! sem expor porta. Senha fica no cofre e nunca entra em log ou resposta.
use crate::{engine::Runtime, model::Profile};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
 collections::HashMap,
 sync::{Arc, Mutex},
 time::Duration,
};
use futures_util::{SinkExt, StreamExt};
pub const MODULE: &str = "obs";
const DEFAULT_PORT: u16 = 4455;
/// Operações seguras expostas a automações. Sem stop/exit/delete/settings.
pub const OPS: &[&str] = &["mute", "unmute", "toggle_mute", "volume", "show", "hide", "toggle_item", "scene"];
pub fn op_name(op: &str) -> &'static str {
 match op {
  "mute" => "Mutar entrada",
  "unmute" => "Desmutar entrada",
  "toggle_mute" => "Alternar mudo",
  "volume" => "Volume da entrada",
  "show" => "Mostrar fonte",
  "hide" => "Esconder fonte",
  "toggle_item" => "Alternar fonte",
  "scene" => "Trocar de cena",
  _ => "Desconhecida",
 }
}
/// Operações que aceitam duração temporária com restauração do estado anterior.
pub fn temp_ok(op: &str) -> bool {
 matches!(op, "mute" | "unmute" | "volume" | "show" | "hide")
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
 pub enabled: bool,
 pub host: String,
 pub port: u16,
}
impl Config {
 fn addr(&self) -> (String, u16) {
  let host = self.host.trim();
  (if host.is_empty() { "127.0.0.1".into() } else { host.into() }, if self.port == 0 { DEFAULT_PORT } else { self.port })
 }
}
pub fn config(rt: &Runtime, p: &str) -> Config {
 let mut c: Config = serde_json::from_value(rt.db.module(p, MODULE)).unwrap_or_default();
 if c.port == 0 {
  c.port = DEFAULT_PORT;
 }
 c
}
/// Trava de recurso temporário com contagem: o estado anterior só volta
/// quando o último efeito ativo expirar. Mesma trava, mesmo desejo soma;
/// desejo diferente com trava ativa é recusado com aviso.
#[derive(Default)]
pub struct Locks(Mutex<HashMap<String, Lock>>);
struct Lock {
 desired: String,
 count: u32,
}
/// Autenticação oficial do protocolo 5.x: base64(sha256(base64(sha256(senha+salt))+challenge)).
pub fn auth_string(password: &str, salt: &str, challenge: &str) -> String {
 use sha2::{Digest, Sha256};
 let secret = base64::Engine::encode(
  &base64::engine::general_purpose::STANDARD,
  Sha256::digest(format!("{password}{salt}").as_bytes()),
 );
 base64::Engine::encode(&base64::engine::general_purpose::STANDARD, Sha256::digest(format!("{secret}{challenge}").as_bytes()))
}
type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
struct Conn {
 ws: Ws,
 seq: u64,
}
async fn next_text(ws: &mut Ws) -> Result<String, String> {
 loop {
  let msg = tokio::time::timeout(Duration::from_secs(10), ws.next())
   .await
   .map_err(|_| "OBS demorou a responder".to_string())?
   .ok_or("OBS fechou a conexão".to_string())?
   .map_err(|_| "Falha de leitura no OBS".to_string())?;
  match msg {
   tokio_tungstenite::tungstenite::Message::Text(t) => return Ok(t.to_string()),
   tokio_tungstenite::tungstenite::Message::Close(_) => return Err("OBS fechou a conexão".into()),
   _ => continue,
  }
 }
}
async fn call(conn: &mut Conn, request_type: &str, data: Value) -> Result<Value, String> {
 conn.seq += 1;
 let id = format!("botlive-{}", conn.seq);
 let msg = json!({"op": 6, "d": {"requestType": request_type, "requestId": id, "requestData": data}}).to_string();
 conn.ws
  .send(tokio_tungstenite::tungstenite::Message::Text(msg.into()))
  .await
  .map_err(|_| "Falha ao enviar pedido ao OBS".to_string())?;
 loop {
  let text = next_text(&mut conn.ws).await?;
  let v: Value = serde_json::from_str(&text).map_err(|_| "Resposta inválida do OBS".to_string())?;
  if v["op"].as_i64() != Some(7) || v["d"]["requestId"].as_str() != Some(id.as_str()) {
   continue;
  }
  if v["d"]["requestStatus"]["result"] == true {
   return Ok(v["d"]["responseData"].clone());
  }
  return Err(v["d"]["requestStatus"]["comment"].as_str().unwrap_or("OBS recusou o pedido").into());
 }
}
async fn connect(rt: &Arc<Runtime>, p: &Profile) -> Result<Conn, String> {
 let c = config(rt, &p.id);
 if !c.enabled {
  return Err("Ative a integração OBS na tela OBS Studio".into());
 }
 let (host, port) = c.addr();
 let url = format!("ws://{host}:{port}");
 let (mut ws, _) = tokio::time::timeout(Duration::from_secs(10), tokio_tungstenite::connect_async(&url))
  .await
  .map_err(|_| format!("Não foi possível alcançar o OBS em {url}. Abra o OBS e ative o WebSocket do OBS em Ferramentas."))?
  .map_err(|_| "Falha ao conectar no OBS".to_string())?;
 let hello: Value = serde_json::from_str(&next_text(&mut ws).await?).map_err(|_| "Resposta inválida do OBS".to_string())?;
 let mut identify = json!({"op": 1, "d": {"rpcVersion": 1, "eventSubscriptions": 0}});
 if let (Some(challenge), Some(salt)) = (
  hello["d"]["authentication"]["challenge"].as_str(),
  hello["d"]["authentication"]["salt"].as_str(),
 ) {
  let password = crate::secrets::get(&p.id, "obs_password").map_err(|_| "O OBS pede senha: salve a senha na tela OBS Studio".to_string())?;
  identify["d"]["authentication"] = json!(auth_string(&password, salt, challenge));
 }
 ws.send(tokio_tungstenite::tungstenite::Message::Text(identify.to_string().into()))
  .await
  .map_err(|_| "Falha ao identificar no OBS".to_string())?;
 let identified: Value = serde_json::from_str(&next_text(&mut ws).await?).map_err(|_| "Resposta inválida do OBS".to_string())?;
 if identified["op"].as_i64() != Some(2) {
  return Err("O OBS recusou a identificação. Confira a senha.".into());
 }
 Ok(Conn { ws, seq: 0 })
}
async fn close(mut conn: Conn) {
 let _ = conn.ws.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
}
/// Cena atual do programa.
async fn program_scene(conn: &mut Conn) -> Result<String, String> {
 let v = call(conn, "GetSceneList", json!({})).await?;
 v["currentProgramSceneName"].as_str().map(str::to_owned).ok_or("OBS não informou a cena atual".into())
}
/// Itens da cena com nome e id, para achar fonte pelo nome.
async fn scene_items(conn: &mut Conn, scene: &str) -> Result<Vec<(String, i64)>, String> {
 let v = call(conn, "GetSceneItemList", json!({"sceneName": scene})).await?;
 Ok(v["sceneItems"]
  .as_array()
  .cloned()
  .unwrap_or_default()
  .into_iter()
  .filter_map(|i| Some((i["sourceName"].as_str()?.to_owned(), i["sceneItemId"].as_i64()?)))
  .collect())
}
async fn find_item(conn: &mut Conn, scene: &str, source: &str) -> Result<i64, String> {
 scene_items(conn, scene)
  .await?
  .into_iter()
  .find(|(name, _)| name.eq_ignore_ascii_case(source))
  .map(|(_, id)| id)
  .ok_or_else(|| format!("Fonte {source} não encontrada na cena {scene}"))
}
fn lock_key(profile: &str, resource: &str) -> String {
 format!("{profile}:{resource}")
}
/// Aplica a operação (efeito imediato). Devolve texto para o Histórico.
async fn apply(conn: &mut Conn, op: &str, target: &str, num: f64) -> Result<String, String> {
 match op {
  "mute" => {
   call(conn, "SetInputMute", json!({"inputName": target, "inputMuted": true})).await?;
   Ok(format!("{} mutado", target))
  }
  "unmute" => {
   call(conn, "SetInputMute", json!({"inputName": target, "inputMuted": false})).await?;
   Ok(format!("{} desmutado", target))
  }
  "toggle_mute" => {
   call(conn, "ToggleInputMute", json!({"inputName": target})).await?;
   Ok(format!("mudo de {} alternado", target))
  }
  "volume" => {
   call(conn, "SetInputVolume", json!({"inputName": target, "inputVolumeDb": num})).await?;
   Ok(format!("volume de {} em {} dB", target, num))
  }
  "show" | "hide" | "toggle_item" => {
   let scene = program_scene(conn).await?;
   let id = find_item(conn, &scene, target).await?;
   if op == "toggle_item" {
    let cur = current_enabled(conn, &scene, id).await?;
    call(conn, "SetSceneItemEnabled", json!({"sceneName": scene, "sceneItemId": id, "sceneItemEnabled": !cur})).await?;
    Ok(format!("fonte {} alternada", target))
   } else {
    call(
     conn,
     "SetSceneItemEnabled",
     json!({"sceneName": scene, "sceneItemId": id, "sceneItemEnabled": op == "show"}),
    )
    .await?;
    Ok(format!("fonte {} {}", target, if op == "show" { "visível" } else { "oculta" }))
   }
  }
  "scene" => {
   call(conn, "SetCurrentProgramScene", json!({"sceneName": target})).await?;
   Ok(format!("cena {}", target))
  }
  _ => Err("Ação do OBS desconhecida".into()),
 }
}
async fn current_enabled(conn: &mut Conn, scene: &str, id: i64) -> Result<bool, String> {
 let v = call(conn, "GetSceneItemEnabled", json!({"sceneName": scene, "sceneItemId": id})).await?;
 Ok(v["sceneItemEnabled"].as_bool().unwrap_or(true))
}
/// Lê o estado anterior para restaurar depois (só operações temporárias).
async fn previous(conn: &mut Conn, op: &str, target: &str) -> Result<String, String> {
 match op {
  "mute" | "unmute" => {
   let v = call(conn, "GetInputMute", json!({"inputName": target})).await?;
   Ok(if v["inputMuted"] == true { "muted".into() } else { "unmuted".into() })
  }
  "volume" => {
   let v = call(conn, "GetInputVolume", json!({"inputName": target})).await?;
   Ok(v["inputVolumeDb"].as_f64().unwrap_or(0.0).to_string())
  }
  "show" | "hide" => {
   let scene = program_scene(conn).await?;
   let id = find_item(conn, &scene, target).await?;
   Ok(if current_enabled(conn, &scene, id).await? { "shown".into() } else { "hidden".into() })
  }
  _ => Err("Operação sem modo temporário".into()),
 }
}
/// Restaura o estado guardado (inverso do apply, pelo valor anterior).
async fn restore(conn: &mut Conn, op: &str, target: &str, prev: &str) -> Result<(), String> {
 match op {
  "mute" | "unmute" => {
   call(conn, "SetInputMute", json!({"inputName": target, "inputMuted": prev == "muted"})).await?;
  }
  "volume" => {
   let db: f64 = prev.parse().unwrap_or(0.0);
   call(conn, "SetInputVolume", json!({"inputName": target, "inputVolumeDb": db})).await?;
  }
  "show" | "hide" => {
   let scene = program_scene(conn).await?;
   let id = find_item(conn, &scene, target).await?;
   call(conn, "SetSceneItemEnabled", json!({"sceneName": scene, "sceneItemId": id, "sceneItemEnabled": prev == "shown"})).await?;
  }
  _ => {}
 }
 Ok(())
}
fn resource_of(op: &str, target: &str) -> String {
 match op {
  "mute" | "unmute" | "toggle_mute" | "volume" => format!("input:{target}"),
  "show" | "hide" | "toggle_item" => format!("item:{target}"),
  "scene" => "scene".to_string(),
  _ => format!("other:{target}"),
 }
}
/// Executa a ação do fluxo (efeito imediato ou temporário com restauração).
/// Temporário nunca bloqueia: aplica, agenda a restauração e devolve na hora.
pub async fn execute_action(rt: &Arc<Runtime>, p: &Profile, op: &str, target: &str, num: f64, secs: u64) -> Result<String, String> {
 if !OPS.contains(&op) {
  return Err("Ação do OBS desconhecida".into());
 }
 if target.trim().is_empty() {
  return Err("Escolha o alvo da ação do OBS".into());
 }
 if secs > 0 && !temp_ok(op) {
  return Err("Duração temporária vale para mutar, volume, mostrar e esconder".into());
 }
 let resource = resource_of(op, target);
 let desired = format!("{op}:{num}");
 if secs > 0 {
  let key = lock_key(&p.id, &resource);
  {
   let mut locks = rt.obs.0.lock().unwrap();
   if let Some(lock) = locks.get_mut(&key) {
    if lock.desired != desired {
     return Err(format!("Recurso ocupado por outro efeito; aguarde liberar"));
    }
    lock.count += 1;
   } else {
    locks.insert(key.clone(), Lock { desired: desired.clone(), count: 1 });
   }
  }
  let mut conn = connect(rt, p).await.map_err(|e| {
   rt.obs.0.lock().unwrap().remove(&key);
   e
  })?;
  let prev = previous(&mut conn, op, target).await.map_err(|e| {
   rt.obs.0.lock().unwrap().remove(&key);
   e
  })?;
  let done = apply(&mut conn, op, target, num).await.map_err(|e| {
   rt.obs.0.lock().unwrap().remove(&key);
   e
  })?;
  close(conn).await;
  rt.log(&p.id, "obs", &format!("{done} por {secs}s"), "success");
  let rt2 = rt.clone();
  let p2 = p.clone();
  let (op2, target2, prev2) = (op.to_owned(), target.to_owned(), prev);
  tokio::spawn(async move {
   tokio::time::sleep(Duration::from_secs(secs)).await;
   let release = {
    let mut locks = rt2.obs.0.lock().unwrap();
    match locks.get_mut(&key) {
     Some(lock) => {
      lock.count = lock.count.saturating_sub(1);
      lock.count == 0
     }
     None => false,
    }
   };
   if !release {
    return;
   }
   match connect(&rt2, &p2).await {
    Ok(mut conn) => {
     if let Err(err) = restore(&mut conn, &op2, &target2, &prev2).await {
      rt2.log(&p2.id, "obs", &err, "error");
     } else {
      rt2.log(&p2.id, "obs", &format!("{target2} restaurado"), "success");
     }
     close(conn).await;
    }
    Err(err) => rt2.log(&p2.id, "obs", &err, "error"),
   }
   rt2.obs.0.lock().unwrap().remove(&key);
  });
  Ok(format!("{done} por {secs}s"))
 }
 let mut conn = connect(rt, p).await?;
 let done = apply(&mut conn, op, target, num).await?;
 close(conn).await;
 Ok(done)
}
/// Descoberta para os dropdowns: cenas, entradas de áudio e fontes da cena atual.
pub async fn discover(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 let mut conn = connect(rt, p).await?;
 let scenes = call(&mut conn, "GetSceneList", json!({})).await?;
 let current = scenes["currentProgramSceneName"].as_str().unwrap_or("").to_owned();
 let scene_names: Vec<Value> = scenes["scenes"]
  .as_array()
  .cloned()
  .unwrap_or_default()
  .into_iter()
  .filter_map(|s| s["sceneName"].as_str().map(|n| json!(n)))
  .collect();
 let inputs = call(&mut conn, "GetInputList", json!({})).await?;
 let input_names: Vec<Value> = inputs["inputs"]
  .as_array()
  .cloned()
  .unwrap_or_default()
  .into_iter()
  .filter_map(|i| {
   json!({"name": i["inputName"], "kind": i["inputKind"]});
   i["inputName"].as_str().map(|n| json!(n))
  })
  .collect();
 let mut sources: Vec<Value> = vec![];
 if !current.is_empty() {
  for (name, _) in scene_items(&mut conn, &current).await.unwrap_or_default() {
   sources.push(json!(name));
  }
 }
 close(conn).await;
 Ok(json!({"scenes": scene_names, "current": current, "inputs": input_names, "sources": sources}))
}
pub async fn status(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 let mut conn = connect(rt, p).await?;
 let v = call(&mut conn, "GetVersion", json!({})).await?;
 let scene = program_scene(&mut conn).await.unwrap_or_default();
 close(conn).await;
 Ok(json!({
  "online": true,
  "obsVersion": v["obsVersion"],
  "wsVersion": v["obsWebSocketVersion"],
  "scene": scene,
 }))
}
fn valid_save(v: &Value) -> Result<Config, String> {
 let host = v["host"].as_str().unwrap_or("").trim();
 if host.chars().count() > 253 {
  return Err("Endereço do OBS inválido".into());
 }
 if !host.is_empty() && !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') {
  return Err("Endereço do OBS inválido: use IP ou nome da máquina".into());
 }
 let port = v["port"].as_u64().unwrap_or(DEFAULT_PORT as u64);
 if port < 1 || port > 65535 {
  return Err("Porta do OBS entre 1 e 65535".into());
 }
 Ok(Config { enabled: v["enabled"] == true, host: host.into(), port: port as u16 })
}
pub async fn operation(rt: &Arc<Runtime>, p: &str, op: &str, args: &Value) -> Result<Value, String> {
 rt.db.profile(p)?;
 let profile = rt.db.profile(p)?;
 if profile.platform != "twitch" && profile.platform != "youtube" && profile.platform != "kick" {
  return Err("Perfil inválido".into());
 }
 match op {
  "obs.get" => Ok(json!({"config": serde_json::to_value(config(rt, p)).map_err(|e| e.to_string())?, "hasPassword": crate::secrets::get(p, "obs_password").is_ok()})),
  "obs.save" => {
   let c = valid_save(&args["config"])?;
   rt.db.set_module(p, MODULE, &serde_json::to_value(&c).map_err(|e| e.to_string())?)?;
   rt.log(p, "obs", "Configuração do OBS salva.", "success");
   Ok(Value::Null)
  },
  "obs.password" => {
   let pw = args["value"].as_str().unwrap_or("");
   if pw.chars().count() > 200 {
    return Err("Senha muito longa".into());
   }
   crate::secrets::set(p, "obs_password", pw)?;
   Ok(Value::Null)
  }
  "obs.status" => status(rt, &profile).await,
  "obs.discover" => discover(rt, &profile).await,
  "obs.test" => {
   let op = args["op"].as_str().unwrap_or("");
   let target = args["target"].as_str().unwrap_or("");
   execute_action(rt, &profile, op, target, 0.0, 0).await.map(|done| json!(done))
  }
  "obs.execute" => {
   let op = args["op"].as_str().unwrap_or("");
   let target = args["target"].as_str().unwrap_or("");
   let num = args["num"].as_f64().unwrap_or(0.0);
   let secs = args["secs"].as_u64().unwrap_or(0);
   execute_action(rt, &profile, op, target, num, secs).await.map(|done| json!(done))
  }
  _ => Err("Operação do OBS desconhecida".into()),
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn auth_matches_the_official_vector_shape() {
  let a = auth_string("supersecretpassword", "saltsalt1234", "challenge5678");
  assert_eq!(a.len(), 44, "base64 de 32 bytes tem 44 caracteres");
  assert_ne!(a, auth_string("outra", "saltsalt1234", "challenge5678"));
  assert_ne!(a, auth_string("supersecretpassword", "outro", "challenge5678"));
  assert_eq!(a, auth_string("supersecretpassword", "saltsalt1234", "challenge5678"), "determinístico");
 }
 #[test]
 fn only_safe_ops_exist_and_temp_is_limited() {
  for op in OPS {
   assert!(op_name(op) != "Desconhecida");
  }
  assert_eq!(OPS.len(), 8);
  for op in ["mute", "unmute", "volume", "show", "hide"] {
   assert!(temp_ok(op), "{op} aceita duração");
  }
  for op in ["toggle_mute", "toggle_item", "scene"] {
   assert!(!temp_ok(op), "{op} é momentânea");
  }
 }
 #[test]
 fn config_defaults_and_rejects_junk() {
  let c = valid_save(&json!({})).unwrap();
  assert!(!c.enabled && c.port == DEFAULT_PORT);
  assert!(valid_save(&json!({"host": "a b"})).is_err());
  assert!(valid_save(&json!({"port": 0})).is_err());
  assert!(valid_save(&json!({"port": 70000})).is_err());
  assert!(valid_save(&json!({"enabled": true, "host": "192.168.0.10", "port": 4455})).is_ok());
 }
}
