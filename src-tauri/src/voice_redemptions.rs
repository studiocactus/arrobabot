//! Resgates de voz: o reward da Twitch liga uma voz real do Voicemod por um
//! tempo combinado e devolve a voz de antes. Uma sessão por máquina (o
//! Voicemod é um dispositivo local), fila limitada e previsível, e
//! FULFILLED só depois que a troca foi confirmada pelo próprio Voicemod —
//! nunca porque a mensagem saiu pela ponta.
//!
//! O resgate é dono só da parte de voz: automações que casem no mesmo reward
//! continuam cumprindo o resto (mensagem, pontos) pelo caminho normal.
//! A deduplicação por ID de resgate mora aqui, por cima da deduplicação de
//! mensagens EventSub (message_id), porque uma ressincronização pode
//! reentregar a mesma resgate com message_id novo.
use crate::engine::Runtime;
use crate::model::{Event, Profile};
use crate::voicemod as vm;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const KIND: &str = "voiceResgate";
pub const MODULE: &str = "voiceRedeem";
/// Memória de resgates atendidos: deduplicação além do EventSub.
const MAX_SEEN: usize = 500;
const MAX_BINDINGS: usize = 20;
const MAX_DURATION: u64 = 3600;
const MAX_COOLDOWN: u64 = 86400;
const MAX_QUEUE_LIMIT: u64 = 10;
const CONCURRENCY: &[&str] = &["queue", "ignore", "replace"];
/// Quem pode usar um comando do chat que troca a voz. O padrão é `moderator`:
/// nada vira público às escondidas durante a migração.
const PERMISSIONS: &[&str] = &["everyone", "moderator", "broadcaster"];
type R<T> = Result<T, String>;

/// Palavra do comando, normalizada: minúsculas e sem o "!" de chamada.
pub fn command_key(s: &str) -> String {
 s.trim().to_lowercase().trim_start_matches('!').to_owned()
}

/// O comando do chat casa quando a primeira palavra é a palavra da regra.
fn command_matches(rule: &str, message: &str) -> bool {
 let key = command_key(rule);
 if key.is_empty() {
  return false;
 }
 message.split_whitespace().next().map(command_key).as_deref() == Some(key.as_str())
}

/// Permissão explícita por regra: broadcaster só broadcaster; moderator aceita
/// moderator e broadcaster; everyone é público (escolha consciente do dono).
fn permission_ok(permission: &str, role: &str) -> bool {
 match permission {
  "broadcaster" => role == "broadcaster",
  "moderator" => role == "broadcaster" || role == "moderator",
  _ => true,
 }
}

// ---------------------------------------------------------------- configuração

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
 /// `"reward"` (recompensa da Twitch) ou `"command"` (comando do chat).
 #[serde(default = "default_kind")]
 pub kind: String,
 pub reward_id: String,
 #[serde(default)]
 pub reward_name: String,
 /// Palavra que dispara no chat quando `kind == "command"`.
 #[serde(default)]
 pub command: String,
 /// Quem pode usar o comando: `everyone`, `moderator` ou `broadcaster`.
 /// Padrão `moderator`: nada vira público às escondidas.
 #[serde(default = "default_permission")]
 pub permission: String,
 /// Regra ligada sem apagar o vínculo.
 #[serde(default = "default_enabled")]
 pub enabled: bool,
 pub voice_id: String,
 #[serde(default)]
 pub voice_name: String,
 /// `durationSec` (sem o "s") porque é o nome que a tela envia e o parser lê.
 #[serde(rename = "durationSec", default = "default_duration")]
 pub duration_secs: u64,
 #[serde(rename = "cooldownSec", default)]
 pub cooldown_secs: u64,
 #[serde(default = "default_concurrency")]
 pub concurrency: String,
}
fn default_kind() -> String { "reward".into() }
fn default_permission() -> String { "moderator".into() }
fn default_enabled() -> bool { true }
fn default_duration() -> u64 { 30 }
fn default_concurrency() -> String { "queue".into() }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
 #[serde(default)]
 pub enabled: bool,
 #[serde(default = "default_queue_limit")]
 pub queue_limit: u64,
 #[serde(default)]
 pub bindings: Vec<Binding>,
 /// Mapeamento explícito de efeitos internos antigos para vozes do Voicemod.
 /// Nomes antigos nunca viram ID por suposição: o dono escolhe na interface.
 #[serde(default)]
 pub legacy_map: HashMap<String, String>,
 /// "Áudio conferido no OBS": só o usuário confirma (ouvir a própria voz na
 /// gravação). Conexão com o Voicemod nunca comprova áudio na cena.
 #[serde(default)]
 pub obs_checked: bool,
 /// Mensagem ao chat quando um resgate entra na fila. Vazia desliga o aviso;
 /// aceita {{user}}, {{pos}} e {{voz}}.
 #[serde(default = "default_queue_notice")]
 pub queue_notice: String,
}
fn default_queue_limit() -> u64 { 3 }
fn default_queue_notice() -> String { "{{user}}, seu resgate de voz está na fila ({{pos}}º) — começa em seguida!".into() }

impl Default for Config {
 fn default() -> Self {
  Self { enabled: false, queue_limit: default_queue_limit(), bindings: Vec::new(), legacy_map: HashMap::new(), obs_checked: false, queue_notice: default_queue_notice() }
 }
}

fn num(v: &Value, key: &str, min: u64, max: u64, fallback: u64) -> u64 {
 v[key].as_u64().map(|n| n.clamp(min, max)).unwrap_or(fallback)
}

/// Lê a configuração gravada e devolve com defaults enxutos. Uma entrada
/// corrompida nunca derruba a tela: vira configuração vazia e o motivo fica
/// no log quando alguém salvar de novo.
pub fn load(db: &crate::db::Db, profile_id: &str) -> Config {
 let raw = db.module(profile_id, MODULE);
 let mut cfg: Config = serde_json::from_value(raw).unwrap_or_default();
 cfg.queue_limit = cfg.queue_limit.clamp(1, MAX_QUEUE_LIMIT);
 cfg.bindings.truncate(MAX_BINDINGS);
 cfg
}

/// Normaliza e grava. Valida cada vínculo por inteiro: recompensa, voz,
/// duração e política de concorrência precisam fazer sentido juntos.
pub fn save(rt: &Runtime, p: &Profile, v: &Value) -> R<Value> {
 let cfg = parse(v)?;
 rt.db.set_module(&p.id, MODULE, &serde_json::to_value(&cfg).map_err(|_| "Configuração inválida".to_string())?)?;
 rt.log(&p.id, KIND, "Vínculos de resgate de voz salvos", "success");
 Ok(serde_json::to_value(view(rt, p)).map_err(|_| "Falha ao montar o estado".to_string())?)
}

