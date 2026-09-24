use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Profile {
 pub id:String, pub name:String, pub platform:String, pub channel:String,
 #[serde(default)] pub channel_id:String,
 #[serde(default)] pub bot_id:String,
 #[serde(default)] pub client_id:String,
 #[serde(default)] pub blocklist:Vec<String>,
 #[serde(default)] pub topics:Vec<String>,
 #[serde(default)] pub editors:Vec<String>,
 #[serde(default)] pub ai:AiConfig,
 #[serde(default)] pub modules:Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AiConfig {
 pub provider:String, pub endpoint:String, pub model:String,
 pub personality:String, pub temperature:f64, pub fallback:String,
 #[serde(default)] pub remember:bool,
}
impl Default for AiConfig {
 fn default()->Self { Self { provider:"ollama".into(), endpoint:"http://localhost:11434".into(), model:"".into(),personality:"Você é um bot amigável de uma comunidade de live. Responda em português, brevemente e com respeito.".into(),temperature:0.7,fallback:"Não consegui responder agora. Tente novamente em instantes.".into(), remember:false } }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Flow {
 #[serde(default)] pub counter:bool,
 #[serde(default="timer_default")] pub timer_seconds:u64,
 pub id:String, pub profile_id:String, pub name:String, pub enabled:bool,
 pub trigger:Trigger, pub actions:Vec<Action>,
 #[serde(default)] pub layout:Value,
}
fn timer_default()->u64{300}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Trigger {
 pub kind:String, #[serde(default)] pub pattern:String,
 #[serde(default="everyone")] pub permission:String,
 #[serde(default)] pub cooldown:u64, #[serde(default)] pub user_cooldown:u64,
}
fn everyone()->String { "everyone".into() }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Action {
 pub kind:String, #[serde(default)] pub text:String,
 #[serde(default)] pub target:String, #[serde(default)] pub value:i64,
 #[serde(default)] pub condition:String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Event {
 pub id:String, pub profile_id:String, pub kind:String,
 #[serde(default)] pub user:String, #[serde(default)] pub user_id:String,
 #[serde(default="everyone")] pub role:String,
 #[serde(default)] pub message:String,
 #[serde(default)] pub data:Value,
 #[serde(default)] pub simulated:bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Log {
 pub id:i64, pub profile_id:String, pub timestamp:String,
 pub kind:String, pub message:String, pub status:String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Note { pub path:String, pub content:String }
pub fn valid_id(id:&str)->bool { uuid::Uuid::parse_str(id).is_ok() }
pub fn blocked(text:&str, p:&Profile)->bool {
 let text=text.to_lowercase();
 p.blocklist.iter().any(|w| !w.trim().is_empty() && text.contains(&w.to_lowercase()))
}
pub fn permitted(required:&str, role:&str)->bool {
 match required {
 "everyone"=>true,
 "subscriber"=>matches!(role,"subscriber"|"moderator"|"broadcaster"),
 "moderator"=>matches!(role,"moderator"|"broadcaster"),
 "broadcaster"=>role=="broadcaster",
 _=>false,
 }
}
pub fn matches(t:&Trigger,e:&Event)->bool {
 if !permitted(&t.permission,&e.role) { return false; }
 match t.kind.as_str() {
 "timer"=>false, // Only the internal scheduler can execute periodic flows.
 "command"=>e.kind=="chat" && e.message.split_whitespace().next().is_some_and(|s| s.eq_ignore_ascii_case(&t.pattern)),
 "contains"=>e.kind=="chat" && !t.pattern.is_empty() && e.message.to_lowercase().contains(&t.pattern.to_lowercase()),
 "voice"=>e.kind=="voice" && (t.pattern.is_empty()||e.message.to_lowercase().contains(&t.pattern.to_lowercase())),
 other=>other==e.kind,
 }
}
pub fn validate_flow(f:&Flow)->Result<(),String> {
 if f.trigger.kind=="timer"&&!(30..=86400).contains(&f.timer_seconds){return Err("Timer: escolha de 30 segundos a 24 horas".into())}
 if f.counter&&f.trigger.kind!="command"{return Err("O contador individual é exclusivo de comandos".into())}
 if !valid_id(&f.id)||!valid_id(&f.profile_id) {return Err("Identificador inválido".into())}
 if f.name.trim().is_empty()||f.actions.is_empty()||f.actions.len()>64 {return Err("Dê um nome e adicione entre 1 e 64 ações".into())}
 if !["everyone","subscriber","moderator","broadcaster"].contains(&f.trigger.permission.as_str()) {return Err("Permissão inválida".into())}
 if f.trigger.cooldown>86400||f.trigger.user_cooldown>86400{return Err("Cooldown máximo: 24 horas".into())}
 if f.trigger.kind=="command" && (!f.trigger.pattern.starts_with('!')||f.trigger.pattern.contains(char::is_whitespace)) {return Err("O comando deve começar com ! e não conter espaços".into())}
 for a in &f.actions {
 if !["chat","ai","ai.generate","memory","webhook","discord","overlay","delay","script","points","tts","variable.set","variable.increment","variable.delete"].contains(&a.kind.as_str()) {return Err("Tipo de ação inválido".into())}
 if a.kind=="ai.generate" {let (scope,name)=crate::variables::target(crate::ai::response_target(a))?;if scope!="local"||name=="aiSuccess"{return Err("Guarde a resposta da IA numa variável local de texto".into())}}
 if a.kind.starts_with("variable."){crate::variables::target(&a.target)?;}
 if a.text.len()>32768 {return Err("Ação muito longa".into())}
 if a.kind=="delay" && !(0..=30000).contains(&a.value) {return Err("Espera máxima: 30 segundos".into())}
 }
 Ok(())
}
#[cfg(test)] mod tests {
 use super::*;
 fn e()->Event {Event{id:"x".into(),profile_id:"p".into(),kind:"chat".into(),user:"ana".into(),user_id:"1".into(),role:"everyone".into(),message:"!oi tudo bem".into(),data:Value::Null,simulated:true}}
 #[test] fn command_boundary_and_roles() {
 let mut t=Trigger{kind:"command".into(),pattern:"!oi".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0};
 assert!(matches(&t,&e()));t.pattern="!o".into();assert!(!matches(&t,&e()));
 t.pattern="!oi".into();t.permission="moderator".into();assert!(!matches(&t,&e()));
 assert!(permitted("moderator","broadcaster"));assert!(!permitted("broadcaster","moderator"));
 }
}
