use crate::{engine::Runtime,model::Event};
use std::{sync::Arc,time::Duration};
use serde_json::{json,Value};
use futures_util::{SinkExt,StreamExt};
use tokio_tungstenite::{accept_async_with_config,tungstenite::{Message,protocol::WebSocketConfig}};
pub async fn serve(rt:Arc<Runtime>)->Result<(),String>{
 let port=rt.db.get("apiPort").as_u64().unwrap_or(9876).clamp(1024,65535) as u16;
 let listener=tokio::net::TcpListener::bind(("127.0.0.1",port)).await.map_err(|_|format!("Porta local {port} ocupada. Altere a porta nas configurações e reinicie."))?;
 *rt.api_port.lock().unwrap()=port;
 let limit=Arc::new(tokio::sync::Semaphore::new(16));
 loop{
 let (socket,_)=listener.accept().await.map_err(|_|"Falha na API local")?;
 let Ok(permit)=limit.clone().try_acquire_owned() else{continue};
 let rt=rt.clone();
 tokio::spawn(async move{
 let _permit=permit;
 let config=WebSocketConfig::default().max_message_size(Some(64*1024)).max_frame_size(Some(64*1024));
 let Ok(Ok(mut ws))=tokio::time::timeout(Duration::from_secs(5),accept_async_with_config(socket,Some(config))).await else{return};
 let Ok(Some(Ok(Message::Text(first))))=tokio::time::timeout(Duration::from_secs(5),ws.next()).await else{return};
 let Ok(auth)=serde_json::from_str::<Value>(&first) else{return};
 if auth["token"].as_str()!=Some(rt.api_token.as_str()){let _=ws.close(None).await;return}
 let _=ws.send(Message::Text(json!({"type":"ready","version":1}).to_string().into())).await;
 for p in rt.db.profiles().unwrap_or_default(){let state=crate::modules::load(&rt,&p.id);if !state.songs.is_empty(){let _=ws.send(Message::Text(json!({"type":"songs","payload":{"profileId":p.id,"queue":state.songs}}).to_string().into())).await;}}
 let mut events=rt.broadcast.subscribe();
 loop{
 tokio::select!{
 item=ws.next()=>{
 let Some(Ok(msg))=item else{break};
 match msg{
 Message::Text(t)=>{
 let result=async {
 let request:Value=serde_json::from_str(&t).map_err(|_|"JSON inválido".to_string())?;
 if request["type"]!="event"{return Err("Use type: event".to_string())}
 let mut e:Event=serde_json::from_value(request["event"].clone()).map_err(|_|"Evento inválido".to_string())?;
 // External events never assert broadcaster/moderator privileges.
 e.role="everyone".into();e.id=uuid::Uuid::new_v4().to_string();
 rt.submit(e).await
 }.await;
 let response=match result{Ok(())=>json!({"type":"ack"}),Err(e)=>json!({"type":"error","message":e})};
 if ws.send(Message::Text(response.to_string().into())).await.is_err(){break}
 },
 Message::Ping(v)=>{if ws.send(Message::Pong(v)).await.is_err(){break}},
 Message::Close(_)=>break,_=>{}
 }
 },
 event=events.recv()=>match event{
 Ok(v)=>{if ws.send(Message::Text(v.to_string().into())).await.is_err(){break}},
 Err(tokio::sync::broadcast::error::RecvError::Lagged(_))=>continue,
 Err(_)=>break
 }
 }
 }
 });
 }
}