fn parse(v: &Value) -> R<Config> {
 let mut cfg = Config {
  enabled: v["enabled"].as_bool().unwrap_or(false),
  queue_limit: num(v, "queueLimit", 1, MAX_QUEUE_LIMIT, 3),
  bindings: Vec::new(),
  legacy_map: HashMap::new(),
  obs_checked: v["obsChecked"].as_bool().unwrap_or(false),
  queue_notice: v["queueNotice"].as_str().map(|s| s.trim().chars().take(300).collect()).unwrap_or_else(default_queue_notice),
 };
 let list = v["bindings"].as_array().cloned().unwrap_or_default();
 if list.len() > MAX_BINDINGS {
  return Err(format!("No máximo {MAX_BINDINGS} vínculos de resgate."));
 }
 let mut seen_reward: Vec<String> = Vec::new();
 for item in &list {
  let kind = item["kind"].as_str().unwrap_or("reward").trim().to_owned();
  if kind != "reward" && kind != "command" {
   return Err(format!("Acionamento desconhecido: {kind}"));
  }
  let enabled = item["enabled"].as_bool().unwrap_or(true);
  let voice_id = item["voiceId"].as_str().unwrap_or("").trim().to_owned();
  if voice_id.is_empty() || voice_id.len() > 64 {
   return Err("Cada troca de voz precisa de uma voz escolhida na lista do Voicemod.".into());
  }
  if voice_id == vm::NORMAL_VOICE {
   return Err("\"Efeito desligado\" não é efeito: escolha uma voz de verdade.".into());
  }
  let concurrency = item["concurrency"].as_str().unwrap_or("queue").to_owned();
  if !CONCURRENCY.contains(&concurrency.as_str()) {
   return Err(format!("Política de concorrência desconhecida: {concurrency}"));
  }
  let mut binding = Binding {
   kind: kind.clone(),
   reward_id: String::new(),
   reward_name: item["rewardName"].as_str().unwrap_or("").chars().take(80).collect(),
   command: String::new(),
   permission: default_permission(),
   enabled,
   voice_id,
   voice_name: item["voiceName"].as_str().unwrap_or("").chars().take(80).collect(),
   duration_secs: item["durationSec"].as_u64().map(|n| n.min(MAX_DURATION)).unwrap_or(default_duration()),
   cooldown_secs: item["cooldownSec"].as_u64().map(|n| n.min(MAX_COOLDOWN)).unwrap_or(0),
   concurrency,
  };
  if kind == "command" {
   let key = command_key(item["command"].as_str().unwrap_or(""));
   if key.is_empty() || key.len() > 40 || key.split_whitespace().count() != 1 {
    return Err("O comando do chat é uma palavra curta, tipo \"!robô\" (sem espaços).".into());
   }
   if seen_reward.contains(&key) {
    return Err("Cada comando pode ter uma voz só: remova a regra duplicada.".into());
   }
   let permission = item["permission"].as_str().unwrap_or("moderator").trim().to_owned();
   if !PERMISSIONS.contains(&permission.as_str()) {
    return Err(format!("Permissão desconhecida: {permission}"));
   }
   seen_reward.push(key.clone());
   binding.command = key;
   binding.permission = permission;
  } else {
   let reward_id = item["rewardId"].as_str().unwrap_or("").trim().to_owned();
   if reward_id.is_empty() || reward_id.len() > 64 {
    return Err("Cada recompensa precisa de um reward escolhido na lista do canal.".into());
   }
   if seen_reward.contains(&reward_id) {
    return Err("Cada recompensa pode ter uma voz só: remova a regra duplicada.".into());
   }
   seen_reward.push(reward_id.clone());
   binding.reward_id = reward_id;
  }
  cfg.bindings.push(binding);
 }
 if let Some(map) = v["legacyMap"].as_object() {
  for (k, val) in map {
   let k = k.trim().to_lowercase();
   let val = val.as_str().unwrap_or("").trim().to_owned();
   if !k.is_empty() && !val.is_empty() && k.len() <= 40 && val.len() <= 64 {
    cfg.legacy_map.insert(k, val);
   }
  }
 }
 Ok(cfg)
}

/// Regra do módulo: o que a tela pode gravar sem inventar estado.
pub fn valid_config(v: &Value) -> bool {
 parse(v).is_ok()
}

// ---------------------------------------------------------------- estado

#[derive(Clone)]
struct Active {
 reward_id: String,
 reward_name: String,
 voice_id: String,
 voice_name: String,
 origin_voice: String,
 origin_name: String,
 origin_changer: Option<bool>,
 /// Changer ligado por nós para o efeito ficar audível; devolvemos como estava.
 changed_changer: bool,
 ends_at: Option<Instant>,
 duration_secs: u64,
 user: String,
}

#[derive(Clone)]
struct Queued {
 reward_id: String,
 reward_name: String,
 voice_id: String,
 voice_name: String,
 duration_secs: u64,
 redemption_id: String,
 profile_id: String,
 user: String,
}

#[derive(Default)]
struct State {
 active: Option<Active>,
 upcoming: VecDeque<Queued>,
 /// Efeito interrompido (queda do Voicemod): restauração pendente, nunca
 /// prometida em voz alta enquanto não houver conexão de novo.
 interrupted: Option<Active>,
 seen: VecDeque<String>,
 last_fire: HashMap<String, Instant>,
 last: String,
 last_kind: String,
}

pub struct Engine {
 state: Mutex<State>,
 /// Qualquer troca nova avança a geração: o temporizador antigo não desfaz
 /// o que um comando mais novo aplicou.
 gen: AtomicU64,
 gate: tokio::sync::Mutex<()>,
 #[cfg(test)]
 fulfilled: Mutex<Vec<(String, String)>>,
}

impl Default for Engine {
 fn default() -> Self {
  Self {
   state: Mutex::new(State::default()),
   gen: AtomicU64::new(0),
   gate: tokio::sync::Mutex::new(()),
   #[cfg(test)]
   fulfilled: Mutex::new(Vec::new()),
  }
 }
}

fn view_locked(st: &State) -> Value {
 let now = Instant::now();
 let active = st.active.as_ref().map(|a| {
  json!({
   "rewardId": a.reward_id, "rewardName": a.reward_name,
   "voiceId": a.voice_id, "voiceName": a.voice_name,
   "user": a.user, "durationSecs": a.duration_secs,
   "remainingMs": a.ends_at.map(|e| e.saturating_duration_since(now).as_millis() as u64).unwrap_or(0),
   "forever": a.ends_at.is_none(),
  })
 });
 let upcoming: Vec<Value> = st.upcoming.iter().map(|q| {
  json!({
   "rewardId": q.reward_id, "rewardName": q.reward_name,
   "voiceId": q.voice_id, "voiceName": q.voice_name,
   "user": q.user, "durationSecs": q.duration_secs,
  })
 }).collect();
 json!({
  "active": active,
  "upcoming": upcoming,
  "last": st.last,
  "lastKind": st.last_kind,
  "pendingRestore": st.interrupted.is_some(),
  "busy": st.active.is_some() || !st.upcoming.is_empty() || st.interrupted.is_some(),
 })
}

pub fn view(rt: &Runtime, p: &Profile) -> Value {
 let queue = view_locked(&rt.voice_resgates.state.lock().unwrap());
 json!({ "queue": queue, "config": load(&rt.db, &p.id) })
}

fn publish(rt: &Runtime) {
 let queue = view_locked(&rt.voice_resgates.state.lock().unwrap());
 rt.emit(KIND, queue);
}

/// Um efeito de voz ocupa o Voicemod agora? Usado pela voz interna para não
/// dividir o microfone (um caminho por vez).
pub fn busy(rt: &Runtime) -> bool {
 let st = rt.voice_resgates.state.lock().unwrap();
 st.active.is_some() || !st.upcoming.is_empty() || st.interrupted.is_some()
}

/// Exatamente um efeito aplicado (não conta fila nem pendência).
pub fn has_active(rt: &Runtime) -> bool {
 rt.voice_resgates.state.lock().unwrap().active.is_some()
}

/// Voz mapeada de um efeito interno antigo, se o dono escolheu uma.
pub fn legacy_voice(rt: &Runtime, profile_id: &str, effect: &str) -> Option<String> {
 load(&rt.db, profile_id).legacy_map.get(&effect.trim().to_lowercase()).cloned()
}

// ---------------------------------------------------------------- resgate

