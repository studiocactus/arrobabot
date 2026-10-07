//! Voicemod Control API — a voz do bot sai pelo microfone virtual do Voicemod.
//!
//! Escopo escolhido: **uma sessão por máquina, não por perfil.** O Voicemod é um
//! dispositivo local compartilhado; dois perfis conectando ao mesmo tempo lutariam
//! pela mesma porta. A chave usada é a do perfil que mandou conectar.
//!
//! Só 127.0.0.1 e as portas publicadas na referência oficial, caminho `/v1`.
//! Nenhuma varredura de rede. A chave fica no cofre do sistema e nunca aparece em
//! log, resposta, preset ou exportação.
//!
//! Conectado só depois do `registerClient` respondido com 200: um socket aberto
//! não prova autorização. Sucesso de troca de voz só quando `getCurrentVoice`
//! devolve o id pedido, nunca porque a mensagem saiu pela ponta.
use crate::{engine::Runtime, model::Profile};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
 collections::HashMap,
 sync::{
  atomic::{AtomicBool, AtomicU64, Ordering},
  Arc, Mutex,
 },
 time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot};

/// Portas publicadas em <https://control-api.voicemod.net/api-reference/> (servidor `voicemod`).
pub const PORTS: &[u16] = &[59129, 20000, 39273, 42152, 43782, 46667, 35679, 37170, 38501, 33952, 30546];
/// Caminho publicado para o WebSocket do aplicativo Voicemod.
pub const WSPATH: &str = "/v1";
const KIND: &str = "voicemod";
/// Janela curta por porta: conexão recusada em 127.0.0.1 falha em milissegundos.
const PORT_TIMEOUT: Duration = Duration::from_millis(400);
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_TEST_SECS: u64 = 10;
const MIN_TEST_SECS: u64 = 1;
const MAX_TEST_SECS: u64 = 60;
const RECONNECT_DELAYS: [u64; 4] = [2, 4, 8, 16];
const APPLY_ATTEMPTS: u32 = 6;
const APPLY_DELAY: Duration = Duration::from_millis(350);

pub const OFF: &str = "disconnected";
pub const SEARCHING: &str = "searching";
pub const AUTHORIZING: &str = "authorizing";
pub const CONNECTED: &str = "connected";
pub const FAILED: &str = "failed";

const NOT_FOUND: &str = "Voicemod não encontrado nesta máquina. Abra o aplicativo Voicemod e clique em Conectar.";
const NO_KEY: &str = "Chave da Control API ausente. Peça sua chave em control-api.voicemod.net e salve em Voicemod → Chave da API.";
const KEY_REFUSED: &str = "O Voicemod recusou a chave (401 unauthorized). Confira a chave salva.";
const BAD_PROTOCOL: &str = "Uma porta respondeu, mas não é o Voicemod (protocolo fora do esperado). Versão do Voicemod incompatível com esta integração.";
const LOST: &str = "A conexão com o Voicemod foi interrompida.";
const NOT_CONNECTED: &str = "Conecte o Voicemod antes de pedir isto.";

// ---------------------------------------------------------------- estado

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
 pub phase: String,
 pub message: String,
 pub port: u16,
 pub voices: Vec<Value>,
 pub current_voice: String,
 pub current_name: String,
 pub voice_changer: Option<bool>,
 pub hear_myself: Option<bool>,
 pub license: String,
 pub attempts: u32,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestView {
 pub active: bool,
 pub phase: String,
 pub voice_id: String,
 pub voice_name: String,
 pub seconds: u64,
 pub remaining_ms: u64,
 pub interrupted: bool,
 pub manual: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
 pub kind: String,
 pub detail: String,
}

#[derive(Clone)]
struct Test {
 voice_id: String,
 voice_name: String,
 prev_voice: String,
 prev_name: String,
 prev_changer: Option<bool>,
 changed_changer: bool,
 manual: bool,
 interrupted: bool,
 seconds: u64,
 ends_at: Instant,
}

struct State {
 status: Status,
 link: Option<Session>,
 /// Teste em curso (relógio rodando).
 test: Option<Test>,
 /// Teste concluído mas com restauração pendente: só sai quando o usuário pedir.
 pending: Option<Test>,
 outcome: Outcome,
 profile: String,
}

impl Default for State {
 fn default() -> Self {
  Self {
   status: Status {
    phase: OFF.into(),
    message: "Abra o Voicemod nesta máquina e clique em Conectar.".into(),
    ..Status::default()
   },
   link: None,
   test: None,
   pending: None,
   outcome: Outcome::default(),
   profile: String::new(),
  }
 }
}

/// Uma sessão: um leitor, um emissor, uma geração. Nunca duas ao mesmo tempo.
struct Session {
 link: Link,
 /// Mantido junto com a sessão; soltar ele desanexa o fio em vez de encerrá-lo.
 _task: tokio::task::JoinHandle<()>,
}

/// Metade compartilhável da conexão, clonada a cada pedido.
#[derive(Clone)]
struct Link {
 tx: mpsc::UnboundedSender<Out>,
 closed: Arc<AtomicBool>,
 pending: Arc<Mutex<HashMap<String, Pending>>>,
 gen: u64,
}

struct Pending {
 expect: &'static [&'static str],
 reply: oneshot::Sender<Result<Value, String>>,
}

enum Out {
 Send(String),
 Close,
}

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Estado global do Voicemod nesta máquina.
pub struct Voice {
 state: Mutex<State>,
 call_lock: tokio::sync::Mutex<()>,
 ports: Mutex<Vec<u16>>,
 delays: Mutex<Vec<u64>>,
 gen: AtomicU64,
 busy: AtomicBool,
 reconnect: Mutex<Option<tokio::task::JoinHandle<()>>>,
 #[cfg(test)]
 test_keys: Mutex<HashMap<String, String>>,
 #[cfg(test)]
 reply_timeout: Mutex<Duration>,
}

impl Voice {
 fn snapshot(&self) -> (Status, TestView, Outcome) {
  let st = self.state.lock().unwrap();
  (st.status.clone(), view(&st), st.outcome.clone())
 }
 fn publish(&self, rt: &Runtime) {
  let (s, t, o) = self.snapshot();
  rt.emit(KIND, json!({"status": s, "test": t, "outcome": o}));
 }
 fn profile(&self) -> String {
  self.state.lock().unwrap().profile.clone()
 }
 #[cfg(test)]
 pub fn set_ports(&self, ports: Vec<u16>) {
  *self.ports.lock().unwrap() = ports;
 }
 #[cfg(test)]
 pub fn set_delays(&self, delays: Vec<u64>) {
  *self.delays.lock().unwrap() = delays;
 }
 #[cfg(test)]
 pub fn set_key(&self, key: Option<String>) {
  let mut keys = self.test_keys.lock().unwrap();
  keys.clear();
  if let Some(k) = key {
   keys.insert(String::new(), k);
  }
 }
 #[cfg(test)]
 pub fn set_reply_timeout(&self, ms: u64) {
  *self.reply_timeout.lock().unwrap() = Duration::from_millis(ms);
 }
 fn reply_timeout(&self) -> Duration {
  #[cfg(test)]
  {
   return *self.reply_timeout.lock().unwrap();
  }
  #[allow(unreachable_code)]
  REPLY_TIMEOUT
 }
}

impl Default for Voice {
 fn default() -> Self {
  Self {
   state: Mutex::new(State::default()),
   call_lock: tokio::sync::Mutex::new(()),
   ports: Mutex::new(PORTS.to_vec()),
   delays: Mutex::new(RECONNECT_DELAYS.to_vec()),
   gen: AtomicU64::new(0),
   busy: AtomicBool::new(false),
   reconnect: Mutex::new(None),
   #[cfg(test)]
   test_keys: Mutex::new(HashMap::new()),
   #[cfg(test)]
   reply_timeout: Mutex::new(REPLY_TIMEOUT),
  }
 }
}

impl Test {
 fn placeholder() -> Self {
  Self {
   voice_id: String::new(),
   voice_name: String::new(),
   prev_voice: String::new(),
   prev_name: String::new(),
   prev_changer: None,
   changed_changer: false,
   manual: false,
   interrupted: false,
   seconds: 0,
   ends_at: Instant::now(),
  }
 }
}

fn view(st: &State) -> TestView {
 match st.test.as_ref() {
  None => TestView::default(),
  Some(t) => TestView {
   active: true,
   phase: "running".into(),
   voice_id: t.voice_id.clone(),
   voice_name: t.voice_name.clone(),
   seconds: t.seconds,
   remaining_ms: t.ends_at.saturating_duration_since(Instant::now()).as_millis() as u64,
   interrupted: t.interrupted,
   manual: t.manual,
  },
 }
}

