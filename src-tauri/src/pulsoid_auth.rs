//! Autorização oficial para desktop: somente Client ID público, sem Client Secret.
use super::*;
const DEVICE:&str="https://pulsoid.net/oauth2/device_authorization";
const TOKEN:&str="https://pulsoid.net/oauth2/token";
struct Consent{device:String,url:String,expires:Duration,interval:Duration}
fn consent(v:&Value)->Result<Consent,String>{
 let device=v["device_code"].as_str().filter(|s|token_ok(s)).ok_or("Resposta de autorização inválida")?.to_owned();
 let code=v["user_code"].as_str().filter(|s|!s.is_empty()&&s.len()<=128&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')).ok_or("Código de autorização inválido")?;
 let raw=v["verification_uri_complete"].as_str().or(v["verification_uri"].as_str()).ok_or("Endereço de autorização ausente")?;
 let mut url=reqwest::Url::parse(raw).map_err(|_|"Endereço de autorização inválido")?;
 if url.scheme()!="https"||url.host_str()!=Some("pulsoid.net")||url.port().is_some()||!url.username().is_empty()||url.password().is_some()||url.path()!="/oauth2/device_authorization_consent"||url.fragment().is_some(){return Err("Endereço de autorização não pertence ao Pulsoid".into())}
 // Mantém somente o código fornecido pelo mesmo contrato, sem parâmetros arbitrários.
 url.set_query(None);url.query_pairs_mut().append_pair("user_code",code);
 let expires=v["expires_in"].as_u64().filter(|n|*n>0&&*n<=1800).ok_or("Prazo de autorização inválido")?;
 let interval=v["interval"].as_u64().unwrap_or(3).max(3).min(1800);
 Ok(Consent{device,url:url.into(),expires:Duration::from_secs(expires),interval:Duration::from_secs(interval)})
}
pub(super) fn start(rt:&Arc<Runtime>,p:&str,args:&Value)->Result<Value,String>{
 if rt.db.profile(p)?.modules["heartRate"]!=true{return Err("Ative Batimentos da live neste perfil".into())}
 let client=args["clientId"].as_str().unwrap_or("").trim();
 if uuid::Uuid::parse_str(client).is_err(){return Err("Copie o Client ID do aplicativo cadastrado no Pulsoid. Não use o Client Secret.".into())}
 let port=*rt.api_port.lock().unwrap();if port==0{return Err("A API local ainda não está disponível".into())}
 let mut map=rt.heart_rate.sessions.lock().unwrap();if map.get(p).is_some_and(|s|s.source=="bluetooth"&&s.connected){return Err("Desconecte a ponte Bluetooth antes de autorizar o Pulsoid.".into())}
 let mut cfg=rt.db.module(p,"heartRate");if !cfg.is_object(){cfg=json!({});}cfg["pulsoidClientId"]=json!(client);rt.db.set_module(p,"heartRate",&cfg)?;
 let mut session=Session::new("pulsoid");if let Some(previous)=map.get(p).filter(|s|s.source=="pulsoid"){session.overlay=previous.overlay.clone();}session.phase="authorizing";
 let key=session.publisher.clone();let overlay=session.overlay.clone();let id=p.to_owned();let client=client.to_owned();let weak=Arc::downgrade(rt);
 session.task=Some(tokio::spawn(async move{authorize(weak,id,key,client,DEVICE.into(),TOKEN.into(),VALIDATE.into(),SOCKET.into()).await;}).abort_handle());map.insert(p.into(),session);drop(map);
 Ok(json!({"overlayUrl":format!("http://127.0.0.1:{port}/heart-overlay#token={overlay}")}))
}
async fn post(rt:&Runtime,url:&str,form:&[(&str,&str)])->Result<(u16,Value),String>{
 tokio::time::timeout(Duration::from_secs(12),async{let response=rt.http.post(url).form(form).send().await.map_err(|_|"Sem conexão com o Pulsoid")?;let status=response.status().as_u16();let value=response.json::<Value>().await.map_err(|_|"Resposta de autorização inválida")?;Ok((status,value))}).await.map_err(|_|"O Pulsoid não respondeu a tempo".to_owned())?
}
async fn authorize(weak:std::sync::Weak<Runtime>,p:String,key:String,client:String,device_endpoint:String,token_endpoint:String,validate:String,socket:String){
 let Some(rt)=weak.upgrade()else{return};
 let fail=|msg:&str|{phase(&rt,&p,&key,"error",msg,false);};
 let device=match post(&rt,&device_endpoint,&[("client_id",&client),("scope","data:heart_rate:read")]).await{
  Ok((200,v))=>match consent(&v){Ok(c)=>c,Err(e)=>{fail(&e);return}},
  Ok((status,v))=>{fail(response_rejection(status,&v).unwrap_or("O Pulsoid recusou o cadastro do aplicativo. Confira o Client ID."));return},Err(e)=>{fail(&e);return}
 };
 let deadline=tokio::time::Instant::now()+device.expires;let mut interval=device.interval;
 {let mut map=rt.heart_rate.sessions.lock().unwrap();let Some(s)=map.get_mut(&p).filter(|s|s.publisher==key)else{return};s.authorization=Some(json!({"url":device.url,"expiresAt":chrono::Utc::now().timestamp()+device.expires.as_secs() as i64}));}
 loop{
  if tokio::time::timeout_at(deadline,tokio::time::sleep(interval)).await.is_err(){fail("O prazo de autorização expirou. Clique em Autorizar Pulsoid para tentar novamente.");return}
  if !rt.heart_rate.sessions.lock().unwrap().get(&p).is_some_and(|s|s.publisher==key){return}
  let result=tokio::time::timeout_at(deadline,post(&rt,&token_endpoint,&[("grant_type","urn:ietf:params:oauth:grant-type:device_code"),("device_code",&device.device),("client_id",&client)])).await;
  match result{
   Ok(Ok((200,v)))=>{
    let Some(token)=v["access_token"].as_str().filter(|s|token_ok(s))else{fail("O Pulsoid não retornou uma autorização válida.");return};
    // O cancelamento não pode gravar uma credencial que chegou para uma sessão antiga.
    {let map=rt.heart_rate.sessions.lock().unwrap();if !map.get(&p).is_some_and(|s|s.publisher==key){return}if let Err(e)=vault::set(&p,token){drop(map);fail(&e);return}}
    if !phase(&rt,&p,&key,"connecting","",false){return}drop(rt);
    run(weak,p,key,token.into(),validate,socket,Duration::from_secs(2)).await;return
   },
   Ok(Ok((status,v)))=>match v["error"].as_str(){
    Some("authorization_pending") if status==400=>{},
    Some("slow_down")=>interval=(interval+Duration::from_secs(5)).min(Duration::from_secs(300)),
    Some("expired_token")=>{fail("O prazo de autorização expirou. Clique em Autorizar Pulsoid para tentar novamente.");return},
    Some("access_denied"|"invalid_grant")=>{fail("A autorização foi recusada ou encerrada no Pulsoid. Clique em Autorizar Pulsoid para tentar novamente.");return},
    _=>{fail(response_rejection(status,&v).unwrap_or("O Pulsoid não concluiu a autorização. Confira o cadastro do aplicativo e tente novamente."));return}
   },
   Ok(Err(e))=>{fail(&e);return},Err(_)=>{fail("O prazo de autorização expirou. Clique em Autorizar Pulsoid para tentar novamente.");return}
  }
 }
}
#[cfg(test)]mod tests{
 use super::*;
 use tokio::{io::{AsyncReadExt,AsyncWriteExt},net::TcpListener};
 fn setup()->(tempfile::TempDir,Arc<Runtime>,String){let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let id=uuid::Uuid::new_v4().to_string();let p=serde_json::from_value(json!({"id":id,"name":"Consentimento QA","platform":"twitch","channel":"qa","modules":{"heartRate":true}})).unwrap();rt.db.save_profile(&p).unwrap();*rt.api_port.lock().unwrap()=12345;(dir,rt,id)}
 async fn server(outcome:&'static str)->(String,tokio::task::JoinHandle<()>,Arc<std::sync::atomic::AtomicUsize>){
  let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let url=format!("http://{}",listener.local_addr().unwrap());let polls=Arc::new(std::sync::atomic::AtomicUsize::new(0));let counter=polls.clone();
  let task=tokio::spawn(async move{loop{let (mut conn,_)=listener.accept().await.unwrap();let mut buf=Vec::new();let mut part=[0;4096];loop{let n=conn.read(&mut part).await.unwrap();if n==0{break}buf.extend_from_slice(&part[..n]);if let Some(end)=buf.windows(4).position(|w|w==b"\r\n\r\n"){let headers=String::from_utf8_lossy(&buf[..end]).to_lowercase();let length=headers.lines().find_map(|l|l.strip_prefix("content-length:").and_then(|v|v.trim().parse::<usize>().ok())).unwrap_or(0);if buf.len()>=end+4+length{break}}}let request=String::from_utf8_lossy(&buf);
   assert!(!request.contains("client_secret"));
   let (status,body)=if request.starts_with("POST /device "){(200,json!({"device_code":"DEVICE_CODE_QA_123456","user_code":"user-code","verification_uri":"https://pulsoid.net/oauth2/device_authorization_consent","expires_in":if outcome=="expire"{1}else{30},"interval":3}))}else if request.starts_with("POST /token "){let n=counter.fetch_add(1,std::sync::atomic::Ordering::SeqCst);if outcome=="denied"{(400,json!({"error":"invalid_grant","error_description":"private response"}))}else if n==0{(400,json!({"error":"authorization_pending"}))}else{(200,json!({"access_token":"AUTH_QA_ACCESS_TOKEN_123456","expires_in":3600}))}}else{(200,json!({"profile_id":"personal","expires_in":3600,"scopes":["data:heart_rate:read"]}))};let body=body.to_string();let reply=format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());let _=conn.write_all(reply.as_bytes()).await;
  }});(url,task,polls)
 }
 fn launch(rt:&Arc<Runtime>,p:&str,base:&str,socket:String)->(String,tokio::task::JoinHandle<()>){let mut s=Session::new("pulsoid");s.phase="authorizing";let key=s.publisher.clone();let overlay=s.overlay.clone();let weak=Arc::downgrade(rt);let p=p.to_owned();let request_id=p.clone();let base=base.to_owned();let task=tokio::spawn(async move{authorize(weak,request_id,key,"00000000-0000-4000-8000-000000000001".into(),format!("{base}/device"),format!("{base}/token"),format!("{base}/validate"),socket).await;});s.task=Some(task.abort_handle());rt.heart_rate.sessions.lock().unwrap().insert(p,s);(overlay,task)}
 async fn wait(rt:&Runtime,p:&str,predicate:impl Fn(&Value)->bool){tokio::time::timeout(Duration::from_secs(12),async{loop{if predicate(&state(rt,p,false)){break}tokio::time::sleep(Duration::from_millis(10)).await;}}).await.unwrap();}
 #[test]fn consentimento_so_abre_o_destino_oficial_e_respeita_intervalo(){let mut v=json!({"device_code":"DEVICE_CODE_QA_123456","user_code":"code-qa","verification_uri":"https://pulsoid.net/oauth2/device_authorization_consent","expires_in":600,"interval":1});let c=consent(&v).unwrap();assert_eq!(c.interval,Duration::from_secs(3));assert!(c.url.ends_with("?user_code=code-qa"));for url in ["http://pulsoid.net/oauth2/device_authorization_consent","https://example.com/oauth2/device_authorization_consent","https://pulsoid.net/another","https://user:pass@pulsoid.net/oauth2/device_authorization_consent"]{v["verification_uri"]=json!(url);assert!(consent(&v).is_err());}}
 #[tokio::test]async fn autorizacao_pendente_salva_no_cofre_e_le_o_mesmo_sensor(){let (_dir,rt,p)=setup();let (base,http,polls)=server("ok").await;let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();let socket=format!("ws://{}",listener.local_addr().unwrap());let ws=tokio::spawn(async move{let (conn,_)=listener.accept().await.unwrap();let mut ws=tokio_tungstenite::accept_async(conn).await.unwrap();ws.send(Message::Text(json!({"measured_at":chrono::Utc::now().timestamp_millis(),"data":{"heart_rate":87}}).to_string().into())).await.unwrap();while let Some(Ok(msg))=ws.next().await{if let Message::Ping(v)=msg{let _=ws.send(Message::Pong(v)).await;}}});let (_,worker)=launch(&rt,&p,&base,socket);wait(&rt,&p,|v|v["authorization"].is_object()).await;assert!(state(&rt,&p,true)["authorization"].is_null());assert!(!state(&rt,&p,false).to_string().contains("DEVICE_CODE_QA"));wait(&rt,&p,|v|v["bpm"]==87).await;assert_eq!(polls.load(std::sync::atomic::Ordering::SeqCst),2);assert!(vault::get(&p).is_ok());assert!(state(&rt,&p,false)["authorization"].is_null());assert!(state(&rt,&p,true)["bpm"].is_null());assert!(!state(&rt,&p,false).to_string().contains("ACCESS_TOKEN"));stop(&rt,&p);worker.abort();ws.abort();http.abort();vault::set(&p,"").unwrap();}
 #[tokio::test]async fn recusa_expiracao_e_cancelamento_nao_gravam_chave(){for outcome in ["denied","expire","cancel"]{let (_dir,rt,p)=setup();let (base,http,polls)=server(outcome).await;let (overlay,worker)=launch(&rt,&p,&base,"ws://127.0.0.1:1".into());wait(&rt,&p,|v|v["authorization"].is_object()).await;if outcome=="cancel"{stop(&rt,&p);assert_eq!(profile_for(&rt,&overlay,false),None);assert_eq!(polls.load(std::sync::atomic::Ordering::SeqCst),0);}else{wait(&rt,&p,|v|v["phase"]=="error").await;assert!(state(&rt,&p,false)["authorization"].is_null());stop(&rt,&p);}assert!(vault::get(&p).is_err());worker.abort();http.abort();}}
 #[tokio::test]async fn client_id_publico_e_publicacao_sao_preservados_sem_segredo(){let (_dir,rt,p)=setup();assert!(start(&rt,&p,&json!({"clientId":"not-an-id"})).is_err());rt.db.set_module(&p,"heartRate",&json!({"pulsoidClientId":"public-id"})).unwrap();super::super::super::operation(&rt,&p,"heartRate.visibility",&json!({"enabled":true})).unwrap();assert_eq!(rt.db.module(&p,"heartRate")["pulsoidClientId"],"public-id");}
}