/// Entrada do motor para um resgate real. Devolve Some quando o reward tem
/// vínculo de voz (o resgate passa a ser da conta do motor de voz).
pub async fn on_redemption(rt: &Arc<Runtime>, p: &Profile, e: &Event) -> Option<String> {
 let cfg = load(&rt.db, &p.id);
 if !cfg.enabled {
  return None;
 }
 let reward_id = e.data["reward"]["id"].as_str().unwrap_or("").to_owned();
 let binding = cfg.bindings.iter().find(|b| b.kind == "reward" && b.enabled && b.reward_id == reward_id)?.clone();
 let redemption_id = e.data["id"].as_str().unwrap_or("").to_owned();
 if redemption_id.is_empty() {
  rt.log(&p.id, KIND, "Resgate sem ID de resgate no payload: não vou cumprir às cegas.", "error");
  return Some("Resgate sem ID reconhecível".into());
 }
 // Deduplicação por ID do resgate, por cima do message_id do EventSub.
 let repeated = {
  let mut st = rt.voice_resgates.state.lock().unwrap();
  let repeated = st.seen.iter().any(|x| x == &redemption_id);
  if !repeated {
   st.seen.push_back(redemption_id.clone());
   while st.seen.len() > MAX_SEEN {
    st.seen.pop_front();
   }
  }
  repeated
 };
 if repeated {
  rt.log(&p.id, KIND, "Resgate repetido (mesmo ID): já atendido, nada aplicado duas vezes.", "info");
  return Some("Resgate repetido".into());
 }
 let user = e.user.clone();
 let item = Queued {
  reward_id: binding.reward_id.clone(),
  reward_name: binding.reward_name.clone(),
  voice_id: binding.voice_id.clone(),
  voice_name: binding.voice_name.clone(),
  duration_secs: if binding.duration_secs == 0 { 0 } else { binding.duration_secs },
  redemption_id,
  profile_id: p.id.clone(),
  user,
 };
 let policy = binding.concurrency.clone();
 let cooldown = binding.cooldown_secs;
 let (rt2, p2) = (rt.clone(), p.clone());
 let decision = {
  let st = rt.voice_resgates.state.lock().unwrap();
  if let Some(t) = st.last_fire.get(&binding.reward_id) {
   if t.elapsed().as_secs() < cooldown {
    Some(format!("{} está em intervalo ({cooldown}s entre resgates)", binding.reward_name))
   } else {
    None
   }
  } else {
   None
  }
 };
 if let Some(msg) = decision {
  rt.log(&p.id, KIND, &msg, "info");
  return Some(msg);
 }
 tokio::spawn(async move { enqueue(&rt2, &p2, item, policy, cooldown).await });
 Some(format!("Resgate de voz recebido: {}", binding.voice_name))
}

/// Entrada do motor para um comando do chat que troca a voz. Não é resgate:
/// não gasta pontos nem marca nada como cumprido — só usa o mesmo serviço de
/// troca, fila, duração e restauração dos resgates. Simulação nunca troca voz.
pub async fn on_chat(rt: &Arc<Runtime>, p: &Profile, e: &Event) {
 if e.kind != "chat" || e.simulated {
  return;
 }
 let cfg = load(&rt.db, &p.id);
 if !cfg.enabled {
  return;
 }
 let Some(binding) = cfg
  .bindings
  .iter()
  .find(|b| b.kind == "command" && b.enabled && command_matches(&b.command, &e.message))
  .cloned()
 else {
  return;
 };
 if !permission_ok(&binding.permission, &e.role) {
  let who = match binding.permission.as_str() {
   "broadcaster" => "só o broadcaster",
   "moderator" => "moderadores e o broadcaster",
   _ => "qualquer pessoa",
  };
  rt.log(&p.id, KIND, &format!("{} pediu o comando de voz {}, mas ele é para {who}.", e.user, binding.command), "info");
  return;
 }
 let key = format!("{}:cmd:{}", p.id, binding.command);
 let decision = {
  let st = rt.voice_resgates.state.lock().unwrap();
  st.last_fire
   .get(&key)
   .filter(|t| t.elapsed().as_secs() < binding.cooldown_secs)
   .map(|_| format!("{} está em intervalo ({}s entre usos).", binding.command, binding.cooldown_secs))
 };
 if let Some(msg) = decision {
  rt.log(&p.id, KIND, &msg, "info");
  return;
 }
 let item = Queued {
  reward_id: key,
  reward_name: format!("Comando {}", binding.command),
  voice_id: binding.voice_id.clone(),
  voice_name: binding.voice_name.clone(),
  duration_secs: binding.duration_secs,
  redemption_id: String::new(),
  profile_id: p.id.clone(),
  user: e.user.clone(),
 };
 let policy = binding.concurrency.clone();
 let cooldown = binding.cooldown_secs;
 let (rt2, p2) = (rt.clone(), p.clone());
 tokio::spawn(async move { enqueue(&rt2, &p2, item, policy, cooldown).await });
}

/// Decide fila ou aplicação com o portão em mãos (uma troca por vez).
async fn enqueue(rt: &Arc<Runtime>, p: &Profile, item: Queued, policy: String, cooldown: u64) {
 let _gate = rt.voice_resgates.gate.lock().await;
 if !rt.voicemod.connected() {
  fail(rt, p, &item, "Voicemod desconectado: o resgate não foi atendido e não foi marcado como cumprido.");
  return;
 }
 if rt.voice_fx.is_running() {
  fail(rt, p, &item, "A Voz da live interna (efeitos do BotLive) está ativa. Pare-a antes de usar resgates do Voicemod: os dois caminhos capturariam o seu microfone.");
  return;
 }
 if rt.voicemod.in_use() {
  fail(rt, p, &item, "Há um teste de voz no Voicemod. Aguarde terminar antes de aplicar um resgate.");
  return;
 }
 if let Err(err) = vm::voice_known(rt, &item.voice_id) {
  fail(rt, p, &item, &err);
  return;
 }
 let limit = queue_limit(rt, p) as usize;
 let queued = {
  let mut st = rt.voice_resgates.state.lock().unwrap();
  if let Some(t) = st.last_fire.get(&item.reward_id) {
   if t.elapsed().as_secs() < cooldown {
    let msg = format!("{} está em intervalo: nada aplicado.", item.reward_name);
    drop(st);
    rt.log(&p.id, KIND, &msg, "info");
    return;
   }
  }
  let has_active = st.active.is_some();
  if has_active {
   match policy.as_str() {
    "ignore" => {
     let msg = "Já há um efeito de voz ativo e este vínculo não empilha: nada aplicado.".to_owned();
     drop(st);
     note(rt, p, &msg, "info");
     return;
    },
    "queue" => {
     if st.upcoming.len() >= limit {
      let msg = "Fila de vozes cheia: resgate não atendido (não cumprido).".to_owned();
      drop(st);
      note(rt, p, &msg, "info");
      return;
     }
     let pos = st.upcoming.len() + 1;
     st.upcoming.push_back(item.clone());
     let msg = format!("{} entrou na fila de vozes ({}º).", item.voice_name, pos);
     drop(st);
     publish(rt);
     rt.log(&p.id, KIND, &msg, "info");
     Some(pos)
    },
    _ => None, // replace: segue e sobrescreve, mantendo a voz de antes
   }
  } else {
   None
  }
 };
 if let Some(pos) = queued {
  send_queue_notice(rt, p, &item, pos).await;
  return;
 }
 let (rt2, p2) = (rt.clone(), p.clone());
 let _ = apply_locked(&rt2, &p2, &item).await;
}

fn queue_limit(rt: &Runtime, p: &Profile) -> u64 {
 load(&rt.db, &p.id).queue_limit.clamp(1, MAX_QUEUE_LIMIT)
}

/// Texto do aviso de fila: {{user}}, {{pos}} e {{voz}} viram o que foi pedido.
pub fn queue_notice_text(template: &str, user: &str, pos: usize, voice: &str) -> String {
 template.replace("{{user}}", user).replace("{{pos}}", &pos.to_string()).replace("{{voz}}", voice)
}

/// Avisa quem resgatou que o pedido entrou na fila e começa em seguida. A
/// mensagem é configurável ("Mensagem quando o resgate entra na fila"); vazio
/// desliga o aviso. Sem nome de quem resgatou, não há o que dirigir.
async fn send_queue_notice(rt: &Arc<Runtime>, p: &Profile, item: &Queued, pos: usize) {
 let cfg = load(&rt.db, &p.id);
 let template = cfg.queue_notice.trim();
 if template.is_empty() || item.user.trim().is_empty() { return; }
 let text = queue_notice_text(template, &item.user, pos, &item.voice_name);
 let e = Event { id: uuid::Uuid::new_v4().to_string(), profile_id: p.id.clone(), kind: "voice".into(), user: item.user.clone(), user_id: String::new(), role: "everyone".into(), message: String::new(), data: Value::Null, simulated: false, rehearsal: false };
 let _ = rt.send(p, &e, &text).await;
}