fn name_of(voices: &[Value], id: &str) -> String {
 voices
  .iter()
  .find(|v| v["id"].as_str() == Some(id))
  .and_then(|v| v["friendlyName"].as_str())
  .unwrap_or(id)
  .to_owned()
}

// ---------------------------------------------------------------- leitura da API

fn read_voices(v: &Value) -> (Vec<Value>, Option<String>) {
 let arr = v["actionObject"]["voices"]
  .as_array()
  .or_else(|| v["voices"].as_array())
  .cloned()
  .unwrap_or_default();
 // A referência descreve `currentVoice` dentro de `actionObject`, mas o exemplo gerado
 // traz no raiz. Aceita as duas formas em vez de assumir uma só.
 let cur = v["actionObject"]["currentVoice"]
  .as_str()
  .or_else(|| v["currentVoice"].as_str())
  .map(str::to_owned);
 (arr, cur)
}

fn read_bool(v: &Value) -> Option<bool> {
 v["actionObject"]["value"].as_bool()
}

const EV_CHANGER: &[&str] = &["toggleVoiceChanger", "voiceChangerEnabledEvent", "voiceChangerDisabledEvent"];
const EV_HEAR: &[&str] = &["toggleHearMyVoice", "getHearMyselfStatus"];

// ---------------------------------------------------------------- estado publicado

fn set_phase(rt: &Runtime, phase: &str, message: &str) {
 {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.status.phase = phase.into();
  st.status.message = message.into();
 }
 rt.voicemod.publish(rt);
}

fn set_outcome(rt: &Runtime, outcome: Outcome, log: &str) {
 {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.outcome = outcome;
 }
 rt.voicemod.publish(rt);
 let profile = rt.voicemod.profile();
 if !profile.is_empty() && !log.is_empty() {
  rt.log(&profile, KIND, log, "error");
 }
}

fn snapshot_json(rt: &Runtime) -> Value {
 let (s, t, o) = rt.voicemod.snapshot();
 json!({
  "status": s,
  "test": t,
  "outcome": o,
  "defaultSecs": DEFAULT_TEST_SECS,
  "minSecs": MIN_TEST_SECS,
  "maxSecs": MAX_TEST_SECS
 })
}

/// Ponto único de leitura da credencial. Em teste usa o cofre falso da sessão,
/// para nenhum teste tocar no cofre do sistema.
fn read_key(_rt: &Runtime, profile: &str) -> Result<String, String> {
 #[cfg(test)]
 {
  let keys = _rt.voicemod.test_keys.lock().unwrap();
  return keys
   .get(profile)
   .or_else(|| keys.get(""))
   .cloned()
   .ok_or_else(|| NO_KEY.to_string());
 }
 #[allow(unreachable_code)]
 {
  if profile.is_empty() {
   return Err(NO_KEY.to_string());
  }
  crate::secrets::get(profile, "vm_key").map_err(|_| NO_KEY.to_string())
 }
}

/// Ponto único de gravação: em teste grava no cofre falso da sessão.
fn write_key(_rt: &Runtime, profile: &str, value: &str) -> Result<(), String> {
 #[cfg(test)]
 {
  let mut keys = _rt.voicemod.test_keys.lock().unwrap();
  if value.is_empty() {
   keys.remove(profile);
  } else {
   keys.insert(profile.to_string(), value.to_string());
  }
  return Ok(());
 }
 #[allow(unreachable_code)]
 { crate::secrets::set(profile, "vm_key", value) }
}

/// A chave pertence ao perfil que fez a chamada, não à sessão aberta. O estado
/// do cofre é montado por `operation`, o único lugar que sabe qual perfil é.
fn stored_key(rt: &Runtime, profile: &str) -> Option<String> {
 if profile.is_empty() {
  None
 } else {
  read_key(rt, profile).ok()
 }
}

// ---------------------------------------------------------------- fio de leitura

fn spawn_link(rt: Arc<Runtime>, gen: u64, ws: Ws) -> (Link, tokio::task::JoinHandle<()>) {
 let (tx, rx) = mpsc::unbounded_channel();
 let closed = Arc::new(AtomicBool::new(false));
 let pending: Arc<Mutex<HashMap<String, Pending>>> = Arc::new(Mutex::new(HashMap::new()));
 let link = Link { tx, closed: closed.clone(), pending: pending.clone(), gen };
 let task = tokio::spawn(reader(rt, gen, ws, rx, pending, closed));
 (link, task)
}

/// Correlaciona a resposta: id ecoado com ação esperada, depois ação esperada,
/// e por último qualquer id — o protocolo documenta `actionID` nulo em alguns casos.
/// Com pedidos serializados, uma ação esperada nunca é ambígua.
fn settle(pending: &Mutex<HashMap<String, Pending>>, v: &Value) -> bool {
 let action = v["action"].as_str().or_else(|| v["actionType"].as_str()).unwrap_or("");
 let id = v["id"].as_str().or_else(|| v["actionId"].as_str()).or_else(|| v["actionID"].as_str());
 let mut map = pending.lock().unwrap();
 if let Some(id) = id {
  let expected = map.get(id).is_some_and(|p| p.expect.contains(&action));
  if expected {
   if let Some(p) = map.remove(id) {
    let _ = p.reply.send(Ok(v.clone()));
    return true;
   }
  }
 }
 let key = map.iter().find(|(_, p)| p.expect.contains(&action)).map(|(k, _)| k.clone());
 if let Some(k) = key {
  if let Some(p) = map.remove(&k) {
   let _ = p.reply.send(Ok(v.clone()));
   return true;
  }
 }
 if let Some(id) = id {
  if let Some(p) = map.remove(id) {
   let _ = p.reply.send(Ok(v.clone()));
   return true;
  }
 }
 false
}

async fn reader(rt: Arc<Runtime>, gen: u64, ws: Ws, rx: mpsc::UnboundedReceiver<Out>, pending: Arc<Mutex<HashMap<String, Pending>>>, closed: Arc<AtomicBool>) {
 let (mut sink, mut stream) = ws.split();
 let writer = tokio::spawn(async move {
  let mut rx = rx;
  while let Some(out) = rx.recv().await {
   match out {
    Out::Close => {
     let _ = sink.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
     let _ = sink.close().await;
     break;
    }
    Out::Send(body) => {
     if sink.send(tokio_tungstenite::tungstenite::Message::Text(body.into())).await.is_err() {
      break;
     }
    }
   }
  }
 });
 loop {
  match stream.next().await {
   None | Some(Err(_)) => break,
   Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => break,
   Some(Ok(tokio_tungstenite::tungstenite::Message::Text(t))) => {
    if let Ok(v) = serde_json::from_str::<Value>(&t) {
     if !settle(&pending, &v) {
      event(&rt, v);
     }
    }
   },
   Some(Ok(_)) => {}
  }
 }
 writer.abort();
 closed.store(true, Ordering::SeqCst);
 for (_, p) in pending.lock().unwrap().drain() {
  let _ = p.reply.send(Err(LOST.into()));
 }
 drop_link(&rt, gen).await;
}

/// Chamado só pelo fio de leitura: nunca sobrescreve uma sessão nova.
async fn drop_link(rt: &Arc<Runtime>, gen: u64) {
 let profile = {
  let mut st = rt.voicemod.state.lock().unwrap();
  if st.link.as_ref().map(|l| l.link.gen) != Some(gen) {
   return;
  }
  st.link = None;
  if st.status.phase != CONNECTED {
   return;
  }
  if let Some(t) = st.test.as_mut() {
   t.interrupted = true;
  }
  st.status.phase = FAILED.into();
  st.status.message = "A conexão com o Voicemod foi interrompida. Reconectando…".into();
  st.profile.clone()
 };
 if profile.is_empty() {
  return;
 }
 rt.voicemod.publish(rt);
 rt.log(&profile, KIND, &format!("{LOST} A restauração só acontece quando você pedir."), "error");
 schedule_reconnect(rt.clone(), profile);
}

