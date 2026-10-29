//! Telemetria de leitura e agregados observados. Não controla a transmissão.
use crate::{engine::Runtime,model::{Profile,Event}};
use serde::{Serialize,Deserialize};
use serde_json::{json,Value};
use std::{collections::{HashMap,VecDeque},sync::{Arc,Mutex,atomic::{AtomicU64,Ordering}},time::{Duration,Instant}};
#[derive(Default)]
pub struct Dashboard {
 gates:Mutex<HashMap<String,Arc<tokio::sync::Mutex<Cache>>>>,
 recent:Mutex<HashMap<String,VecDeque<(i64,String)>>>,
 sessions:Mutex<()>,
 pub deliveries:Mutex<HashMap<String,Value>>,
 pub requests:Mutex<HashMap<String,Value>>,
 pub subscriptions:Mutex<HashMap<String,Value>>,
 generation:AtomicU64,
}
#[derive(Default)]
struct Cache { platform:Value, attempted:Option<Instant>, failures:u32, cooldown:u64 }
#[derive(Default,Serialize,Deserialize,Clone)]
#[serde(default,rename_all="camelCase")]
pub struct Session {
 pub id:String,pub channel_id:String,pub started_at:String,pub observed_at:i64,pub last_at:i64,pub last_viewers:Option<u64>,pub ended_at:Option<i64>,
 pub peak:u64,pub viewer_seconds:u64,pub covered_seconds:u64,
 pub samples:Vec<(i64,Option<u64>)>,pub event_ids:Vec<String>,pub messages:u64,pub follows:u64,pub subs:u64,pub resubs:u64,pub gifts:u64,pub bits:u64,pub raids:u64,pub redemptions:u64,
}
impl Dashboard {
 pub fn clear_profile(&self,id:&str){self.recent.lock().unwrap().remove(id);self.requests.lock().unwrap().remove(id);self.deliveries.lock().unwrap().remove(id);self.subscriptions.lock().unwrap().remove(id);}
 pub fn reset_after<T>(&self,work:impl FnOnce()->Result<T,String>)->Result<T,String>{let _session=self.sessions.lock().unwrap();self.generation.fetch_add(1,Ordering::SeqCst);let result=work();self.gates.lock().unwrap().clear();self.recent.lock().unwrap().clear();self.requests.lock().unwrap().clear();self.deliveries.lock().unwrap().clear();result}
}
pub fn now()->i64 {chrono::Utc::now().timestamp()}
fn session(rt:&Runtime,p:&str)->Session {serde_json::from_value(rt.db.module(p,"dashboardSession")).unwrap_or_default()}
fn save(rt:&Runtime,p:&str,s:&Session)->Result<(),String>{rt.db.set_module(p,"dashboardSession",&json!(s))}
/// Estrutura estrita: HTTP/JSON inválido jamais vira zero ou offline.
pub fn parse_stream(v:&Value)->Result<Value,String>{
 let rows=v["data"].as_array().ok_or("Resposta da Twitch sem dados válidos")?;
 if rows.is_empty(){return Ok(json!({"live":false}))}
 let s=&rows[0];
 let id=s["id"].as_str().filter(|s|!s.is_empty()).ok_or("Transmissão sem identificador")?;
 let started=s["started_at"].as_str().filter(|s|chrono::DateTime::parse_from_rfc3339(s).is_ok()).ok_or("Início da transmissão inválido")?;
 let viewers=s["viewer_count"].as_u64().ok_or("Contagem de espectadores inválida")?;
 Ok(json!({"live":true,"id":id,"startedAt":started,"viewers":viewers,"title":s["title"].as_str().unwrap_or(""),"category":s["game_name"].as_str().unwrap_or("")}))
}
fn sample(s:&mut Session,v:&Value,at:i64){
 if v["live"]!=true {if !s.id.is_empty()&&s.last_viewers.is_some(){s.samples.push((at,None));if s.samples.len()>2000{s.samples.remove(0);}}s.last_viewers=None;if !s.id.is_empty()&&s.ended_at.is_none(){s.ended_at=Some(at);}return}
 let id=v["id"].as_str().unwrap_or("");
 if s.id!=id {*s=Session{id:id.into(),started_at:v["startedAt"].as_str().unwrap_or("").into(),observed_at:at,..Default::default()};}
 s.ended_at=None;
 if at<=s.last_at{return}
 let viewers=v["viewers"].as_u64();
 let delta=at-s.last_at;
 if delta>0&&delta<=60 {if let Some(last)=s.last_viewers{s.viewer_seconds=s.viewer_seconds.saturating_add(last.saturating_mul(delta as u64));s.covered_seconds+=delta as u64;}}
 s.peak=s.peak.max(viewers.unwrap_or(0));s.samples.push((at,viewers));
 if s.samples.len()>2000{s.samples.drain(..s.samples.len()-2000);}
 s.last_at=at;s.last_viewers=viewers;
}
pub async fn platform(rt:&Arc<Runtime>,p:&Profile)->Value {
 if p.platform!="twitch"{return json!({"state":"unsupported","error":"Audiência disponível nesta etapa para Twitch","data":null})}
 if p.channel_id.is_empty()||p.client_id.is_empty(){return json!({"state":"unconfigured","error":"Autorize a conta do canal em Perfis","data":null})}
 let generation=rt.dashboard.generation.load(Ordering::SeqCst);
 let key=format!("{}|{}|{}|{}",p.id,p.platform,p.channel_id,p.client_id);
 let gate=rt.dashboard.gates.lock().unwrap().entry(key).or_default().clone();
 let mut cache=gate.lock().await;
 if rt.dashboard.generation.load(Ordering::SeqCst)!=generation{return json!({"state":"unavailable","data":null,"error":"Dados restaurados; atualize a leitura"})}
 let wait=(30u64.saturating_mul(1u64<<cache.failures.min(4))).min(300).max(cache.cooldown);
 if cache.attempted.is_some_and(|t|t.elapsed()<Duration::from_secs(wait)){return cache.platform.clone()}
 let first_attempt=cache.attempted.is_none();
 cache.attempted=Some(Instant::now());
 let mut retry_after=0;
 let mut result=tokio::time::timeout(Duration::from_secs(12),async {
  let token=crate::oauth::token(&rt.http,p,"channel").await?;
  let response=crate::helix::get(&rt.http,p,&token,"streams",&[("user_id",p.channel_id.as_str())]).send().await.map_err(|_|"Falha de rede ao consultar a Twitch".to_owned())?;
  let code=response.status().as_u16();
  if code==429 {retry_after=response.headers().get("retry-after").and_then(|v|v.to_str().ok()).and_then(|s|s.parse::<u64>().ok()).unwrap_or(0);let reset=response.headers().get("ratelimit-reset").and_then(|v|v.to_str().ok()).and_then(|s|s.parse::<i64>().ok()).map(|n|(n-now()).max(0) as u64).unwrap_or(0);retry_after=retry_after.max(reset);}
  if !response.status().is_success(){return Err(match code {401|403=>"A Twitch recusou a autorização. Confira a conta do canal em Perfis".into(),429=>"Limite de consultas da Twitch. A próxima tentativa será espaçada".into(),_=>format!("A Twitch está indisponível (HTTP {code})")})}
  let raw=response.json::<Value>().await.map_err(|_|"Resposta inválida da Twitch".to_owned())?;
  parse_stream(&raw)
 }).await.unwrap_or_else(|_|Err("A consulta da Twitch excedeu o tempo limite".into()));
 // Metadados opcionais fora do ar não invalidam o estado confirmado de transmissão.
 if let Ok(value)=result.as_mut(){if value["live"]==false {
  let info=tokio::time::timeout(Duration::from_secs(2),async {
   let token=crate::oauth::token(&rt.http,p,"channel").await?;
   let response=crate::helix::get(&rt.http,p,&token,"channels",&[("broadcaster_id",p.channel_id.as_str())]).send().await.map_err(|_|"Sem metadados")?;
   if !response.status().is_success(){return Err("Sem metadados".to_owned())}response.json::<Value>().await.map_err(|_|"Sem metadados".to_owned())
  }).await;
  if let Ok(Ok(info))=info{value["title"]=info["data"][0]["title"].clone();value["category"]=info["data"][0]["game_name"].clone();}
 }}

 // Um perfil reconfigurado não recebe resposta da conta anterior.
 if rt.dashboard.generation.load(Ordering::SeqCst)!=generation||!rt.db.profile(&p.id).is_ok_and(|current|current.channel_id==p.channel_id&&current.client_id==p.client_id&&current.platform==p.platform){return json!({"state":"unavailable","data":null,"error":"Perfil alterado; atualize a tela"})}
 let at=now();
 match result {
  Ok(data)=>{
   cache.failures=0;cache.cooldown=0;
   let _lock=rt.dashboard.sessions.lock().unwrap();if rt.dashboard.generation.load(Ordering::SeqCst)!=generation{return json!({"state":"unavailable","data":null,"error":"Dados restaurados; atualize a leitura"})}let mut s=session(rt,&p.id);let previous=s.id.clone();if first_attempt&&!s.id.is_empty(){s.last_viewers=None;if s.last_at>0&&s.last_at<at{s.samples.push((s.last_at+1,None));}}sample(&mut s,&data,at);s.channel_id=p.channel_id.clone();if previous!=s.id{rt.dashboard.recent.lock().unwrap().remove(&p.id);}
   let persistence=save(rt,&p.id,&s).err();
   cache.platform=json!({"state":"fresh","observedAt":at,"lastSuccessAt":at,"data":data,"error":null,"persistenceError":persistence});
  },
  Err(error)=>{
   cache.failures=cache.failures.saturating_add(1);cache.cooldown=retry_after;
   let previous=cache.platform["data"].clone();let last=cache.platform["lastSuccessAt"].clone();
   cache.platform=json!({"state":if previous.is_object(){"stale"}else{"unavailable"},"observedAt":at,"lastSuccessAt":last,"data":previous,"error":error});
   let _lock=rt.dashboard.sessions.lock().unwrap();if rt.dashboard.generation.load(Ordering::SeqCst)!=generation{return json!({"state":"unavailable","data":null,"error":"Dados restaurados; atualize a leitura"})}let mut s=session(rt,&p.id);
   if !s.id.is_empty(){s.samples.push((at,None));s.last_viewers=None;if s.samples.len()>2000{s.samples.remove(0);}if let Err(err)=save(rt,&p.id,&s){cache.platform["persistenceError"]=json!(err);}}
  }
 }
 cache.platform.clone()
}
/// Continua observando a audiência com a janela aberta em outra página. Uma tarefa sequencial por ciclo.
pub async fn monitor(rt:Arc<Runtime>){
 loop {for p in rt.db.profiles().unwrap_or_default(){if p.platform=="twitch"&&!p.channel_id.is_empty(){let _=platform(&rt,&p).await;}}tokio::time::sleep(Duration::from_secs(30)).await;}
}
pub fn record(rt:&Runtime,p:&Profile,e:&Event){
 if e.simulated||e.rehearsal||e.data["botliveTest"]==true||e.user_id==p.bot_id||e.data["bot"]==true{return}
 if !matches!(e.kind.as_str(),"chat"|"follow"|"subscription"|"resub"|"gift"|"cheer"|"raid"|"redemption"){return}
 let _lock=rt.dashboard.sessions.lock().unwrap();let mut s=session(rt,&p.id);
 if s.id.is_empty()||s.channel_id!=p.channel_id||s.last_viewers.is_none(){return}
 let at=now();if at-s.last_at>90{return}
 if s.event_ids.iter().any(|id|id==&e.id){return} s.event_ids.push(e.id.clone());if s.event_ids.len()>2000{s.event_ids.remove(0);}
 match e.kind.as_str(){
  "chat"=>{s.messages+=1;let mut all=rt.dashboard.recent.lock().unwrap();let q=all.entry(p.id.clone()).or_default();q.push_back((at,e.user_id.clone()));while q.front().is_some_and(|(t,_)|at-t>300)||q.len()>10000{q.pop_front();}},
  "follow"=>s.follows+=1,"subscription"=>{if e.data["is_gift"]!=true{s.subs+=1}},"resub"=>s.resubs+=1,
  "gift"=>s.gifts=s.gifts.saturating_add(e.data["total"].as_u64().unwrap_or(1)),"cheer"=>s.bits=s.bits.saturating_add(e.data["bits"].as_u64().unwrap_or(0)),"raid"=>s.raids+=1,"redemption"=>s.redemptions+=1,_=>{}
 }
 if let Err(err)=save(rt,&p.id,&s){rt.log(&p.id,"dashboard",&format!("Não foi possível guardar os totais observados: {err}"),"error");}
}
pub fn local(rt:&Runtime,p:&Profile)->Value {
 let mut s=session(rt,&p.id);if s.channel_id!=p.channel_id{s=Session::default();}let at=now();
 let (messages,people)={let all=rt.dashboard.recent.lock().unwrap();let entries=if s.id.is_empty(){None}else{all.get(&p.id)};let messages=entries.map_or(0,|q|q.iter().filter(|(t,_)|at-t<=60).count());let people=entries.map_or(0,|q|q.iter().filter(|(t,_)|at-t<=300).map(|(_,u)|u).collect::<std::collections::HashSet<_>>().len());(messages,people)};
 let goal_config=crate::livepix::config(rt,&p.id);let goal_data=crate::livepix::data(rt,&p.id);let goal=goal_config.goals.get(&goal_config.active_goal).map(|goal|json!({"enabled":goal_config.enabled,"title":goal.title,"target":goal.target,"current":goal_data["current"],"updatedAt":goal_data["updated_at"],"error":if goal_data["last_error"].is_string(){Some("A coleta da meta precisa de atenção. Confira a integração LivePix.")}else{None},"startAt":goal_config.start_timestamp}));
 let subs=rt.dashboard.subscriptions.lock().unwrap().get(&p.id).cloned().unwrap_or(Value::Null);
 json!({"session":s,"messagesPerMinute":messages,"activeChatters":people,"subscriptions":subs,"audio":rt.dashboard.deliveries.lock().unwrap().get(&p.id).cloned(),"request":rt.dashboard.requests.lock().unwrap().get(&p.id).cloned(),"observedAt":at,"pendingConfirmation":rt.intents.pending(&p.id),"goal":goal})
}
pub async fn get(rt:&Arc<Runtime>,p:&Profile)->Value {let platform=platform(rt,p).await;json!({"platform":platform,"local":local(rt,p)})}
#[cfg(test)]mod tests{
 use super::*;
 fn live(id:&str,n:u64)->Value{json!({"live":true,"id":id,"startedAt":"2026-10-29T20:00:00Z","viewers":n})}
 #[test]fn respostas_invalidas_nao_viram_zero_ou_offline(){assert!(parse_stream(&json!({"status":429})).is_err());assert!(parse_stream(&json!({"data":[{"id":"a","started_at":"invalido","viewer_count":0}]})).is_err());assert_eq!(parse_stream(&json!({"data":[]})).unwrap()["live"],false);}
 #[test]fn sessao_mantem_medias_lacunas_e_troca_sem_inflar(){let mut s=Session::default();sample(&mut s,&live("a",10),100);sample(&mut s,&live("a",20),130);assert_eq!(s.covered_seconds,30);assert_eq!(s.viewer_seconds,300);sample(&mut s,&live("a",50),300);assert_eq!(s.covered_seconds,30);assert_eq!(s.peak,50);sample(&mut s,&live("b",5),330);assert_eq!(s.peak,5);assert_eq!(s.covered_seconds,0);assert_eq!(s.samples.len(),1);}
 #[test]fn encerramento_nao_desliza_com_novas_consultas_offline(){let mut s=Session::default();sample(&mut s,&live("a",10),100);sample(&mut s,&json!({"live":false}),130);sample(&mut s,&json!({"live":false}),190);assert_eq!(s.ended_at,Some(130));assert_eq!(s.samples.last().unwrap().1,None);sample(&mut s,&live("a",20),200);assert_eq!(s.ended_at,None);assert_eq!(s.covered_seconds,0);}
 #[test]fn persistencia_recupera_id_e_observacao(){let mut s=Session::default();sample(&mut s,&live("a",2),100);let mut reopened:Session=serde_json::from_value(json!(s)).unwrap();sample(&mut reopened,&live("a",4),400);assert_eq!(reopened.observed_at,100);assert_eq!(reopened.covered_seconds,0);assert_eq!(reopened.samples.len(),2);}
}