fn note(rt: &Runtime, p: &Profile, msg: &str, status: &str) {
 rt.log(&p.id, KIND, msg, status);
 {
  let mut st = rt.voice_resgates.state.lock().unwrap();
  st.last = msg.to_owned();
  st.last_kind = match status { "error" => "error", "success" => "ok", _ => "skip" }.into();
 }
 publish(rt);
}

fn fail(rt: &Runtime, p: &Profile, item: &Queued, msg: &str) {
 // Nunca fingir cumprido: o resgate fica sem resposta nossa e o log diz por quê.
 note(rt, p, &format!("{} · {}", item.reward_name, msg), "error");
}

/// Aplica a voz do item com o portão em mãos. Só a primeira troca captura a
/// origem: continuações herdam a mesma, senão a restauração devolveria a voz
/// de um efeito, não a de antes.
async fn apply_locked(rt: &Arc<Runtime>, p: &Profile, item: &Queued) -> R<()> {
 if rt.voice_resgates.state.lock().unwrap().interrupted.is_some() {
  let err = "Há restauração pendente: use Recuperar antes de aplicar outra voz.";
  fail(rt, p, item, err);
  return Err(err.into());
 }

 // Origem: só a primeira troca captura. Continuações herdam a mesma origem,
 // senão a restauração devolveria a voz de um efeito, não a de antes.
 let inherited = {
  let st = rt.voice_resgates.state.lock().unwrap();
  st.active.as_ref().map(|a| (a.origin_voice.clone(), a.origin_name.clone(), a.origin_changer))
 };
 let continuing = inherited.is_some();
 let (origin_voice, origin_name, origin_changer) = match inherited {
  Some(t) => t,
  None => {
   let cur = vm::current_voice_id(rt).await?;
   let name = vm::voice_name(rt, &cur);
   let changer = vm::read_changer(rt).await.unwrap_or(None);
   (cur, name, changer)
  },
 };
 let mut changed_changer = false;
 if origin_changer == Some(false) {
  // Efeito soado no Voicemod exige o Voice Changer ligado; devolvemos como estava.
  vm::set_changer(rt, true).await.map_err(|e| format!("Não consegui ligar o Voice Changer: {e}"))?;
  changed_changer = true;
 }
 if let Err(err) = vm::load_confirmed(rt, &item.voice_id).await {
  if changed_changer && !continuing {
   vm::set_changer(rt, false).await.ok();
  }
  fail(rt, p, item, &err);
  return Err(err);
 }
 let voice_name = vm::voice_name(rt, &item.voice_id);
 let gen = {
  let mut st = rt.voice_resgates.state.lock().unwrap();
  let ends_at = if item.duration_secs == 0 { None } else { Some(Instant::now() + Duration::from_secs(item.duration_secs)) };
  let replacing = st.active.is_some();
  st.active = Some(Active {
   reward_id: item.reward_id.clone(),
   reward_name: item.reward_name.clone(),
   voice_id: item.voice_id.clone(),
   voice_name: voice_name.clone(),
   origin_voice: origin_voice.clone(),
   origin_name: origin_name.clone(),
   origin_changer,
   changed_changer,
   ends_at,
   duration_secs: item.duration_secs,
   user: item.user.clone(),
  });
  st.last_fire.insert(item.reward_id.clone(), Instant::now());
  let msg = if replacing {
   format!("{voice_name} no lugar do efeito anterior ({}), por {}", item.user, when(item.duration_secs))
  } else {
   format!("{voice_name} por {} (resgate de {})", when(item.duration_secs), item.user)
  };
  st.last = msg.clone();
  st.last_kind = "ok".into();
  let gen = rt.voice_resgates.gen.fetch_add(1, Ordering::SeqCst) + 1;
  drop(st);
  rt.log(&p.id, KIND, &msg, "success");
  gen
 };
 publish(rt);
 // Cumprido só agora: a troca foi confirmada pelo próprio Voicemod.
 if !item.redemption_id.is_empty() {
  fulfill(rt, p, &item.reward_id, &item.redemption_id).await;
 }
 let secs = item.duration_secs;
 if secs > 0 {
  let rt2 = rt.clone();
  let profile_id = p.id.clone();
  tokio::spawn(async move {
   tokio::time::sleep(Duration::from_secs(secs.max(1))).await;
   if rt2.voice_resgates.gen.load(Ordering::SeqCst) != gen {
    // Um comando mais novo assumiu: este temporizador não desfaz nada.
    return;
   }
   let _gate = rt2.voice_resgates.gate.lock().await;
   if rt2.voice_resgates.gen.load(Ordering::SeqCst) != gen {
    return;
   }
   finish(rt2.clone(), profile_id).await;
  });
 }
 Ok(())
}

fn when(secs: u64) -> String {
 if secs == 0 { "até o próximo efeito ou cancelamento".into() }
 else if secs % 3600 == 0 { format!("{}h", secs / 3600) }
 else if secs % 60 == 0 { format!("{}min", secs / 60) }
 else { format!("{secs}s") }
}

/// Restauração completa: só confirma sucesso após voz e modificador.
async fn restore_origin(rt: &Arc<Runtime>, active: &Active) -> R<()> {
 vm::load_confirmed(rt, &active.origin_voice).await?;
 if active.changed_changer {
  if let Some(want) = active.origin_changer {
   vm::set_changer(rt, want).await?;
  }
 }
 Ok(())
}

/// Fim do efeito: devolve a voz de antes, respeita troca manual do usuário e
/// segue a fila quando houver. Sempre com o portão em mãos e geração conferida.
/// O futuro vem caixado de propósito: `finish` e `apply_locked` se chamam uma
/// pela outra, e sem a caixa a pergunta "isto é Send?" entra em ciclo e o
/// compilador recusa o agendamento com resposta conservadora.
fn finish(rt: Arc<Runtime>, profile_id: String) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
 Box::pin(async move {
  let rt = &rt;
  let profile_id = profile_id.as_str();
  // Troca manual no aplicativo: o usuário escolheu outra voz e não será
  // sobrescrito. O sinal é consumido aqui (uma vez).
  let manual = rt.voicemod.take_manual();
  let active = {
   let mut st = rt.voice_resgates.state.lock().unwrap();
   st.active.take()
  };
  let Some(active) = active else { return };
  if manual {
   let msg = format!("Você trocou a voz no Voicemod durante {}; nada foi sobrescrito. Sua escolha vale.", active.voice_name);
   rt.log(profile_id, KIND, &msg, "info");
   {
    let mut st = rt.voice_resgates.state.lock().unwrap();
    st.last = msg;
    st.last_kind = "skip".into();
    st.upcoming.clear();
   }
   publish(rt);
   return;
  }
  if !rt.voicemod.connected() {
   // Queda no meio: guarda a promessa de restauração e diz a verdade.
   let msg = format!("A conexão com o Voicemod caiu durante {}. A restauração da voz de antes fica pendente até reconectar e pedir Recuperar.", active.origin_name);
   rt.log(profile_id, KIND, &msg, "error");
   {
    let mut st = rt.voice_resgates.state.lock().unwrap();
    st.interrupted = Some(active.clone());
    st.last = msg;
    st.last_kind = "error".into();
   }
   publish(rt);
   return;
  }
  // Próximo da fila: aplica sem restaurar a origem (a origem continua a mesma).
  let next = { rt.voice_resgates.state.lock().unwrap().upcoming.pop_front() };
  if let Some(next) = next {
   let item = Queued { reward_id: next.reward_id.clone(), voice_id: next.voice_id.clone(), duration_secs: next.duration_secs, redemption_id: next.redemption_id.clone(), reward_name: next.reward_name.clone(), voice_name: next.voice_name.clone(), profile_id: next.profile_id.clone(), user: next.user.clone() };
   // Reinsere a origem atual para a continuação herdar a mesma.
   {
    let mut st = rt.voice_resgates.state.lock().unwrap();
    st.active = Some(active.clone());
   }
   let result = match rt.db.profile(profile_id) {
    Ok(p) => apply_locked(rt, &p, &item).await,
    Err(err) => Err(err),
   };
   if let Err(err) = result {
    // A origem continua guardada; sem temporizador novo, não deixe um efeito
    // antigo preso como ativo nem repita pedidos depois da recuperação.
    let msg = format!("A próxima voz falhou: {err}. Fila interrompida; use Recuperar para devolver a voz de antes.");
    {
     let mut st = rt.voice_resgates.state.lock().unwrap();
     st.active = None;
     st.interrupted = Some(active);
     st.upcoming.clear();
     st.last = msg.clone();
     st.last_kind = "error".into();
    }
    rt.log(profile_id, KIND, &msg, "error");
    publish(rt);
   }
   return;
  }
  // Fim da fila: devolve a voz de antes, com confirmação.
  let restore = restore_origin(rt, &active).await;
  match restore {
   Ok(()) => {
    rt.voicemod.forget_applied();
    let msg = if active.origin_voice == vm::NORMAL_VOICE {
     format!("Efeito encerrado: voz normal restaurada ({}).", active.origin_name)
    } else {
     format!("Efeito encerrado: voz {} de antes restaurada.", active.origin_name)
    };
    rt.log(profile_id, KIND, &msg, "success");
    {
     let mut st = rt.voice_resgates.state.lock().unwrap();
     st.last = msg;
     st.last_kind = "ok".into();
    }
   },
   Err(err) => {
    let msg = format!("Não consegui devolver a voz de antes ({}): {err}", active.origin_name);
    rt.log(profile_id, KIND, &msg, "error");
    {
     let mut st = rt.voice_resgates.state.lock().unwrap();
     st.interrupted = Some(active.clone());
     st.last = msg;
     st.last_kind = "error".into();
    }
   },
  }
  publish(rt);
 })
}

