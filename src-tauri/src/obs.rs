//! OBS Studio local via OBS WebSocket 5.x. O app é desktop e o OBS está na
//! mesma máquina: conexão direta em ws://localhost, sem nuvem, sem túnel,
//! sem expor porta. Senha fica no cofre e nunca entra em log ou resposta.
use crate::{engine::Runtime, model::Profile};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
 collections::HashMap,
 sync::{Arc, Mutex, Weak},
 time::Duration,
};
use futures_util::{SinkExt, StreamExt};
pub const MODULE: &str = "obs";
const DEFAULT_PORT: u16 = 4455;
/// Operações seguras expostas a automações. Sem stop/exit/delete/settings.
pub const OPS: &[&str] = &["mute", "unmute", "toggle_mute", "volume", "show", "hide", "toggle_item", "scene"];
/// Operações que aceitam duração temporária com restauração do estado anterior.
pub fn temp_ok(op: &str) -> bool {
 matches!(op, "mute" | "unmute" | "volume" | "show" | "hide")
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
 pub enabled: bool,
 pub host: String,
 pub port: u16,
}
impl Config {
 fn addr(&self) -> (String, u16) {
  let host = self.host.trim();
  (if host.is_empty() { "127.0.0.1".into() } else { host.into() }, if self.port == 0 { DEFAULT_PORT } else { self.port })
 }
}
pub fn config(rt: &Runtime, p: &str) -> Config {
 let mut c: Config = rt.db.module_cfg(p, MODULE);
 if c.port == 0 {
  c.port = DEFAULT_PORT;
 }
 c
}
/// Um portão por destino OBS; efeitos temporários compartilham a origem.
#[derive(Default)]
pub struct Locks {
 active:Mutex<HashMap<String,Lock>>,
 gates:Mutex<HashMap<String,Weak<Gate>>>,
}
struct Gate {serial:Arc<tokio::sync::Mutex<()>>,slots:Arc<tokio::sync::Semaphore>}
impl Default for Gate {fn default()->Self{Self{serial:Arc::new(tokio::sync::Mutex::new(())),slots:Arc::new(tokio::sync::Semaphore::new(9))}}}
impl Locks {
 fn gate(&self,key:&str)->Arc<Gate>{
  let mut gates=self.gates.lock().unwrap();gates.retain(|_,v|v.strong_count()>0);
  if let Some(g)=gates.get(key).and_then(Weak::upgrade){return g}
  let gate=Arc::new(Gate::default());gates.insert(key.into(),Arc::downgrade(&gate));gate
 }
}
#[derive(Clone)]
struct Lock {
 desired:String,count:u32,previous:String,expected:String,generation:String,
 profile:Profile,config:Config,pinned:Option<(String,i64)>,op:String,target:String,
 gate:Arc<Gate>,
}
struct Release {rt:Arc<Runtime>,key:String,generation:String,seconds:u64,gate:Arc<Gate>}
impl Drop for Release {
 fn drop(&mut self){let (rt,key,generation,seconds,gate)=(self.rt.clone(),self.key.clone(),self.generation.clone(),self.seconds,self.gate.clone());
  tokio::spawn(async move{tokio::time::sleep(Duration::from_secs(seconds)).await;release_temporary(rt,key,generation,gate).await;});
 }
}
fn endpoint(c:&Config)->String {let (host,port)=c.addr();let host=host.to_ascii_lowercase();let host=if host=="localhost"||host=="::1"{"127.0.0.1"}else{&host};format!("{host}:{port}")}
async fn cancelled(rt:&Runtime,run:Option<&str>){
 if let Some(id)=run{loop{if rt.runs.cancelled(id){return}tokio::time::sleep(Duration::from_millis(25)).await;}}
 else{std::future::pending::<()>().await}
}
async fn admit(gate:&Arc<Gate>,rt:&Runtime,run:Option<&str>)->Result<(tokio::sync::OwnedSemaphorePermit,tokio::sync::OwnedMutexGuard<()>),String>{
 admit_with_timeout(gate,rt,run,Duration::from_secs(10)).await
}
async fn admit_with_timeout(gate:&Arc<Gate>,rt:&Runtime,run:Option<&str>,wait:Duration)->Result<(tokio::sync::OwnedSemaphorePermit,tokio::sync::OwnedMutexGuard<()>),String>{
 let slot=gate.slots.clone().try_acquire_owned().map_err(|_|"Fila do OBS cheia: há oito pedidos aguardando".to_string())?;
 let guard=tokio::select!{
  result=tokio::time::timeout(wait,gate.serial.clone().lock_owned())=>result.map_err(|_|"Pedido ao OBS excedeu 10 segundos na fila; não foi enviado".to_string())?,
  _=cancelled(rt,run)=>return Err("Execução cancelada".into()),
 };
 if run.is_some_and(|id|rt.runs.cancelled(id)){return Err("Execução cancelada".into())}
 Ok((slot,guard))
}
pub fn health(rt:&Runtime,p:&str)->Value {
 let prefix=format!("{}|",endpoint(&config(rt,p)));let all=rt.obs.active.lock().unwrap();
 let entries=all.iter().filter(|(key,_)|key.starts_with(&prefix)).map(|(_,v)|v).collect::<Vec<_>>();
 json!({"temporary":entries.iter().filter(|v|v.count>0).count(),"pendingRestore":entries.iter().filter(|v|v.count==0).count()})
}
/// Autenticação oficial do protocolo 5.x: base64(sha256(base64(sha256(senha+salt))+challenge)).
pub fn auth_string(password: &str, salt: &str, challenge: &str) -> String {
 use sha2::{Digest, Sha256};
 let secret = base64::Engine::encode(
  &base64::engine::general_purpose::STANDARD,
  Sha256::digest(format!("{password}{salt}").as_bytes()),
 );
 base64::Engine::encode(&base64::engine::general_purpose::STANDARD, Sha256::digest(format!("{secret}{challenge}").as_bytes()))
}
type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
struct Conn {
 ws: Ws,
 seq: u64,
 pinned:Option<(String,i64)>,
}
async fn next_text(ws:&mut Ws)->Result<String,String>{
 tokio::time::timeout(Duration::from_secs(10),next_text_inner(ws)).await.map_err(|_|"OBS demorou a responder".to_string())?
}
async fn next_text_inner(ws: &mut Ws) -> Result<String, String> {
 loop {
  let msg = tokio::time::timeout(Duration::from_secs(10), ws.next())
   .await
   .map_err(|_| "OBS demorou a responder".to_string())?
   .ok_or("OBS fechou a conexão".to_string())?
   .map_err(|_| "Falha de leitura no OBS".to_string())?;
  match msg {
   tokio_tungstenite::tungstenite::Message::Text(t) => return Ok(t.to_string()),
   tokio_tungstenite::tungstenite::Message::Close(_) => return Err("OBS fechou a conexão".into()),
   _ => continue,
  }
 }
}
async fn call(conn:&mut Conn,request_type:&str,data:Value)->Result<Value,String>{
 tokio::time::timeout(Duration::from_secs(10),call_inner(conn,request_type,data)).await.map_err(|_|"OBS demorou a confirmar o pedido".to_string())?
}
async fn call_inner(conn: &mut Conn, request_type: &str, data: Value) -> Result<Value, String> {
 conn.seq += 1;
 let id = format!("botlive-{}", conn.seq);
 let msg = json!({"op": 6, "d": {"requestType": request_type, "requestId": id, "requestData": data}}).to_string();
 conn.ws
  .send(tokio_tungstenite::tungstenite::Message::Text(msg.into()))
  .await
  .map_err(|_| "Falha ao enviar pedido ao OBS".to_string())?;
 loop {
  let text = next_text(&mut conn.ws).await?;
  let v: Value = serde_json::from_str(&text).map_err(|_| "Resposta inválida do OBS".to_string())?;
  if v["op"].as_i64() != Some(7) || v["d"]["requestId"].as_str() != Some(id.as_str()) {
   continue;
  }
  if v["d"]["requestStatus"]["result"] == true {
   return Ok(v["d"]["responseData"].clone());
  }
  return Err(v["d"]["requestStatus"]["comment"].as_str().unwrap_or("OBS recusou o pedido").into());
 }
}
async fn connect(rt: &Arc<Runtime>, p: &Profile) -> Result<Conn, String> {
 let c = config(rt, &p.id);connect_to(p,&c).await
}
async fn connect_to(p:&Profile,c:&Config)->Result<Conn,String>{
 if !c.enabled {
  return Err("Ative a integração OBS na tela OBS Studio".into());
 }
 let (host, port) = c.addr();
 let url = format!("ws://{host}:{port}");
 let (mut ws, _) = tokio::time::timeout(Duration::from_secs(10), tokio_tungstenite::connect_async(&url))
  .await
  .map_err(|_| format!("Não foi possível alcançar o OBS em {url}. Abra o OBS e ative o WebSocket do OBS em Ferramentas."))?
  .map_err(|_| "Falha ao conectar no OBS".to_string())?;
 let hello: Value = serde_json::from_str(&next_text(&mut ws).await?).map_err(|_| "Resposta inválida do OBS".to_string())?;
 let mut identify = json!({"op": 1, "d": {"rpcVersion": 1, "eventSubscriptions": 0}});
 if let (Some(challenge), Some(salt)) = (
  hello["d"]["authentication"]["challenge"].as_str(),
  hello["d"]["authentication"]["salt"].as_str(),
 ) {
  let password = crate::secrets::get(&p.id, "obs_password").map_err(|_| "O OBS pede senha: salve a senha na tela OBS Studio".to_string())?;
  identify["d"]["authentication"] = json!(auth_string(&password, salt, challenge));
 }
 ws.send(tokio_tungstenite::tungstenite::Message::Text(identify.to_string().into()))
  .await
  .map_err(|_| "Falha ao identificar no OBS".to_string())?;
 let identified: Value = serde_json::from_str(&next_text(&mut ws).await?).map_err(|_| "Resposta inválida do OBS".to_string())?;
 if identified["op"].as_i64() != Some(2) {
  return Err("O OBS recusou a identificação. Confira a senha.".into());
 }
 Ok(Conn { ws, seq: 0, pinned:None })
}
async fn close(mut conn: Conn) {
 let _ = conn.ws.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
}
/// Cena atual do programa.
async fn program_scene(conn: &mut Conn) -> Result<String, String> {
 let v = call(conn, "GetSceneList", json!({})).await?;
 v["currentProgramSceneName"].as_str().map(str::to_owned).ok_or("OBS não informou a cena atual".into())
}
/// Itens da cena com nome e id, para achar fonte pelo nome.
async fn scene_items(conn: &mut Conn, scene: &str) -> Result<Vec<(String, i64)>, String> {
 let v = call(conn, "GetSceneItemList", json!({"sceneName": scene})).await?;
 Ok(v["sceneItems"]
  .as_array()
  .cloned()
  .unwrap_or_default()
  .into_iter()
  .filter_map(|i| Some((i["sourceName"].as_str()?.to_owned(), i["sceneItemId"].as_i64()?)))
  .collect())
}
async fn find_item(conn: &mut Conn, scene: &str, source: &str) -> Result<i64, String> {
 scene_items(conn, scene)
  .await?
  .into_iter()
  .find(|(name, _)| name.eq_ignore_ascii_case(source))
  .map(|(_, id)| id)
  .ok_or_else(|| format!("Fonte {source} não encontrada na cena {scene}"))
}
async fn item(conn:&mut Conn,target:&str)->Result<(String,i64),String>{
 if let Some((scene,id))=conn.pinned.clone(){
  if scene_items(conn,&scene).await?.iter().any(|(name,n)|*n==id&&name.eq_ignore_ascii_case(target)){return Ok((scene,id))}
  return Err("A fonte original do efeito não existe mais; nenhuma outra fonte foi alterada".into())
 }
 let scene=program_scene(conn).await?;let id=find_item(conn,&scene,target).await?;Ok((scene,id))
}
/// Aplica a operação (efeito imediato). Devolve texto para o Histórico.
async fn apply(conn: &mut Conn, op: &str, target: &str, num: f64) -> Result<String, String> {
 match op {
  "mute" => {
   call(conn, "SetInputMute", json!({"inputName": target, "inputMuted": true})).await?;
   Ok(format!("{} mutado", target))
  }
  "unmute" => {
   call(conn, "SetInputMute", json!({"inputName": target, "inputMuted": false})).await?;
   Ok(format!("{} desmutado", target))
  }
  "toggle_mute" => {
   call(conn, "ToggleInputMute", json!({"inputName": target})).await?;
   Ok(format!("mudo de {} alternado", target))
  }
  "volume" => {
   call(conn, "SetInputVolume", json!({"inputName": target, "inputVolumeDb": num})).await?;
   Ok(format!("volume de {} em {} dB", target, num))
  }
  "show" | "hide" | "toggle_item" => {
   let (scene,id)=item(conn,target).await?;
   if op == "toggle_item" {
    let cur = current_enabled(conn, &scene, id).await?;
    call(conn, "SetSceneItemEnabled", json!({"sceneName": scene, "sceneItemId": id, "sceneItemEnabled": !cur})).await?;
    Ok(format!("fonte {} alternada", target))
   } else {
    call(
     conn,
     "SetSceneItemEnabled",
     json!({"sceneName": scene, "sceneItemId": id, "sceneItemEnabled": op == "show"}),
    )
    .await?;
    Ok(format!("fonte {} {}", target, if op == "show" { "visível" } else { "oculta" }))
   }
  }
  "scene" => {
   call(conn, "SetCurrentProgramScene", json!({"sceneName": target})).await?;
   Ok(format!("cena {}", target))
  }
  _ => Err("Ação do OBS desconhecida".into()),
 }
}
async fn current_enabled(conn: &mut Conn, scene: &str, id: i64) -> Result<bool, String> {
 let v = call(conn, "GetSceneItemEnabled", json!({"sceneName": scene, "sceneItemId": id})).await?;
 v["sceneItemEnabled"].as_bool().ok_or("OBS não informou a visibilidade da fonte".into())
}
/// Lê o estado anterior para restaurar depois (só operações temporárias).
async fn previous(conn: &mut Conn, op: &str, target: &str) -> Result<String, String> {
 match op {
  "mute" | "unmute" => {
   let v = call(conn, "GetInputMute", json!({"inputName": target})).await?;
   let muted=v["inputMuted"].as_bool().ok_or("OBS não informou o estado de mudo")?;
   Ok(if muted { "muted".into() } else { "unmuted".into() })
  }
  "volume" => {
   let v = call(conn, "GetInputVolume", json!({"inputName": target})).await?;
   Ok(v["inputVolumeDb"].as_f64().filter(|n|n.is_finite()).ok_or("OBS não informou o volume anterior")?.to_string())
  }
  "show" | "hide" => {
   let (scene,id)=item(conn,target).await?;
   Ok(if current_enabled(conn, &scene, id).await? { "shown".into() } else { "hidden".into() })
  }
  _ => Err("Operação sem modo temporário".into()),
 }
}
/// Restaura o estado guardado (inverso do apply, pelo valor anterior).
async fn restore(conn: &mut Conn, op: &str, target: &str, prev: &str) -> Result<(), String> {
 match op {
  "mute" | "unmute" => {
   call(conn, "SetInputMute", json!({"inputName": target, "inputMuted": prev == "muted"})).await?;
  }
  "volume" => {
   let db: f64 = prev.parse().unwrap_or(0.0);
   call(conn, "SetInputVolume", json!({"inputName": target, "inputVolumeDb": db})).await?;
  }
  "show" | "hide" => {
   let (scene,id)=item(conn,target).await?;
   call(conn, "SetSceneItemEnabled", json!({"sceneName": scene, "sceneItemId": id, "sceneItemEnabled": prev == "shown"})).await?;
  }
  _ => {}
 }
 Ok(())
}
fn resource_of(op: &str, target: &str) -> String {
 match op {
  "mute" | "unmute" | "toggle_mute" | "volume" => format!("input:{target}"),
  "show" | "hide" | "toggle_item" => format!("item:{target}"),
  "scene" => "scene".to_string(),
  _ => format!("other:{target}"),
 }
}
fn same_state(a:&str,b:&str,op:&str)->bool {
 if op=="volume"{match (a.parse::<f64>(),b.parse::<f64>()){(Ok(a),Ok(b))=>(a-b).abs()<0.01,_=>false}}else{a==b}
}
async fn release_temporary(rt:Arc<Runtime>,key:String,generation:String,gate:Arc<Gate>){
 let Ok(_slot)=gate.slots.clone().acquire_owned().await else{return};
 let _guard=gate.serial.clone().lock_owned().await;
 let lease={let mut all=rt.obs.active.lock().unwrap();let Some(lock)=all.get_mut(&key) else{return};
  if lock.generation!=generation{return}lock.count=lock.count.saturating_sub(1);if lock.count>0{return}lock.clone()};
 let result=async {
  let mut conn=connect_to(&lease.profile,&lease.config).await?;conn.pinned=lease.pinned.clone();
  let current=previous(&mut conn,&lease.op,&lease.target).await?;
  if !same_state(&current,&lease.expected,&lease.op){
   close(conn).await;return Ok(false)
  }
  restore(&mut conn,&lease.op,&lease.target,&lease.previous).await?;
  let confirmed=previous(&mut conn,&lease.op,&lease.target).await?;close(conn).await;
  if !same_state(&confirmed,&lease.previous,&lease.op){return Err("OBS não confirmou o estado anterior".to_string())}Ok(true)
 }.await;
 match result {
  Ok(restored)=>{rt.log(&lease.profile.id,"obs",&if restored{format!("{} restaurado com confirmação",lease.target)}else{format!("{} mudou fora do efeito; restauração não aplicada",lease.target)},"info");rt.obs.active.lock().unwrap().remove(&key);}
  Err(err)=>rt.log(&lease.profile.id,"obs",&format!("Restauração não confirmada de {}: {err}. Estado anterior: {}. Confira no OBS antes de enviar outro efeito.",lease.target,lease.previous),"error"),
 }
}
/// Mesma operação usada por testes manuais e automações; não há outro motor.
pub async fn execute_action(rt:&Arc<Runtime>,p:&Profile,op:&str,target:&str,num:f64,secs:u64)->Result<String,String>{execute(rt,p,op,target,num,secs,None).await}
pub async fn execute_for_run(rt:&Arc<Runtime>,p:&Profile,op:&str,target:&str,num:f64,secs:u64,run:&str,index:usize)->Result<String,String>{execute(rt,p,op,target,num,secs,Some((run,index))).await}
async fn execute(rt:&Arc<Runtime>,p:&Profile,op:&str,target:&str,num:f64,secs:u64,run:Option<(&str,usize)>)->Result<String,String>{
 if !OPS.contains(&op){return Err("Ação do OBS desconhecida".into())}
 if target.trim().is_empty(){return Err("Escolha o alvo da ação do OBS".into())}
 if secs>3600{return Err("Ação do OBS: duração de até 1 hora".into())}
 if !num.is_finite(){return Err("Valor do OBS inválido".into())}
 if secs>0&&!temp_ok(op){return Err("Duração temporária vale para mutar, volume, mostrar e esconder".into())}
 let config=config(rt,&p.id);let endpoint=endpoint(&config);let gate=rt.obs.gate(&endpoint);
 if let Some((id,index))=run{rt.runs.step(id,index,"WAITING");}
 let (_slot,_guard)=admit(&gate,rt,run.map(|(id,_)|id)).await?;
 if let Some((id,index))=run{rt.runs.step(id,index,"RUNNING");}
 let mut conn=connect_to(p,&config).await?;
 if matches!(op,"show"|"hide"|"toggle_item"){conn.pinned=Some(item(&mut conn,target).await?);}
 let resource=if let Some((scene,id))=&conn.pinned{format!("item:{scene}:{id}")}else{resource_of(op,&target.to_lowercase())};
 let key=format!("{endpoint}|{resource}");let desired=format!("{op}:{}",if op=="volume"{num}else{0.0});
 let existing={rt.obs.active.lock().unwrap().get(&key).cloned()};
 if let Some(lock)=existing {
  if lock.count==0 {
   let current=previous(&mut conn,&lock.op,&lock.target).await?;
   if same_state(&current,&lock.previous,&lock.op){rt.obs.active.lock().unwrap().remove(&key);}
   else{return Err(format!("Restauração pendente de {}. Devolva ao estado anterior ({}) no OBS antes de tentar novamente",lock.target,lock.previous))}
  }else{
   if secs==0||lock.desired!=desired{return Err("Recurso ocupado por outro efeito temporário; aguarde liberar".into())}
   let current=previous(&mut conn,op,target).await?;
   if !same_state(&current,&lock.expected,op){rt.obs.active.lock().unwrap().remove(&key);return Err("O alvo foi alterado fora do efeito; alteração preservada. Tente novamente para iniciar outro efeito".into())}
   if run.is_some_and(|(id,_)|rt.runs.cancelled(id)){return Err("Execução cancelada".into())}
   rt.obs.active.lock().unwrap().get_mut(&key).unwrap().count+=1;
   let _release=Release{rt:rt.clone(),key,generation:lock.generation,seconds:secs,gate:lock.gate};
   close(conn).await;return Ok(format!("Efeito de {target} compartilhado por mais {secs}s; origem preservada"))
  }
 }
 if run.is_some_and(|(id,_)|rt.runs.cancelled(id)){return Err("Execução cancelada".into())}
 if secs==0{let done=apply(&mut conn,op,target,num).await?;close(conn).await;return Ok(done)}
 let prev=previous(&mut conn,op,target).await?;
 let expected=match op{"mute"=>"muted".into(),"unmute"=>"unmuted".into(),"show"=>"shown".into(),"hide"=>"hidden".into(),_=>num.to_string()};
 if run.is_some_and(|(id,_)|rt.runs.cancelled(id)){return Err("Execução cancelada".into())}
 let generation=uuid::Uuid::new_v4().to_string();
 rt.obs.active.lock().unwrap().insert(key.clone(),Lock{desired,count:1,previous:prev,expected,generation:generation.clone(),profile:p.clone(),config,pinned:conn.pinned.clone(),op:op.into(),target:target.into(),gate:gate.clone()});
 // Drop também agenda recuperação se a tarefa for interrompida aguardando o OBS.
 let mut release=Release{rt:rt.clone(),key,generation,seconds:0,gate};
 let done=apply(&mut conn,op,target,num).await?;release.seconds=secs;drop(release);
 close(conn).await;Ok(format!("{done} por {secs}s"))
}
/// Descoberta para os dropdowns: cenas, entradas de áudio e fontes da cena atual.
pub async fn discover(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 let mut conn = connect(rt, p).await?;
 let scenes = call(&mut conn, "GetSceneList", json!({})).await?;
 let current = scenes["currentProgramSceneName"].as_str().unwrap_or("").to_owned();
 let scene_names: Vec<Value> = scenes["scenes"]
  .as_array()
  .cloned()
  .unwrap_or_default()
  .into_iter()
  .filter_map(|s| s["sceneName"].as_str().map(|n| json!(n)))
  .collect();
 let inputs = call(&mut conn, "GetInputList", json!({})).await?;
 let input_names: Vec<Value> = inputs["inputs"]
  .as_array()
  .cloned()
  .unwrap_or_default()
  .into_iter()
  .filter_map(|i| {
   json!({"name": i["inputName"], "kind": i["inputKind"]});
   i["inputName"].as_str().map(|n| json!(n))
  })
  .collect();
 let mut sources: Vec<Value> = vec![];
 if !current.is_empty() {
  for (name, _) in scene_items(&mut conn, &current).await.unwrap_or_default() {
   sources.push(json!(name));
  }
 }
 close(conn).await;
 Ok(json!({"scenes": scene_names, "current": current, "inputs": input_names, "sources": sources}))
}
pub async fn status(rt: &Arc<Runtime>, p: &Profile) -> Result<Value, String> {
 let mut conn = connect(rt, p).await?;
 let v = call(&mut conn, "GetVersion", json!({})).await?;
 let scene = program_scene(&mut conn).await.unwrap_or_default();
 close(conn).await;
 Ok(json!({
  "online": true,
  "obsVersion": v["obsVersion"],
  "wsVersion": v["obsWebSocketVersion"],
  "scene": scene,
 }))
}
fn valid_save(v: &Value) -> Result<Config, String> {
 let host = v["host"].as_str().unwrap_or("").trim();
 if host.chars().count() > 253 {
  return Err("Endereço do OBS inválido".into());
 }
 if !host.is_empty() && !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_') {
  return Err("Endereço do OBS inválido: use IP ou nome da máquina".into());
 }
 let port = v["port"].as_u64().unwrap_or(DEFAULT_PORT as u64);
 if port < 1 || port > 65535 {
  return Err("Porta do OBS entre 1 e 65535".into());
 }
 Ok(Config { enabled: v["enabled"] == true, host: host.into(), port: port as u16 })
}
pub async fn operation(rt: &Arc<Runtime>, p: &str, op: &str, args: &Value) -> Result<Value, String> {
 rt.db.profile(p)?;
 let profile = rt.db.profile(p)?;
 if profile.platform != "twitch" && profile.platform != "youtube" && profile.platform != "kick" {
  return Err("Perfil inválido".into());
 }
 match op {
  "obs.get" => Ok(json!({"config": serde_json::to_value(config(rt, p)).map_err(|e| e.to_string())?, "hasPassword": crate::secrets::get(p, "obs_password").is_ok()})),
  "obs.save" => {
   let c = valid_save(&args["config"])?;
   rt.db.set_module(p, MODULE, &serde_json::to_value(&c).map_err(|e| e.to_string())?)?;
   rt.log(p, "obs", "Configuração do OBS salva.", "success");
   Ok(Value::Null)
  },
  "obs.password" => {
   let pw = args["value"].as_str().unwrap_or("");
   if pw.chars().count() > 200 {
    return Err("Senha muito longa".into());
   }
   crate::secrets::set(p, "obs_password", pw)?;
   Ok(Value::Null)
  }
  "obs.status" => status(rt, &profile).await,
  "obs.discover" => discover(rt, &profile).await,
  "obs.test" => {
   let op = args["op"].as_str().unwrap_or("");
   let target = args["target"].as_str().unwrap_or("");
   execute_action(rt, &profile, op, target, 0.0, 0).await.map(|done| json!(done))
  }
  "obs.execute" => {
   let op = args["op"].as_str().unwrap_or("");
   let target = args["target"].as_str().unwrap_or("");
   let num = args["num"].as_f64().unwrap_or(0.0);
   let secs = args["secs"].as_u64().unwrap_or(0);
   execute_action(rt, &profile, op, target, num, secs).await.map(|done| json!(done))
  }
  _ => Err("Operação do OBS desconhecida".into()),
 }
}
#[cfg(test)]
mod tests {
 use super::*;