pub async fn obs(rt:&Arc<Runtime>,p:&Profile)->Value {
 let config=crate::obs::config(rt,&p.id);
 if !config.enabled{return json!({"state":"unconfigured","data":null,"error":"Integração OBS desligada"})}
 let key=format!("obs|{}|{}|{}",p.id,config.host,config.port);
 let gate=rt.dashboard.gates.lock().unwrap().entry(key).or_default().clone();let mut cache=gate.lock().await;
 let wait=if cache.failures>0{30}else{5};
 if cache.attempted.is_some_and(|t|t.elapsed()<Duration::from_secs(wait)){return cache.platform.clone()}
 cache.attempted=Some(Instant::now());
 let result=crate::obs::telemetry(rt,p).await;let at=now();
 if result["state"]=="fresh"{cache.failures=0;cache.platform=result;}else{cache.failures=1;let previous=cache.platform["data"].clone();let last=cache.platform["lastSuccessAt"].clone();cache.platform=json!({"state":if previous.is_object(){"stale"}else{"unavailable"},"data":previous,"lastSuccessAt":last,"observedAt":at,"error":result["error"]});}
 cache.platform.clone()
}

pub async fn chatters(rt:&Arc<Runtime>,p:&Profile)->Result<(i64,Vec<String>),String>{
 let key=format!("chat|{}|{}|{}|{}",p.id,p.channel_id,p.bot_id,p.client_id);
 let gate=rt.dashboard.gates.lock().unwrap().entry(key).or_default().clone();let mut cache=gate.lock().await;
 if !cache.attempted.is_some_and(|t|t.elapsed()<Duration::from_secs(if cache.failures>0{60}else{30})) {
  cache.attempted=Some(Instant::now());match crate::labels::fetch_chatters(rt,p).await{Ok((total,users))=>{cache.failures=0;cache.platform=json!({"total":total,"users":users});},Err(error)=>{cache.failures=1;cache.platform=json!({"error":error});}}
 }
 if let Some(error)=cache.platform["error"].as_str(){return Err(error.into())}
 Ok((cache.platform["total"].as_i64().ok_or("Presença indisponível")?,serde_json::from_value(cache.platform["users"].clone()).map_err(|_|"Lista de presença inválida")?))
}

