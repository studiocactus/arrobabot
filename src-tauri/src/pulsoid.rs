//! Ponte oficial de leitura Pulsoid. Credencial no cofre; nunca em estado, log ou overlay.
use super::*;
use futures_util::{SinkExt,StreamExt};
use std::time::Duration;
use tokio_tungstenite::{connect_async_with_config,tungstenite::{Message,protocol::WebSocketConfig}};

#[cfg(not(test))]mod vault{
 pub fn get(p:&str)->Result<String,String>{crate::secrets::get(p,"pulsoid_token")}
 pub fn set(p:&str,v:&str)->Result<(),String>{crate::secrets::set(p,"pulsoid_token",v)}
}
#[cfg(test)]mod vault{
 use std::{collections::HashMap,sync::{Mutex,OnceLock}};
 fn store()->&'static Mutex<HashMap<String,String>>{static DATA:OnceLock<Mutex<HashMap<String,String>>>=OnceLock::new();DATA.get_or_init(Mutex::default)}
 pub fn get(p:&str)->Result<String,String>{store().lock().unwrap().get(p).cloned().ok_or("Chave não configurada".into())}
 pub fn set(p:&str,v:&str)->Result<(),String>{let mut m=store().lock().unwrap();if v.is_empty(){m.remove(p);}else{m.insert(p.into(),v.into());}Ok(())}
}
const VALIDATE:&str="https://dev.pulsoid.net/api/v1/token/validate";
const SOCKET:&str="wss://dev.pulsoid.net/api/v1/data/real_time";
#[path="pulsoid_auth.rs"]mod auth;
fn token_ok(t:&str)->bool{(16..=2048).contains(&t.len())&&t.bytes().all(|b|b.is_ascii_alphanumeric()||b"-_.".contains(&b))}
fn authorization(v:&Value)->Result<(),String>{
 if !v["scopes"].as_array().is_some_and(|a|a.iter().any(|s|s=="data:heart_rate:read")){return Err("A chave precisa da permissão data:heart_rate:read. Crie uma nova chave no Pulsoid.".into())}
 if !v["profile_id"].as_str().is_some_and(|s|!s.is_empty())||!v["expires_in"].as_u64().is_some_and(|s|s>0){return Err("A chave do Pulsoid está expirada ou não pertence a uma conta pessoal.".into())}Ok(())
}
fn rejection(status:u16)->Option<&'static str>{match status{
 401=>Some("A chave do Pulsoid expirou ou foi revogada. Gere outra chave e conecte novamente."),
 402=>Some("O Pulsoid exige acesso à API pelo plano BRO ou teste ativo. Confira sua assinatura no Pulsoid."),
 403=>Some("O Pulsoid recusou a permissão desta chave. Confira o acesso à API e data:heart_rate:read."),_=>None}}
