use crate::{cmd_manager, db::Db,model::*,ai,oauth,platforms,vault,modules};
use serde_json::{json,Value};
use std::{collections::HashMap,path::PathBuf,sync::{Arc,Mutex},time::{Instant,Duration}};
use tokio::sync::{broadcast,mpsc,Semaphore};
use tauri::Emitter;
pub struct Runtime {
 pub db:Db,pub base:PathBuf,pub http:reqwest::Client,
 pub tx:mpsc::Sender<Event>,pub broadcast:broadcast::Sender<Value>,
 pub app:Mutex<Option<tauri::AppHandle>>,
 pub connections:Mutex<HashMap<String,tokio::task::JoinHandle<()>>>,
 pub statuses:Mutex<HashMap<String,String>>,
 pub discord_tasks:Mutex<HashMap<String,tokio::task::JoinHandle<()>>>,
 pub discord_status:Mutex<HashMap<String,String>>,
 pub discord_cache:Mutex<HashMap<String,crate::discord::Cache>>,
 pub timer_pending:Mutex<std::collections::HashSet<String>>,
 pub cooldowns:Mutex<HashMap<String,Instant>>,
 pub conversation:Mutex<crate::conversation::History>,
 pub live:crate::live_state::LiveStates,
 pub listen:crate::listen::Sessions,
 pub chat_extras:Mutex<crate::chat_extras::State>,
 pub seen:Mutex<HashMap<String,Instant>>,
 pub seen_chatters:Mutex<HashMap<String,(String,Instant)>>,
 pub module_lock:Mutex<()>,pub vault_lock:Mutex<()>,pub send_locks:Mutex<HashMap<String,Arc<tokio::sync::Mutex<Instant>>>>,
 pub api_token:String,pub api_port:Mutex<u16>,pub actor:Mutex<String>,
}
impl Runtime {
 pub fn new(base:PathBuf)->Result<Arc<Self>,String>{
 std::fs::create_dir_all(&base).map_err(|e|e.to_string())?;
 let db=Db::open(&base.join("botlive.sqlite"))?;
 let (tx,rx)=mpsc::channel(512);let (broadcast,_)=broadcast::channel(512);
 let http=reqwest::Client::builder().timeout(Duration::from_secs(30)).redirect(reqwest::redirect::Policy::none()).build().map_err(|e|e.to_string())?;
 let actor=if db.get("accessEnabled")==true{"locked"}else{"owner"}.to_owned();
 let rt=Arc::new(Self{actor:Mutex::new(actor),db,base,http,tx,broadcast,app:Mutex::new(None),connections:Mutex::new(HashMap::new()),statuses:Mutex::new(HashMap::new()),discord_tasks:Mutex::new(HashMap::new()),discord_status:Mutex::new(HashMap::new()),discord_cache:Mutex::new(HashMap::new()),timer_pending:Mutex::new(std::collections::HashSet::new()),cooldowns:Mutex::new(HashMap::new()),conversation:Mutex::new(crate::conversation::History::default()),live:crate::live_state::LiveStates::default(),listen:crate::listen::Sessions::default(),chat_extras:Mutex::new(crate::chat_extras::State::default()),seen:Mutex::new(HashMap::new()),seen_chatters:Mutex::new(HashMap::new()),module_lock:Mutex::new(()),vault_lock:Mutex::new(()),send_locks:Mutex::new(HashMap::new()),api_token:format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple()),api_port:Mutex::new(0)});
 tokio::spawn(worker(rt.clone(),rx));Ok(rt)
 }
 pub fn emit(&self,kind:&str,payload:Value){
 let _=self.broadcast.send(json!({"type":kind,"payload":payload}));
 let actor=crate::access::current(self);
 let visible=actor=="owner"||(actor!="locked"&&payload["profileId"].as_str().is_some_and(|id|self.db.profile(id).is_ok_and(|p|p.editors.contains(&actor))));
 if visible{if let Some(app)=self.app.lock().unwrap().as_ref(){let _=app.emit(kind,payload);}}
 }
 pub fn log(&self,p:&str,kind:&str,msg:&str,status:&str){let log=self.db.log(p,kind,msg,status);self.emit("activity",serde_json::to_value(log).unwrap());}
 pub fn status(&self,p:&str,status:&str){self.statuses.lock().unwrap().insert(p.into(),status.into());self.emit("connection",json!({"profileId":p,"status":status}));}
 pub async fn submit(&self,e:Event)->Result<(),String>{
 if e.kind=="timer"&&!e.simulated{return Err("Timers reais são executados apenas pelo agendador interno".into())}
 self.db.profile(&e.profile_id)?;
 if e.message.len()>16000||e.id.len()>200{return Err("Evento muito grande".into())}
 {
 let mut seen=self.seen.lock().unwrap();seen.retain(|_,t|t.elapsed()<Duration::from_secs(600));
 let key=format!("{}:{}",e.profile_id,e.id);if seen.contains_key(&key){return Ok(())}
 self.tx.try_send(e).map_err(|_|"Fila cheia. Aguarde e tente novamente.".to_string())?;
 seen.insert(key,Instant::now());
 }
 Ok(())
 }
 pub async fn connect(self:&Arc<Self>,id:&str)->Result<(),String>{
 let p=self.db.profile(id)?;
 self.disconnect(id);
 if p.platform=="kick"{return Err("A recepção de eventos Kick exige um webhook público. Configure uma ponte que encaminhe eventos para a API local; o envio de mensagens já está disponível após OAuth.".into())}
 let _=oauth::token(&self.http,&p,"bot").await?;
 self.status(id,"connecting");
 let rt=self.clone();let key=id.to_string();
 let task=tokio::spawn(async move{
 let mut delay=2;
 loop {
 let result=if p.platform=="twitch"{platforms::twitch(rt.clone(),p.clone()).await}else{platforms::youtube(rt.clone(),p.clone()).await};
 if let Err(e)=result {rt.log(&p.id,"connection",&e,"error");}
 rt.status(&p.id,"reconnecting");
 tokio::time::sleep(Duration::from_secs(delay)).await;delay=(delay*2).min(60);
 }
 });
 self.connections.lock().unwrap().insert(key,task);Ok(())
 }
 pub fn disconnect(&self,id:&str){if let Some(h)=self.connections.lock().unwrap().remove(id){h.abort();}self.status(id,"offline");}
 pub async fn send(&self,p:&Profile,e:&Event,text:&str)->Result<(),String>{self.send_with(p,e,text,None).await}
 pub async fn send_with(&self,p:&Profile,e:&Event,text:&str,f:Option<&Flow>)->Result<(),String>{
 let current=self.db.profile(&p.id)?;let p=&current;
 let (kind,color)=f.map(|f|(f.send_type.as_str(),f.send_color.as_str())).unwrap_or(("chat","primary"));
 let label=if kind=="announce"{"Anúncio"}else if kind=="pin"{"Mensagem fixada"}else if kind=="shoutout"{"Destaque"}else{"Mensagem"};
 if text.trim().is_empty(){return Ok(())}
 if blocked(text,p){return Err("Mensagem bloqueada pelas restrições do perfil".into())}
 if e.simulated {let tag=if kind=="chat" {String::new()}else{format!(" [{label}]")};self.log(&p.id,"chat",&format!("[Simulação]{tag} {text}"),"success");return Ok(())}
 let limiter={self.send_locks.lock().unwrap().entry(p.id.clone()).or_insert_with(||Arc::new(tokio::sync::Mutex::new(Instant::now()-Duration::from_secs(60)))).clone()};
 let mut last=limiter.lock().await;
 let interval=Duration::from_millis(1600);
 if last.elapsed()<interval{tokio::time::sleep(interval-last.elapsed()).await;}
 let fresh=self.db.profile(&p.id)?;
 if e.kind=="timer"&&e.data["timerId"].is_string()&&!e.simulated&&!crate::timers::event_active(self,e){return Err("Timer pausado ou perfil desconectado".into())}
 if blocked(text,&fresh){return Err("Mensagem bloqueada pelas restrições atuais do perfil".into())}
 // A reply always goes back to the channel the question came from.
 let note=if kind=="chat" {None}else{Some("Anúncio, fixação e destaque valem só para o chat da Twitch: publicado como mensagem comum.")};
 // Responder à pessoa: a Twitch prende a mensagem no fio de quem falou e o Discord
 // cita a original. Quem não tem essa API avisa no Histórico e envia a mensagem comum.
 let want_reply=f.map(|f|f.reply_to).unwrap_or(false)&&kind=="chat";
 let mut reply=String::new();let mut reply_note:Option<String>=None;
 if want_reply {
  if crate::discord::from_discord(e).is_some(){reply=e.data["discord"]["id"].as_str().unwrap_or("").to_owned();}
  else if fresh.platform=="twitch"{reply=e.data["message_id"].as_str().unwrap_or("").to_owned();}
  else {let who=if fresh.platform=="kick"{"O Kick"}else if fresh.platform=="youtube"{"O YouTube"}else{"Esta plataforma"};reply_note=Some(format!("{who} não tem resposta direcionada: a mensagem saiu sem o fio de quem falou."));}
 }
 if let Some(channel)=crate::discord::from_discord(e){
  if crate::discord::config(self,&fresh.id)["enabled"]!=true{return Err("O bot do Discord está desativado".into())}
  if channel.is_empty(){return Err("O Discord não informou o canal de origem".into())}
  if reply.is_empty(){crate::discord::post(self,&fresh,&channel,text).await?;}else{crate::discord::post_reply(self,&fresh,&channel,text,Some(&reply)).await?;}
  if let Some(note)=note{self.log(&p.id,"discord",note,"info");}
 }else{
  platforms::send_as(self,&fresh,text,kind,color,if reply.is_empty(){None}else{Some(reply.as_str())}).await?;
  if kind!="chat"&&fresh.platform!="twitch"{if let Some(note)=note{self.log(&p.id,"chat",note,"info");}}
 }
 if let Some(note)=reply_note{self.log(&p.id,"chat",&note,"info");}
 *last=Instant::now();
 self.conversation.lock().unwrap().sent(&p.id,text);
 self.log(&p.id,"chat",text,"success");Ok(())
 }
}
async fn worker(rt:Arc<Runtime>,mut rx:mpsc::Receiver<Event>){
 let permits=Arc::new(Semaphore::new(16));
 while let Some(event)=rx.recv().await{
 let Ok(permit)=permits.clone().acquire_owned().await else {break};
 let rt=rt.clone();
 tokio::spawn(async move{let _permit=permit;process(rt,event).await;});
 }
}
pub async fn process(rt:Arc<Runtime>,e:Event){
 let _pending=crate::timers::Pending(rt.clone(),if e.kind=="timer"&&!e.simulated{e.data["timerId"].as_str().map(str::to_owned)}else{None});
 let Ok(p)=rt.db.profile(&e.profile_id) else{return};
 if e.kind=="chat"&&!e.simulated&&!p.bot_id.is_empty()&&e.user_id==p.bot_id{return}
 if e.kind=="chat"{crate::twitch_ops::note_chatter(&rt,&e);}
  // Novo: Tratar !cmd apenas para moderadores e streamer
  if e.kind=="chat"&&!e.simulated&&!p.bot_id.is_empty()&&e.user_id!=p.bot_id&&e.message.starts_with("!cmd") {
    // Verificar se o usuário tem permissão de moderador ou broadcaster
    if crate::model::permitted("moderator", &e.role) {
      let op_result = cmd_manager::parse(&e.message);
      match op_result {
        Ok(op) => {
          let exec_result = cmd_manager::execute(&rt.db, &e.profile_id, op);
          match exec_result {
            Ok(msg) => {
                // Envia a resposta para o chat
                let _ = rt.send(&p, &e, &msg).await;
            }
            Err(err) => {
                let _ = rt.send(&p, &e, &err).await;
            }
          }
        }
        Err(parse_err) => {
            let _ = rt.send(&p, &e, &format!("Erro ao processar comando: {}", parse_err)).await;
        }
      }
      return;
    }
  }
 // Nativos !setgame e !settitle: moderadores e streamer digitam no chat e o
 // bot executa pela própria conta, sem precisar de automação. Quem criar uma
 // automação própria com o mesmo comando usa o fluxo em vez do nativo.
 if e.kind=="chat"&&!e.simulated&&!p.bot_id.is_empty()&&e.user_id!=p.bot_id{
  if let Some((op,rest))=crate::twitch_ops::builtin_command(&e.message){
   if crate::model::permitted("moderator",&e.role){
    let first=e.message.split_whitespace().next().unwrap_or("");
    let custom=rt.db.flows(&e.profile_id).unwrap_or_default().iter().any(|f|f.enabled&&f.trigger.kind=="command"&&crate::model::command_any(&f.trigger.pattern,first));
    if !custom{
     if rest.trim().is_empty(){let _=rt.send(&p,&e,if op=="game"{"Use: !setgame Nome do Jogo"}else{"Use: !settitle Novo título"}).await;}
     else{
      let a=crate::model::Action{kind:"twitch".into(),text:String::new(),target:String::new(),value:0,condition:String::new(),punish:String::new(),tw_op:op.into(),..Default::default()};
      let trigger=if op=="game"{"!setgame"}else{"!settitle"};
      match crate::twitch_ops::run(&rt,&p,&e,&a,rest,trigger).await{
       Ok(msg)=>{let _=rt.send(&p,&e,&msg).await;},
       Err(err)=>{let _=rt.send(&p,&e,&err).await;}
      }
     }
    }
    return;
   }
  }
 }
 if e.kind=="voice"&&p.modules["voice"]!=true{rt.log(&p.id,"voice","Controle por voz desativado","info");return}
 rt.log(&p.id,&e.kind,&format!("{}: {}",e.user,e.message),"info");
 rt.emit("platform-event",serde_json::to_value(&e).unwrap());
 if e.kind=="chat" {
 if let Some(reason)=crate::moderation::detect(&rt,&p,&e){rt.log(&p.id,"moderation",reason,"info");if let Err(err)=crate::moderation::act(&rt,&p,&e,reason).await{rt.log(&p.id,"moderation",&err,"error");}return}
 if let Some(reply)=crate::discord_engage::link_command(&rt,&p,&e){if let Err(err)=rt.send(&p,&e,&reply).await{rt.log(&p.id,"discord",&err,"error");}return}
 crate::discord::mirror_chat(&rt,&p,&e);
 }
 if matches!(e.kind.as_str(),"follow"|"subscription"|"cheer"|"raid"){crate::discord::notify(&rt,&p,&e.kind,&e);}
 let history=rt.conversation.lock().unwrap().receive(&e);
 rt.live.record(&p.id,&e);
 crate::chat_extras::sound(&rt,&p,&e);
 match crate::chat_extras::response(&rt,&p,&e){Ok(Some(text))=>{if let Err(err)=rt.send(&p,&e,&text).await{rt.log(&p.id,"txt",&err,"error");}return},Err(err)=>{rt.log(&p.id,"txt",&err,"error");return},_=>{}}
 if e.kind=="chat" {
 match modules::chat(&rt,&p,&e) {
 Ok(Some(reply))=>{if let Err(err)=rt.send(&p,&e,&reply).await{rt.log(&p.id,"module",&err,"error");}return},
 Err(err)=>{rt.log(&p.id,"module",&err,"error");return},_=>{}
 }
 }
 let matched:Vec<Flow>=rt.db.flows(&p.id).unwrap_or_default().into_iter().filter(|f|f.enabled&&(matches(&f.trigger,&e)||(e.kind=="timer"&&f.trigger.kind=="timer"&&e.data["timerId"]==f.id))).collect();
 let mut ready:Vec<Flow>=Vec::new();
 for f in matched {
 if e.kind=="timer"&&!e.simulated&&!crate::timers::event_active(&rt,&e){continue}
 let allowed={
 let mut cd=rt.cooldowns.lock().unwrap();
 cd.retain(|_,t|t.elapsed()<Duration::from_secs(86400));
 let global=format!("{}:{}:{}",p.id,f.id,e.simulated);let user=format!("{global}:{}",e.user_id);
 let blocked=cd.get(&global).is_some_and(|t|t.elapsed().as_secs()<f.trigger.cooldown)||cd.get(&user).is_some_and(|t|t.elapsed().as_secs()<f.trigger.user_cooldown);
 if !blocked{cd.insert(global,Instant::now());cd.insert(user,Instant::now());} !blocked
 };
 if !allowed{rt.log(&p.id,"cooldown",&format!("{} está em intervalo",f.name),"info");continue}
 ready.push(f);
 }
 let publisher=responder(&ready);
 for f in ready {
  let answers=publisher.as_ref().is_none_or(|id|id==&f.id);
  rt.log(&p.id,"flow",&format!("Iniciando {}",f.name),"info");
  if !answers&&replies(&f){rt.log(&p.id,"action",&format!("{} · Outra automação já respondeu esta mensagem; os demais efeitos continuam",f.name),"info")}
 let count=if f.counter{match if e.simulated{crate::command_counter::get(&rt.db,&p.id,&f.id).and_then(|n|n.checked_add(1).ok_or("Contador excedeu o limite".into()))}else{crate::command_counter::change(&rt.db,&p.id,&f.id,None)}{Ok(n)=>Some(n),Err(err)=>{rt.log(&p.id,"counter",&err,"error");continue}}}else{None};
 if let Some(n)=count{if !e.simulated{rt.emit("command-counter",json!({"profileId":p.id,"id":f.id,"value":n}));}}
 let mut variables=match crate::variables::Context::new(&rt.db,&p,&e,Some(&f)){Ok(v)=>v,Err(err)=>{rt.log(&p.id,"variables",&err,"error");continue}};
 if let Some(n)=count{variables.set_command_count(n);}
 crate::chat_extras::play_flow(&rt,&p,&f,&e);
 for a in &f.actions {
  if !answers&&matches!(a.kind.as_str(),"chat"|"ai"|"ai.generate"){continue}
 if e.kind=="timer"&&!e.simulated&&!crate::timers::event_active(&rt,&e){break}
 if !a.condition.is_empty()&&!e.message.to_lowercase().contains(&a.condition.to_lowercase()){continue}
 let result=tokio::time::timeout(Duration::from_secs(65),action(&rt,&p,&e,a,&mut variables,&history,&f)).await;
 match result{
 Ok(Ok(()))=>rt.log(&p.id,"action",&format!("{} · {}",f.name,a.kind),"success"),
 Ok(Err(err))=>{rt.log(&p.id,"action",&format!("{} · {err}",f.name),"error");break},
 Err(_)=>{rt.log(&p.id,"action","A ação excedeu o tempo permitido","error");break}
 }
 }
 }
}
/// Ações que publicam a resposta no chat ou produzem o texto que a ação de chat envia.
const REPLY:&[&str]=&["chat","ai","ai.generate"];
/// True quando a automação tem alguma ação de resposta no chat.
fn replies(f:&Flow)->bool{f.actions.iter().any(|a|REPLY.contains(&a.kind.as_str()))}
/// Escolhe qual automação publica a resposta quando várias casam na mesma mensagem.
/// A ordem é a do tipo do gatilho (comando, chamada pelo nome, contém, mensagem e os
/// demais), depois o gatilho mais específico (menos opções e texto mais longo) e, no
/// empate, a posição na lista do perfil. Sem concorrência devolve None e todos respondem.
fn responder(flows:&[Flow])->Option<String>{
 if flows.len()<2{return None}
 let rank=|kind:&str|match kind{"command"=>0,"mention"=>1,"contains"=>2,"voice"=>3,"chat"=>4,_=>5};
 let mut best:Option<((usize,usize,usize),usize)>=None;
 for (i,f) in flows.iter().enumerate(){
  let options=f.trigger.pattern.split(',').map(str::trim).filter(|s|!s.is_empty()).count().max(1);
  let candidate=((rank(&f.trigger.kind),options,usize::MAX-f.trigger.pattern.len()),i);
  if best.is_none_or(|b|candidate<b){best=Some(candidate)}
 }
 best.map(|(_,i)|flows[i].id.clone())
}
async fn action(rt:&Arc<Runtime>,p:&Profile,e:&Event,a:&Action,variables:&mut crate::variables::Context,history:&[Value],f:&Flow)->Result<(),String>{
 let text=if a.kind=="variable.delete"{String::new()}else if a.kind=="script"{a.text.clone()}else{variables.render(&a.text)?};
 if a.kind.starts_with("variable."){variables.change(&rt.db,p,e,&a.target,a.kind.trim_start_matches("variable."),crate::variables::typed(&text))?;return Ok(())}
 if blocked(&text,p)&&matches!(a.kind.as_str(),"chat"|"tts"|"overlay"|"discord"){return Err("Conteúdo bloqueado".into())}
 // Simulations never perform external effects or change persistent module/vault state.
 if e.simulated && a.kind!="chat" {if matches!(a.kind.as_str(),"ai"|"ai.generate"){crate::ai::save_response(variables,rt,p,e,a,"[Prévia: resposta contextual da IA]",false)?;}rt.log(&p.id,"simulation",&format!("Executaria {}: {}",a.kind,text.chars().take(200).collect::<String>()),"success");return Ok(())}
 match a.kind.as_str(){
 "punish"=>crate::moderation::punish(rt,p,e,a,&text).await,
 "twitch"=>crate::twitch_ops::run(rt,p,e,a,&text,&f.trigger.pattern).await.map(|_|()),
 "chat"=>rt.send_with(p,e,&text,Some(f)).await,
 "ai"|"ai.generate"=>{
 let opts=crate::ai::Options{
  knowledge:if a.knowledge(&p.ai){crate::knowledge::context_enabled(&rt.base,p,Some(e),&e.message)}else{String::new()},
  live:rt.live.summary(&p.id),
  speech:crate::speech::summary(&rt.db,&p.id),
  anchor:a.anchor(&p.ai).to_owned(),length:a.length(&p.ai).to_owned(),
  no_repeat:a.no_repeat(&p.ai),style:a.ai_style.clone(),
  emotes:crate::emotes::list(&rt.db,&p.id),
 };
 let (output,success)=match ai::conversation(&rt.http,&rt.base,p,e,&text,history,&opts).await{
 Ok(answer)=>(answer,true),Err(err)=>{ai::save_response(variables,rt,p,e,a,"",false)?;return Err(err)}
 };
 if blocked(&output,p){return Err("Resposta bloqueada pelas restrições do perfil".into())}
 ai::save_response(variables,rt,p,e,a,&output,success)?;
 if a.kind=="ai"{rt.send_with(p,e,&output,Some(f)).await?;}
 if p.ai.remember && e.kind=="chat" {let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p.id)?;let platform=if crate::discord::from_discord(e).is_some(){"discord"}else{p.platform.as_str()};vault::record_interaction(&rt.base,&p.id,platform,&e.user_id,&e.user,&e.message)?;}
 Ok(())
 },
 "memory"=>{let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p.id)?;vault::write(&rt.base,&p.id,&variables.render(&a.target)?,&text,true)},
 "delay"=>{tokio::time::sleep(Duration::from_millis(a.value.clamp(0,30000) as u64)).await;Ok(())},
 "overlay"=>{rt.emit("overlay",json!({"profileId":p.id,"text":text}));Ok(())},
 "tts"=>{if p.modules["tts"]!=true{return Err("Ative o módulo Texto para voz".into())}let config=rt.db.module(&p.id,"tts");rt.emit("tts",json!({"profileId":p.id,"text":text.chars().take(config["limit"].as_u64().unwrap_or(300).clamp(1,500) as usize).collect::<String>(),"voice":config["voice"],"rate":config["rate"]}));Ok(())},
 "points"=>{if p.modules["points"]!=true{return Err("Ative o módulo de pontos".into())}modules::change_points(rt,&p.id,&e.user_id,a.value)?;Ok(())},
 "webhook"|"discord"=>{
 let target=if a.kind=="discord"{if p.modules["discord"]!=true{return Err("Ative o módulo Discord".into())}crate::secrets::get(&p.id,"discord_webhook")?}else{a.target.clone()};
 ai::validate_url(&target)?;
 let payload=if a.kind=="discord"{json!({"content":text,"allowed_mentions":{"parse":[]}})}else{json!({"text":text,"user":e.user,"channel":p.channel})};
 let res=rt.http.post(target).json(&payload).send().await.map_err(|_|"Não foi possível chamar o webhook")?;
 if !res.status().is_success(){return Err(format!("Webhook retornou HTTP {}",res.status().as_u16()))}Ok(())
 },
 "script"=>{
 let mut engine=rhai::Engine::new();engine.set_max_operations(10000);engine.set_max_expr_depths(32,16);engine.set_max_string_size(16000);engine.set_max_array_size(1000);engine.set_max_map_size(100);
 let mut scope=rhai::Scope::new();scope.push_constant("user",e.user.clone());scope.push_constant("message",e.message.clone());scope.push_constant("channel",p.channel.clone());
 let reply=engine.eval_with_scope::<String>(&mut scope,&a.text).map_err(|_|"O script falhou ou ultrapassou os limites")?;
 rt.send_with(p,e,&reply,Some(f)).await
 },
 _=>Err("Ação desconhecida".into())
 }
}
#[cfg(test)] mod tests {
 use super::*;
 fn f(name:&str,kind:&str,pattern:&str,actions:&[&str])->Flow {
  serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"profileId":uuid::Uuid::new_v4(),"name":name,"enabled":true,
  "trigger":{"kind":kind,"pattern":pattern},
  "actions":actions.iter().map(|k|json!({"kind":k,"text":"Oi"})).collect::<Vec<Value>>()})).unwrap()
 }
 #[test] fn only_the_most_specific_trigger_publishes_when_several_match() {
  let comando=f("Comando","command","!oi",&["chat"]);
  let contem=f("Contém","contains","oi, ola",&["overlay","chat"]);
  let chamada=f("Chamada","mention","bot",&["chat"]);
  let unico=f("Único","command","!oi",&["chat"]);
  assert!(responder(std::slice::from_ref(&unico)).is_none(),"sem concorrência todo mundo responde");
  assert_eq!(responder(&[contem.clone(),comando.clone(),chamada.clone()]).unwrap(),comando.id,"comando vence por tipo do gatilho");
  let mais=f("Detalhado","command","!oi tudo",&["chat"]);
  assert_eq!(responder(&[comando.clone(),mais.clone()]).unwrap(),mais.id,"o gatilho mais específico vence");
  assert_eq!(responder(&[mais.clone(),comando.clone()]).unwrap(),mais.id,"a posição na lista não inverte a especificidade");
  let primeiro=f("Primeiro","command","!oi",&["chat"]);
  let segundo=f("Segundo","command","!oi",&["chat"]);
  assert_eq!(responder(&[primeiro.clone(),segundo.clone()]).unwrap(),primeiro.id,"no empate vale a ordem da lista");
  let varias=f("Variações","command","!oi, !ola, !eai",&["chat"]);
  assert_eq!(responder(&[varias.clone(),comando.clone()]).unwrap(),comando.id,"uma opção é mais específica que três");
 }
 #[test] fn only_reply_actions_are_held_back_from_the_losing_flow() {
  assert!(replies(&f("A","command","!a",&["chat"])));
  assert!(replies(&f("B","command","!b",&["overlay","ai"])));
  assert!(replies(&f("C","command","!c",&["ai.generate"])));
  assert!(!replies(&f("D","command","!d",&["overlay","points","punish"])),"efeitos paralelos não são resposta");
 }
}