#[cfg(test)]mod integration {
 use super::*;
 fn profile()->Profile{serde_json::from_value(json!({"id":uuid::Uuid::new_v4().to_string(),"name":"Dashboard QA","platform":"twitch","channel":"qa","channelId":"1","botId":"bot"})).unwrap()}
 #[tokio::test]async fn eventos_deduplicados_persistidos_e_isolados_sem_simulacoes(){
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile();rt.db.save_profile(&p).unwrap();let n=now();let s=Session{id:"stream1".into(),channel_id:p.channel_id.clone(),last_at:n,last_viewers:Some(0),observed_at:n,started_at:chrono::Utc::now().to_rfc3339(),..Default::default()};save(&rt,&p.id,&s).unwrap();
  let mut e:Event=serde_json::from_value(json!({"id":"event1","profileId":p.id,"kind":"chat","user":"Pessoa","user_id":"u1","role":"everyone","message":"oi","data":{},"simulated":false})).unwrap();
  record(&rt,&p,&e);record(&rt,&p,&e);assert_eq!(session(&rt,&p.id).messages,1);assert_eq!(local(&rt,&p)["activeChatters"],1);
  e.id="sim".into();e.simulated=true;record(&rt,&p,&e);e.simulated=false;e.rehearsal=true;record(&rt,&p,&e);assert_eq!(session(&rt,&p.id).messages,1);
  e.rehearsal=false;e.id="gift1".into();e.kind="gift".into();e.data=json!({"total":3});record(&rt,&p,&e);assert_eq!(session(&rt,&p.id).gifts,3);
  let reopened=crate::db::Db::open(&dir.path().join("botlive.sqlite")).unwrap();let stored:Session=serde_json::from_value(reopened.module(&p.id,"dashboardSession")).unwrap();assert_eq!(stored.messages,1);assert_eq!(stored.gifts,3);
  let mut changed=p.clone();changed.channel_id="other".into();assert_eq!(local(&rt,&changed)["session"]["id"],"");
 }
 #[tokio::test]async fn dados_locais_nao_expoem_tokens_e_api_respeita_acesso(){
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile();rt.db.save_profile(&p).unwrap();
  rt.db.set_module(&p.id,"livepix",&json!({"overlayToken":"SEGREDO_QA","clientId":"SEGREDO_ID","enabled":true,"goals":{}})).unwrap();assert!(!local(&rt,&p).to_string().contains("SEGREDO"));
  let value=crate::dispatch(rt.clone(),"dashboard.get",json!({"profileId":p.id})).await.unwrap();assert_eq!(value["platform"]["state"],"unconfigured");
  *rt.actor.lock().unwrap()="editor-sem-acesso".into();assert!(crate::dispatch(rt.clone(),"dashboard.get",json!({"profileId":p.id})).await.is_err());assert!(crate::dispatch(rt.clone(),"dashboard.obs",json!({"profileId":p.id})).await.is_err());
 }
}
