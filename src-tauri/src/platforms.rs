use crate::{engine::Runtime,model::*,oauth};
use serde_json::{json,Value};
use std::{sync::Arc,time::Duration};
use futures_util::{StreamExt,SinkExt};
use tokio_tungstenite::{connect_async,tungstenite::Message};
pub async fn send(rt:&Runtime,p:&Profile,text:&str)->Result<(),String>{
 let token=oauth::token(&rt.http,p,"bot").await?;
 let text:String=text.chars().take(450).collect();
 let req=match p.platform.as_str(){
 "twitch"=>rt.http.post("https://api.twitch.tv/helix/chat/messages").header("Client-Id",&p.client_id).bearer_auth(token).json(&json!({"broadcaster_id":p.channel_id,"sender_id":p.bot_id,"message":text})),
 "youtube"=>{
 let chat=rt.db.module(&p.id,"youtube_chat");let id=chat.as_str().ok_or("Conecte uma transmissão ativa do YouTube")?;
 rt.http.post("https://www.googleapis.com/youtube/v3/liveChat/messages").query(&[("part","snippet")]).bearer_auth(token).json(&json!({"snippet":{"liveChatId":id,"type":"textMessageEvent","textMessageDetails":{"messageText":text}}}))
 },
 "kick"=>rt.http.post("https://api.kick.com/public/v1/chat").bearer_auth(token).json(&json!({"broadcaster_user_id":p.channel_id.parse::<u64>().map_err(|_|"Informe o ID numérico do canal Kick")?,"content":text,"type":"bot"})),
 _=>return Err("Plataforma desconhecida".into())
 };
 let res=req.send().await.map_err(|_|"Não foi possível enviar a mensagem")?;
 if !res.status().is_success(){return Err(format!("Envio recusado: HTTP {}. Confira autorização e permissões.",res.status().as_u16()))}
 if p.platform=="twitch" {
 let v:Value=res.json().await.map_err(|_|"Resposta de envio inválida")?;
 if v["data"][0]["is_sent"]!=true{return Err("A Twitch não publicou a mensagem (limite ou restrição do canal)".into())}
 }
 Ok(())
}
pub async fn twitch(rt:Arc<Runtime>,p:Profile)->Result<(),String>{
 tokio::select!{result=twitch_events(rt.clone(),p.clone())=>result,_=crate::presence::run(rt,p)=>Ok(())}
}
async fn twitch_events(rt:Arc<Runtime>,p:Profile)->Result<(),String>{
 if crate::secrets::get(&p.id,"channel_token").is_ok() {
 tokio::try_join!(twitch_session(rt.clone(),p.clone(),"bot"),twitch_session(rt,p,"channel"))?;Ok(())
 }else{twitch_session(rt,p,"bot").await}
}
async fn twitch_session(rt:Arc<Runtime>,p:Profile,account:&str)->Result<(),String>{
 if p.channel_id.is_empty()||p.bot_id.is_empty(){return Err("Autorize as contas do canal e do bot antes de conectar".into())}
 let bot=oauth::token(&rt.http,&p,account).await?;
 let (mut ws,_)=connect_async("wss://eventsub.wss.twitch.tv/ws?keepalive_timeout_seconds=30").await.map_err(|_|"Não foi possível conectar ao EventSub")?;
 loop{
 let msg=tokio::time::timeout(Duration::from_secs(40),ws.next()).await.map_err(|_|"Twitch sem resposta; reconectando")?.ok_or("Conexão Twitch encerrada")?.map_err(|_|"Conexão Twitch interrompida")?;
 match msg{
 Message::Ping(v)=>{ws.send(Message::Pong(v)).await.map_err(|_|"Falha de conexão")?;continue},
 Message::Close(_)=>return Err("Conexão Twitch encerrada".into()),
 Message::Text(text)=>{
 let v:Value=serde_json::from_str(&text).map_err(|_|"Evento Twitch inválido")?;
 match v["metadata"]["message_type"].as_str().unwrap_or(""){
 "session_welcome"=>{
 let session=v["payload"]["session"]["id"].as_str().ok_or("Sessão Twitch inválida")?;
 if account=="bot" {subscribe(&rt,&p,&bot,session,"channel.chat.message","1",json!({"broadcaster_user_id":p.channel_id,"user_id":p.bot_id})).await?;
 rt.status(&p.id,"online");}
 if account=="channel"{let channel=bot.clone();
 for (kind,version,condition) in [
 ("channel.follow","2",json!({"broadcaster_user_id":p.channel_id,"moderator_user_id":p.channel_id})),
 ("channel.subscribe","1",json!({"broadcaster_user_id":p.channel_id})),
 ("channel.cheer","1",json!({"broadcaster_user_id":p.channel_id})),
 ("channel.raid","1",json!({"to_broadcaster_user_id":p.channel_id})),
 ("channel.channel_points_custom_reward_redemption.add","1",json!({"broadcaster_user_id":p.channel_id}))
 ] {
 if let Err(err)=subscribe(&rt,&p,&channel,session,kind,version,condition).await{rt.log(&p.id,"subscription",&format!("{kind}: {err}"),"error");}
 }
 }
 },
 "notification"=>{if let Some(e)=normalize_twitch(&p,&v){rt.submit(e).await?;}},
 "session_reconnect"=>{
 let target=v["payload"]["session"]["reconnect_url"].as_str().ok_or("Reconexão sem URL")?;
 let u=url::Url::parse(target).map_err(|_|"URL de reconexão inválida")?;
 if u.scheme()!="wss"||!u.host_str().is_some_and(|h|h=="eventsub.wss.twitch.tv"||h.ends_with(".twitch.tv")){return Err("Servidor de reconexão inválido".into())}
 let (mut next,_)=connect_async(target).await.map_err(|_|"Falha ao reconectar Twitch")?;
 let welcome=tokio::time::timeout(Duration::from_secs(15),next.next()).await.map_err(|_|"Reconexão excedeu o tempo")?.ok_or("Reconexão fechada")?.map_err(|_|"Falha de reconexão")?;
 if !welcome.is_text(){return Err("Reconexão sem confirmação".into())}
 let _=ws.close(None).await;ws=next;
 },
 "revocation"=>{rt.log(&p.id,"subscription","Uma autorização Twitch foi revogada. Reconecte a conta.","error");},
 _=>{}
 }
 },_=>{}
 }
 }
}
async fn subscribe(rt:&Runtime,p:&Profile,token:&str,session:&str,kind:&str,version:&str,condition:Value)->Result<(),String>{
 let res=rt.http.post("https://api.twitch.tv/helix/eventsub/subscriptions").header("Client-Id",&p.client_id).bearer_auth(token).json(&json!({"type":kind,"version":version,"condition":condition,"transport":{"method":"websocket","session_id":session}})).send().await.map_err(|_|"Falha ao assinar evento")?;
 if !res.status().is_success(){return Err(format!("HTTP {}: confira escopos e elegibilidade do canal",res.status().as_u16()))}Ok(())
}
pub fn normalize_twitch(p:&Profile,v:&Value)->Option<Event>{
 let payload=&v["payload"]["event"];
 let kind=match v["metadata"]["subscription_type"].as_str()?{
 "channel.chat.message"=>"chat","channel.follow"=>"follow","channel.subscribe"=>"subscription","channel.cheer"=>"cheer","channel.raid"=>"raid","channel.channel_points_custom_reward_redemption.add"=>"redemption",_=>return None};
 let user_id=payload["chatter_user_id"].as_str().or(payload["user_id"].as_str()).or(payload["from_broadcaster_user_id"].as_str()).unwrap_or("");
 if user_id==p.bot_id{return None}
 let mut role="everyone";
 if let Some(badges)=payload["badges"].as_array(){
 for b in badges{if b["set_id"]=="subscriber"{role="subscriber"}}
 for b in badges{if b["set_id"]=="moderator"{role="moderator"}}
 }
 if user_id==p.channel_id{role="broadcaster"}
 Some(Event{id:v["metadata"]["message_id"].as_str()?.into(),profile_id:p.id.clone(),kind:kind.into(),user:payload["chatter_user_name"].as_str().or(payload["user_name"].as_str()).or(payload["from_broadcaster_user_name"].as_str()).unwrap_or("espectador").into(),user_id:user_id.into(),role:role.into(),message:payload["message"]["text"].as_str().or(payload["message"].as_str()).or(payload["user_input"].as_str()).unwrap_or("").into(),data:payload.clone(),simulated:false})
}
pub async fn youtube(rt:Arc<Runtime>,p:Profile)->Result<(),String>{
 let token=oauth::token(&rt.http,&p,"bot").await?;
 // A channel profile can point to a live video ID, without requiring broadcaster ownership.
 let res=rt.http.get("https://www.googleapis.com/youtube/v3/videos").query(&[("part","liveStreamingDetails"),("id",p.channel_id.as_str())]).bearer_auth(&token).send().await.map_err(|_|"Não foi possível consultar a transmissão")?;
 if !res.status().is_success(){return Err(format!("YouTube HTTP {}",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Resposta YouTube inválida")?;
 let chat=v["items"][0]["liveStreamingDetails"]["activeLiveChatId"].as_str().ok_or("Informe o ID de um vídeo ao vivo com chat ativo")?.to_owned();
 rt.db.set_module(&p.id,"youtube_chat",&json!(chat))?;rt.status(&p.id,"online");
 let mut page=String::new();let mut first=true;
 loop {
 let res=rt.http.get("https://www.googleapis.com/youtube/v3/liveChat/messages").query(&[("liveChatId",chat.as_str()),("part","snippet,authorDetails"),("pageToken",page.as_str())]).bearer_auth(&token).send().await.map_err(|_|"Falha ao consultar o chat YouTube")?;
 if !res.status().is_success(){return Err(format!("Chat YouTube HTTP {}. Confira quota e transmissão.",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Evento YouTube inválido")?;
 if !first {if let Some(items)=v["items"].as_array(){for item in items{
 let author=&item["authorDetails"];let snippet=&item["snippet"];
 let uid=author["channelId"].as_str().unwrap_or("");if uid==p.bot_id{continue}
 let kind=match snippet["type"].as_str().unwrap_or(""){"textMessageEvent"=>"chat","superChatEvent"|"superStickerEvent"=>"cheer","newSponsorEvent"|"memberMilestoneChatEvent"=>"subscription",_=>continue};
 rt.submit(Event{id:item["id"].as_str().unwrap_or("").into(),profile_id:p.id.clone(),kind:kind.into(),user:author["displayName"].as_str().unwrap_or("espectador").into(),user_id:uid.into(),role:if author["isChatOwner"]==true{"broadcaster"}else if author["isChatModerator"]==true{"moderator"}else if author["isChatSponsor"]==true{"subscriber"}else{"everyone"}.into(),message:snippet["displayMessage"].as_str().unwrap_or("").into(),data:snippet.clone(),simulated:false}).await?;
 }}}
 first=false;page=v["nextPageToken"].as_str().unwrap_or("").into();
 tokio::time::sleep(Duration::from_millis(v["pollingIntervalMillis"].as_u64().unwrap_or(5000).max(1000))).await;
 }
}