/// Marca o resgate como cumprido na Twitch. Testes registram a chamada;
/// produção só reporta sucesso depois do PATCH aceito pela Helix.
async fn fulfill(rt: &Arc<Runtime>, p: &Profile, reward: &str, redemption: &str) {
 #[cfg(test)]
 {
  let _ = p;
  rt.voice_resgates.fulfilled.lock().unwrap().push((reward.to_owned(), redemption.to_owned()));
  return;
 }
 #[allow(unreachable_code)]
 {
  match crate::twitch_ops::fulfill_redemption(rt, p, reward, redemption).await {
   Ok(_) => rt.log(&p.id, KIND, "Resgate marcado como cumprido depois da voz confirmada", "success"),
   Err(err) => rt.log(&p.id, KIND, &format!("Voz aplicada, mas o cumprimento falhou: {err}"), "error"),
  }
 }
}

// ---------------------------------------------------------------- comandos da tela

/// Encerra tudo e devolve a voz de antes, confirmada (botão "Encerrar efeitos
/// e restaurar minha voz"). Se a pessoa estava numa voz com efeito antes, é
/// essa voz que volta — nunca prometemos "voz normal" por engano. Mata a
/// geração primeiro: nenhum temporizador antigo pode religar uma voz.
pub async fn cancel_all(rt: &Arc<Runtime>, p: &Profile) -> R<Value> {
 rt.voice_resgates.gen.fetch_add(1, Ordering::SeqCst);
 let _gate = rt.voice_resgates.gate.lock().await;
 let taken = {
  let mut st = rt.voice_resgates.state.lock().unwrap();
  let taken = st.active.take().or_else(|| st.interrupted.take());
  st.upcoming.clear();
  taken
 };
 match taken {
  None => {
   let msg = "Fila de vozes limpa: nenhum efeito ativo, sua voz no Voicemod não foi mexida.";
   rt.log(&p.id, KIND, msg, "success");
   note(rt, p, msg, "success");
  },
  Some(active) => {
   if !rt.voicemod.connected() {
    // Sem conexão não há como confirmar nada: a promessa de restauração fica.
    {
     let mut st = rt.voice_resgates.state.lock().unwrap();
     st.interrupted = Some(active.clone());
    }
    let msg = "Efeitos encerrados, mas o Voicemod está desconectado: a restauração da voz de antes fica pendente até reconectar e pedir Recuperar.".to_owned();
    rt.log(&p.id, KIND, &msg, "error");
    note(rt, p, &msg, "error");
   } else {
    match restore_origin(rt, &active).await {
     Ok(()) => {
      rt.voicemod.forget_applied();
      let msg = if active.origin_voice == vm::NORMAL_VOICE {
       "Efeitos encerrados: voz sem efeito restaurada.".to_owned()
      } else {
       format!("Efeitos encerrados: voz {} de antes restaurada.", active.origin_name)
      };
      rt.log(&p.id, KIND, &msg, "success");
      note(rt, p, &msg, "success");
     },
     Err(err) => {
      {
       let mut st = rt.voice_resgates.state.lock().unwrap();
       st.interrupted = Some(active.clone());
      }
      let msg = format!("Não consegui devolver a voz de antes ({}): {err}", active.origin_name);
      rt.log(&p.id, KIND, &msg, "error");
      note(rt, p, &msg, "error");
      return Err(msg);
     },
    }
   }
  },
 }
 Ok(serde_json::to_value(view(rt, p)).map_err(|_| "Falha ao montar o estado".to_string())?)
}

/// Recupera a restauração pendente depois de reconectar. Nada é prometido
/// antes: sem conexão não há como devolver a voz de antes.
pub async fn recover(rt: &Arc<Runtime>, p: &Profile) -> R<Value> {
 let pending = rt.voice_resgates.state.lock().unwrap().interrupted.clone();
 let Some(pending) = pending else {
  return Ok(serde_json::to_value(view(rt, p)).map_err(|_| "Falha ao montar o estado".to_string())?);
 };
 if !rt.voicemod.connected() {
  return Err("O Voicemod continua desconectado. Abra o aplicativo, clique em Conectar e tente de novo.".into());
 }
 let _gate = rt.voice_resgates.gate.lock().await;
 match restore_origin(rt, &pending).await {
  Ok(()) => {
   rt.voicemod.forget_applied();
   {
    let mut st = rt.voice_resgates.state.lock().unwrap();
    st.interrupted = None;
   }
   let msg = format!("Voz {} devolvida após a reconexão.", pending.origin_name);
   rt.log(&p.id, KIND, &msg, "success");
   note(rt, p, &msg, "success");
  },
  Err(err) => {
   let msg = format!("Ainda não consegui devolver a voz de antes: {err}");
   rt.log(&p.id, KIND, &msg, "error");
   return Err(msg);
  },
 }
 Ok(serde_json::to_value(view(rt, p)).map_err(|_| "Falha ao montar o estado".to_string())?)
}

