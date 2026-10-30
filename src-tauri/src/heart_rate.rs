//! Leitura BLE por ponte local de navegador. Chaves distintas para publicação e overlay.
use crate::{engine::Runtime,model::Event};
use serde_json::{json,Value};
use std::{collections::HashMap,sync::{Arc,Mutex},time::Instant};
#[path="pulsoid.rs"]pub mod pulsoid;
#[derive(Default)]pub struct Sensors{sessions:Mutex<HashMap<String,Session>>,events:Mutex<HashMap<String,(String,Instant)>>}
struct Session{publisher:String,overlay:String,bpm:Option<u16>,at:Option<Instant>,previous:Option<u16>,last_event:Option<Instant>,connected:bool,source:&'static str,phase:&'static str,error:String,authorization:Option<Value>,task:Option<tokio::task::AbortHandle>}
impl Drop for Session{fn drop(&mut self){if let Some(task)=self.task.take(){task.abort();}}}
impl Session{fn new(source:&'static str)->Self{Self{publisher:token(),overlay:token(),bpm:None,at:None,previous:None,last_event:None,connected:false,source,phase:"disconnected",error:String::new(),authorization:None,task:None}}}
fn token()->String{format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple())}
/// Heart Rate Measurement 0x2A37: UINT8/UINT16, contato e campos opcionais validados.
pub fn decode(bytes:&[u8])->Result<Option<u16>,String>{
 let flags=*bytes.first().ok_or("Pacote Bluetooth vazio")?;
 if flags&0xe0!=0{return Err("Flags Bluetooth não reconhecidas".into())}
 let wide=flags&1!=0;let needed=if wide{3}else{2};if bytes.len()<needed{return Err("Pacote Bluetooth incompleto".into())}
 let bpm=if wide{u16::from_le_bytes([bytes[1],bytes[2]])}else{bytes[1] as u16};
 let mut offset=needed;if flags&8!=0{offset+=2;}if bytes.len()<offset{return Err("Energia Bluetooth incompleta".into())}
 if flags&16!=0{if bytes.len()<offset+2||(bytes.len()-offset)%2!=0{return Err("Intervalos Bluetooth incompletos".into())}}else if bytes.len()!=offset{return Err("Tamanho de pacote Bluetooth inválido".into())}
 if flags&4!=0&&flags&2==0{return Ok(None)}
 if !(1..=300).contains(&bpm){return Err("Medição Bluetooth fora do formato aceito".into())}Ok(Some(bpm))
}
pub fn state(rt:&Runtime,p:&str,public:bool)->Value{
 let cfg=rt.db.module(p,"heartRate");let enabled=rt.db.profile(p).is_ok_and(|p|p.modules["heartRate"]==true);let allowed=enabled&&(!public||cfg["showOnObs"]==true);
 let map=rt.heart_rate.sessions.lock().unwrap();let entry=map.get(p);
 let age=entry.and_then(|s|s.at.map(|a|a.elapsed().as_secs()));
 let fresh=entry.is_some_and(|s|s.connected&&s.bpm.is_some()&&s.at.is_some_and(|at|at.elapsed()<=std::time::Duration::from_secs(10)));
 json!({"profileId":p,"state":if !allowed{"hidden"}else if fresh{"fresh"}else if entry.is_some_and(|s|s.connected){"waiting"}else{"disconnected"},"bpm":if allowed&&fresh{entry.and_then(|s|s.bpm)}else{None},"ageSeconds":age,"showOnObs":cfg["showOnObs"]==true,"source":entry.map(|s|s.source),"phase":entry.map(|s|s.phase),"error":if public{None}else{entry.map(|s|s.error.as_str())},"authorization":if public{None}else{entry.and_then(|s|s.authorization.clone())}})
}
pub fn profile_for(rt:&Runtime,key:&str,publisher:bool)->Option<String>{
 if key.len()!=64{return None}
 let id=rt.heart_rate.sessions.lock().unwrap().iter().find(|(_,s)|if publisher{s.source=="bluetooth"&&s.publisher==key}else{s.overlay==key}).map(|(id,_)|id.clone())?;
 let p=rt.db.profile(&id).ok()?;if p.modules["heartRate"]!=true{return None}Some(id)
}
pub fn connected(rt:&Runtime,p:&str,key:&str,next:bool)->Result<(),String>{
 let mut map=rt.heart_rate.sessions.lock().unwrap();let s=map.get_mut(p).filter(|s|s.source=="bluetooth"&&s.publisher==key).ok_or("Conexão de batimentos expirada")?;
 if next&&s.connected{return Err("Já há uma ponte de batimentos conectada neste perfil".into())}
 s.connected=next;s.phase=if next{"waiting"}else{"disconnected"};s.bpm=None;s.previous=None;s.at=None;Ok(())
}
pub async fn receive(rt:&Arc<Runtime>,p:&str,key:&str,raw:&Value)->Result<(),String>{
 let data=raw.as_array().filter(|a|a.len()<=64).ok_or("Pacote Bluetooth inválido")?;
 let bytes:Vec<u8>=data.iter().map(|v|v.as_u64().filter(|n|*n<=255).map(|n|n as u8).ok_or("Byte Bluetooth inválido")).collect::<Result<_,_>>()?;
 let bpm=decode(&bytes)?;
 if profile_for(rt,key,true).as_deref()!=Some(p){return Err("Conexão de batimentos expirada".into())}
 accept(rt,p,key,bpm,std::time::Duration::ZERO).await
}
/// Único caminho validado para prévia, overlay e eventos, independente da ponte.
async fn accept(rt:&Arc<Runtime>,p:&str,key:&str,bpm:Option<u16>,age:std::time::Duration)->Result<(),String>{
 let profile=rt.db.profile(p)?;if profile.modules["heartRate"]!=true{return Err("Módulo de batimentos desligado".into())}
 if bpm.is_some_and(|n|!(1..=300).contains(&n))||age>std::time::Duration::from_secs(10){return Err("Medição de batimentos inválida ou antiga".into())}
 let source;
 let previous={let mut map=rt.heart_rate.sessions.lock().unwrap();let s=map.get_mut(p).filter(|s|s.publisher==key&&s.connected).ok_or("Conexão de batimentos expirada")?;
  if s.at.is_some_and(|a|a.elapsed().as_millis()<200){return Ok(())}
  source=s.source;let previous=if s.at.is_some_and(|a|a.elapsed()<=std::time::Duration::from_secs(10)){s.bpm}else{None};s.previous=previous;s.bpm=bpm;s.at=Some(Instant::now()-age);s.phase="waiting";s.error.clear();
  let emit=bpm.is_some()&&previous.is_some()&&bpm!=previous&&s.last_event.map_or(true,|a|a.elapsed().as_secs()>=1);
  if emit{s.last_event=Some(Instant::now());previous}else{None}
 };
 rt.emit("heart-rate",state(rt,p,true));
 if let (Some(bpm),Some(previous))=(bpm,previous){
  // Só aciona fluxos explicitamente configurados; não grava uma série de saúde no histórico.
  if rt.db.flows(p)?.iter().any(|f|f.enabled&&f.trigger.kind=="heartRate"){
   let id=uuid::Uuid::new_v4().to_string();{let mut events=rt.heart_rate.events.lock().unwrap();events.retain(|_,(_,at)|at.elapsed().as_secs()<30);events.insert(id.clone(),(p.into(),Instant::now()));}
   rt.submit(Event{id,profile_id:p.into(),kind:"heartRate".into(),user:"Sensor de batimentos".into(),user_id:"local-heart-sensor".into(),role:"everyone".into(),message:String::new(),data:json!({"bpm":bpm,"previousBpm":previous,"source":if source=="pulsoid"{"pulsoid-heart-rate"}else{"bluetooth-heart-rate"}}),simulated:false,rehearsal:false}).await?;
  }
 }
 Ok(())
}
pub fn trusted(rt:&Runtime,e:&Event)->bool{rt.heart_rate.events.lock().unwrap().remove(&e.id).is_some_and(|(p,at)|p==e.profile_id&&at.elapsed().as_secs()<30)}
pub fn matches(pattern:&str,e:&Event)->bool{
 if e.kind!="heartRate"{return false}let Some(current)=e.data["bpm"].as_u64() else{return false};let Some(previous)=e.data["previousBpm"].as_u64() else{return false};
 if pattern.is_empty()||pattern=="change"{return current!=previous}
 let Some((op,value))=pattern.split_once(':') else{return false};let Ok(n)=value.parse::<u64>() else{return false};match op{"above"=>current>n&&previous<=n,"below"=>current<n&&previous>=n,_=>false}
}
pub fn stop(rt:&Runtime,p:&str){rt.heart_rate.sessions.lock().unwrap().remove(p);rt.emit("heart-rate",state(rt,p,true));}
pub fn operation(rt:&Runtime,p:&str,op:&str,args:&Value)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 match op{
 "heartRate.get"=>Ok(state(rt,p,false)),
 "heartRate.links"=>{let map=rt.heart_rate.sessions.lock().unwrap();let port=*rt.api_port.lock().unwrap();Ok(json!({"overlayUrl":map.get(p).filter(|_|port>0).map(|s|format!("http://127.0.0.1:{port}/heart-overlay#token={}",s.overlay))}))},
 "heartRate.visibility"=>{let enabled=args["enabled"].as_bool().ok_or("Escolha se quer exibir no OBS")?;let mut cfg=rt.db.module(p,"heartRate");if !cfg.is_object(){cfg=json!({});}cfg["showOnObs"]=json!(enabled);rt.db.set_module(p,"heartRate",&cfg)?;rt.emit("heart-rate",state(rt,p,true));Ok(Value::Null)},
 "heartRate.stop"=>{stop(rt,p);Ok(Value::Null)},
 "heartRate.prepare"=>{
  if profile.modules["heartRate"]!=true{return Err("Ative Batimentos da live neste perfil".into())}
  let mut map=rt.heart_rate.sessions.lock().unwrap();if map.get(p).is_some_and(|s|s.connected||s.task.is_some()){return Err("Desconecte a ponte atual antes de preparar outra".into())}
  let session=Session::new("bluetooth");let publisher=session.publisher.clone();let overlay=session.overlay.clone();map.insert(p.into(),session);drop(map);
  let port=*rt.api_port.lock().unwrap();if port==0{stop(rt,p);return Err("A API local ainda não está disponível".into())}
  Ok(json!({"bridgeUrl":format!("http://127.0.0.1:{port}/heart-connect#token={publisher}"),"overlayUrl":format!("http://127.0.0.1:{port}/heart-overlay#token={overlay}")}))
 },_=>Err("Operação de batimentos desconhecida".into())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn setup()->(tempfile::TempDir,Arc<Runtime>,String){
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let id=uuid::Uuid::new_v4().to_string();
  let p=serde_json::from_value(json!({"id":id,"name":"Sensor QA","platform":"twitch","channel":"qa","modules":{"heartRate":true}})).unwrap();rt.db.save_profile(&p).unwrap();*rt.api_port.lock().unwrap()=12345;(dir,rt,id)
 }
 fn prepare(rt:&Runtime,p:&str)->(String,String){let links=operation(rt,p,"heartRate.prepare",&json!({})).unwrap();let key=|name:&str|links[name].as_str().unwrap().split("#token=").nth(1).unwrap().to_owned();(key("bridgeUrl"),key("overlayUrl"))}
 #[tokio::test]async fn chaves_separadas_privacidade_e_revogacao(){
  let (_dir,rt,p)=setup();let (publisher,overlay)=prepare(&rt,&p);
  assert_ne!(publisher,overlay);assert_eq!(profile_for(&rt,&publisher,true),Some(p.clone()));assert_eq!(profile_for(&rt,&overlay,false),Some(p.clone()));assert_eq!(profile_for(&rt,&publisher,false),None);assert_eq!(profile_for(&rt,&overlay,true),None);
  connected(&rt,&p,&publisher,true).unwrap();assert!(connected(&rt,&p,&publisher,true).is_err());assert!(connected(&rt,&p,&overlay,false).is_err());
  receive(&rt,&p,&publisher,&json!([0,82])).await.unwrap();assert_eq!(state(&rt,&p,false)["bpm"],82);assert!(state(&rt,&p,true)["bpm"].is_null());
  operation(&rt,&p,"heartRate.visibility",&json!({"enabled":true})).unwrap();assert_eq!(state(&rt,&p,true)["bpm"],82);
  stop(&rt,&p);assert!(state(&rt,&p,false)["bpm"].is_null());assert_eq!(profile_for(&rt,&publisher,true),None);assert_eq!(profile_for(&rt,&overlay,false),None);
  let (next,_)=prepare(&rt,&p);assert_ne!(next,publisher);assert!(connected(&rt,&p,&publisher,false).is_err());
 }
 #[tokio::test]async fn sinal_antigo_e_modulo_desligado_nao_expoem_bpm(){
  let (_dir,rt,p)=setup();let (key,_)=prepare(&rt,&p);connected(&rt,&p,&key,true).unwrap();receive(&rt,&p,&key,&json!([0,79])).await.unwrap();
  {let mut map=rt.heart_rate.sessions.lock().unwrap();map.get_mut(&p).unwrap().at=Some(Instant::now()-std::time::Duration::from_millis(10500));}
  assert!(state(&rt,&p,false)["bpm"].is_null());assert_eq!(state(&rt,&p,false)["state"],"waiting");
  receive(&rt,&p,&key,&json!([0,80])).await.unwrap();assert!(rt.heart_rate.sessions.lock().unwrap().get(&p).unwrap().previous.is_none());
  let mut profile=rt.db.profile(&p).unwrap();profile.modules["heartRate"]=json!(false);rt.db.save_profile(&profile).unwrap();assert_eq!(profile_for(&rt,&key,true),None);assert!(receive(&rt,&p,&key,&json!([0,80])).await.is_err());assert_eq!(state(&rt,&p,false)["state"],"hidden");
 }
 #[tokio::test]async fn evento_externo_nao_se_passa_por_sensor_e_ticket_nao_reutiliza(){
  let (_dir,rt,p)=setup();let e:Event=serde_json::from_value(json!({"id":"trusted-id","profileId":p,"kind":"heartRate","user":"Sensor","user_id":"local-heart-sensor","role":"everyone","message":"","data":{"bpm":80,"previousBpm":79}})).unwrap();
  assert!(!trusted(&rt,&e));rt.heart_rate.events.lock().unwrap().insert(e.id.clone(),(p,Instant::now()));assert!(trusted(&rt,&e));assert!(!trusted(&rt,&e));
 }
 #[test]fn formatos_ble_validos_e_contato(){assert_eq!(decode(&[0,75]).unwrap(),Some(75));assert_eq!(decode(&[1,4,1]).unwrap(),Some(260));assert_eq!(decode(&[4,75]).unwrap(),None);assert_eq!(decode(&[6,75]).unwrap(),Some(75));assert_eq!(decode(&[16,80,0,4]).unwrap(),Some(80));}
 #[tokio::test]async fn cruzamento_real_dispara_fluxo_mas_evento_forjado_nao(){
  use crate::model::{Action,Flow,Trigger};
  let (_dir,rt,p)=setup();let flow=Flow{id:uuid::Uuid::new_v4().to_string(),profile_id:p.clone(),name:"Cruzamento QA".into(),trigger:Trigger{kind:"heartRate".into(),pattern:"above:120".into(),..Default::default()},actions:vec![Action{kind:"variable.increment".into(),target:"global.crossings".into(),text:"1".into(),..Default::default()}],..Default::default()};rt.db.save_flow(&flow).unwrap();
  let e:Event=serde_json::from_value(json!({"id":"forged","profileId":p,"kind":"heartRate","user":"Sensor","user_id":"local-heart-sensor","role":"everyone","message":"","data":{"bpm":121,"previousBpm":119}})).unwrap();crate::engine::process(rt.clone(),e).await;assert_eq!(crate::variables::list(&rt.db,&p,"").unwrap(),json!([]));
  let (key,_)=prepare(&rt,&p);connected(&rt,&p,&key,true).unwrap();receive(&rt,&p,&key,&json!([0,119])).await.unwrap();assert_eq!(crate::variables::list(&rt.db,&p,"").unwrap(),json!([]));
  rt.heart_rate.sessions.lock().unwrap().get_mut(&p).unwrap().at=Some(Instant::now()-std::time::Duration::from_millis(300));receive(&rt,&p,&key,&json!([0,121])).await.unwrap();
  tokio::time::timeout(std::time::Duration::from_secs(3),async{loop{let values=crate::variables::list(&rt.db,&p,"").unwrap();if values.as_array().unwrap().iter().any(|v|v["name"]=="crossings"&&v["value"]==1){break}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}}).await.unwrap();
 }
 #[test]fn pacotes_invalidos_nao_viram_medicao(){for raw in [&[][..],&[1,3][..],&[0,0][..],&[1,255,255][..],&[16,80,1][..],&[8,70][..],&[128,70][..]]{assert!(decode(raw).is_err(),"{raw:?}");}}
 #[test]fn cruzamento_exige_medicao_anterior(){let mut e:Event=serde_json::from_value(json!({"id":"a","profileId":"p","kind":"heartRate","user":"Sensor","user_id":"sensor","role":"everyone","message":"","data":{"bpm":121,"previousBpm":119},"simulated":false})).unwrap();assert!(matches("above:120",&e));e.data["previousBpm"]=json!(122);assert!(!matches("above:120",&e));e.data["previousBpm"]=Value::Null;assert!(!matches("change",&e));}
}