/// Reconexão espaçada e limitada. Nunca reenvia a voz do teste.
fn schedule_reconnect(rt: Arc<Runtime>, profile: String) {
 // O guarda fica preso só enquanto o fio novo ainda não roda: `spawn` agenda e a
 // tarefa atual continua até o fim desta função, então não há corrida no candado.
 let voice = rt.clone();
 let mut slot = voice.voicemod.reconnect.lock().unwrap();
 if slot.is_some() {
  return;
 }
 let handle = tokio::spawn(async move {
  let delays = rt.voicemod.delays.lock().unwrap().clone();
  let empty = delays.is_empty();
  let mut last = String::new();
  for (i, secs) in delays.iter().enumerate() {
   if *secs > 0 {
    tokio::time::sleep(Duration::from_secs(*secs)).await;
   }
   {
    let st = rt.voicemod.state.lock().unwrap();
    if st.link.is_some() || st.status.phase == OFF {
     *rt.voicemod.reconnect.lock().unwrap() = None;
     return;
    }
   }
   if rt.voicemod.busy.swap(true, Ordering::SeqCst) {
    continue;
   }
   let result = match rt.db.profile(&profile) {
    Ok(p) => open_and_install(&rt, &p, true).await,
    Err(e) => Err(e),
   };
   rt.voicemod.busy.store(false, Ordering::SeqCst);
   match result {
    Ok(()) => {
     *rt.voicemod.reconnect.lock().unwrap() = None;
     return;
    },
    Err(e) => {
     last = e;
     set_phase(&rt, FAILED, &format!("Reconexão {} de {}: {}", i + 1, delays.len().max(1), last));
    },
   }
  }
  if !empty {
   set_phase(&rt, FAILED, &format!("Não foi possível reconectar ao Voicemod ({} tentativas). Clique em Conectar.", delays.len()));
  }
  *rt.voicemod.reconnect.lock().unwrap() = None;
  if !last.is_empty() {
   rt.log(&profile, KIND, &format!("Reconexão ao Voicemod desistiu: {last}"), "error");
  }
 });
 *slot = Some(handle);
}

// ---------------------------------------------------------------- pedidos

fn current_link(rt: &Runtime) -> Result<Link, String> {
 let st = rt.voicemod.state.lock().unwrap();
 let link = st.link.as_ref().ok_or(NOT_CONNECTED)?;
 if link.link.closed.load(Ordering::SeqCst) {
  return Err(LOST.into());
 }
 Ok(link.link.clone())
}

async fn request(rt: &Arc<Runtime>, link: &Link, action: &str, expect: &'static [&'static str], body: Value) -> Result<Value, String> {
 let _gate = rt.voicemod.call_lock.lock().await;
 if link.closed.load(Ordering::SeqCst) {
  return Err(LOST.into());
 }
 let id = uuid::Uuid::new_v4().to_string();
 let text = json!({"id": id, "action": action, "payload": body}).to_string();
 let (tx, rx) = oneshot::channel();
 link.pending.lock().unwrap().insert(id.clone(), Pending { expect, reply: tx });
 if link.tx.send(Out::Send(text)).is_err() {
  link.pending.lock().unwrap().remove(&id);
  return Err(LOST.into());
 }
 match tokio::time::timeout(rt.voicemod.reply_timeout(), rx).await {
  Ok(Ok(v)) => v,
  Ok(Err(_)) => Err(LOST.into()),
  Err(_) => {
   link.pending.lock().unwrap().remove(&id);
   Err(format!("O Voicemod não respondeu a {action}."))
  },
 }
}

async fn call(rt: &Arc<Runtime>, action: &str, expect: &'static [&'static str]) -> Result<Value, String> {
 let link = current_link(rt)?;
 request(rt, &link, action, expect, json!({})).await
}

/// Envia sem esperar resposta: `loadVoice` só se confirma pelo estado depois.
async fn fire(rt: &Arc<Runtime>, action: &str, body: Value) -> Result<(), String> {
 let _gate = rt.voicemod.call_lock.lock().await;
 let link = current_link(rt)?;
 let text = json!({"id": uuid::Uuid::new_v4().to_string(), "action": action, "payload": body}).to_string();
 link.tx.send(Out::Send(text)).map_err(|_| LOST.to_string())
}

// ---------------------------------------------------------------- conexão

async fn discover(rt: &Runtime) -> Result<(u16, Ws), String> {
 let ports = rt.voicemod.ports.lock().unwrap().clone();
 let mut listened = false;
 for port in ports {
  let tcp = tokio::time::timeout(PORT_TIMEOUT, tokio::net::TcpStream::connect(("127.0.0.1", port))).await;
  if !matches!(tcp, Ok(Ok(_))) {
   continue;
  }
  listened = true;
  let url = format!("ws://127.0.0.1:{port}{WSPATH}");
  if let Ok(Ok((ws, _))) = tokio::time::timeout(REPLY_TIMEOUT, tokio_tungstenite::connect_async(&url)).await {
   return Ok((port, ws));
  }
 }
 if listened {
  Err(BAD_PROTOCOL.into())
 } else {
  Err(NOT_FOUND.into())
 }
}

async fn open_and_install(rt: &Arc<Runtime>, p: &Profile, from_reconnect: bool) -> Result<(), String> {
 let key = read_key(rt, &p.id).map_err(|e| {
  set_phase(rt, FAILED, &e);
  e
 })?;
 set_phase(rt, SEARCHING, "Procurando o Voicemod nesta máquina…");
 let (port, ws) = discover(rt).await.map_err(|e| {
  set_phase(rt, FAILED, &e);
  e
 })?;
 set_phase(rt, AUTHORIZING, "Autorizando no Voicemod…");
 let gen = rt.voicemod.gen.fetch_add(1, Ordering::SeqCst);
 let (link, task) = spawn_link(rt.clone(), gen, ws);
 let key_len = key.chars().count();
 let body = json!({"id": uuid::Uuid::new_v4().to_string(), "action": "registerClient", "payload": {"clientKey": key}});
 let reply = request(rt, &link, "registerClient", &["registerClient"], body).await;
 let register = match reply {
  Ok(v) => v,
  Err(e) => {
   link.tx.send(Out::Close).ok();
   set_phase(rt, FAILED, &e);
   return Err(e);
  },
 };
 match register["payload"]["status"]["code"].as_i64() {
  Some(200) => {},
  Some(401) => {
   link.tx.send(Out::Close).ok();
   // O protocolo está certo (o Voicemod respondeu): só o valor da chave foi
   // recusado. Dizer isso sem mexer no cofre evita outra tentativa às cegas.
   let refused = format!(
    "{KEY_REFUSED} A chave guardada tem {key_len} caracteres: confira se é exatamente a recebida por e-mail, sem rótulo e sem aspas, e se o aplicativo Voicemod está aberto na mesma conta que pediu a chave. Ainda em dúvida, peça outra em control-api.voicemod.net/getting-started ou escreva para devservices@voicemod.net."
   );
   set_phase(rt, FAILED, &refused);
   return Err(refused);
  },
  _ => {
   link.tx.send(Out::Close).ok();
   set_phase(rt, FAILED, BAD_PROTOCOL);
   return Err(BAD_PROTOCOL.into());
  },
 }
 {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.status.phase = CONNECTED.into();
  st.status.port = port;
  st.status.attempts = 0;
  st.status.message = "Conectado e autorizado no Voicemod.".into();
  st.status.voices.clear();
  st.status.current_voice = String::new();
  st.status.current_name = String::new();
  st.status.voice_changer = None;
  st.status.hear_myself = None;
  st.status.license = String::new();
  st.link = Some(Session { link, _task: task });
  st.profile = p.id.clone();
 }
 rt.voicemod.publish(rt);
 if let Err(e) = refresh(rt).await {
  let session = {
   let mut st = rt.voicemod.state.lock().unwrap();
   st.status.phase = FAILED.into();
   st.status.message = e.clone();
   st.link.take()
  };
  if let Some(s) = session {
   s.link.tx.send(Out::Close).ok();
  }
  rt.voicemod.publish(rt);
  // Quem chamou registra no Histórico: `operation` ou o fio de reconexão.
  return Err(e);
 }
 rt.voicemod.publish(rt);
 let msg = if from_reconnect {
  format!("Reconectado ao Voicemod na porta {port}.")
 } else {
  format!("Conectado ao Voicemod na porta {port}.")
 };
 rt.log(&p.id, KIND, &msg, "success");
 Ok(())
}

// ---------------------------------------------------------------- leitura de estado

/// Lê tudo o que a tela precisa. `getVoices` e `getCurrentVoice` são obrigatórios;
/// licença e estados de áudio são o melhor esforço (a referência muda por versão).
async fn refresh(rt: &Arc<Runtime>) -> Result<(), String> {
 let v = call(rt, "getVoices", &["getVoices"]).await?;
 let (voices, cur) = read_voices(&v);
 let c = call(rt, "getCurrentVoice", &["getCurrentVoice"]).await?;
 let current = c["actionObject"]["voiceID"].as_str().unwrap_or("").to_owned();
 let current = if current.is_empty() { cur.unwrap_or_default() } else { current };
 let changer = call(rt, "getVoiceChangerStatus", EV_CHANGER).await.ok().and_then(|v| read_bool(&v));
 let hear = call(rt, "getHearMyselfStatus", EV_HEAR).await.ok().and_then(|v| read_bool(&v));
 let license = call(rt, "getUserLicense", &["getUserLicense"])
  .await
  .ok()
  .and_then(|v| v["actionObject"]["licenseType"].as_str().map(str::to_owned))
  .unwrap_or_default();
 let name = name_of(&voices, &current);
 {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.status.voices = voices;
  st.status.current_voice = current;
  st.status.current_name = name;
  st.status.voice_changer = changer;
  st.status.hear_myself = hear;
  st.status.license = license;
 }
 Ok(())
}

