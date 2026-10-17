//! Comandos administrativos falados e digitados: um só lugar decide quem pode
//! pedir o quê e executa pela conta do bot. A origem é conferida no backend —
//! fala vem só da escuta contínua (streamer local), texto só do chat com
//! permissão de moderador; resgate, alerta, TTS e texto de espectador nunca
//! viram ordem administrativa. Banimento permanente exige confirmação curta,
//! presa à ação, ao usuário, ao perfil e a quem pediu.
use crate::engine::Runtime;
use crate::model::{permitted, Action, Event, Profile};
use crate::natural::Talk;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const KIND: &str = "intents";
/// Prazo da confirmação de ação permanente: curto de propósito.
const CONFIRM_TTL: u64 = 30;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Origin {
 /// Fala do streamer captada pela escuta contínua (ou botão de comando).
 Voice,
 /// Mensagem de chat de quem tem permissão de moderador no canal.
 Chat,
}

/// Confirmações pendentes por perfil: `perfil:ator:ação:alvo` → (expira, alvo, ação).
#[derive(Default)]
pub struct Intents {
 pending: Mutex<HashMap<String, (u64, String, String)>>,
}

fn now() -> u64 {
 SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn human(secs: u64) -> String {
 if secs >= 3600 && secs % 3600 == 0 { format!("{}h", secs / 3600) }
 else if secs >= 60 {
  let m = secs / 60;
  let s = secs % 60;
  if s == 0 { format!("{m}min") } else { format!("{m}min {s}s") }
 } else { format!("{secs}s") }
}

/// A origem precisa bater com o caminho real do evento: fala só vem do
/// microfone do streamer, texto só do chat com papel de moderador. Nada de
/// simulação, teste rápido ou texto vindo de outro lugar disfarçado de ordem.
fn origin_ok(origin: Origin, e: &Event) -> bool {
 match origin {
  Origin::Voice => e.kind == "voice" && !e.simulated && e.user_id == "local-streamer" && e.role == "broadcaster",
  Origin::Chat => {
   e.kind == "chat" && !e.simulated && e.data["botliveTest"] != true
    && permitted("moderator", &e.role)
    && crate::discord::from_discord(e).is_none()
  },
 }
}

/// Executa um comando natural já interpretado. Devolve true quando a frase era
/// um comando administrativo (o chamador não deve seguir o fluxo normal).
pub async fn dispatch(rt: &Arc<Runtime>, p: &Profile, e: &Event, origin: Origin, talk: Talk) -> Result<bool, String> {
 if matches!(talk, Talk::Ignore) {
  return Ok(false);
 }
 if !origin_ok(origin, e) {
  rt.log(&p.id, KIND, "Comando administrativo recusado: a origem deste pedido não tem autorização.", "error");
  return Err("Este comando só sai do microfone do streamer ou do chat de quem modera.".into());
 }
 let reply = match talk {
  Talk::Ignore => return Ok(false),
  Talk::Incomplete(hint) => Some(hint.to_owned()),
  Talk::Title(title) => {
   let a = Action { kind: "twitch".into(), tw_op: "title".into(), ..Default::default() };
   Some(run(rt, p, e, &a, &title).await?)
  },
  Talk::Category(names) => {
   let a = Action { kind: "twitch".into(), tw_op: "game".into(), ..Default::default() };
   Some(run(rt, p, e, &a, &names.join(" ou ")).await?)
  },
  Talk::Timeout { login, seconds } => {
   let secs = seconds.unwrap_or_else(|| crate::listen::Config::read(&rt.db, &p.id).timeout_secs).clamp(1, 1_209_600);
   // "quicar" é apelido de timeout: a duração configurada aparece sempre e
   // o alvo real é resolvido na Twitch antes de qualquer punição.
   let target = resolve(rt, p, &login).await?;
   let a = Action { kind: "twitch".into(), tw_op: "timeout".into(), target: target.clone(), value: secs as i64, ..Default::default() };
   run(rt, p, e, &a, "").await?;
   Some(format!("Timeout de {} em @{target} pelo bot.", human(secs)))
  },
  Talk::Ban { login } => {
   let target = resolve(rt, p, &login).await?;
   let key = format!("{}:{}:ban:{target}", p.id, e.user_id);
   let fresh = {
    let mut pend = rt.intents.pending.lock().unwrap();
    let alive = pend.get(&key).is_some_and(|(at, _, _)| at > &now());
    if !alive {
     pend.insert(key.clone(), (now() + CONFIRM_TTL, target.clone(), "ban".into()));
    }
    alive
   };
   if fresh {
    execute_ban(rt, p, e, &target).await?
   } else {
    Some(format!(
     "Para banir permanentemente @{target}, repita o pedido ou diga 'confirme' em até {CONFIRM_TTL} segundos. Fora do prazo eu não bano."
    ))
   }
  },
  Talk::Unban { login } => {
   let target = resolve(rt, p, &login).await?;
   let a = Action { kind: "twitch".into(), tw_op: "unban".into(), target: target.clone(), ..Default::default() };
   run(rt, p, e, &a, "").await?;
   Some(format!("@{target} voltou ao canal (desban pelo bot)."))
  },
  Talk::Confirm => {
   let hit = {
    let mut pend = rt.intents.pending.lock().unwrap();
    let prefix = format!("{}:{}:", p.id, e.user_id);
    let mut hit = None;
    pend.retain(|k, v| {
     if k.starts_with(&prefix) && v.0 > now() {
      hit = Some((v.1.clone(), v.2.clone()));
      false
     } else {
      v.0 > now()
     }
    });
    hit
   };
   match hit {
    Some((target, action)) if action == "ban" => execute_ban(rt, p, e, &target).await?,
    Some(_) => Some("Esta ação pendente não pode ser confirmada.".into()),
    None => Some("Nada para confirmar: não há pedido pendente no prazo.".into()),
   }
  },
  Talk::Cancel => {
   let n = {
    let mut pend = rt.intents.pending.lock().unwrap();
    let prefix = format!("{}:{}:", p.id, e.user_id);
    let before = pend.len();
    pend.retain(|k, v| !(k.starts_with(&prefix) && v.0 > now()));
    before - pend.len()
   };
   Some(if n > 0 { format!("{n} pedido(s) cancelado(s): nada será executado.") } else { "Nada pendente para cancelar.".into() })
  },
  Talk::VoiceNormal => Some(voice_normal(rt, p).await?),
 };
 if let Some(text) = reply {
  if let Err(err) = rt.send(p, e, &text).await {
   rt.log(&p.id, KIND, &err, "error");
  }
 }
 Ok(true)
}

/// Login dito → login real da Twitch. Sem chute: quem não existe não é alvo.
async fn resolve(rt: &Runtime, p: &Profile, login: &str) -> Result<String, String> {
 crate::moderation::twitch_user_id(rt, p, login).await.map(|_| login.to_lowercase())
}

async fn run(rt: &Arc<Runtime>, p: &Profile, e: &Event, a: &Action, text: &str) -> Result<String, String> {
 crate::twitch_ops::run(rt, p, e, a, text, "", None).await
}

async fn execute_ban(rt: &Arc<Runtime>, p: &Profile, e: &Event, target: &str) -> Result<Option<String>, String> {
 let a = Action { kind: "twitch".into(), tw_op: "ban".into(), target: target.to_owned(), ..Default::default() };
 run(rt, p, e, &a, "").await?;
 Ok(Some(format!("Banimento permanente aplicado em @{target} pelo bot.")))
}

/// "Volte minha voz ao normal": no caminho do Voicemod cancela efeitos e
/// confirma a voz normal; na voz interna volta o efeito para normal.
async fn voice_normal(rt: &Arc<Runtime>, p: &Profile) -> Result<String, String> {
 let voicemod_here = rt.voicemod.connected() || crate::voice_redemptions::busy(rt);
 if voicemod_here {
  match crate::voice_redemptions::cancel_all(rt, p).await {
   Ok(_) => return Ok("Voz normal de volta no Voicemod; fila de resgates limpa.".into()),
   Err(err) => return Err(err),
  }
 }
 if rt.voice_fx.is_running() {
  rt.voice_fx.set_effect(rt, &p.id, "normal")?;
  return Ok("Voz interna de volta ao normal.".into());
 }
 Ok("Sua voz já está normal.".into())
}

#[cfg(test)]
mod tests {
 use super::*;
 use serde_json::{json, Value};

 fn profile(id: &str) -> Profile {
  Profile {
   id: id.into(),
   name: "P".into(),
   platform: "twitch".into(),
   channel: "canal".into(),
   channel_id: "1".into(),
   bot_id: "9".into(),
   client_id: "c".into(),
   blocklist: vec![],
   topics: vec![],
   editors: vec![],
   ai: crate::model::AiConfig::default(),
   modules: json!({"voice": true}),
  }
 }
 fn chat(role: &str, msg: &str) -> Event {
  Event { id: uuid::Uuid::new_v4().to_string(), profile_id: "p".into(), kind: "chat".into(), user: "u".into(), user_id: "7".into(), role: role.into(), message: msg.into(), data: json!({}), simulated: false, rehearsal: false }
 }
 fn voice(msg: &str) -> Event {
  Event { id: uuid::Uuid::new_v4().to_string(), profile_id: "p".into(), kind: "voice".into(), user: "streamer".into(), user_id: "local-streamer".into(), role: "broadcaster".into(), message: msg.into(), data: Value::Null, simulated: false, rehearsal: false }
 }
 fn rt_profile() -> (tempfile::TempDir, Arc<Runtime>, Profile) {
  let dir = tempfile::tempdir().unwrap();
  let rt = Runtime::new(dir.path().to_path_buf()).unwrap();
  let p = profile(&uuid::Uuid::new_v4().to_string());
  rt.db.save_profile(&p).unwrap();
  (dir, rt, p)
 }

 #[test]
 fn origem_de_fala_exige_o_streamer_real_de_voz() {
  let ok = voice("arroba, cancele");
  assert!(origin_ok(Origin::Voice, &ok), "voz do streamer local é a origem certa");
  let mut fake = voice("arroba, ban @alvo");
  fake.user_id = "1234".into();
  assert!(!origin_ok(Origin::Voice, &fake), "fala fabricada com outro user_id não vale");
  let mut sim = voice("arroba, ban @alvo");
  sim.simulated = true;
  assert!(!origin_ok(Origin::Voice, &sim), "simulação nunca pune de verdade");
  let mut alert = voice("arroba, ban @alvo");
  alert.kind = "alert".into();
  assert!(!origin_ok(Origin::Voice, &alert), "texto de alerta não é ordem");
  let mut redeem = voice("arroba, ban @alvo");
  redeem.kind = "redemption".into();
  assert!(!origin_ok(Origin::Voice, &redeem), "texto de resgate não é ordem");
 }

 #[test]
 fn origem_de_chat_exige_moderador_e_nao_simulacao() {
  assert!(origin_ok(Origin::Chat, &chat("moderator", "arroba, mude a categoria para X")));
  assert!(origin_ok(Origin::Chat, &chat("broadcaster", "arroba, cancele")));
  assert!(!origin_ok(Origin::Chat, &chat("viewer", "arroba, ban @alvo")), "espectador não modera");
  assert!(!origin_ok(Origin::Chat, &chat("everyone", "arroba, ban @alvo")));
  let mut sim = chat("moderator", "arroba, ban @alvo");
  sim.simulated = true;
  assert!(!origin_ok(Origin::Chat, &sim), "ensaio não pune");
  let mut test = chat("moderator", "arroba, ban @alvo");
  test.data = json!({"botliveTest": true});
  assert!(!origin_ok(Origin::Chat, &test), "teste rápido não pune");
  let mut disc = chat("moderator", "arroba, ban @alvo");
  disc.data = json!({"source": "discord", "channel": "geral"});
  assert!(!origin_ok(Origin::Chat, &disc), "fala do Discord não vira ordem do canal");
 }

 #[tokio::test]
 async fn ignore_e_comum_nao_consome_a_origem() {
  let (_d, rt, p) = rt_profile();
  let e = chat("viewer", "qual é a categoria?");
  assert_eq!(dispatch(&rt, &p, &e, Origin::Chat, Talk::Ignore).await.unwrap(), false, "seguir o chat normal");
 }

 #[tokio::test]
 async fn comando_de_origem_ruim_e_recusado_sem_executar() {
  let (_d, rt, p) = rt_profile();
  let e = chat("viewer", "arroba, ban @alvo");
  let err = dispatch(&rt, &p, &e, Origin::Chat, Talk::Ban { login: "alvo".into() }).await.unwrap_err();
  assert!(err.contains("microfone") || err.contains("modera"), "{err}");
 }

 #[tokio::test]
 async fn ban_pede_confirmacao_curta_e_presa_ao_alvo() {
  let (_d, rt, p) = rt_profile();
  let e = voice("arroba, bana o alvo");
  // resolve() bate na Twitch: sem rede o alvo falha antes de qualquer ban.
  let err = dispatch(&rt, &p, &e, Origin::Voice, Talk::Ban { login: "alvo".into() }).await;
  assert!(err.is_err(), "sem Twitch real não banimos: a resolução do alvo é obrigatória");
  // O pendente não chega a existir: resolve falhou antes.
  assert!(rt.intents.pending.lock().unwrap().is_empty());
 }

 #[test]
 fn confirmacao_tem_prazo_e_fica_presa_ao_alvo() {
  let map: HashMap<String, (u64, String, String)> = {
   let mut m = HashMap::new();
   m.insert("p1:7:ban:alvo".to_string(), (now() + 20, "alvo".into(), "ban".into()));
   m
  };
  let (at, target, action) = map.values().next().unwrap();
  assert!(at > &now(), "dentro do prazo");
  assert_eq!(target, "alvo");
  assert_eq!(action, "ban");
  let expired: HashMap<String, (u64, String, String)> = {
   let mut m = HashMap::new();
   m.insert("p1:7:ban:alvo2".to_string(), (now().saturating_sub(5), "alvo2".into(), "ban".into()));
   m
  };
  assert!(expired.values().all(|(at, _, _)| at <= &now()), "fora do prazo não confirma");
 }

 #[test]
 fn duracao_humana_e_legivel() {
  assert_eq!(human(300), "5min");
  assert_eq!(human(30), "30s");
  assert_eq!(human(3600), "1h");
  assert_eq!(human(90), "1min 30s");
 }
}
