use crate::{db::Db,model::*,ai,oauth,platforms,vault,modules};
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
 pub chat_extras:Mutex<crate::chat_extras::State>,
 pub seen:Mutex<HashMap<String,Instant>>,
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
 let rt=Arc::new(Self{actor:Mutex::new(actor),db,base,http,tx,broadcast,app:Mutex::new(None),connections:Mutex::new(HashMap::new()),statuses:Mutex::new(HashMap::new()),discord_tasks:Mutex::new(HashMap::new()),discord_status:Mutex::new(HashMap::new()),discord_cache:Mutex::new(HashMap::new()),timer_pending:Mutex::new(std::collections::HashSet::new()),cooldowns:Mutex::new(HashMap::new()),conversation:Mutex::new(crate::conversation::History::default()),chat_extras:Mutex::new(crate::chat_extras::State::default()),seen:Mutex::new(HashMap::new()),module_lock:Mutex::new(()),vault_lock:Mutex::new(()),send_locks:Mutex::new(HashMap::new()),api_token:format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple()),api_port:Mutex::new(0)});
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
 pub async fn send(&self,p:&Profile,e:&Event,text:&str)->Result<(),String>{
 let current=self.db.profile(&p.id)?;let p=&current;
 if text.trim().is_empty(){return Ok(())}
 if blocked(text,p){return Err("Mensagem bloqueada pelas restrições do perfil".into())}
 if e.simulated {self.log(&p.id,"chat",&format!("[Simulação] {text}"),"success");return Ok(())}
 let limiter={self.send_locks.lock().unwrap().entry(p.id.clone()).or_insert_with(||Arc::new(tokio::sync::Mutex::new(Instant::now()-Duration::from_secs(60)))).clone()};
 let mut last=limiter.lock().await;
 let interval=Duration::from_millis(1600);
 if last.elapsed()<interval{tokio::time::sleep(interval-last.elapsed()).await;}
 let fresh=self.db.profile(&p.id)?;
 if e.kind=="timer"&&e.data["timerId"].is_string()&&!e.simulated&&!crate::timers::event_active(self,e){return Err("Timer pausado ou perfil desconectado".into())}
 if blocked(text,&fresh){return Err("Mensagem bloqueada pelas restrições atuais do perfil".into())}
 // A reply always goes back to the channel the question came from.
 if let Some(channel)=crate::discord::from_discord(e){
  if crate::discord::config(self,&fresh.id)["enabled"]!=true{return Err("O bot do Discord está desativado".into())}
  if channel.is_empty(){return Err("O Discord não informou o canal de origem".into())}
  crate::discord::post(self,&fresh,&channel,text).await?;
 }else{
  platforms::send(self,&fresh,text).await?;
 }
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
 crate::chat_extras::sound(&rt,&p,&e);
 match crate::chat_extras::response(&rt,&p,&e){Ok(Some(text))=>{if let Err(err)=rt.send(&p,&e,&text).await{rt.log(&p.id,"txt",&err,"error");}return},Err(err)=>{rt.log(&p.id,"txt",&err,"error");return},_=>{}}
 if e.kind=="chat" {
 match modules::chat(&rt,&p,&e) {
 Ok(Some(reply))=>{if let Err(err)=rt.send(&p,&e,&reply).await{rt.log(&p.id,"module",&err,"error");}return},
 Err(err)=>{rt.log(&p.id,"module",&err,"error");return},_=>{}
 }
 }
 for f in rt.db.flows(&p.id).unwrap_or_default().into_iter().filter(|f|f.enabled&&(matches(&f.trigger,&e)||(e.kind=="timer"&&f.trigger.kind=="timer"&&e.data["timerId"]==f.id))){
 if e.kind=="timer"&&!e.simulated&&!crate::timers::event_active(&rt,&e){continue}
 let allowed={
 let mut cd=rt.cooldowns.lock().unwrap();
 cd.retain(|_,t|t.elapsed()<Duration::from_secs(86400));
 let global=format!("{}:{}:{}",p.id,f.id,e.simulated);let user=format!("{global}:{}",e.user_id);
 let blocked=cd.get(&global).is_some_and(|t|t.elapsed().as_secs()<f.trigger.cooldown)||cd.get(&user).is_some_and(|t|t.elapsed().as_secs()<f.trigger.user_cooldown);
 if !blocked{cd.insert(global,Instant::now());cd.insert(user,Instant::now());} !blocked
 };
 if !allowed{rt.log(&p.id,"cooldown",&format!("{} está em intervalo",f.name),"info");continue}
 rt.log(&p.id,"flow",&format!("Iniciando {}",f.name),"info");
 let count=if f.counter{match if e.simulated{crate::command_counter::get(&rt.db,&p.id,&f.id).and_then(|n|n.checked_add(1).ok_or("Contador excedeu o limite".into()))}else{crate::command_counter::change(&rt.db,&p.id,&f.id,None)}{Ok(n)=>Some(n),Err(err)=>{rt.log(&p.id,"counter",&err,"error");continue}}}else{None};
 if let Some(n)=count{if !e.simulated{rt.emit("command-counter",json!({"profileId":p.id,"id":f.id,"value":n}));}}
 let mut variables=match crate::variables::Context::new(&rt.db,&p,&e,Some(&f)){Ok(v)=>v,Err(err)=>{rt.log(&p.id,"variables",&err,"error");continue}};
 if let Some(n)=count{variables.set_command_count(n);}
 for a in &f.actions {
 if e.kind=="timer"&&!e.simulated&&!crate::timers::event_active(&rt,&e){break}
 if !a.condition.is_empty()&&!e.message.to_lowercase().contains(&a.condition.to_lowercase()){continue}
 let result=tokio::time::timeout(Duration::from_secs(65),action(&rt,&p,&e,a,&mut variables,&history)).await;
 match result{
 Ok(Ok(()))=>rt.log(&p.id,"action",&format!("{} · {}",f.name,a.kind),"success"),
 Ok(Err(err))=>{rt.log(&p.id,"action",&format!("{} · {err}",f.name),"error");break},
 Err(_)=>{rt.log(&p.id,"action","A ação excedeu o tempo permitido","error");break}
 }
 }
 }
}
async fn action(rt:&Arc<Runtime>,p:&Profile,e:&Event,a:&Action,variables:&mut crate::variables::Context,history:&[Value])->Result<(),String>{
 let text=if a.kind=="variable.delete"{String::new()}else if a.kind=="script"{a.text.clone()}else{variables.render(&a.text)?};
 if a.kind.starts_with("variable."){variables.change(&rt.db,p,e,&a.target,a.kind.trim_start_matches("variable."),crate::variables::typed(&text))?;return Ok(())}
 if blocked(&text,p)&&matches!(a.kind.as_str(),"chat"|"tts"|"overlay"|"discord"){return Err("Conteúdo bloqueado".into())}
 // Simulations never perform external effects or change persistent module/vault state.
 if e.simulated && a.kind!="chat" {if matches!(a.kind.as_str(),"ai"|"ai.generate"){crate::ai::save_response(variables,rt,p,e,a,"[Prévia: resposta contextual da IA]",false)?;}rt.log(&p.id,"simulation",&format!("Executaria {}: {}",a.kind,text.chars().take(200).collect::<String>()),"success");return Ok(())}
 match a.kind.as_str(){
 "chat"=>rt.send(p,e,&text).await,
 "ai"|"ai.generate"=>{
 let (output,success)=match ai::conversation(&rt.http,&rt.base,p,e,&text,history).await{
 Ok(answer)=>(answer,true),Err(err)=>{rt.log(&p.id,"ai",&err,"error");(p.ai.fallback.clone(),false)}
 };
 if blocked(&output,p){return Err("Resposta alternativa bloqueada pelas restrições do perfil".into())}
 ai::save_response(variables,rt,p,e,a,&output,success)?;
 if a.kind=="ai"{rt.send(p,e,&output).await?;}
 if p.ai.remember {let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p.id)?;let name=format!("usuarios/{}.md",safe_name(&e.user_id));vault::write(&rt.base,&p.id,&name,&format!("{} disse: {}",e.user,e.message),true)?;}
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
 rt.send(p,e,&reply).await
 },
 _=>Err("Ação desconhecida".into())
 }
}
fn safe_name(s:&str)->String {let out:String=s.chars().filter(|c|c.is_ascii_alphanumeric()||*c=='-'||*c=='_').take(80).collect();if out.is_empty(){"anonimo".into()}else{out}}