fn response_rejection(status:u16,v:&Value)->Option<&'static str>{
 match v["error_code"].as_u64(){Some(7007)=>rejection(402),Some(7005|7006)=>rejection(401),_=>rejection(status)}
}
/// Não transforma pacotes antigos, duplicados, fora de ordem ou futuros em sinal novo.
fn measurement(v:&Value,now:i64,last:Option<i64>)->Option<(u16,i64,Duration)>{
 let bpm=v["data"]["heart_rate"].as_u64().filter(|n|(1..=300).contains(n))? as u16;
 let at=v["measured_at"].as_i64()?;
 if at<=0||last.is_some_and(|last|at<=last)||at>now+5000||now.saturating_sub(at)>10000{return None}
 Some((bpm,at,Duration::from_millis(now.saturating_sub(at).max(0) as u64)))
}
fn phase(rt:&Runtime,p:&str,key:&str,next:&'static str,error:&str,connected:bool)->bool{
 let mut map=rt.heart_rate.sessions.lock().unwrap();let Some(s)=map.get_mut(p).filter(|s|s.publisher==key&&s.source=="pulsoid")else{return false};
 s.phase=next;s.error=error.into();s.connected=connected;if next!="authorizing"{s.authorization=None;}if !connected{s.bpm=None;s.at=None;s.previous=None;s.last_event=None;}drop(map);rt.emit("heart-rate",state(rt,p,true));true
}
pub fn operation(rt:&Arc<Runtime>,p:&str,op:&str,args:&Value)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 match op{
  "heartRate.pulsoid.get"=>Ok(json!({"hasToken":vault::get(p).is_ok_and(|s|!s.is_empty()),"clientId":rt.db.module(p,"heartRate")["pulsoidClientId"].as_str().unwrap_or("")})),
  "heartRate.pulsoid.auth.start"=>auth::start(rt,p,args),
  "heartRate.pulsoid.forget"=>{vault::set(p,"")?;if rt.heart_rate.sessions.lock().unwrap().get(p).is_some_and(|s|s.source=="pulsoid"){stop(rt,p);}Ok(Value::Null)},
  "heartRate.pulsoid.connect"=>{
   if profile.modules["heartRate"]!=true{return Err("Ative Batimentos da live neste perfil".into())}
   let port=*rt.api_port.lock().unwrap();if port==0{return Err("A API local ainda não está disponível".into())}
   let supplied=args["token"].as_str().unwrap_or("").trim();let credential=if supplied.is_empty(){vault::get(p)?}else{supplied.into()};
   if !token_ok(&credential){return Err("Cole somente a chave pessoal de acesso do Pulsoid, sem endereço ou espaços.".into())}
   let mut map=rt.heart_rate.sessions.lock().unwrap();
   if map.get(p).is_some_and(|s|s.source=="bluetooth"&&s.connected){return Err("Desconecte a ponte Bluetooth antes de conectar pelo celular.".into())}
   // Salva antes de substituir: uma falha no cofre preserva a conexão atual.
   vault::set(p,&credential)?;
   let mut session=Session::new("pulsoid");if let Some(previous)=map.get(p).filter(|s|s.source=="pulsoid"){session.overlay=previous.overlay.clone();}session.phase="connecting";let key=session.publisher.clone();let overlay=session.overlay.clone();
   let runtime=Arc::downgrade(rt);let id=p.to_owned();
   let task=tokio::spawn(async move{run(runtime,id,key,credential,VALIDATE.into(),SOCKET.into(),Duration::from_secs(2)).await;});
   session.task=Some(task.abort_handle());map.insert(p.into(),session);drop(map);
   Ok(json!({"overlayUrl":format!("http://127.0.0.1:{port}/heart-overlay#token={overlay}")}))
  },_=>Err("Operação Pulsoid desconhecida".into())
 }
}