/// Aplicação direta, sem resgate: ação "Mudar minha voz" em modo Voicemod e
/// teste da tela de resgates. Mesmo portão, mesma restauração com geração.
pub async fn apply_manual(rt: &Arc<Runtime>, p: &Profile, voice_id: &str, origin_label: &str, secs: u64, user: &str) -> R<String> {
 let voice_id = voice_id.trim().to_owned();
 if voice_id.is_empty() {
  return Err("Escolha uma voz do Voicemod.".into());
 }
 let secs = secs.min(MAX_DURATION);
 if rt.voice_fx.is_running() {
  return Err("A Voz da live interna está ativa. Pare-a antes de aplicar vozes do Voicemod: os dois caminhos capturariam o seu microfone.".into());
 }
 if rt.voicemod.in_use() {
  return Err("Há um teste de voz em andamento no Voicemod. Aguarde ou encerre-o.".into());
 }
 if !rt.voicemod.connected() {
  return Err(vm::NOT_CONNECTED.into());
 }
 vm::voice_known(rt, &voice_id)?;
 let _gate = rt.voice_resgates.gate.lock().await;
 let item = Queued {
  reward_id: String::new(),
  reward_name: origin_label.to_owned(),
  voice_id,
  voice_name: String::new(),
  duration_secs: secs,
  redemption_id: String::new(),
  profile_id: p.id.clone(),
  user: user.trim().to_owned(),
 };
 // Automações respeitam a mesma fila dos resgates: cada efeito cumpre o seu
 // tempo e os próximos esperam — ninguém é cancelado no meio. O efeito "até a
 // próxima troca" (duração 0) é, pela própria definição, substituído pelo que
 // vier depois.
 let queued = {
  let mut st = rt.voice_resgates.state.lock().unwrap();
  if st.active.as_ref().is_some_and(|a| a.duration_secs > 0) {
   let limit = queue_limit(rt, p) as usize;
   if st.upcoming.len() >= limit {
    return Err("Fila de vozes cheia: aguarde o efeito atual terminar.".into());
   }
   let pos = st.upcoming.len() + 1;
   st.upcoming.push_back(item.clone());
   Some(pos)
  } else {
   None
  }
 };
 let name = vm::voice_name(rt, &item.voice_id);
 if let Some(pos) = queued {
  publish(rt);
  send_queue_notice(rt, p, &item, pos).await;
  return Ok(format!("{name} entrou na fila de vozes ({pos}º)"));
 }
 apply_locked(rt, p, &item).await?;
 let msg = if secs == 0 { format!("{name} aplicada até a próxima troca") } else { format!("{name} por {}", when(secs)) };
 Ok(msg)
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::voicemod::testkit::*;
 use serde_json::json;

 fn cfg_json(bindings: &[Value], queue_limit: u64) -> Value {
  json!({"enabled": true, "queueLimit": queue_limit, "bindings": bindings})
 }
 fn binding(reward: &str, voice: &str, secs: u64, cooldown: u64, concurrency: &str) -> Value {
  json!({"rewardId": reward, "rewardName": "Voz engraçada", "voiceId": voice, "voiceName": voice, "durationSec": secs, "cooldownSec": cooldown, "concurrency": concurrency})
 }
 fn command_binding(command: &str, permission: &str, voice: &str, secs: u64, cooldown: u64, concurrency: &str) -> Value {
  json!({"kind": "command", "command": command, "permission": permission, "voiceId": voice, "voiceName": voice, "durationSec": secs, "cooldownSec": cooldown, "concurrency": concurrency})
 }
 fn chat(profile: &str, text: &str, role: &str) -> Event {
  Event {
   id: format!("chat-{}", uuid::Uuid::new_v4()),
   profile_id: profile.into(),
   kind: "chat".into(),
   user: "Ana".into(),
   user_id: "77".into(),
   role: role.into(),
   message: text.into(),
   data: Value::Null,
   simulated: false,
   rehearsal: false,
  }
 }
 fn chat_sim(profile: &str, text: &str, role: &str) -> Event {
  Event { simulated: true, ..chat(profile, text, role) }
 }
 fn redeem(profile: &str, reward: &str, redemption: &str, user: &str) -> Event {
  Event {
   id: format!("msg-{redemption}"),
   profile_id: profile.into(),
   kind: "redemption".into(),
   user: user.into(),
   user_id: "77".into(),
   role: "viewer".into(),
   message: String::new(),
   data: json!({"id": redemption, "reward": {"id": reward, "title": "Voz engraçada"}, "user_name": user}),
   simulated: false,
   rehearsal: false,
  }
 }
 async fn fired(rt: &Runtime, n: usize) -> bool {
  for _ in 0..200 {
   if rt.voice_resgates.fulfilled.lock().unwrap().len() >= n {
    return true;
   }
   tokio::time::sleep(Duration::from_millis(25)).await;
  }
  false
 }
 async fn connected_runtime() -> (tempfile::TempDir, Arc<Runtime>, Profile, Mock) {
  let (dir, rt, p) = setup().await;
  let (port, m, _accept) = mock_server().await;
  boot(&rt, port);
  crate::voicemod::connect(&rt, &p).await.unwrap();
  (dir, rt, p, m)
 }
 async fn setup_with(rt: &Runtime, p: &Profile, cfg: Value) {
  crate::voice_redemptions::save(rt, p, &cfg).unwrap();
 }

 #[tokio::test]
 async fn refused_modifier_restore_stays_pending_until_confirmed_recovery() {
  let (_dir, rt, p, m) = connected_runtime().await;
  apply_manual(&rt, &p, "robot", "QA", 60, "tester").await.unwrap();
  assert!(m.changer());
  m.refuse_toggle.store(true, Ordering::SeqCst);
  assert!(cancel_all(&rt, &p).await.is_err(), "não pode declarar restauração completa");
  assert_eq!(view(&rt, &p)["queue"]["pendingRestore"], json!(true));
  assert!(m.changer(), "modificador recusou desligar");
  assert!(recover(&rt, &p).await.is_err(), "recuperação também exige confirmação");
  assert_eq!(view(&rt, &p)["queue"]["pendingRestore"], json!(true));
  m.refuse_toggle.store(false, Ordering::SeqCst);
  recover(&rt, &p).await.unwrap();
  assert!(!m.changer());
  assert_eq!(view(&rt, &p)["queue"]["pendingRestore"], json!(false));
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn failed_next_voice_preserves_origin_and_does_not_leave_a_stuck_active_effect() {
  let (_dir, rt, p, m) = connected_runtime().await;
  apply_manual(&rt, &p, "robot", "QA", 60, "tester").await.unwrap();
  apply_manual(&rt, &p, "alien", "QA", 60, "tester").await.unwrap();
  apply_manual(&rt, &p, "robot", "QA", 60, "tester").await.unwrap();
  *m.refuse_voice.lock().unwrap() = "alien".into();
  {
   let _gate = rt.voice_resgates.gate.lock().await;
   finish(rt.clone(), p.id.clone()).await;
  }
  let v = view(&rt, &p);
  assert_eq!(v["queue"]["pendingRestore"], json!(true));
  assert!(v["queue"]["active"].is_null());
  assert!(v["queue"]["upcoming"].as_array().unwrap().is_empty());
  assert_eq!(m.voice(), "robot");
  assert!(m.changer(), "não deve desligar o efeito anterior durante continuação falha");
  assert!(apply_manual(&rt, &p, "robot", "QA", 60, "tester").await.is_err(),
   "pedido novo não pode sobrescrever a origem pendente");
  assert_eq!(view(&rt, &p)["queue"]["pendingRestore"], json!(true));
  recover(&rt, &p).await.unwrap();
  assert_eq!(m.voice(), "nofx");
  assert!(!m.changer());
  assert_eq!(view(&rt, &p)["queue"]["pendingRestore"], json!(false));
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn binding_applies_and_fulfills_with_the_redemption_id_not_the_message_id() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 60, 0, "queue")], 3)).await;
  let e = redeem(&p.id, "r1", "redem-111", "viewer1");
  assert!(on_redemption(&rt, &p, &e).await.is_some(), "reward com vínculo é do motor de voz");
  assert!(fired(&rt, 1).await, "troca confirmada cumprida");
  let done = rt.voice_resgates.fulfilled.lock().unwrap().clone();
  assert_eq!(done, vec![("r1".to_string(), "redem-111".to_string())], "cumprido pelo ID do resgate, não pelo message_id");
  assert_eq!(m.voice(), "robot");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn duplicated_redemption_id_is_never_applied_twice() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 60, 0, "queue")], 3)).await;
  let e = redeem(&p.id, "r1", "redem-222", "viewer1");
  on_redemption(&rt, &p, &e).await.unwrap();
  assert!(fired(&rt, 1).await);
  let again = redeem(&p.id, "r1", "redem-222", "viewer1");
  let msg = on_redemption(&rt, &p, &again).await.unwrap();
  assert!(msg.contains("repetido"), "{msg}");
  tokio::time::sleep(Duration::from_millis(200)).await;
  assert_eq!(m.loads(), vec!["robot".to_string()], "uma só aplicação para o mesmo ID");
  assert_eq!(rt.voice_resgates.fulfilled.lock().unwrap().len(), 1);
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn cooldown_blocks_the_second_redemption_of_the_same_reward() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 60, 3600, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-a", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  let msg = on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-b", "u2")).await.unwrap();
  assert!(msg.contains("intervalo"), "{msg}");
  tokio::time::sleep(Duration::from_millis(200)).await;
  assert_eq!(m.loads(), vec!["robot".to_string()]);
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn queue_policy_waits_and_the_next_voice_applies_after_the_first() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 1, 0, "queue"), binding("r2", "alien", 60, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  on_redemption(&rt, &p, &redeem(&p.id, "r2", "redem-2", "u2")).await.unwrap();
  // A fila é decidida na tarefa do portão: deixa ela entrar antes de olhar.
  tokio::time::sleep(Duration::from_millis(200)).await;
  let view = view(&rt, &p);
  assert_eq!(view["queue"]["upcoming"].as_array().unwrap().len(), 1, "segundo espera na fila visível");
  tokio::time::sleep(Duration::from_millis(2200)).await;
  assert_eq!(m.loads(), vec!["robot".to_string(), "alien".to_string()], "fila aplica na ordem");
  assert!(fired(&rt, 2).await, "cada troca confirmada cumpre o seu resgate");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn full_queue_refuses_instead_of_growing() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 600, 0, "queue")], 1)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-2", "u2")).await.unwrap();
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-3", "u3")).await.unwrap();
  tokio::time::sleep(Duration::from_millis(300)).await;
  let v = view(&rt, &p);
  assert_eq!(v["queue"]["upcoming"].as_array().unwrap().len(), 1, "fila limitada ao configurado");
  assert!(v["queue"]["last"].as_str().unwrap().contains("cheia"), "{}", v["queue"]["last"]);
  assert_eq!(m.loads().len(), 1, "só o primeiro aplicado");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn ignore_policy_skips_while_another_effect_runs_replace_overwrites() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 600, 0, "ignore")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-2", "u2")).await.unwrap();
  tokio::time::sleep(Duration::from_millis(250)).await;
  assert_eq!(m.loads().len(), 1, "ignore não empilha");
  let v = view(&rt, &p);
  assert!(v["queue"]["last"].as_str().unwrap().contains("não empilha"), "{}", v["queue"]["last"]);
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 600, 0, "replace")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-3", "u3")).await.unwrap();
  assert!(fired(&rt, 2).await, "replace cumpre o novo resgate");
  tokio::time::sleep(Duration::from_millis(250)).await;
  assert_eq!(m.loads().len(), 2, "replace sobrescreve sem empilhar");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn manual_voice_change_is_respected_and_restore_is_skipped() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 1, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  // Usuário troca no aplicativo: o Voicemod avisa por voiceChangedEvent.
  m.user_picks("alien");
  tokio::time::sleep(Duration::from_millis(2200)).await;
  assert_eq!(m.voice(), "alien", "a escolha do usuário permanece");
  let v = view(&rt, &p);
  assert!(v["queue"]["last"].as_str().unwrap().contains("sobrescrito"), "{}", v["queue"]["last"]);
  assert!(!v["queue"]["busy"].as_bool().unwrap());
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn drop_during_effect_keeps_restore_pending_until_recover() {
  let (_dir, rt, p, m) = connected_runtime().await;
  // Reconexão automática não pode socorrer antes do relógio do efeito: o
  // teste precisa ver a queda de verdade.
  rt.voicemod.set_delays(vec![60_000, 60_000]);
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 1, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  m.push(KILL.to_string());
  tokio::time::sleep(Duration::from_millis(2400)).await;
  let v = view(&rt, &p);
  assert_eq!(v["queue"]["pendingRestore"], json!(true), "restauração pendente, sem promessa");
  assert_eq!(v["queue"]["active"], serde_json::Value::Null);
  crate::voicemod::connect(&rt, &p).await.unwrap();
  let recovered = recover(&rt, &p).await.unwrap();
  assert_eq!(recovered["queue"]["pendingRestore"], json!(false));
  assert_eq!(m.voice(), "nofx", "origem era a voz normal");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn unknown_or_blocked_voice_is_refused_and_not_fulfilled() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "ghost", 60, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  tokio::time::sleep(Duration::from_millis(400)).await;
  assert!(rt.voice_resgates.fulfilled.lock().unwrap().is_empty(), "voz desconhecida não cumprir");
  assert!(view(&rt, &p)["queue"]["last"].as_str().unwrap().contains("disponível"));
  setup_with(&rt, &p, cfg_json(&[binding("r1", "blocked", 60, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-2", "u1")).await.unwrap();
  tokio::time::sleep(Duration::from_millis(400)).await;
  assert!(rt.voice_resgates.fulfilled.lock().unwrap().is_empty(), "voz bloqueada não cumprir");
  assert!(view(&rt, &p)["queue"]["last"].as_str().unwrap().contains("licença"));
  assert!(m.loads().is_empty(), "nada aplicado");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn redemptions_without_connection_are_reported_never_fulfilled() {
  let (dir, rt, p) = setup().await;
  let _ = dir;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 60, 0, "queue")], 3)).await;
  let msg = on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(msg.contains("recebido"), "aceito, mas o aplicado é decidido no portão");
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert!(rt.voice_resgates.fulfilled.lock().unwrap().is_empty());
  assert!(view(&rt, &p)["queue"]["last"].as_str().unwrap().contains("desconectado"));
 }

 #[tokio::test]
 async fn cancel_all_stops_everything_and_restores_the_previous_voice() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 600, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  let _ = on_redemption(&rt, &p, &redeem(&p.id, "r2x", "redem-2", "u2")).await;
  cancel_all(&rt, &p).await.unwrap();
  assert_eq!(m.voice(), "nofx", "a voz de antes era a normal");
  let v = view(&rt, &p);
  assert!(!v["queue"]["busy"].as_bool().unwrap());
  assert_eq!(v["queue"]["upcoming"].as_array().unwrap().len(), 0, "fila limpa");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn cancelar_restaura_a_voz_de_antes_e_nao_promete_normal() {
  let (_dir, rt, p, m) = connected_runtime().await;
  // A pessoa já estava numa voz com efeito (escolha dela) antes do resgate.
  m.user_picks("alien");
  rt.voicemod.take_manual(); // consome o sinal: não houve troca durante efeito
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 600, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  assert_eq!(m.voice(), "robot");
  cancel_all(&rt, &p).await.unwrap();
  assert_eq!(m.voice(), "alien", "cancelar devolve a voz de antes, não “a normal”");
  let v = view(&rt, &p);
  let last = v["queue"]["last"].as_str().unwrap().to_owned();
  assert!(last.to_lowercase().contains("alien"), "a mensagem diz qual voz voltou: {last}");
  assert!(!last.contains("voz normal"), "nada de prometer “voz normal” quando volta uma voz com efeito");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn comando_do_chat_troca_a_voz_pelo_mesmo_servico_sem_ser_resgate() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[command_binding("!robo", "moderator", "robot", 1, 0, "queue")], 3)).await;
  // Moderador pede o comando: troca pelo mesmo serviço, sem cumprir resgate.
  on_chat(&rt, &p, &chat(&p.id, "!robo agora", "moderator")).await;
  tokio::time::sleep(Duration::from_millis(400)).await;
  assert_eq!(m.voice(), "robot", "comando do chat usa o mesmo serviço de troca");
  assert!(rt.voice_resgates.fulfilled.lock().unwrap().is_empty(), "chat nunca vira resgate de pontos");
  // Simulação (teste/ensaio) nunca troca voz.
  on_chat(&rt, &p, &chat_sim(&p.id, "!robo de novo", "moderator")).await;
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert_eq!(m.loads().len(), 1, "simulação não toca a voz");
  // O efeito curto termina e devolve a voz de antes antes do teste de perfis.
  tokio::time::sleep(Duration::from_millis(1500)).await;
  let before = m.loads().len();
  // Outro perfil tem as regras dele: o chat deste perfil nunca pega a do
  // vizinho, e o chat do vizinho dispara a voz do vizinho.
  let other = Profile { id: uuid::Uuid::new_v4().to_string(), name: "Outro".into(), platform: "twitch".into(), channel: "outro".into(), channel_id: "2".into(), bot_id: "b".into(), client_id: "c".into(), blocklist: vec![], topics: vec![], editors: vec![], ai: Default::default(), modules: json!({}) };
  rt.db.save_profile(&other).unwrap();
  setup_with(&rt, &other, cfg_json(&[command_binding("!alien", "moderator", "alien", 60, 0, "queue")], 3)).await;
  on_chat(&rt, &p, &chat(&p.id, "!alien", "broadcaster")).await;
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert_eq!(m.loads().len(), before, "a regra do outro perfil não dispara com o chat deste");
  on_chat(&rt, &other, &chat(&other.id, "!alien", "moderator")).await;
  tokio::time::sleep(Duration::from_millis(400)).await;
  assert_eq!(m.voice(), "alien", "cada perfil dispara as regras dele");
  let applied: Vec<String> = m.loads().into_iter().filter(|v| v.as_str() != "nofx").collect();
  assert_eq!(applied, vec!["robot".to_string(), "alien".to_string()], "só vozes de efeito aplicadas, sem contar a devolução da origem");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn comando_do_chat_respeita_permissao_cooldown_e_regra_desligada() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[command_binding("!robo", "moderator", "robot", 60, 3600, "queue")], 3)).await;
  // Quem não tem permissão não dispara — e o motivo fica claro no log.
  on_chat(&rt, &p, &chat(&p.id, "!robo", "viewer")).await;
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert!(m.loads().is_empty(), "viewer não usa comando de moderator");
  assert!(rt.db.logs(&p.id).unwrap().iter().any(|l| l.message.contains("pediu o comando de voz")), "o aviso diz quem pode usar o comando");
  // Moderator dispara; o segundo uso cai no cooldown da regra.
  on_chat(&rt, &p, &chat(&p.id, "!robo", "moderator")).await;
  tokio::time::sleep(Duration::from_millis(400)).await;
  assert_eq!(m.loads(), vec!["robot".to_string()]);
  on_chat(&rt, &p, &chat(&p.id, "!robo", "broadcaster")).await;
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert_eq!(m.loads().len(), 1, "cooldown segura o segundo uso (broadcaster inclusive)");
  // Regra desligada não dispara, mesmo com permissão.
  let mut off = command_binding("!alien", "everyone", "alien", 60, 0, "queue");
  off["enabled"] = json!(false);
  setup_with(&rt, &p, cfg_json(&[off], 3)).await;
  on_chat(&rt, &p, &chat(&p.id, "!alien", "broadcaster")).await;
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert_eq!(m.loads().len(), 1, "regra desligada fica quieta");
  leave(&rt, &p).await;
 }

 /// Automações ("Mudar minha voz") respeitam a mesma fila dos resgates: o
 /// efeito no ar cumpre o seu tempo e o próximo espera — ninguém é cancelado.
 #[tokio::test]
 async fn manual_action_queues_instead_of_cancelling_the_running_effect() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[], 3)).await;
  apply_manual(&rt, &p, "robot", "Regra A", 2, "Ana").await.unwrap();
  assert_eq!(m.loads(), vec!["robot".to_string()], "primeiro aplicado");
  let msg = apply_manual(&rt, &p, "alien", "Regra B", 1, "Bia").await.unwrap();
  assert!(msg.contains("fila"), "o segundo entra em vez de cancelar: {msg}");
  tokio::time::sleep(Duration::from_millis(200)).await;
  assert_eq!(m.loads(), vec!["robot".to_string()], "o primeiro continua no ar, sem cancelamento");
  tokio::time::sleep(Duration::from_millis(2300)).await;
  assert_eq!(m.loads(), vec!["robot".to_string(), "alien".to_string()], "e o segundo começa na sequência");
  leave(&rt, &p).await;
 }

 /// O efeito "até a próxima troca" (duração 0) é substituído — é a definição dele.
 #[tokio::test]
 async fn zero_duration_effect_is_replaced_by_the_next_change() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[], 3)).await;
  apply_manual(&rt, &p, "robot", "Até a próxima troca", 0, "Ana").await.unwrap();
  let msg = apply_manual(&rt, &p, "alien", "Regra B", 5, "Bia").await.unwrap();
  assert!(!msg.contains("fila"), "duração zero não trava a fila: {msg}");
  assert_eq!(m.loads(), vec!["robot".to_string(), "alien".to_string()]);
  leave(&rt, &p).await;
 }

 /// O aviso de fila é personalizável e dirigido a quem resgatou.
 #[test]
 fn queue_notice_is_customizable() {
  assert_eq!(queue_notice_text("{{user}}, espera aí: {{voz}} ({{pos}}º)", "Ana", 2, "Robô"), "Ana, espera aí: Robô (2º)");
  assert_eq!(queue_notice_text("", "Ana", 1, "Robô"), "", "vazio desliga o aviso");
 }

 #[tokio::test]
 async fn old_timer_never_undoes_a_newer_command() {
  let (_dir, rt, p, m) = connected_runtime().await;
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 1, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  assert!(fired(&rt, 1).await);
  // Antes do fim do primeiro, um comando manual entra na cena (na fila aqui, o
  // primeiro cumpre o segundo inteiro antes dele).
  crate::voice_redemptions::apply_manual(&rt, &p, "alien", "Teste da tela", 600, "").await.unwrap();
  tokio::time::sleep(Duration::from_millis(2200)).await;
  assert_eq!(m.voice(), "alien", "o temporizador antigo não desfez o comando novo");
  leave(&rt, &p).await;
 }

 #[tokio::test]
 async fn redemption_is_refused_while_internal_fx_runs() {
  let (_dir, rt, p, m) = connected_runtime().await;
  rt.voice_fx.test_force_running();
  setup_with(&rt, &p, cfg_json(&[binding("r1", "robot", 60, 0, "queue")], 3)).await;
  on_redemption(&rt, &p, &redeem(&p.id, "r1", "redem-1", "u1")).await.unwrap();
  tokio::time::sleep(Duration::from_millis(300)).await;
  assert!(m.loads().is_empty(), "um caminho por vez: nada aplicado");
  assert!(rt.voice_resgates.fulfilled.lock().unwrap().is_empty());
  assert!(view(&rt, &p)["queue"]["last"].as_str().unwrap().contains("interna"));
 }

 #[tokio::test]
 async fn legacy_map_never_guesses_an_effect_name() {
  let (dir, rt, p) = setup().await;
  let _ = dir;
  assert_eq!(legacy_voice(&rt, &p.id, "robot"), None, "sem mapeamento explícito não há voz");
  setup_with(&rt, &p, json!({"enabled": false, "legacyMap": {"Robo": "robot"}})).await;
  assert_eq!(legacy_voice(&rt, &p.id, "robo").as_deref(), Some("robot"), "o dono escolheu");
  assert_eq!(legacy_voice(&rt, &p.id, "chipmunk"), None, "efeito sem escolha continua sem voz");
 }

 #[test]
 fn config_validation_refuses_silent_nonsense() {
  assert!(!valid_config(&json!({"enabled": true, "bindings": [{"rewardId": "r", "voiceId": ""}]})), "vínculo sem voz");
  assert!(!valid_config(&json!({"enabled": true, "bindings": [{"rewardId": "r", "voiceId": "nofx"}]})), "nofx não é efeito");
  assert!(!valid_config(&json!({"enabled": true, "bindings": [
   {"rewardId": "r", "voiceId": "robot"}, {"rewardId": "r", "voiceId": "alien"}
  ]})), "reward repetido");
  assert!(!valid_config(&json!({"enabled": true, "bindings": [{"rewardId": "r", "voiceId": "robot", "concurrency": "explode"}]})), "política desconhecida");
  assert!(valid_config(&json!({"enabled": true, "bindings": [{"rewardId": "r", "voiceId": "robot", "durationSec": 30, "concurrency": "queue"}]})));
 }
}