async fn refresh_now(rt: &Arc<Runtime>) -> Result<(), String> {
 refresh(rt).await?;
 rt.voicemod.publish(rt);
 Ok(())
}

// ---------------------------------------------------------------- troca de voz

/// Aplica a voz e só declara sucesso quando `getCurrentVoice` devolve o id pedido.
async fn apply_voice(rt: &Arc<Runtime>, voice_id: &str) -> Result<(), String> {
 fire(rt, "loadVoice", json!({"voiceID": voice_id})).await?;
 for _ in 0..APPLY_ATTEMPTS {
  tokio::time::sleep(APPLY_DELAY).await;
  let cur = call(rt, "getCurrentVoice", &["getCurrentVoice"]).await?;
  if cur["actionObject"]["voiceID"].as_str() == Some(voice_id) {
   return Ok(());
  }
 }
 Err(format!("O Voicemod não confirmou a troca para {voice_id}: voz indisponível ou comando não confirmado."))
}

async fn read_changer(rt: &Arc<Runtime>) -> Result<Option<bool>, String> {
 call(rt, "getVoiceChangerStatus", EV_CHANGER).await.map(|v| read_bool(&v))
}

/// `toggleVoiceChanger` é um interruptor: só dispara se o estado atual já for o
/// desejado, e confere a leitura depois. Nunca mexe em "Ouvir minha voz".
async fn set_changer(rt: &Arc<Runtime>, want: bool) -> Result<bool, String> {
 match read_changer(rt).await? {
  None => Err("Não foi possível ler o estado do modificador de voz no Voicemod.".into()),
  Some(now) if now == want => Ok(false),
  _ => {
   fire(rt, "toggleVoiceChanger", json!({})).await?;
   for _ in 0..4 {
    tokio::time::sleep(APPLY_DELAY).await;
    if read_changer(rt).await? == Some(want) {
     return Ok(true);
    }
   }
   Err("O Voicemod não confirmou a mudança do modificador de voz.".into())
  },
 }
}

// ---------------------------------------------------------------- eventos espontâneos

fn event(rt: &Arc<Runtime>, v: Value) {
 let kind = v["actionType"].as_str().unwrap_or("").to_owned();
 match kind.as_str() {
  "voiceChangedEvent" => {
   let id = v["actionObject"]["voiceID"].as_str().unwrap_or("").to_owned();
   let mut changed = false;
   {
    let mut st = rt.voicemod.state.lock().unwrap();
    if id != st.status.current_voice {
     st.status.current_voice = id.clone();
     st.status.current_name = name_of(&st.status.voices, &id);
    }
    if let Some(t) = st.test.as_mut() {
     if t.voice_id != id {
      t.manual = true;
      changed = true;
     }
    }
   }
   if changed {
    let profile = rt.voicemod.profile();
    let name = rt.voicemod.state.lock().unwrap().status.current_name.clone();
    rt.voicemod.publish(rt);
    rt.log(&profile, KIND, &format!("Uma voz foi escolhida no Voicemod durante o teste ({name}); o BotLive não vai sobrescrever."), "info");
   }
  },
  "voiceChangerEnabledEvent" | "voiceChangerDisabledEvent" => {
   if let Some(val) = read_bool(&v) {
    rt.voicemod.state.lock().unwrap().status.voice_changer = Some(val);
    rt.voicemod.publish(rt);
   }
  },
  "getVoices" => {
   let (voices, cur) = read_voices(&v);
   let mut st = rt.voicemod.state.lock().unwrap();
   st.status.voices = voices;
   if let Some(cur) = cur {
    st.status.current_voice = cur.clone();
    st.status.current_name = name_of(&st.status.voices, &cur);
   }
   drop(st);
   rt.voicemod.publish(rt);
  },
  _ => {},
 }
}

// ---------------------------------------------------------------- teste com restauração

async fn test_start(rt: &Arc<Runtime>, p: &Profile, voice_id: &str, secs: u64) -> Result<Value, String> {
 let voice_id = voice_id.trim().to_owned();
 if voice_id.is_empty() {
  return Err("Escolha uma voz para testar.".into());
 }
 let secs = if secs == 0 {
  DEFAULT_TEST_SECS
 } else {
  secs.clamp(MIN_TEST_SECS, MAX_TEST_SECS)
 };
 let (voices, connected) = {
  let st = rt.voicemod.state.lock().unwrap();
  (
   st.status.voices.clone(),
   st.status.phase == CONNECTED && st.link.is_some() && st.test.is_none() && st.pending.is_none(),
  )
 };
 if !connected {
  let blocked = {
   let st = rt.voicemod.state.lock().unwrap();
   st.test.is_some() || st.pending.is_some()
  };
  if blocked {
   return Err("Já existe um teste de voz em andamento no Voicemod. Aguarde terminar ou encerre-o antes de começar outro.".into());
  }
  return Err(NOT_CONNECTED.into());
 }
 match voices.iter().find(|v| v["id"].as_str() == Some(voice_id.as_str())) {
  None => return Err(format!("A voz {voice_id} não está mais disponível no Voicemod. Atualize a lista.")),
  Some(v) if v["enabled"].as_bool() == Some(false) => {
   let n = v["friendlyName"].as_str().unwrap_or(&voice_id).to_owned();
   return Err(format!("A voz {n} não está liberada pela sua licença no Voicemod."));
  },
  _ => {},
 }
 // 1. estado anterior, sempre lido antes de qualquer mudança
 let cur = call(rt, "getCurrentVoice", &["getCurrentVoice"]).await?;
 let prev_voice = cur["actionObject"]["voiceID"].as_str().unwrap_or("").to_owned();
 if prev_voice.is_empty() {
  return Err("Não foi possível determinar a voz atual no Voicemod; o teste não começou.".into());
 }
 let prev_name = name_of(&voices, &prev_voice);
 let prev_changer = read_changer(rt).await?.ok_or("Não foi possível determinar o estado do modificador de voz no Voicemod; o teste não começou.")?;
 // 2. garante efeito audível, sem tocar em "Ouvir minha voz" nem no microfone do Windows
 let changed_changer = set_changer(rt, true).await?;
 // 3. aplica a voz com confirmação de estado
 let voice_name = name_of(&voices, &voice_id);
 if let Err(e) = apply_voice(rt, &voice_id).await {
  let mut restored_changer = true;
  if changed_changer {
   restored_changer = set_changer(rt, prev_changer).await.is_ok();
  }
  if restored_changer {
   set_outcome(rt, Outcome { kind: "failed".into(), detail: e.clone() }, "");
  } else {
   let mut t = Test::placeholder();
   t.voice_id = voice_id;
   t.voice_name = voice_name;
   t.prev_voice = prev_voice;
   t.prev_name = prev_name;
   t.prev_changer = Some(prev_changer);
   t.changed_changer = changed_changer;
   t.seconds = secs;
   {
    let mut st = rt.voicemod.state.lock().unwrap();
    st.pending = Some(t);
   }
   let detail = format!("{e} O modificador de voz também não pôde ser devolvido: use Restaurar agora.");
   set_outcome(rt, Outcome { kind: "unconfirmed".into(), detail: detail.clone() }, "");
  }
  return Err(e);
 }
 let test = Test {
  voice_id: voice_id.clone(),
  voice_name: voice_name.clone(),
  prev_voice,
  prev_name,
  prev_changer: Some(prev_changer),
  changed_changer,
  manual: false,
  interrupted: false,
  seconds: secs,
  ends_at: Instant::now() + Duration::from_secs(secs),
 };
 {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.test = Some(test);
  st.outcome = Outcome::default();
 }
 rt.voicemod.publish(rt);
 rt.log(&p.id, KIND, &format!("Teste de voz iniciado: {voice_name} por {secs}s. O microfone real muda até o fim do teste."), "info");
 let rt2 = rt.clone();
 tokio::spawn(async move { countdown(rt2).await });
 Ok(json!(view(&rt.voicemod.state.lock().unwrap())))
}

async fn countdown(rt: Arc<Runtime>) {
 loop {
  let rem = {
   let st = rt.voicemod.state.lock().unwrap();
   match st.test.as_ref() {
    None => return,
    Some(t) => t.ends_at.saturating_duration_since(Instant::now()),
   }
  };
  if rem.is_zero() {
   break;
  }
  tokio::time::sleep(rem.min(Duration::from_secs(1))).await;
  rt.voicemod.publish(&rt);
 }
 finish(rt).await;
}

