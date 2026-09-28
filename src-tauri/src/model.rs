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
 /// Base de conhecimento importada: entra no prompt quando ativada.
 #[serde(default="yes")] pub knowledge:bool,
 #[serde(default)] pub knowledge_nicho:String,
 #[serde(default="std_depth")] pub knowledge_depth:String,
 #[serde(default)] pub knowledge_off:Vec<String>,
 #[serde(default)] pub knowledge_source:String,
 /// Padrões para ações novas; cada ação pode sobrescrever com valor vazio herdando estes.
 #[serde(default)] pub anchor:String,
 #[serde(default)] pub answer_length:String,
 #[serde(default)] pub no_repeat:bool,
}
fn yes()->bool {true}
fn std_depth()->String {"standard".into()}
impl Default for AiConfig {
 fn default()->Self { Self { provider:"ollama".into(), endpoint:"http://localhost:11434".into(), model:"".into(),personality:"Você é um bot amigável de uma comunidade de live. Responda em português, brevemente e com respeito.".into(),temperature:0.7,fallback:String::new(), remember:false, knowledge:true, knowledge_nicho:String::new(), knowledge_depth:std_depth(), knowledge_off:Vec::new(), knowledge_source:String::new(), anchor:String::new(), answer_length:String::new(), no_repeat:false } }
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
 /// Responde no fio de quem disparou: reply na Twitch, citação no Discord.
 #[serde(default)] pub reply_to:bool,
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
/// Ancoragem da resposta com IA e tamanho máximo dela.
pub const ANCHORS:[&str;4]=["all","message","chat","fixed"];
/// Modos aceitos pela ação de punição e os dois modos de escolher quem leva ela.
pub const PUNISH_MODES:[&str;3]=["timeout","ban","warn"];
pub const PUNISH_TARGETS:[&str;2]=["sender","first"];
pub const LENGTHS:[&str;3]=["short","medium","free"];
pub const KNOWLEDGE_DEPTHS:[&str;3]=["light","standard","full"];
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
 /// Modo da ação de punição: silenciar, banir ou avisar. Só é usado por "punish".
 #[serde(default)] pub punish:String,
 /// Operação da ação na Twitch: setgame, settitle, timeout, ban, unban, warn,
 /// vip, unvip, slow, slowoff, followers, followersoff, subsonly, subsonlyoff,
 /// emoteonly, emoteonlyoff, shoutout ou mention. Só é usado por "twitch".
 #[serde(default)] pub tw_op:String,
 /// Controles da resposta com IA. Vazio ou ausente herda o padrão do perfil.
 #[serde(default)] pub ai_anchor:String,
 #[serde(default)] pub ai_knowledge:String,
 #[serde(default)] pub ai_length:String,
 #[serde(default)] pub ai_style:String,
 #[serde(default)] pub ai_no_repeat:Option<bool>,
}
impl Default for Action {
 fn default()->Self { Self{kind:"chat".into(),text:String::new(),target:String::new(),value:0,condition:String::new(),punish:String::new(),tw_op:String::new(),ai_anchor:String::new(),ai_knowledge:String::new(),ai_length:String::new(),ai_style:String::new(),ai_no_repeat:None} }
}
impl Action {
 pub fn anchor<'a>(&'a self,ai:&'a AiConfig)->&'a str { self.pick(&self.ai_anchor,&ai.anchor,"all") }
 pub fn length<'a>(&'a self,ai:&'a AiConfig)->&'a str { self.pick(&self.ai_length,&ai.answer_length,"") }
 fn pick<'a>(&'a self,mine:&'a str,fallback:&'a str,empty:&'a str)->&'a str {
  if mine.is_empty() { if fallback.is_empty() {empty} else {fallback} } else {mine}
 }
 /// "" herda o perfil, "on"/"off" obriga.
 pub fn knowledge(&self,ai:&AiConfig)->bool {
  match self.ai_knowledge.as_str() { "on"=>true, "off"=>false, _=>ai.knowledge }
 }
 pub fn no_repeat(&self,ai:&AiConfig)->bool { self.ai_no_repeat.unwrap_or(ai.no_repeat) }
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
 "command"=>e.kind=="chat" && e.message.split_whitespace().next().is_some_and(|s| command_any(&t.pattern,s)),
 "contains"=>e.kind=="chat" && contains_any(&t.pattern,&e.message),
 "mention"=>e.kind=="chat" && mention_any(&t.pattern,&e.message),
 "voice"=>e.kind=="voice" && (t.pattern.is_empty()||contains_any(&t.pattern,&e.message)),
 other=>other==e.kind,
 }
}
/// "Mensagem contém" aceita opções separadas por vírgula: basta uma delas aparecer
/// na mensagem. Sem vírgula o comportamento continua o de antes, um trecho só.
fn contains_any(pattern:&str,message:&str)->bool {
 if pattern.trim().is_empty() { return false; }
 let text=message.to_lowercase();
 pattern.split(',').map(str::trim).any(|alt|!alt.is_empty()&&text.contains(&alt.to_lowercase()))
}
/// "Comando de chat" aceita variações separadas por vírgula, como !whislist, !whishlist:
/// qualquer uma delas como primeira palavra da mensagem dispara o fluxo. Sem vírgula o
/// comportamento continua o de antes, um comando exato.
pub(crate) fn command_any(pattern:&str,first:&str)->bool {
 if pattern.trim().is_empty() { return false; }
 pattern.split(',').map(str::trim).any(|alt|!alt.is_empty()&&first.eq_ignore_ascii_case(alt))
}
/// "Chamada pelo nome do bot": dispara quando um dos nomes da lista aparece como
/// palavra inteira. Sem a barreira de palavra, "arromba" dispararia dentro de
/// "arrombado"; com ela, "@Arroba", "ArrobaSrv," e "chama o arromba" continuam passando.
pub fn mention_any(pattern:&str,message:&str)->bool {
 if pattern.trim().is_empty() { return false; }
 let text=message.to_lowercase();
 pattern.split(',').map(str::trim).filter(|n|!n.is_empty()).any(|n| {
  let name=n.to_lowercase();
  let mut from=0;
  while let Some(at)=text[from..].find(&name) {
   let start=from+at;let end=start+name.len();
   let before=text[..start].chars().next_back().is_none_or(|c|!c.is_alphanumeric());
   let after=text[end..].chars().next().is_none_or(|c|!c.is_alphanumeric());
   if before&&after { return true; }
   from=end;
  }
  false
 })
}
/// Primeira opção da lista: é ela que a prévia usa para simular o gatilho.
fn first_option(pattern:&str)->&str {
 pattern.split(',').map(str::trim).find(|alt|!alt.is_empty()).unwrap_or(pattern.trim())
}
pub fn preview_event(f:&Flow,e:Event)->Event {
 match f.trigger.kind.as_str() {
 "timer"=>crate::timers::event(f,true),
 "voice"=>Event{kind:"voice".into(),user:"streamer".into(),user_id:"local-streamer".into(),role:"broadcaster".into(),..e},
 "command" if !f.trigger.pattern.is_empty()=>{
  let words:Vec<&str>=e.message.split_whitespace().collect();
  let rest:&[&str]=if words.first().is_some_and(|w|w.starts_with('!')){&words[1..]}else{&words[..]};
  let message=std::iter::once(first_option(&f.trigger.pattern)).chain(rest.iter().copied()).collect::<Vec<_>>().join(" ");
  Event{kind:"chat".into(),message,..e}
 },
 "contains" if !f.trigger.pattern.is_empty()=>{
  let alt=first_option(&f.trigger.pattern);
  let message=if e.message.to_lowercase().contains(&alt.to_lowercase()){e.message}
   else if e.message.trim().is_empty(){alt.to_owned()}
   else{format!("{} {}",e.message.trim(),alt)};
  Event{kind:"chat".into(),message,..e}
 },
 // A prévia chama o bot pelo nome antes da fala, como faria um espectador.
 "mention" if !f.trigger.pattern.is_empty()=>{
  let alt=first_option(&f.trigger.pattern);
  let message=if e.message.to_lowercase().contains(&alt.to_lowercase()){e.message}
   else if e.message.trim().is_empty(){alt.to_owned()}
   else{format!("{} {}",alt,e.message.trim())};
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
 if f.trigger.kind=="command" {let alts:Vec<&str>=f.trigger.pattern.split(',').map(str::trim).filter(|s|!s.is_empty()).collect();if alts.is_empty()||alts.iter().any(|c|!c.starts_with('!')||c.contains(char::is_whitespace)){return Err("Cada comando começa com ! e não tem espaço. Separe variações com vírgula, como !whislist, !whishlist".into())}}
 if f.trigger.kind=="mention" && f.trigger.pattern.trim().is_empty() {return Err("Informe pelo menos um nome do bot, separados por vírgula".into())}
 if !SEND_TYPES.contains(&f.send_type.as_str()){return Err("Forma de envio inválida".into())}
 if !SEND_COLORS.contains(&f.send_color.as_str()){return Err("Cor do anúncio inválida".into())}
 if f.send_type=="shoutout"&&!needs_message(f){return Err("O destaque de canal usa o texto de uma ação que envia mensagem: escreva ali o canal de destino".into())}
 if !f.audio.is_empty()&&(f.audio.len()>64||f.audio.contains(char::is_whitespace)){return Err("Áudio inválido: escolha um som da biblioteca".into())}
 if !f.audio_volume.is_finite()||!(0.0..=1.0).contains(&f.audio_volume){return Err("Volume do áudio: escolha de 0% a 100%".into())}
 for a in &f.actions {
 if !["chat","ai","ai.generate","memory","webhook","discord","overlay","delay","script","points","tts","variable.set","variable.increment","variable.delete","punish","twitch"].contains(&a.kind.as_str()) {return Err("Tipo de ação inválido".into())}
 if a.kind=="ai.generate" {let (scope,name)=crate::variables::target(crate::ai::response_target(a))?;if scope!="local"||name=="aiSuccess"{return Err("Guarde a resposta da IA numa variável local de texto".into())}}
 if a.kind.starts_with("variable."){crate::variables::target(&a.target)?;}
 if a.text.len()>32768 {return Err("Ação muito longa".into())}
 if a.kind=="delay" && !(0..=30000).contains(&a.value) {return Err("Espera máxima: 30 segundos".into())}
 if a.kind=="punish" {
  if !PUNISH_MODES.contains(&a.punish.as_str()){return Err("Punição: escolha silenciar, banir ou avisar".into())}
  if !PUNISH_TARGETS.contains(&a.target.as_str()){return Err("Punição: escolha quem leva a ação, quem enviou ou o primeiro argumento".into())}
  if a.punish=="timeout"&&!(1..=1209600).contains(&a.value){return Err("Punição: duração de 1 segundo a 14 dias".into())}
  if a.text.len()>500{return Err("Punição: o motivo pode ter até 500 caracteres".into())}
 }
 if a.kind=="twitch" {
  if !crate::twitch_ops::OPS.contains(&a.tw_op.as_str()){return Err("Ação na Twitch: escolha uma operação válida".into())}
  if ["timeout","slow"].contains(&a.tw_op.as_str())&&a.value<0{return Err("Ação na Twitch: a duração não pode ser negativa".into())}
  if a.tw_op=="title"&&a.text.chars().count()>140{return Err("Ação na Twitch: o título pode ter até 140 caracteres".into())}
 }
 if !a.ai_anchor.is_empty()&&!ANCHORS.contains(&a.ai_anchor.as_str()){return Err("Ancoragem da IA inválida".into())}
 if !a.ai_length.is_empty()&&!LENGTHS.contains(&a.ai_length.as_str()){return Err("Tamanho da resposta da IA inválido".into())}
 if !a.ai_knowledge.is_empty()&&!["on","off"].contains(&a.ai_knowledge.as_str()){return Err("Base de conhecimento inválida".into())}
 if a.ai_style.len()>600 {return Err("Tom deste bloco muito longo".into())}
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
 #[test] fn gatilho_de_voz_aceita_variacoes_com_virgula() {
  let t=Trigger{kind:"voice".into(),pattern:"troca o jogo, minecraft".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0};
  let mut v=e();v.kind="voice".into();v.message="troca o jogo para minecraft".into();
  assert!(matches(&t,&v));
  v.message="muda o jogo agora".into();assert!(!matches(&t,&v));
  v.message="poe minecraft ai".into();assert!(matches(&t,&v));
 }
 #[test] fn preview_event_follows_the_trigger() {
  let mut f=Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),reply_to:false,id:"f".into(),profile_id:"p".into(),name:"Minecraft".into(),enabled:true,trigger:Trigger{kind:"timer".into(),pattern:String::new(),permission:"everyone".into(),cooldown:0,user_cooldown:0},actions:vec![],layout:Value::Null};
  let t=preview_event(&f,e());
  assert_eq!((t.kind.as_str(),t.user.as_str(),t.user_id.as_str(),t.message.as_str(),t.simulated),("timer","BotLive","","",true));
  f.trigger.kind="command".into();f.trigger.pattern="!minecraft".into();
  let t=preview_event(&f,e());
  assert_eq!((t.kind.as_str(),t.message.as_str()),("chat","!minecraft tudo bem"));
  f.trigger.kind="contains".into();f.trigger.pattern="minecraft".into();
  assert_eq!(preview_event(&f,e()).message,"!oi tudo bem minecraft");
  f.trigger.pattern="minecraft, cs2".into();
  assert_eq!(preview_event(&f,e()).message,"!oi tudo bem minecraft","a prévia usa a primeira opção da lista");
  f.trigger.kind="voice".into();f.trigger.pattern.clear();
  let t=preview_event(&f,e());
  assert_eq!((t.kind.as_str(),t.user_id.as_str(),t.message.as_str()),("voice","local-streamer","!oi tudo bem"));
  f.trigger.kind="follow".into();
  assert_eq!(preview_event(&f,e()).kind,"follow");
 }
 #[test] fn contains_trigger_accepts_a_list_separated_by_commas() {
  let mut t=Trigger{kind:"contains".into(),pattern:"comprei, comprar , gastei".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0};
  let mut m=e();
  m.message="amigo comprou ontem".into();assert!(!matches(&t,&m),"não precisa colar com o radical");
  m.message="oi eu comprei sim".into();assert!(matches(&t,&m));
  m.message="vou comprar hoje".into();assert!(matches(&t,&m));
  m.message="só conversa por aqui".into();assert!(!matches(&t,&m));
  m.message="agora é pix".into();t.pattern="pix".into();assert!(matches(&t,&m),"sem vírgula continua sendo um trecho só");
  t.pattern.clear();assert!(!matches(&t,&m));
 }
 #[test] fn mention_trigger_waits_for_one_of_the_bot_names() {
  let mut t=Trigger{kind:"mention".into(),pattern:"Arroba, ArrobaSrv, arromba".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0};
  let mut m=e();
  m.message="Arroba, chega aí".into();assert!(matches(&t,&m));
  m.message="@arrobasrv bora jogar".into();assert!(matches(&t,&m),"maiúsculas e arroba do Twitter não atrapalham");
  m.message="ArrobaSrv mandou salve".into();assert!(matches(&t,&m));
  m.message="que arrombado, hein".into();assert!(!matches(&t,&m),"só palavra inteira: arromba não entra em arrombado");
  m.message="só conversa por aqui".into();assert!(!matches(&t,&m));
  t.pattern.clear();assert!(!matches(&t,&m),"lista vazia não dispara nada");
  t.pattern="arromba".into();m.kind="follow".into();assert!(!matches(&t,&m),"só mensagem de chat");
 }
 #[test] fn mention_preview_calls_the_bot_and_the_flow_needs_a_name() {
  let mut f=Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),reply_to:false,id:uuid::Uuid::new_v4().to_string(),profile_id:uuid::Uuid::new_v4().to_string(),name:"Salve".into(),enabled:true,trigger:Trigger{kind:"mention".into(),pattern:"Arroba, arromba".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0},actions:vec![Action{kind:"chat".into(),text:"Oi!".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}],layout:Value::Null};
  assert!(validate_flow(&f).is_ok());
  assert_eq!(preview_event(&f,e()).message,"Arroba !oi tudo bem","a prévia já começa chamando o bot");
  f.trigger.pattern.clear();assert_eq!(validate_flow(&f).unwrap_err(),"Informe pelo menos um nome do bot, separados por vírgula");
 }
 #[test] fn delivery_type_color_and_flow_audio_rules() {
  let mut f=Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),reply_to:false,id:uuid::Uuid::new_v4().to_string(),profile_id:uuid::Uuid::new_v4().to_string(),name:"ifood".into(),enabled:true,trigger:Trigger{kind:"command".into(),pattern:"!ifood".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0},actions:vec![Action{kind:"chat".into(),text:"Bora".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}],layout:Value::Null};
  assert!(validate_flow(&f).is_ok());
  f.send_type="announce".into();f.send_color="purple".into();assert!(validate_flow(&f).is_ok());
  f.send_type="letras".into();assert!(validate_flow(&f).is_err());
  f.send_type="pin".into();f.send_color="vermelho".into();assert!(validate_flow(&f).is_err());
  f.send_color="primary".into();f.audio="audio com espaco".into();assert!(validate_flow(&f).is_err());
  f.audio=uuid::Uuid::new_v4().to_string();f.audio_volume=1.5;assert!(validate_flow(&f).is_err());
  f.audio_volume=0.8;assert!(validate_flow(&f).is_ok());
 // Responder a quem enviou entra como um campo a mais: fluxos salvos antes continuam válidos.
 let mut saved=serde_json::to_value(&f).unwrap();saved.as_object_mut().unwrap().remove("replyTo");
 let back:Flow=serde_json::from_value(saved).unwrap();assert!(!back.reply_to);
 f.reply_to=true;assert!(validate_flow(&f).is_ok());
 let round:Flow=serde_json::from_value(serde_json::to_value(&f).unwrap()).unwrap();assert!(round.reply_to);
 f.reply_to=false;
  f.send_type="shoutout".into();f.actions.clear();assert!(validate_flow(&f).is_err());
  f.actions.push(Action{kind:"chat".into(),text:"outrocanal".into(),target:String::new(),value:0,condition:String::new(),..Default::default()});
  assert!(validate_flow(&f).is_ok());
  assert_eq!(SEND_TYPES,["chat","announce","pin","shoutout"]);
 }
 #[test] fn action_controls_override_or_inherit_the_profile() {
  let mut ai=AiConfig::default();let mut a=Action::default();
  assert_eq!(a.anchor(&ai),"all");assert_eq!(a.length(&ai),"");assert!(a.knowledge(&ai));assert!(!a.no_repeat(&ai));
  ai.anchor="fixed".into();ai.answer_length="short".into();ai.knowledge=false;ai.no_repeat=true;
  assert_eq!(a.anchor(&ai),"fixed");assert_eq!(a.length(&ai),"short");assert!(!a.knowledge(&ai));assert!(a.no_repeat(&ai));
  a.ai_anchor="message".into();a.ai_length="free".into();a.ai_knowledge="on".into();a.ai_no_repeat=Some(false);
  assert_eq!(a.anchor(&ai),"message");assert_eq!(a.length(&ai),"free");assert!(a.knowledge(&ai));assert!(!a.no_repeat(&ai));
  assert_eq!(ANCHORS,["all","message","chat","fixed"]);
  assert_eq!(LENGTHS,["short","medium","free"]);
  assert_eq!(KNOWLEDGE_DEPTHS,["light","standard","full"]);
 }
 #[test] fn command_trigger_accepts_variations_separated_by_commas() {
  let mut t=Trigger{kind:"command".into(),pattern:"!whislist, !whishlist, !wishlist".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0};
  let mut m=e();
  m.message="!whislist por favor".into();assert!(matches(&t,&m));
  m.message="!WHISHLIST".into();assert!(matches(&t,&m),"maiúsculas não atrapalham");
  m.message="!whishlist oloco".into();assert!(matches(&t,&m),"quem erra também dispara");
  m.message="!wishlist".into();assert!(matches(&t,&m));
  m.message="whishlist sem exclamação".into();assert!(!matches(&t,&m),"precisa ser a primeira palavra");
  m.message="!whisli".into();assert!(!matches(&t,&m),"trecho do comando não vale");
  m.kind="follow".into();assert!(!matches(&t,&m),"só mensagem de chat");
  t.pattern.clear();m.kind="chat".into();assert!(!matches(&t,&m),"lista vazia não dispara nada");
  t.pattern="!whislist, , !wishlist".into();m.message="!whislist".into();assert!(matches(&t,&m),"opção vazia no meio é ignorada");
 }
 #[test] fn command_variations_and_punish_rules_are_validated() {
  let mut f=Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),reply_to:false,id:uuid::Uuid::new_v4().to_string(),profile_id:uuid::Uuid::new_v4().to_string(),name:"Lista".into(),enabled:true,trigger:Trigger{kind:"command".into(),pattern:"!whislist, !whishlist".into(),permission:"everyone".into(),cooldown:0,user_cooldown:0},actions:vec![Action{kind:"chat".into(),text:"Pronto".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}],layout:Value::Null};
  assert!(validate_flow(&f).is_ok());
  f.trigger.pattern="!whislist, whishlist".into();
  assert!(validate_flow(&f).is_err(),"cada opção começa com !");
  f.trigger.pattern="!whis list".into();
  assert!(validate_flow(&f).is_err(),"nenhuma opção tem espaço");
  f.trigger.pattern="whislist".into();
  assert!(validate_flow(&f).is_err(),"sem exclamação não é comando");
  f.trigger.pattern="!whislist, !whishlist".into();
  f.actions=vec![Action{kind:"punish".into(),text:"Regras da comunidade".into(),target:"sender".into(),value:60,condition:String::new(),punish:"timeout".into(),..Default::default()}];
  assert!(validate_flow(&f).is_ok());
  f.actions[0].punish="mute".into();assert_eq!(validate_flow(&f).unwrap_err(),"Punição: escolha silenciar, banir ou avisar");
  f.actions[0].punish="timeout".into();f.actions[0].value=0;assert_eq!(validate_flow(&f).unwrap_err(),"Punição: duração de 1 segundo a 14 dias");
  f.actions[0].value=1209601;assert!(validate_flow(&f).is_err());
  f.actions[0].value=1209600;assert!(validate_flow(&f).is_ok(),"14 dias cabe");
  f.actions[0].target="qualquer".into();assert_eq!(validate_flow(&f).unwrap_err(),"Punição: escolha quem leva a ação, quem enviou ou o primeiro argumento");
  f.actions[0].target="first".into();f.actions[0].punish="ban".into();f.actions[0].value=0;
  assert!(validate_flow(&f).is_ok(),"ban e aviso não usam duração");
  assert_eq!(PUNISH_MODES,["timeout","ban","warn"]);
  assert_eq!(PUNISH_TARGETS,["sender","first"]);
  // Ações salvas antes da punição existir continuam recebendo o campo vazio.
  let mut saved=serde_json::to_value(&f.actions[0]).unwrap();saved.as_object_mut().unwrap().remove("punish");
  let back:Action=serde_json::from_value(saved).unwrap();assert_eq!(back.punish,"");
 }
}