 #[derive(Default)]
 struct MockState {muted:bool,volume:f64,scene:String,a:bool,b:bool,sets:usize,drop_next_set:bool,reject_restore:bool,missing_state:bool}
 struct MockObs {state:Arc<Mutex<MockState>>,port:u16,task:tokio::task::JoinHandle<()>}
 impl Drop for MockObs{fn drop(&mut self){self.task.abort()}}
 async fn mock_obs()->MockObs {
  let tcp=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let port=tcp.local_addr().unwrap().port();
  let state=Arc::new(Mutex::new(MockState{scene:"A".into(),a:true,b:true,..Default::default()}));let st=state.clone();
  let task=tokio::spawn(async move{while let Ok((stream,_))=tcp.accept().await{let st=st.clone();tokio::spawn(async move{
   let Ok(mut ws)=tokio_tungstenite::accept_async(stream).await else{return};
   if ws.send(tokio_tungstenite::tungstenite::Message::Text(json!({"op":0,"d":{"rpcVersion":1}}).to_string().into())).await.is_err(){return}
   while let Some(Ok(message))=ws.next().await {
    let Ok(text)=message.to_text() else{continue};let Ok(v)=serde_json::from_str::<Value>(text) else{continue};
    if v["op"]==1 {if ws.send(tokio_tungstenite::tungstenite::Message::Text(json!({"op":2,"d":{"negotiatedRpcVersion":1}}).to_string().into())).await.is_err(){break}continue}
    let d=&v["d"];let kind=d["requestType"].as_str().unwrap_or("");let args=&d["requestData"];
    let (data,ok,drop_connection)={let mut state=st.lock().unwrap();let mut ok=true;let mut drop_connection=false;
     let data=match kind {
      "GetStreamStatus"=>json!({"outputActive":true,"outputReconnecting":false,"outputDuration":120000,"outputBytes":1000000,"outputSkippedFrames":0,"outputTotalFrames":7200}),
      "GetStats"=>json!({"activeFps":60.0,"cpuUsage":10.0,"renderSkippedFrames":0}),
      "GetRecordStatus"=>json!({"outputActive":false}),
      "GetInputMute"=>if state.missing_state{json!({})}else{json!({"inputMuted":state.muted})},"GetInputVolume"=>json!({"inputVolumeDb":state.volume}),
      "SetInputMute"=>{if state.reject_restore&&args["inputMuted"]==false{ok=false}else{state.muted=args["inputMuted"]==true;state.sets+=1;drop_connection=state.drop_next_set;state.drop_next_set=false;}json!({})},
      "SetInputVolume"=>{state.volume=args["inputVolumeDb"].as_f64().unwrap_or(0.0);state.sets+=1;json!({})},
      "GetSceneList"=>json!({"currentProgramSceneName":state.scene}),
      "GetSceneItemList"=>json!({"sceneItems":[{"sourceName":"Cam","sceneItemId":if args["sceneName"]=="A"{1}else{2}}]}),
      "GetSceneItemEnabled"=>json!({"sceneItemEnabled":if args["sceneName"]=="A"{state.a}else{state.b}}),
      "SetSceneItemEnabled"=>{if args["sceneName"]=="A"{state.a=args["sceneItemEnabled"]==true}else{state.b=args["sceneItemEnabled"]==true}state.sets+=1;json!({})},
      "SetCurrentProgramScene"=>{state.scene=args["sceneName"].as_str().unwrap().into();json!({})},
      _=>{ok=false;json!({})}
     };(data,ok,drop_connection)};
    if drop_connection{break}
    let response=json!({"op":7,"d":{"requestId":d["requestId"],"requestStatus":{"result":ok,"comment":"Recusado no simulador"},"responseData":data}});
    if ws.send(tokio_tungstenite::tungstenite::Message::Text(response.to_string().into())).await.is_err(){break}
   }
  });}});MockObs{state,port,task}
 }
 fn test_profile(rt:&Arc<Runtime>,port:u16)->Profile {
  let p:Profile=serde_json::from_value(json!({"id":uuid::Uuid::new_v4().to_string(),"name":"QA OBS","platform":"twitch","channel":"qa"})).unwrap();rt.db.save_profile(&p).unwrap();
  rt.db.set_module(&p.id,MODULE,&json!({"enabled":true,"host":"127.0.0.1","port":port})).unwrap();p
 }
 async fn expire(rt:&Arc<Runtime>){let (key,lock)=rt.obs.active.lock().unwrap().iter().next().map(|(k,v)|(k.clone(),v.clone())).unwrap();release_temporary(rt.clone(),key,lock.generation,lock.gate).await;}
 #[tokio::test]
 async fn telemetria_observa_sem_mutacoes_ou_travas_de_efeitos(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);
  let value=telemetry(&rt,&p).await;assert_eq!(value["state"],"fresh");assert_eq!(value["data"]["streaming"],true);assert_eq!(value["data"]["recording"],false);assert_eq!(value["data"]["scene"],"A");assert_eq!(server.state.lock().unwrap().sets,0);assert!(rt.obs.active.lock().unwrap().is_empty());
 }
 #[tokio::test]
 async fn shared_temporary_origin_is_preserved_across_profiles_and_conflicts_are_refused(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);let q=test_profile(&rt,server.port);
  execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap();execute_action(&rt,&q,"mute","Mic",0.0,60).await.unwrap();
  assert_eq!(server.state.lock().unwrap().sets,1,"efeito igual não relê nem reaplica a origem");
  assert!(execute_action(&rt,&q,"unmute","Mic",0.0,60).await.unwrap_err().contains("ocupado"));
  assert!(execute_action(&rt,&q,"unmute","Mic",0.0,0).await.unwrap_err().contains("ocupado"));
  expire(&rt).await;assert!(server.state.lock().unwrap().muted);expire(&rt).await;
  assert!(!server.state.lock().unwrap().muted);assert!(rt.obs.active.lock().unwrap().is_empty());
 }
 #[tokio::test]
 async fn concurrent_effects_restore_once_after_the_last_real_timer(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);let q=test_profile(&rt,server.port);
  let (a,b)=tokio::join!(execute_action(&rt,&p,"mute","Mic",0.0,1),execute_action(&rt,&q,"mute","Mic",0.0,2));a.unwrap();b.unwrap();
  assert_eq!(server.state.lock().unwrap().sets,1);
  tokio::time::timeout(Duration::from_secs(4),async{loop{if rt.obs.active.lock().unwrap().is_empty(){break}tokio::time::sleep(Duration::from_millis(10)).await}}).await.unwrap();
  assert!(!server.state.lock().unwrap().muted);assert_eq!(server.state.lock().unwrap().sets,2);
 }
 #[tokio::test]
 async fn restore_keeps_original_scene_and_respects_manual_changes(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);
  execute_action(&rt,&p,"hide","Cam",0.0,60).await.unwrap();server.state.lock().unwrap().scene="B".into();
  expire(&rt).await;let state=server.state.lock().unwrap();assert!(state.a&&state.b);drop(state);
  execute_action(&rt,&p,"volume","Mic",-10.0,60).await.unwrap();server.state.lock().unwrap().volume=-5.0;
  expire(&rt).await;assert_eq!(server.state.lock().unwrap().volume,-5.0);assert!(rt.obs.active.lock().unwrap().is_empty());
 }
 #[tokio::test]
 async fn old_timer_never_releases_a_new_generation_and_config_change_keeps_the_original_obs(){
  let server=mock_obs().await;let other=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);
  execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap();
  let (key,old)=rt.obs.active.lock().unwrap().iter().next().map(|(k,v)|(k.clone(),v.clone())).unwrap();
  server.state.lock().unwrap().muted=false;
  assert!(execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap_err().contains("alterado"));
  execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap();release_temporary(rt.clone(),key,old.generation,old.gate).await;assert!(server.state.lock().unwrap().muted);
  rt.db.set_module(&p.id,MODULE,&json!({"enabled":true,"host":"127.0.0.1","port":other.port})).unwrap();
  expire(&rt).await;assert!(!server.state.lock().unwrap().muted);assert_eq!(other.state.lock().unwrap().sets,0);
 }
 #[tokio::test]
 async fn failed_restore_blocks_the_resource_until_manually_recovered(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);
  execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap();server.state.lock().unwrap().reject_restore=true;expire(&rt).await;
  assert!(execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap_err().contains("pendente"));
  assert!(server.state.lock().unwrap().muted);server.state.lock().unwrap().muted=false;server.state.lock().unwrap().reject_restore=false;
  execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap();expire(&rt).await;assert!(!server.state.lock().unwrap().muted);
 }
 #[tokio::test]
 async fn missing_previous_state_never_applies_a_temporary_effect(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);server.state.lock().unwrap().missing_state=true;
  assert!(execute_action(&rt,&p,"mute","Mic",0.0,60).await.unwrap_err().contains("estado de mudo"));assert_eq!(server.state.lock().unwrap().sets,0);assert!(rt.obs.active.lock().unwrap().is_empty());
 }
 #[tokio::test]
 async fn lost_ack_restores_the_state_instead_of_losing_the_origin(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);
  server.state.lock().unwrap().drop_next_set=true;assert!(execute_action(&rt,&p,"mute","Mic",0.0,60).await.is_err());
  tokio::time::timeout(Duration::from_secs(3),async{while !rt.obs.active.lock().unwrap().is_empty(){tokio::time::sleep(Duration::from_millis(10)).await}}).await.unwrap();
  assert!(!server.state.lock().unwrap().muted);assert_eq!(server.state.lock().unwrap().sets,2);
 }
 #[tokio::test]
 async fn queue_is_bounded_and_waiting_can_timeout_or_cancel_without_sending(){
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,1);let gate=rt.obs.gate("queue");
  let hold=gate.serial.clone().lock_owned().await;let full=gate.slots.clone().acquire_many_owned(9).await.unwrap();
  assert!(admit(&gate,&rt,None).await.err().unwrap().contains("cheia"));drop(full);
  assert!(admit_with_timeout(&gate,&rt,None,Duration::from_millis(10)).await.err().unwrap().contains("10 segundos"));
  let run=rt.runs.start(&p.id,"f","Esperando",&[],true);rt.runs.cancel(&p.id,&run);
  assert_eq!(admit(&gate,&rt,Some(&run)).await.err().unwrap(),"Execução cancelada");
  assert_eq!(gate.slots.available_permits(),9);drop(hold);let _admitted=admit(&gate,&rt,None).await.unwrap();
  assert_eq!(rt.obs.gates.lock().unwrap().len(),1);
 }
 #[tokio::test]
 async fn cancelled_flow_never_applies_obs_after_waiting(){
  let server=mock_obs().await;let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=test_profile(&rt,server.port);
  let gate=rt.obs.gate(&endpoint(&config(&rt,&p.id)));let hold=gate.serial.clone().lock_owned().await;
  let run=rt.runs.start(&p.id,"f","Fila",&["obs".into()],true);let (rt2,p2,run2)=(rt.clone(),p.clone(),run.clone());
  let task=tokio::spawn(async move{execute_for_run(&rt2,&p2,"mute","Mic",0.0,0,&run2,0).await});
  tokio::time::timeout(Duration::from_secs(2),async{while rt.runs.get(&run).unwrap().steps[0].status!="WAITING"{tokio::time::sleep(Duration::from_millis(5)).await}}).await.unwrap();
  rt.runs.cancel(&p.id,&run);drop(hold);assert_eq!(task.await.unwrap().unwrap_err(),"Execução cancelada");assert_eq!(server.state.lock().unwrap().sets,0);
 }
 #[test]
 fn endpoint_aliases_and_volume_comparison_are_stable(){
  let a=Config{host:"localhost".into(),port:4455,enabled:true};let b=Config{host:"127.0.0.1".into(),..a.clone()};assert_eq!(endpoint(&a),endpoint(&b));
  assert!(same_state("-10.00001","-10","volume"));assert!(!same_state("-5","-10","volume"));
 }
 #[test]
 fn auth_matches_the_official_vector_shape() {
  let a = auth_string("supersecretpassword", "saltsalt1234", "challenge5678");
  assert_eq!(a.len(), 44, "base64 de 32 bytes tem 44 caracteres");
  assert_ne!(a, auth_string("outra", "saltsalt1234", "challenge5678"));
  assert_ne!(a, auth_string("supersecretpassword", "outro", "challenge5678"));
  assert_eq!(a, auth_string("supersecretpassword", "saltsalt1234", "challenge5678"), "determinístico");
 }
 #[test]
 fn only_safe_ops_exist_and_temp_is_limited() {
  assert_eq!(OPS, &["mute", "unmute", "toggle_mute", "volume", "show", "hide", "toggle_item", "scene"]);
  for op in ["mute", "unmute", "volume", "show", "hide"] {
   assert!(temp_ok(op), "{op} aceita duração");
  }
  for op in ["toggle_mute", "toggle_item", "scene"] {
   assert!(!temp_ok(op), "{op} é momentânea");
  }
 }
 #[test]
 fn config_defaults_and_rejects_junk() {
  let c = valid_save(&json!({})).unwrap();
  assert!(!c.enabled && c.port == DEFAULT_PORT);
  assert!(valid_save(&json!({"host": "a b"})).is_err());
  assert!(valid_save(&json!({"port": 0})).is_err());
  assert!(valid_save(&json!({"port": 70000})).is_err());
  assert!(valid_save(&json!({"enabled": true, "host": "192.168.0.10", "port": 4455})).is_ok());
 }
}