/// Move o teste para a faixa de restauração. Só um chamador vence a corrida.
fn take_test(rt: &Runtime) -> Option<(Test, bool)> {
 let mut st = rt.voicemod.state.lock().unwrap();
 let t = st.test.take()?;
 let linked = st.status.phase == CONNECTED && st.link.is_some();
 let interrupted = t.interrupted || !linked;
 Some((t, interrupted))
}

async fn finish(rt: Arc<Runtime>) {
 let Some((t, interrupted)) = take_test(&rt) else { return };
 if interrupted {
  {
   let mut st = rt.voicemod.state.lock().unwrap();
   st.pending = Some(t);
  }
  let detail = "O Voicemod ficou desconectado durante o teste. A restauração não foi confirmada; use Restaurar agora.".to_string();
  set_outcome(&rt, Outcome { kind: "unconfirmed".into(), detail: detail.clone() }, &detail);
  return;
 }
 if t.manual {
  let detail = format!("Teste encerrado. {} foi trocada por você no Voicemod durante o teste e nada foi sobrescrito.", t.voice_name);
  set_outcome(&rt, Outcome { kind: "manual".into(), detail: detail.clone() }, "");
  let profile = rt.voicemod.profile();
  rt.log(&profile, KIND, &detail, "info");
  return;
 }
 restore_and_report(rt, t, true).await;
}

/// `log_failure=false` quando quem chamou já devolve o erro ao `operation`,
/// que registra no Histórico uma única vez.
async fn restore_and_report(rt: Arc<Runtime>, t: Test, log_failure: bool) {
 let profile = {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.pending = Some(t.clone());
  st.outcome = Outcome { kind: "restoring".into(), detail: "Restaurando o estado anterior…".into() };
  st.profile.clone()
 };
 rt.voicemod.publish(&rt);
 match restore(&rt, &t).await {
  Ok(detail) => {
   {
    let mut st = rt.voicemod.state.lock().unwrap();
    st.pending = None;
    st.outcome = Outcome { kind: "restored".into(), detail: detail.clone() };
   }
   rt.voicemod.publish(&rt);
   rt.log(&profile, KIND, &detail, "success");
  },
  Err(e) => {
   {
    let mut st = rt.voicemod.state.lock().unwrap();
    st.outcome = Outcome { kind: "unconfirmed".into(), detail: e.clone() };
   }
   rt.voicemod.publish(&rt);
   if log_failure {
    rt.log(&profile, KIND, &e, "error");
   }
  },
 }
}

async fn restore(rt: &Arc<Runtime>, t: &Test) -> Result<String, String> {
 let cur = call(rt, "getCurrentVoice", &["getCurrentVoice"]).await?;
 let now = cur["actionObject"]["voiceID"].as_str().unwrap_or("").to_owned();
 if now != t.prev_voice {
  apply_voice(rt, &t.prev_voice).await?;
 }
 let confirm = call(rt, "getCurrentVoice", &["getCurrentVoice"]).await?;
 let confirmed = confirm["actionObject"]["voiceID"].as_str().unwrap_or("");
 if confirmed != t.prev_voice {
  return Err(format!("Restauração não confirmada: o Voicemod ficou em {confirmed}."));
 }
 if t.changed_changer {
  if let Some(prev) = t.prev_changer {
   set_changer(rt, prev).await?;
   if read_changer(rt).await? != Some(prev) {
    return Err("Restauração não confirmada: o modificador de voz não voltou ao estado anterior.".into());
   }
  }
 }
 let name = t.prev_name.clone();
 {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.status.current_voice = t.prev_voice.clone();
  st.status.current_name = name_of(&st.status.voices, &t.prev_voice);
 }
 Ok(format!("Voz restaurada: {name}."))
}

async fn stop_test(rt: &Arc<Runtime>) -> Result<Value, String> {
 let Some((t, interrupted)) = take_test(rt) else {
  if rt.voicemod.state.lock().unwrap().pending.is_some() {
   return Err("A restauração está pendente; use Restaurar agora.".into());
  }
  return Err("Não há teste de voz ativo no Voicemod.".into());
 };
 if interrupted {
  {
   let mut st = rt.voicemod.state.lock().unwrap();
   st.pending = Some(t);
  }
  let detail = "O teste foi encerrado, mas o Voicemod está desconectado: restauração não confirmada.".to_string();
  set_outcome(rt, Outcome { kind: "unconfirmed".into(), detail: detail.clone() }, &detail);
  return Ok(snapshot_json(rt));
 }
 if t.manual {
  let detail = format!("Teste encerrado. {} foi trocada por você no Voicemod e nada foi sobrescrito.", t.voice_name);
  set_outcome(rt, Outcome { kind: "manual".into(), detail: detail.clone() }, "");
  let profile = rt.voicemod.profile();
  rt.log(&profile, KIND, &detail, "info");
  return Ok(snapshot_json(rt));
 }
 restore_and_report(rt.clone(), t, true).await;
 Ok(snapshot_json(rt))
}

async fn recover(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 if rt.voicemod.state.lock().unwrap().status.phase != CONNECTED {
  open_and_install(rt, p, false).await?;
 }
 let t = {
  let st = rt.voicemod.state.lock().unwrap();
  match st.pending.as_ref() {
   None => return Err("Não há restauração pendente no Voicemod.".into()),
   Some(t) => t.clone(),
  }
 };
 // O erro é devolvido para a tela e registrado pelo `operation`; aqui só o resultado.
 restore_and_report(rt.clone(), t, false).await;
 let outcome = rt.voicemod.state.lock().unwrap().outcome.clone();
 if outcome.kind == "restored" {
  Ok(snapshot_json(rt))
 } else {
  Err(outcome.detail)
 }
}

// ---------------------------------------------------------------- operações públicas

pub async fn connect(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 if let Some(h) = rt.voicemod.reconnect.lock().unwrap().take() {
  h.abort();
 }
 let already = {
  let st = rt.voicemod.state.lock().unwrap();
  st.link.as_ref().is_some_and(|s| !s.link.closed.load(Ordering::SeqCst))
 };
 if already {
  refresh_now(rt).await.ok();
  return Ok(snapshot_json(rt));
 }
 if rt.voicemod.busy.swap(true, Ordering::SeqCst) {
  return Err("Já existe uma conexão em andamento com o Voicemod.".into());
 }
 let result = open_and_install(rt, p, false).await;
 rt.voicemod.busy.store(false, Ordering::SeqCst);
 result?;
 Ok(snapshot_json(rt))
}

pub async fn disconnect(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 if let Some(h) = rt.voicemod.reconnect.lock().unwrap().take() {
  h.abort();
 }
 let mut stopped = false;
 let mut restored = false;
 let mut detail = String::new();
 if rt.voicemod.state.lock().unwrap().test.is_some() {
  stopped = true;
  match stop_test(rt).await {
   Ok(v) => {
    let kind = v["outcome"]["kind"].as_str().unwrap_or("");
    restored = kind == "restored";
    detail = v["outcome"]["detail"].as_str().unwrap_or("").to_owned();
   },
   Err(e) => detail = e,
  }
 }
 let session = {
  let mut st = rt.voicemod.state.lock().unwrap();
  st.status.phase = OFF.into();
  st.status.message = "Desconectado a pedido.".into();
  st.link.take()
 };
 if let Some(s) = session {
  s.link.tx.send(Out::Close).ok();
 }
 rt.voicemod.publish(rt);
 let msg = if stopped {
  format!("Desconectado do Voicemod. {detail}")
 } else {
  "Desconectado do Voicemod.".to_string()
 };
 rt.log(&p.id, KIND, &msg, if restored { "success" } else { "info" });
 Ok(json!({"stoppedTest": stopped, "restored": restored, "detail": detail}))
}

/// Caracteres que um copiar-e-colar do e-mail traz junto sem serem parte da chave.
const INVISIBLE: [char; 6] = ['\u{feff}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{00ad}'];

/// Tira só o que veio junto da cópia; a chave em si não é alterada.
fn clean_key(value: &str) -> String {
 value
  .chars()
  .filter(|c| !INVISIBLE.contains(c))
  .collect::<String>()
  .trim()
  .to_string()
}

pub fn save_key(rt: &Runtime, profile: &str, value: &str) -> Result<Value, String> {
 let v = clean_key(value);
 if v.chars().count() > 400 {
  return Err("Chave da Control API longa demais.".into());
 }
 // Uma chave não tem espaço interno, aspas ou quebra de linha: isso é o rótulo
 // do e-mail copiado junto, e chegaria ao Voicemod como um 401 sem explicação.
 if v.split_whitespace().count() > 1 || v.contains('"') || v.contains('\'') || v.contains('`') {
  return Err(
   "A chave veio com espaço, aspas ou quebra de linha: cole só a chave do e-mail, sem o rótulo e sem aspas.".into(),
  );
 }
 write_key(rt, profile, &v)?;
 Ok(json!({"hasKey": !v.is_empty(), "keyLen": v.chars().count()}))
}

