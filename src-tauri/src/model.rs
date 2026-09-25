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
 #[serde(default)] pub audio:String,
 #[serde(default="full_volume")] pub audio_volume:f64,
 #[serde(default="send_chat")] pub send_type:String,
 #[serde(default="send_primary")] pub send_color:String,
 pub id:String, pub profile_id:String, pub name:String, pub enabled:bool,
 pub trigger:Trigger, pub actions:Vec<Action>,
 #[serde(default)] pub layout:Value,
}
fn timer_default()->u64{300}
fn full_volume()->f64{1.0}
fn send_chat()->String{"chat".into()}
fn send_primary()->String{"primary".into()}
pub const SEND_TYPES:[&str;4]=["chat","announce","pin","shoutout"];
pub const SEND_COLORS:[&str;5]=["primary","blue","green","orange","purple"];
/// True when the flow needs a message-sending action to make this delivery useful.
pub fn needs_message(f:&Flow)->bool{f.actions.iter().any(|a|matches!(a.kind.as_str(),"chat"|"ai"|"script"))}
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
pub fn preview_event(f:&Flow,e:Event)->Event {
 match f.trigger.kind.as_str() {
 "timer"=>crate::timers::event(f,true),
 "voice"=>Event{kind:"voice".into(),user:"streamer".into(),user_id:"local-streamer".into(),role:"broadcaster".into(),..e},
 "command" if !f.trigger.pattern.is_empty()=>{
  let words:Vec<&str>=e.message.split_whitespace().collect();
  let rest:&[&str]=if words.first().is_some_and(|w|w.starts_with('!')){&words[1..]}else{&words[..]};
  let message=std::iter::once(f.trigger.pattern.as_str()).chain(rest.iter().copied()).collect::<Vec<_>>().join(" ");
  Event{kind:"chat".into(),message,..e}
 },
 "contains" if !f.trigger.pattern.is_empty()=>{
  let message=if e.message.to_lowercase().contains(&f.trigger.pattern.to_lowercase()){e.message}
   else if e.message.trim().is_empty(){f.trigger.pattern.clone()}
   else{format!("{} {}",e.message.trim(),f.trigger.pattern)};
  Event{kind:"chat".into(),message,..e}
 },
 kind=>Event{kind:kind.into(),..e},
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
 if !SEND_TYPES.contains(&f.send_type.as_str()){return Err("Forma de envio inválida".into())}
 if !SEND_COLORS.contains(&f.send_color.as_str()){return Err("Cor do anúncio inválida".into())}
 if f.send_type=="shoutout"&&!needs_message(f){return Err("O destaque de canal usa o texto de uma ação que envia mensagem: escreva ali o canal de destino".into())}
 if !f.audio.is_empty()&&(f.audio.len()>64||f.audio.contains(char::is_whitespace)){return Err("Áudio inválido: escolha um som da biblioteca".into())}
 if !f.audio_volume.is_finite()||!(0.0..=1.0).contains(&f.audio_volume){return Err("Volume do áudio: escolha de 0% a 100%".into())}
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
 #[test] fn preview_event_follows_the_trigger() {
  let mut f=Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),id:"f".into(),profile_id:"p".into(),name:"Minecraft".into(),enabled:true,trigger:Trigger{kind:"timer".into(),pattern:String::new(),permission:"everyone".into(),cooldown:0,user_cooldown:0},actions:vec![],layout:Value::Null};
  let t=preview_event(&f,e());
  assert_eq!((t.kind.as_str(),t.user.as_str(),t.user_id.as_str(),t.message.as_str(),t.simulated),("timer","BotLive","","",true));
  f.trigger.kind="command".into();f.trigger.pattern="!minecraft".into();
  let t=preview_event(&f,e());
  assert_eq!((t.kind.as_str(),t.message.as_str()),("chat","!minecraft tudo bem"));
  f.trigger.kind="contains".into();f.trigger.pattern="minecraft".into();
  assert_eq!(preview_event(&f,e()).message,"!oi tudo bem minecraft");
  f.trigger.kind="voice".into();f.trigger.pattern.clear();
  let t=preview_event(&f,e());
  assert_eq!((t.kind.as_str(),t.user_id.as_str(),t.message.as_str()),("voice","local-streamer","!oi tudo bem"));
  f.trigger.kind="follow".into();
  assert_eq!(preview_event(&f,e()).kind,"follow");
 }
 #[test] fn delivery_type_color_and_flow_audio_rules() {
  let mut f=Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),id:uuid::Uuid::new_v4().to_string(),profile_id:uuid::Uuid::new_v4().to_string(),name:"ifood".into(),enabled:true,trigger:Trigger{kind:"command".into(),pattern:"!ifood".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0},actions:vec![Action{kind:"chat".into(),text:"Bora".into(),target:String::new(),value:0,condition:String::new()}],layout:Value::Null};
  assert!(validate_flow(&f).is_ok());
  f.send_type="announce".into();f.send_color="purple".into();assert!(validate_flow(&f).is_ok());
  f.send_type="letras".into();assert!(validate_flow(&f).is_err());
  f.send_type="pin".into();f.send_color="vermelho".into();assert!(validate_flow(&f).is_err());
  f.send_color="primary".into();f.audio="audio com espaco".into();assert!(validate_flow(&f).is_err());
  f.audio=uuid::Uuid::new_v4().to_string();f.audio_volume=1.5;assert!(validate_flow(&f).is_err());
  f.audio_volume=0.8;assert!(validate_flow(&f).is_ok());
  f.send_type="shoutout".into();f.actions.clear();assert!(validate_flow(&f).is_err());
  f.actions.push(Action{kind:"chat".into(),text:"outrocanal".into(),target:String::new(),value:0,condition:String::new()});
  assert!(validate_flow(&f).is_ok());
  assert_eq!(SEND_TYPES,["chat","announce","pin","shoutout"]);
 }
}
