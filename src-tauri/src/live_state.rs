//! O que está acontecendo agora na live: categoria/jogo, eventos recentes e calor.
//! Guardado só em memória — é transitório e reseta junto com a sessão.
use crate::model::Event;
use std::{collections::{HashMap,VecDeque},sync::Mutex,time::{Duration,Instant}};
pub const TTL:Duration=Duration::from_secs(90);
const MAX_EVENTS:usize=6;
const MAX_CHAT:usize=40;
const BURST:Duration=Duration::from_secs(20);

#[derive(Default)]
struct State {
 category:String,
 since:Option<Instant>,
 history:Vec<String>,
 events:Vec<(Instant,String)>,
 chat:VecDeque<Instant>,
}

#[derive(Default)]
pub struct LiveStates(Mutex<HashMap<String,State>>);

fn prune(s:&mut State,now:Instant,ttl:Duration) {
 s.events.retain(|(at,_)| now.saturating_duration_since(*at)<=ttl);
 s.chat.retain(|at| now.saturating_duration_since(*at)<=BURST);
}
fn event_text(e:&Event)->Option<String> {
 let user:String=e.user.chars().take(32).collect();
 Some(match e.kind.as_str() {
  "subscription" if e.data["cumulative_months"].as_u64().unwrap_or(0)>1 => format!("re-sub de {user}"),
  "resub" => format!("re-sub de {user}"),
  "subscription"|"gift" => format!("sub de {user}"),
  "cheer" => {let bits=e.data["bits"].as_u64().unwrap_or(0);if bits>0 {format!("{bits} bits de {user}")} else {format!("bits de {user}")}},
  "raid" => format!("raid de {user}"),
  "follow" => format!("follow de {user}"),
  "redemption" => format!("resgate de {user}"),
  "clip" => format!("clip de {user}"),
  _ => return None,
 })
}
fn category_text(s:&State)->String {
 if s.category.is_empty() {return "categoria não registrada".into();}
 let mins=s.since.map(|t|t.elapsed().as_secs()/60).unwrap_or(0);
 let ago=if mins<1 {"menos de 1 min".into()} else {format!("{mins} min")};
 if s.history.iter().any(|c|*c==s.category) {format!("voltou para {} há {}",s.category,ago)}
 else if s.category.eq_ignore_ascii_case("Just Chatting") {format!("só de conversa (Just Chatting) há {}",ago)}
 else {format!("jogando {} há {}",s.category,ago)}
}
fn heat(s:&State)->&'static str {
 let now=Instant::now();
 let burst=s.chat.iter().filter(|at| now.saturating_duration_since(**at)<=BURST).count();
 let hype=s.events.iter().any(|(at,_)| now.saturating_duration_since(*at)<=Duration::from_secs(15));
 if hype||burst>=8 {"alto"} else if burst>=3 {"médio"} else {"baixo"}
}
impl LiveStates {
 pub fn clear(&self,p:&str) {self.0.lock().unwrap().remove(p);}
 pub fn set_category(&self,p:&str,category:&str) {
  let category=category.trim();if category.is_empty() {return;}
  let mut map=self.0.lock().unwrap();let s: &mut State=map.entry(p.into()).or_default();
  if s.category.eq_ignore_ascii_case(category) {return;}
  if !s.category.is_empty() {s.history.push(s.category.clone());let cut=s.history.len().saturating_sub(4);s.history.drain(..cut);}
  s.category=category.to_owned();s.since=Some(Instant::now());
 }
 /// Registra o que acabou de acontecer. Simulações nunca alteram o estado real.
 pub fn record(&self,p:&str,e:&Event) {
  if e.simulated {return;}
  if e.kind=="category" {
   let name=e.data["category_name"].as_str().unwrap_or(&e.message);
   self.set_category(p,name);return;
  }
  let now=Instant::now();
  let mut map=self.0.lock().unwrap();let s=map.entry(p.into()).or_default();prune(s,now,TTL);
  if e.kind=="chat" {s.chat.push_back(now);while s.chat.len()>MAX_CHAT {s.chat.pop_front();}return;}
  if let Some(text)=event_text(e) {
   s.events.push((now,text));
   while s.events.len()>MAX_EVENTS {s.events.remove(0);}
  }
 }
 /// Linha curta para o prompt: sobre o que a live está falando neste momento.
 pub fn summary(&self,p:&str)->String {
  let now=Instant::now();
  let mut map=self.0.lock().unwrap();
  let Some(s)=map.get_mut(p) else {return String::new()};
  prune(s,now,TTL);
  if s.category.is_empty()&&s.events.is_empty()&&s.chat.is_empty() {return String::new();}
  let mut parts=vec![category_text(s)];
  if !s.events.is_empty() {
   let recent:Vec<String>=s.events.iter().rev().take(2)
    .map(|(at,text)|format!("{text} há {}s",now.saturating_duration_since(*at).as_secs())).collect();
   parts.push(format!("evento recente: {}",recent.join(", ")));
  }
  parts.push(format!("calor {}",heat(s)));
  format!("[Contexto agora: {}]",parts.join(", ")).chars().take(500).collect()
 }
}
#[cfg(test)] mod tests {
 use super::*;
 fn event(kind:&str,user:&str,message:&str,simulated:bool)->Event {
  serde_json::from_value(serde_json::json!({"id":"1","profileId":"p","kind":kind,"user":user,"userId":"u","role":"everyone","message":message,"simulated":simulated})).unwrap()
 }
 #[test] fn category_changes_expire_events_and_calm_down() {
  let live=LiveStates::default();
  assert_eq!(live.summary("p"),"","sem nada acontecendo não gasta tokens");
  live.record("p",&event("chat","Ana","boa jogada",false));
  let first=live.summary("p");
  assert!(first.contains("categoria não registrada"),"sem sinal de categoria a frase fica explícita");
  assert!(first.contains("calor baixo"));
  live.set_category("p","Counter-Strike 2");
  assert!(live.summary("p").contains("jogando Counter-Strike 2"));
  live.set_category("p","Just Chatting");
  assert!(!live.summary("p").contains("voltou para"),"a primeira troca não é volta");
  assert!(live.summary("p").contains("só de conversa (Just Chatting)"),"seção sem jogo é dita");
  live.set_category("p","Counter-Strike 2");
  assert!(live.summary("p").contains("voltou para Counter-Strike 2"),"a troca de volta é dita");
  live.record("p",&event("subscription","Bia","",false));
  let hot=live.summary("p");
  assert!(hot.contains("sub de Bia")&&hot.contains("calor alto"),"um sub recente sobe o calor");
  live.record("p",&event("raid","Mia","",true));
  assert!(!live.summary("p").contains("de Mia"),"simulação não entra no estado real");
  assert!(live.summary("p").chars().count()<=500,"o resumo é curto");
  live.clear("p");assert_eq!(live.summary("p"),"");
 }
 #[test] fn events_leave_the_summary_after_their_ttl() {
  let mut s=State::default();
  s.events.push((Instant::now(),"bits de Ana".into()));
  prune(&mut s,Instant::now(),Duration::from_nanos(1));
  assert!(s.events.is_empty(),"evento mais velho que o TTL é removido");
  s.events.push((Instant::now(),"bits de Ana".into()));
  prune(&mut s,Instant::now(),TTL);
  assert_eq!(s.events.len(),1,"evento dentro do TTL continua");
 }
 #[test] fn heat_grows_with_chat_burst() {
  let live=LiveStates::default();
  for _ in 0..9 {live.record("p",&event("chat","Ana","kkkk",false));}
  assert!(live.summary("p").contains("calor alto"));
  live.clear("p");
  for _ in 0..4 {live.record("p",&event("chat","Ana","kkkk",false));}
  assert!(live.summary("p").contains("calor médio"));
 }
}