pub async fn operation(rt: &Arc<Runtime>, p: &str, op: &str, args: &Value) -> Result<Value, String> {
 let profile = rt.db.profile(p)?;
 let out = match op {
  "voicemod.get" => Ok(snapshot_json(rt)),
  "voicemod.key" => save_key(rt, p, args["value"].as_str().unwrap_or("")),
  "voicemod.connect" => connect(rt, &profile).await,
  "voicemod.disconnect" => disconnect(rt, &profile).await,
  "voicemod.refresh" => match refresh_now(rt).await {
   Ok(()) => Ok(snapshot_json(rt)),
   Err(e) => Err(e),
  },
  "voicemod.testStart" => test_start(rt, &profile, args["voiceId"].as_str().unwrap_or(""), args["seconds"].as_u64().unwrap_or(0)).await,
  "voicemod.testStop" => stop_test(rt).await,
  "voicemod.recover" => recover(rt, &profile).await,
  _ => Err("Operação do Voicemod desconhecida".into()),
 };
 // O cofre é do perfil que fez a chamada, nunca da sessão aberta. A sessão só
 // ganha perfil depois da primeira conexão aceita, então calcular isso lá dentro
 // fazia a tela voltar a dizer "nenhuma chave guardada" logo após salvar.
 let out = out.map(|mut v| {
  if v.is_object() {
   let stored = stored_key(rt, p);
   v["hasKey"] = json!(stored.is_some());
   v["keyLen"] = json!(stored.as_ref().map(|k| k.chars().count()).unwrap_or(0));
  }
  v
 });
 // Erro de integração entra no Histórico, nunca vira mensagem de chat.
 if let Err(e) = &out {
  if op != "voicemod.get" {
   rt.log(&profile.id, KIND, e, "error");
  }
 }
 out
}

// ---------------------------------------------------------------- testes
//
// Servidor WebSocket controlado na máquina: os testes provam o protocolo, a
// autorização, a correlação de respostas e o ciclo de teste/restauração.
// Nada aqui prova conexão com um Voicemod de verdade.

#[cfg(test)]
mod tests {
 use super::*;
 use serde_json::{json, Value};

 const KEY: &str = "chave-secreta-da-control-api-98765";
 /// Sentinela interna: manda o servidor fechar o socket (queda da conexão).
 const KILL: &str = "\u{0}KILL";

 fn profile(name: &str) -> Profile {
  Profile {
   id: uuid::Uuid::new_v4().to_string(),
   name: name.into(),
   platform: "twitch".into(),
   channel: name.into(),
   channel_id: "123".into(),
   bot_id: "456".into(),
   client_id: "client".into(),
   blocklist: vec![],
   topics: vec![],
   editors: vec![],
   ai: crate::model::AiConfig::default(),
   modules: json!({"points": true, "raffles": true, "predictions": true, "queue": true}),
  }
 }

 async fn setup() -> (tempfile::TempDir, Arc<Runtime>, Profile) {
  let dir = tempfile::tempdir().unwrap();
  let rt = Runtime::new(dir.path().to_path_buf()).unwrap();
  let p = profile("Voicemod");
  rt.db.save_profile(&p).unwrap();
  (dir, rt, p)
 }

 fn voices() -> Vec<Value> {
  vec![
   json!({"id":"nofx","friendlyName":"Efeito desligado","enabled":true,"favorited":false,"isNew":false,"isCustom":false,"bitmapChecksum":"1"}),
   json!({"id":"robot","friendlyName":"Robô","enabled":true,"favorited":true,"isNew":false,"isCustom":false,"bitmapChecksum":"2"}),
   json!({"id":"alien","friendlyName":"Alien","enabled":true,"favorited":false,"isNew":true,"isCustom":false,"bitmapChecksum":"3"}),
   json!({"id":"blocked","friendlyName":"Voz bloqueada","enabled":false,"favorited":false,"isNew":false,"isCustom":false,"bitmapChecksum":"4"}),
  ]
 }

 #[derive(Clone)]
 struct Mock {
  key_ok: Arc<std::sync::atomic::AtomicBool>,
  silent: Arc<std::sync::atomic::AtomicBool>,
  omit_ids: Arc<std::sync::atomic::AtomicBool>,
  raw: Arc<Mutex<Option<String>>>,
  registers: Arc<std::sync::atomic::AtomicU64>,
  connects: Arc<std::sync::atomic::AtomicU64>,
  loads: Arc<Mutex<Vec<String>>>,
  voice: Arc<Mutex<String>>,
  changer: Arc<Mutex<bool>>,
  hear: Arc<Mutex<bool>>,
  license: Arc<Mutex<String>>,
  voices: Arc<Mutex<Vec<Value>>>,
  push: Arc<Mutex<Option<mpsc::UnboundedSender<String>>>>,
 }

 impl Mock {
  fn new() -> Self {
   Self {
    key_ok: Arc::new(std::sync::atomic::AtomicBool::new(true)),
    silent: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    omit_ids: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    raw: Arc::new(Mutex::new(None)),
    registers: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    connects: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    loads: Arc::new(Mutex::new(vec![])),
    voice: Arc::new(Mutex::new("nofx".into())),
    changer: Arc::new(Mutex::new(false)),
    hear: Arc::new(Mutex::new(false)),
    license: Arc::new(Mutex::new("free".into())),
    voices: Arc::new(Mutex::new(voices())),
    push: Arc::new(Mutex::new(None)),
   }
  }
  fn push(&self, text: String) {
   let g = self.push.lock().unwrap();
   g.as_ref().expect("servidor sem conexão ativa").send(text).unwrap();
  }
  fn event(&self, action: &str, body: Value) {
   self.push(json!({"actionType": action, "actionObject": body}).to_string());
  }
  fn loads(&self) -> Vec<String> {
   self.loads.lock().unwrap().clone()
  }
  fn voice(&self) -> String {
   self.voice.lock().unwrap().clone()
  }
  fn changer(&self) -> bool {
   *self.changer.lock().unwrap()
  }
 }

 /// Gera as respostas no formato documentado. `id` é ecoado salvo em `omit_ids`.
 fn build(m: &Mock, action: &str, id: &str, v: &Value) -> Vec<String> {
  fn tagged(m: &Mock, mut o: Value, id: &str) -> Value {
   if !m.omit_ids.load(Ordering::SeqCst) {
    o["id"] = json!(id);
   }
   o
  }
  match action {
   "registerClient" => {
    m.registers.fetch_add(1, Ordering::SeqCst);
    let code = if m.key_ok.load(Ordering::SeqCst) { 200 } else { 401 };
    let desc = if code == 200 { "Authorized" } else { "Unauthorized" };
    vec![tagged(m, json!({"action":"registerClient","payload":{"status":{"code":code,"description":desc}}}), id).to_string()]
   },
   "getVoices" => vec![tagged(m, json!({"actionType":"getVoices","actionObject":{"voices":m.voices.lock().unwrap().clone(),"currentVoice":m.voice.lock().unwrap().clone()}}), id).to_string()],
   "getCurrentVoice" => vec![tagged(m, json!({"actionType":"getCurrentVoice","actionObject":{"voiceID":m.voice.lock().unwrap().clone(),"parameters":[]}}), id).to_string()],
   "getVoiceChangerStatus" => vec![tagged(m, json!({"actionType":"toggleVoiceChanger","actionObject":{"value":*m.changer.lock().unwrap()}}), id).to_string()],
   "getHearMyselfStatus" => vec![tagged(m, json!({"actionType":"toggleHearMyVoice","actionObject":{"value":*m.hear.lock().unwrap()}}), id).to_string()],
   "getUserLicense" => vec![tagged(m, json!({"actionType":"getUserLicense","actionObject":{"licenseType":m.license.lock().unwrap().clone()}}), id).to_string()],
   "toggleVoiceChanger" => {
    let now = {
     let mut g = m.changer.lock().unwrap();
     *g = !*g;
     *g
    };
    let kind = if now { "voiceChangerEnabledEvent" } else { "voiceChangerDisabledEvent" };
    vec![tagged(m, json!({"actionType":kind,"actionObject":{"value":now}}), id).to_string()]
   },
   "loadVoice" => {
    let target = v["payload"]["voiceID"].as_str().unwrap_or("").to_owned();
    m.loads.lock().unwrap().push(target.clone());
    let known = m.voices.lock().unwrap().iter().any(|x| x["id"].as_str() == Some(target.as_str()));
    if !known {
     return vec![];
    }
    *m.voice.lock().unwrap() = target.clone();
    vec![tagged(m, json!({"actionType":"voiceChangedEvent","actionObject":{"voiceID":target}}), id).to_string()]
   },
   _ => vec![],
  }
 }