/// Observação em conexão independente: não entra na fila que restaura efeitos.
pub async fn telemetry(rt:&Arc<Runtime>,p:&Profile)->Value {
 let result=tokio::time::timeout(Duration::from_secs(4),async {
  let mut conn=connect(rt,p).await?;
  let stream=call(&mut conn,"GetStreamStatus",json!({})).await?;
  let stats=call(&mut conn,"GetStats",json!({})).await?;
  let record=call(&mut conn,"GetRecordStatus",json!({})).await?;
  let scene=program_scene(&mut conn).await?;
  close(conn).await;
  // Campos permitidos: não expor chave de transmissão, senha, caminhos ou configurações.
  if !stream["outputActive"].is_boolean()||!stream["outputReconnecting"].is_boolean()||!record["outputActive"].is_boolean()||!stats["activeFps"].is_number()||!stats["cpuUsage"].is_number()||["outputDuration","outputBytes","outputSkippedFrames","outputTotalFrames"].iter().any(|key|stream[*key].as_u64().is_none()){return Err("Telemetria incompleta do OBS".to_string())}
  Ok::<Value,String>(json!({"streaming":stream["outputActive"],"reconnecting":stream["outputReconnecting"],"durationMs":stream["outputDuration"],"bytes":stream["outputBytes"],"skipped":stream["outputSkippedFrames"],"total":stream["outputTotalFrames"],"recording":record["outputActive"],"fps":stats["activeFps"],"cpu":stats["cpuUsage"],"renderSkipped":stats["renderSkippedFrames"],"scene":scene}))
 }).await.unwrap_or_else(|_|Err("OBS sem resposta no prazo de observação".into()));
 let at=crate::dashboard::now();match result{Ok(data)=>json!({"state":"fresh","observedAt":at,"lastSuccessAt":at,"data":data}),Err(error)=>json!({"state":"unavailable","observedAt":at,"data":null,"error":error})}
}