#[cfg(test)]mod tests{
 use super::*;
 use tokio::{io::{AsyncReadExt,AsyncWriteExt},net::TcpListener};
 use tokio_tungstenite::accept_async;
 const SECRET:&str="QA_PULSOID_PRIVATE_TOKEN_123456";
 fn setup()->(tempfile::TempDir,Arc<Runtime>,String){let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=uuid::Uuid::new_v4().to_string();let profile=serde_json::from_value(json!({"id":p,"name":"Pulsoid QA","platform":"twitch","channel":"qa","modules":{"heartRate":true}})).unwrap();rt.db.save_profile(&profile).unwrap();*rt.api_port.lock().unwrap()=12345;(dir,rt,p)}
 async fn http(status:u16,body:Value)->(String,tokio::task::JoinHandle<()>){let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let url=format!("http://{}/validate",listener.local_addr().unwrap());let task=tokio::spawn(async move{loop{let Ok((mut conn,_))=listener.accept().await else{break};let mut buf=[0;4096];let _=conn.read(&mut buf).await;let body=body.to_string();let response=format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());let _=conn.write_all(response.as_bytes()).await;}});(url,task)}
 fn authorized()->Value{json!({"profile_id":"personal","expires_in":3600,"scopes":["data:heart_rate:read"],"token":SECRET})}
 fn launch(rt:&Arc<Runtime>,p:&str,validate:String,socket:String)->(String,String){let mut s=Session::new("pulsoid");let key=s.publisher.clone();let overlay=s.overlay.clone();let weak=Arc::downgrade(rt);let id=p.to_owned();let worker_key=key.clone();s.task=Some(tokio::spawn(async move{run(weak,id,worker_key,SECRET.into(),validate,socket,Duration::from_millis(5)).await}).abort_handle());rt.heart_rate.sessions.lock().unwrap().insert(p.into(),s);(key,overlay)}
 async fn until(rt:&Runtime,p:&str,check:impl Fn(&Value)->bool){tokio::time::timeout(Duration::from_secs(5),async{loop{if check(&state(rt,p,false)){return}tokio::time::sleep(Duration::from_millis(10)).await;}}).await.expect("Estado do sensor não chegou ao esperado");}
 #[test]fn formato_permissao_e_erros_nao_revelam_chave(){assert!(token_ok(SECRET));for t in ["short","https://pulsoid.net/token","test token\nprivate"]{assert!(!token_ok(t));}assert!(authorization(&authorized()).is_ok());let mut v=authorized();v["scopes"]=json!([]);assert!(authorization(&v).is_err());v=authorized();v["profile_id"]=Value::Null;assert!(authorization(&v).is_err());for status in [401,402,403]{assert!(!rejection(status).unwrap().contains(SECRET));}assert!(rejection(429).is_none());}
 #[test]fn pacote_antigo_duplicado_futuro_e_invalido_nao_atualiza_sinal(){let now=100000;let v=json!({"measured_at":now-1000,"data":{"heart_rate":89}});assert_eq!(measurement(&v,now,None),Some((89,now-1000,Duration::from_secs(1))));assert!(measurement(&v,now,Some(now-1000)).is_none());for v in [json!({"measured_at":now-10001,"data":{"heart_rate":90}}),json!({"measured_at":now+5001,"data":{"heart_rate":90}}),json!({"measured_at":now,"data":{"heart_rate":0}}),json!({"measured_at":now,"data":{"heart_rate":301}}),json!({"measured_at":now,"data":{"heart_rate":"90"}})]{assert!(measurement(&v,now,None).is_none());}}
 #[tokio::test]async fn recusa_de_autorizacao_ou_plano_nao_vira_conectado_nem_expoe_credencial(){
  let (_dir,rt,p)=setup();
  for status in [401,402,403]{let (url,server)=http(status,json!({"token":SECRET,"error_message":SECRET})).await;launch(&rt,&p,url,"ws://127.0.0.1:1".into());until(&rt,&p,|v|v["phase"]=="error").await;let v=state(&rt,&p,false);assert!(v["bpm"].is_null());assert!(!v.to_string().contains(SECRET));assert!(rt.db.logs(&p).unwrap().is_empty());stop(&rt,&p);server.abort();}
 }
 #[tokio::test]async fn escopo_incorreto_nao_abre_sensor(){let (_dir,rt,p)=setup();let mut body=authorized();body["scopes"]=json!(["profile:read"]);let (url,server)=http(200,body).await;launch(&rt,&p,url,"ws://127.0.0.1:1".into());until(&rt,&p,|v|v["phase"]=="error").await;assert!(state(&rt,&p,false)["error"].as_str().unwrap().contains("data:heart_rate:read"));stop(&rt,&p);server.abort();}
 #[tokio::test]async fn sinal_real_alimenta_mesmo_overlay_e_reconecta_sem_cruzamento_falso(){
  let (_dir,rt,p)=setup();let (url,http)=http(200,authorized()).await;let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let socket=format!("ws://{}",listener.local_addr().unwrap());let (release,wait)=tokio::sync::oneshot::channel();
  let server=tokio::spawn(async move{let (conn,_)=listener.accept().await.unwrap();let mut ws=accept_async(conn).await.unwrap();ws.send(Message::Text(json!({"measured_at":chrono::Utc::now().timestamp_millis(),"data":{"heart_rate":89}}).to_string().into())).await.unwrap();let _=wait.await;let _=ws.close(None).await;let (conn,_)=listener.accept().await.unwrap();let mut ws=accept_async(conn).await.unwrap();ws.send(Message::Text(json!({"measured_at":chrono::Utc::now().timestamp_millis(),"data":{"heart_rate":130}}).to_string().into())).await.unwrap();while let Some(Ok(msg))=ws.next().await{if let Message::Ping(v)=msg{let _=ws.send(Message::Pong(v)).await;}}});
  let (key,overlay)=launch(&rt,&p,url,socket);until(&rt,&p,|v|v["bpm"]==89).await;
  assert_eq!(profile_for(&rt,&key,true),None);assert_eq!(profile_for(&rt,&overlay,false),Some(p.clone()));assert!(state(&rt,&p,true)["bpm"].is_null());assert!(connected(&rt,&p,&key,true).is_err());assert!(receive(&rt,&p,&key,&json!([0,99])).await.is_err());
  super::super::operation(&rt,&p,"heartRate.visibility",&json!({"enabled":true})).unwrap();assert_eq!(state(&rt,&p,true)["bpm"],89);
  let flow:crate::model::Flow=serde_json::from_value(json!({"id":uuid::Uuid::new_v4().to_string(),"profileId":p,"name":"QA","enabled":true,"trigger":{"kind":"heartRate","pattern":"above:120"},"actions":[{"kind":"variable.increment","target":"global.crossings","text":"1"}]})).unwrap();rt.db.save_flow(&flow).unwrap();release.send(()).unwrap();until(&rt,&p,|v|v["bpm"]==130).await;
  assert!(rt.heart_rate.sessions.lock().unwrap().get(&p).unwrap().previous.is_none());assert_eq!(crate::variables::list(&rt.db,&p,"").unwrap(),json!([]));assert_eq!(profile_for(&rt,&overlay,false),Some(p.clone()));
  stop(&rt,&p);assert_eq!(profile_for(&rt,&overlay,false),None);assert!(state(&rt,&p,false)["bpm"].is_null());server.abort();http.abort();
 }
 #[tokio::test]async fn parar_durante_validacao_impede_resultado_tardio_e_editor_nao_acessa_chave(){
  let (_dir,rt,p)=setup();let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let url=format!("http://{}",listener.local_addr().unwrap());let (accepted,wait)=tokio::sync::oneshot::channel();let server=tokio::spawn(async move{let (_conn,_)=listener.accept().await.unwrap();let _=accepted.send(());tokio::time::sleep(Duration::from_secs(10)).await;});let (_,overlay)=launch(&rt,&p,url,"ws://127.0.0.1:1".into());wait.await.unwrap();stop(&rt,&p);assert_eq!(profile_for(&rt,&overlay,false),None);assert!(state(&rt,&p,false)["source"].is_null());
  *rt.actor.lock().unwrap()="editor".into();for op in ["heartRate.pulsoid.get","heartRate.pulsoid.connect","heartRate.pulsoid.forget","heartRate.links"]{assert!(crate::dispatch(rt.clone(),op,json!({"profileId":p,"token":SECRET})).await.is_err());}server.abort();
 }
 #[tokio::test]async fn chave_salva_so_sinaliza_presenca_e_remocao_revoga_sessao(){let (_dir,rt,p)=setup();vault::set(&p,SECRET).unwrap();assert_eq!(operation(&rt,&p,"heartRate.pulsoid.get",&json!({})).unwrap(),json!({"hasToken":true,"clientId":""}));let mut s=Session::new("pulsoid");s.connected=true;let overlay=s.overlay.clone();rt.heart_rate.sessions.lock().unwrap().insert(p.clone(),s);operation(&rt,&p,"heartRate.pulsoid.forget",&json!({})).unwrap();assert!(vault::get(&p).is_err());assert_eq!(profile_for(&rt,&overlay,false),None);}
}
async fn run(runtime:std::sync::Weak<Runtime>,p:String,key:String,credential:String,validate:String,socket:String,retry:Duration){
 let mut attempts=0u32;let mut last=None;
 loop{
  let Some(rt)=runtime.upgrade()else{return};
  if !rt.db.profile(&p).is_ok_and(|p|p.modules["heartRate"]==true){return}
  if !phase(&rt,&p,&key,if attempts==0{"connecting"}else{"reconnecting"},"",false){return}
  let result=rt.http.get(&validate).bearer_auth(&credential).timeout(Duration::from_secs(12)).send().await;
  let mut fatal=None;let mut ready=false;
  if let Ok(response)=result{
   let status=response.status().as_u16();
   if rejection(status).is_some(){let body=tokio::time::timeout(Duration::from_secs(12),response.json::<Value>()).await.ok().and_then(Result::ok).unwrap_or(Value::Null);fatal=response_rejection(status,&body).map(str::to_owned);}
   else if response.status().is_success(){match tokio::time::timeout(Duration::from_secs(12),response.json::<Value>()).await{Ok(Ok(v))=>match authorization(&v){Ok(())=>ready=true,Err(e)=>fatal=Some(e)},_=>{}}}
  }
  if let Some(message)=fatal{phase(&rt,&p,&key,"error",&message,false);return}
  if ready{
   let mut url=match reqwest::Url::parse(&socket){Ok(v)=>v,Err(_)=>return};url.query_pairs_mut().append_pair("access_token",&credential);
   let config=WebSocketConfig::default().max_message_size(Some(16384)).max_frame_size(Some(16384));
   let opened=tokio::time::timeout(Duration::from_secs(12),connect_async_with_config(url.as_str(),Some(config),false)).await;
   match opened{
    Ok(Ok((mut ws,_)))=>{
     if !phase(&rt,&p,&key,"waiting","",true){return}
     let mut ping=tokio::time::interval(Duration::from_secs(20));ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
     let mut alive=Instant::now();
     loop{tokio::select!{
      _=ping.tick()=>{if alive.elapsed()>Duration::from_secs(60)||ws.send(Message::Ping(Vec::new().into())).await.is_err(){break}},
      frame=ws.next()=>{match frame{
       Some(Ok(Message::Text(raw)))=>{alive=Instant::now();if let Ok(v)=serde_json::from_str::<Value>(&raw){if let Some((bpm,at,age))=measurement(&v,chrono::Utc::now().timestamp_millis(),last){if accept(&rt,&p,&key,Some(bpm),age).await.is_err(){return}last=Some(at);attempts=0;}}},
       Some(Ok(Message::Ping(v)))=>{alive=Instant::now();if ws.send(Message::Pong(v)).await.is_err(){break}},
       Some(Ok(Message::Pong(_)))=>alive=Instant::now(),
       Some(Ok(Message::Close(_)))|Some(Err(_))|None=>break,_=>{}
      }}
     }}
    },
    Ok(Err(tokio_tungstenite::tungstenite::Error::Http(response)))=>{if let Some(message)=rejection(response.status().as_u16()){phase(&rt,&p,&key,"error",message,false);return}},_=>{}
   }
  }
  attempts+=1;
  if attempts>=8{phase(&rt,&p,&key,"error","Não foi possível manter a conexão com o Pulsoid. Confira a internet e clique em Conectar Pulsoid para tentar novamente.",false);return}
  if !phase(&rt,&p,&key,"reconnecting","Conexão interrompida. Tentando novamente…",false){return}
  drop(rt);tokio::time::sleep((retry*2u32.pow(attempts.saturating_sub(1))).min(Duration::from_secs(30))).await;
 }
}