 async fn serve(m: Mock, sock: tokio::net::TcpStream) {
  let Ok(ws) = tokio_tungstenite::accept_async(sock).await else { return };
  m.connects.fetch_add(1, Ordering::SeqCst);
  let (mut sink, mut stream) = ws.split();
  let (tx, mut rx) = mpsc::unbounded_channel::<String>();
  *m.push.lock().unwrap() = Some(tx.clone());
  let pump = tokio::spawn(async move {
   while let Some(s) = rx.recv().await {
    if s == KILL {
     let _ = sink.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
     break;
    }
    if sink.send(tokio_tungstenite::tungstenite::Message::Text(s.into())).await.is_err() {
     break;
    }
   }
  });
  while let Some(next) = stream.next().await {
   let text = match next {
    Ok(tokio_tungstenite::tungstenite::Message::Text(t)) => t.to_string(),
    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
    Ok(_) => continue,
    Err(_) => break,
   };
   if m.silent.load(Ordering::SeqCst) {
    continue;
   }
   if let Some(raw) = m.raw.lock().unwrap().clone() {
    tx.send(raw.clone()).ok();
    continue;
   }
   let Ok(v) = serde_json::from_str::<Value>(&text) else { continue };
   let action = v["action"].as_str().unwrap_or("").to_owned();
   let id = v["id"].as_str().unwrap_or("").to_owned();
   for out in build(&m, &action, &id, &v) {
    tx.send(out).ok();
   }
  }
  pump.abort();
  let mut g = m.push.lock().unwrap();
  if g.as_ref().is_some_and(|s| s.same_channel(&tx)) {
   *g = None;
  }
 }

 async fn mock_server() -> (u16, Mock, tokio::task::JoinHandle<()>) {
  let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
  let port = listener.local_addr().unwrap().port();
  let m = Mock::new();
  let m2 = m.clone();
  let accept = tokio::spawn(async move {
   while let Ok((sock, _)) = listener.accept().await {
    let m = m2.clone();
    tokio::spawn(serve(m, sock));
   }
  });
  (port, m, accept)
 }

 fn dead_port() -> u16 {
  let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
  l.local_addr().unwrap().port()
 }

 async fn until(rt: &Arc<Runtime>, ms: u64, ok: impl Fn(&Value) -> bool) -> Value {
  let end = Instant::now() + Duration::from_millis(ms);
  loop {
   let v = snapshot_json(rt);
   if ok(&v) || Instant::now() >= end {
    return v;
   }
   tokio::time::sleep(Duration::from_millis(40)).await;
  }
 }

 async fn leave(rt: &Arc<Runtime>, p: &Profile) {
  if let Some(h) = rt.voicemod.reconnect.lock().unwrap().take() {
   h.abort();
  }
  let _ = disconnect(rt, p).await;
 }

 fn boot(rt: &Arc<Runtime>, port: u16) {
  rt.voicemod.set_ports(vec![port]);
  rt.voicemod.set_delays(vec![0, 0]);
  rt.voicemod.set_key(Some(KEY.into()));
 }

 // ---------------------------------------------------------------- contrato

 #[test]
 fn ports_and_path_follow_the_official_reference() {
  assert_eq!(WSPATH, "/v1");
  assert_eq!(PORTS.len(), 11);
  assert!(PORTS.contains(&59129) && PORTS.contains(&39273) && PORTS.contains(&30546));
  for op in [OFF, SEARCHING, AUTHORIZING, CONNECTED, FAILED] {
   assert!(["disconnected", "searching", "authorizing", "connected", "failed"].contains(&op));
  }
 }

 // ---------------------------------------------------------------- conexão

 #[tokio::test]
 async fn connects_authorizes_and_lists_voices_keeping_their_ids() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  let v = connect(&rt, &p).await.unwrap();
  assert_eq!(v["status"]["phase"], "connected");
  assert_eq!(v["status"]["port"].as_u64(), Some(port as u64));
  assert_eq!(v["status"]["currentVoice"], "nofx");
  assert_eq!(v["status"]["currentName"], "Efeito desligado");
  assert_eq!(v["status"]["voiceChanger"], json!(false));
  assert_eq!(v["status"]["hearMyself"], json!(false));
  assert_eq!(v["status"]["license"], "free");
  let ids: Vec<&str> = v["status"]["voices"].as_array().unwrap().iter().map(|x| x["id"].as_str().unwrap()).collect();
  assert_eq!(ids, vec!["nofx", "robot", "alien", "blocked"]);
  let names: Vec<&str> = v["status"]["voices"].as_array().unwrap().iter().map(|x| x["friendlyName"].as_str().unwrap()).collect();
  assert!(names.contains(&"Robô"));
  assert_eq!(m.connects.load(Ordering::SeqCst), 1);
  assert_eq!(m.registers.load(Ordering::SeqCst), 1);
  assert!(!v.to_string().contains(KEY), "a chave não pode aparecer numa resposta");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn refused_key_is_reported_and_never_reads_as_connected() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  m.key_ok.store(false, Ordering::SeqCst);
  let err = connect(&rt, &p).await.unwrap_err();
  assert!(err.contains("recusou a chave"), "{err}");
  let v = snapshot_json(&rt);
  assert_ne!(v["status"]["phase"], "connected");
  assert_eq!(v["status"]["phase"], "failed");
  assert!(v["status"]["message"].as_str().unwrap().contains("401"));
  assert!(v["status"]["voices"].as_array().unwrap().is_empty());
  assert_eq!(m.registers.load(Ordering::SeqCst), 1);
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn missing_key_stops_before_touching_the_network() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  rt.voicemod.set_key(None);
  let err = connect(&rt, &p).await.unwrap_err();
  assert!(err.contains("Chave da Control API ausente"), "{err}");
  assert_eq!(snapshot_json(&rt)["status"]["phase"], "failed");
  assert_eq!(m.connects.load(Ordering::SeqCst), 0, "sem chave não pode nem abrir socket");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn no_voicemod_reports_not_found() {
  let (_dir, rt, p) = setup().await;
  boot(&rt, dead_port());
  let err = connect(&rt, &p).await.unwrap_err();
  assert!(err.contains("Voicemod não encontrado"), "{err}");
  let v = snapshot_json(&rt);
  assert_eq!(v["status"]["phase"], "failed");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn invalid_response_is_reported_as_protocol_failure() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  rt.voicemod.set_reply_timeout(400);
  *m.raw.lock().unwrap() = Some("isto nao e json".into());
  let err = connect(&rt, &p).await.unwrap_err();
  assert!(err.contains("não respondeu a registerClient"), "{err}");
  assert_eq!(snapshot_json(&rt)["status"]["phase"], "failed");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn silent_server_times_out_instead_of_guessing_success() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  rt.voicemod.set_reply_timeout(300);
  m.silent.store(true, Ordering::SeqCst);
  assert!(connect(&rt, &p).await.is_err());
  let v = snapshot_json(&rt);
  assert_ne!(v["status"]["phase"], "connected");
  assert!(v["status"]["voices"].as_array().unwrap().is_empty());
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn repeated_connect_keeps_a_single_session() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  connect(&rt, &p).await.unwrap();
  connect(&rt, &p).await.unwrap();
  assert_eq!(m.connects.load(Ordering::SeqCst), 1, "uma sessão, um leitor");
  assert_eq!(m.registers.load(Ordering::SeqCst), 1, "uma autorização, não três");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn responses_without_id_are_still_correlated_by_action() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  m.omit_ids.store(true, Ordering::SeqCst);
  let v = connect(&rt, &p).await.unwrap();
  assert_eq!(v["status"]["phase"], "connected");
  assert_eq!(v["status"]["currentName"], "Efeito desligado");
  leave(&rt, &p).await;
 }

 // ---------------------------------------------------------------- reconexão

 #[tokio::test]
 async fn reconnect_is_limited_and_never_reapplies_a_voice() {
  let (_dir, rt, p) = setup().await;
  let (port, m, accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  accept.abort();
  tokio::time::sleep(Duration::from_millis(300)).await;
  m.push(KILL.to_string());
  let v = until(&rt, 20_000, |s| {
   s["status"]["phase"] == "failed" && s["status"]["message"].as_str().is_some_and(|m| m.contains("Não foi possível reconectar"))
  })
  .await;
  assert_eq!(v["status"]["phase"], "failed");
  assert!(v["status"]["message"].as_str().unwrap().contains("tentativas"));
  assert!(m.loads().is_empty(), "reconexão não pode reaplicar voz: {:?}", m.loads());
  leave(&rt, &p).await;
 }

 // ---------------------------------------------------------------- teste manual

 #[tokio::test]
 async fn test_applies_voice_and_restores_voice_and_modifier_on_early_stop() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  assert!(!m.changer(), "o modificador começa desligado no cenário");
  let started = test_start(&rt, &p, "robot", 60).await.unwrap();
  assert_eq!(started["active"], json!(true));
  assert_eq!(started["voiceId"], "robot");
  assert_eq!(m.voice(), "robot");
  assert!(m.changer(), "o teste liga o modificador para o efeito ser audível");
  let stopped = stop_test(&rt).await.unwrap();
  assert_eq!(stopped["outcome"]["kind"], "restored");
  assert!(stopped["outcome"]["detail"].as_str().unwrap().contains("restaurada"));
  assert_eq!(m.voice(), "nofx", "voz anterior devolvida");
  assert!(!m.changer(), "modificador devolvido ao estado anterior");
  assert_eq!(m.loads().join(","), "robot,nofx");
  assert_eq!(stopped["test"]["active"], json!(false));
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn two_simultaneous_tests_are_refused() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  test_start(&rt, &p, "robot", 60).await.unwrap();
  let err = test_start(&rt, &p, "alien", 60).await.unwrap_err();
  assert!(err.contains("Já existe um teste"), "{err}");
  assert_eq!(m.loads().join(","), "robot", "a segunda tentativa não chegou ao Voicemod");
  stop_test(&rt).await.unwrap();
  assert_eq!(m.voice(), "nofx");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn test_on_an_unknown_or_blocked_voice_is_refused() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  assert!(test_start(&rt, &p, "voz-que-nao-existe", 60).await.unwrap_err().contains("não está mais disponível"));
  let err = test_start(&rt, &p, "blocked", 60).await.unwrap_err();
  assert!(err.contains("Voz bloqueada"), "{err}");
  assert!(m.loads().is_empty());
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn manual_voice_change_during_a_test_is_never_overwritten() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  test_start(&rt, &p, "robot", 60).await.unwrap();
  m.event("voiceChangedEvent", json!({"voiceID":"alien"}));
  tokio::time::sleep(Duration::from_millis(250)).await;
  let stopped = stop_test(&rt).await.unwrap();
  assert_eq!(stopped["outcome"]["kind"], "manual");
  assert_eq!(m.loads().join(","), "robot", "nada foi devolvido por cima da escolha do usuário");
  let logs = serde_json::to_string(&rt.db.logs("").unwrap()).unwrap();
  assert!(logs.contains("nada foi sobrescrito"));
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn test_cannot_start_while_disconnected() {
  let (_dir, rt, p) = setup().await;
  let err = test_start(&rt, &p, "robot", 60).await.unwrap_err();
  assert!(err.contains("Conecte o Voicemod"), "{err}");
 }

 #[tokio::test]
 async fn drop_during_test_reports_unconfirmed_restore_until_user_recovers() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  test_start(&rt, &p, "robot", 1).await.unwrap();
  assert_eq!(m.loads().join(","), "robot");
  m.push(KILL.to_string());
  let v = until(&rt, 20_000, |s| s["outcome"]["kind"] == "unconfirmed").await;
  assert_eq!(v["status"]["phase"], "connected", "a reconexão automática volta, mas não reaplica");
  assert_eq!(v["test"]["active"], json!(false));
  assert_eq!(m.loads().join(","), "robot", "sem reapply automático depois da queda");
  let recovered = recover(&rt, &p).await.unwrap();
  assert_eq!(recovered["outcome"]["kind"], "restored");
  assert_eq!(m.voice(), "nofx");
  assert_eq!(m.loads().join(","), "robot,nofx");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn disconnect_during_a_test_restores_first_and_reports_the_result() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  test_start(&rt, &p, "robot", 60).await.unwrap();
  let v = disconnect(&rt, &p).await.unwrap();
  assert_eq!(v["stoppedTest"], json!(true));
  assert_eq!(v["restored"], json!(true));
  assert!(v["detail"].as_str().unwrap().contains("restaurada"));
  assert_eq!(m.voice(), "nofx");
  assert_eq!(m.loads().join(","), "robot,nofx");
  assert_eq!(snapshot_json(&rt)["status"]["phase"], "disconnected");
  let logs = serde_json::to_string(&rt.db.logs("").unwrap()).unwrap();
  assert!(logs.contains("Desconectado do Voicemod"));
 }

 // ---------------------------------------------------------------- credenciais

 #[tokio::test]
 async fn key_never_reaches_logs_or_responses() {
  let (_dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  connect(&rt, &p).await.unwrap();
  test_start(&rt, &p, "robot", 60).await.unwrap();
  stop_test(&rt).await.unwrap();
  let snapshot = snapshot_json(&rt).to_string();
  assert!(!snapshot.contains(KEY), "chave vazou na resposta da tela");
  let logs = serde_json::to_string(&rt.db.logs("").unwrap()).unwrap();
  assert!(!logs.contains(KEY), "chave vazou no Histórico");
  let db = String::from_utf8_lossy(&std::fs::read(_dir.path().join("botlive.sqlite")).unwrap()).into_owned();
  assert!(!db.contains(KEY), "chave vazou no banco, que é o que backup e exportação carregam");
  assert!(m.registers.load(Ordering::SeqCst) >= 1);
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn only_the_owner_saves_the_key_even_for_an_editor_of_the_profile() {
  use crate::access;
  let (_dir, rt, mut p) = setup().await;
  p.editors = vec!["ana".into()];
  rt.db.save_profile(&p).unwrap();
  let args = json!({"profileId": p.id, "value": KEY});
  assert!(access::guard(&rt, "voicemod.key", &args).is_ok());
  *rt.actor.lock().unwrap() = "ana".into();
  assert!(access::guard(&rt, "voicemod.get", &args).is_ok(), "editor pode usar a tela");
  let err = access::guard(&rt, "voicemod.key", &args).unwrap_err();
  assert!(err.contains("Somente o proprietário"), "{err}");
 }

 // A tela tem que confirmar a chave logo depois de salvar, sem conexão nenhuma:
 // a sessão só ganha perfil depois da primeira conexão aceita pelo Voicemod.
 #[tokio::test]
 async fn saving_the_key_is_reported_even_before_the_first_connection() {
  let (_dir, rt, p) = setup().await;
  assert!(rt.voicemod.profile().is_empty(), "nenhuma conexão aconteceu");
  let saved = save_key(&rt, &p.id, KEY).unwrap();
  assert_eq!(saved["hasKey"], json!(true));
  assert_eq!(saved["keyLen"], json!(KEY.chars().count()));
  let view = operation(&rt, &p.id, "voicemod.get", &json!({})).await.unwrap();
  assert_eq!(view["hasKey"], json!(true), "a tela precisa ver a chave logo após salvar");
  assert_eq!(view["keyLen"], json!(KEY.chars().count()));
 }

 #[tokio::test]
 async fn the_key_belongs_to_the_profile_that_saved_it() {
  let (_dir, rt, p) = setup().await;
  let outro = profile("Outro");
  rt.db.save_profile(&outro).unwrap();
  save_key(&rt, &p.id, KEY).unwrap();
  let view = operation(&rt, &outro.id, "voicemod.get", &json!({})).await.unwrap();
  assert_eq!(view["hasKey"], json!(false), "um perfil não herda a chave do outro");
 }

 // Rótulo do e-mail colado junto viraria um 401 do Voicemod sem explicar nada.
 #[tokio::test]
 async fn a_paste_with_label_spaces_or_quotes_is_refused_before_it_reaches_the_voicemod() {
  let (_dir, rt, p) = setup().await;
  for bad in ["API Key: controlapi-1234", "\"controlapi-1234\"", "controlapi-1234\noutra"] {
   let err = save_key(&rt, &p.id, bad).unwrap_err();
   assert!(err.contains("cole só a chave"), "{bad} => {err}");
  }
  assert!(stored_key(&rt, &p.id).is_none(), "nada foi guardado");
  let view = operation(&rt, &p.id, "voicemod.get", &json!({})).await.unwrap();
  assert_eq!(view["hasKey"], json!(false));
 }

 #[tokio::test]
 async fn invisible_characters_copied_from_an_email_do_not_change_the_key() {
  let (_dir, rt, p) = setup().await;
  let raw = format!("\u{feff}{KEY}\u{200b}");
  let saved = save_key(&rt, &p.id, &raw).unwrap();
  assert_eq!(saved["keyLen"], json!(KEY.chars().count()));
  assert_eq!(read_key(&rt, &p.id).unwrap(), KEY, "a chave enviada ao Voicemod é a do e-mail");
 }
}
