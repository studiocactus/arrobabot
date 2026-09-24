//! Optional IRC membership observer. EventSub remains responsible for messages.
use crate::{chat_extras,engine::Runtime,model::{Profile,Event},oauth};
use futures_util::{SinkExt,StreamExt};
use serde_json::{json,Value};
use std::{collections::HashSet,sync::Arc,time::Duration};
use tokio_tungstenite::{connect_async,tungstenite::Message};
fn enabled(rt:&Runtime,p:&Profile)->bool{let c=chat_extras::config(rt,&p.id);c.sounds_enabled&&c.presence_enabled&&c.people.iter().any(|a|a.enabled&&a.trigger!="message")}
fn login(s:&str)->bool{!s.is_empty()&&s.len()<=25&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')}
fn parts(line:&str)->(&str,&str,&str){
 let line=if line.starts_with('@'){line.split_once(' ').map_or("",|(_,s)|s)}else{line};
 let (prefix,rest)=if let Some(line)=line.strip_prefix(':'){line.split_once(' ').unwrap_or(("",""))}else{("",line)};
 let (command,args)=rest.split_once(' ').unwrap_or((rest,""));(prefix,command,args)
}
pub async fn run(rt:Arc<Runtime>,p:Profile){
 loop{
  if enabled(&rt,&p){if let Err(error)=session(&rt,&p).await{rt.log(&p.id,"presence",&error,"error");tokio::time::sleep(Duration::from_secs(60)).await;}}
  tokio::time::sleep(Duration::from_secs(5)).await;
 }
}
async fn session(rt:&Runtime,p:&Profile)->Result<(),String>{
 let token=oauth::token(&rt.http,p,"bot").await?;
 let identity:Value=rt.http.get("https://id.twitch.tv/oauth2/validate").header("Authorization",format!("OAuth {token}")).send().await.map_err(|_|"Falha ao validar monitor de entradas")?.json().await.map_err(|_|"Identidade Twitch inválida")?;
 if !identity["scopes"].as_array().is_some_and(|v|v.iter().any(|s|s=="chat:read")){return Err("Para entradas silenciosas, autorize novamente a conta do bot no perfil (permissão chat:read). As mensagens continuam pelo EventSub.".into())}
 let nick=identity["login"].as_str().filter(|s|login(s)).ok_or("Login do bot inválido")?;
 let channel:Value=rt.http.get("https://api.twitch.tv/helix/users").header("Client-Id",&p.client_id).bearer_auth(&token).query(&[("id",&p.channel_id)]).send().await.map_err(|_|"Falha ao consultar canal para entradas")?.json().await.map_err(|_|"Canal inválido")?;
 let channel=channel["data"][0]["login"].as_str().filter(|s|login(s)).ok_or("Canal Twitch não encontrado para entradas")?;
 let room=format!("#{channel}");
 let (mut ws,_)=tokio::time::timeout(Duration::from_secs(20),connect_async("wss://irc-ws.chat.twitch.tv:443")).await.map_err(|_|"Tempo esgotado ao conectar entradas Twitch")?.map_err(|_|"Não foi possível conectar entradas Twitch")?;
 ws.send(Message::Text(format!("CAP REQ :twitch.tv/membership\r\nPASS oauth:{token}\r\nNICK {nick}\r\nJOIN {room}\r\n").into())).await.map_err(|_|"Falha ao autorizar entradas Twitch")?;
 let mut ready=false;let mut present=HashSet::<String>::new();let mut tick=tokio::time::interval(Duration::from_secs(5));let mut last=std::time::Instant::now();let started=last;
 loop{
  tokio::select!{
   _=tick.tick()=>{
    if !enabled(rt,p){let _=ws.close(None).await;return Ok(())}
    if last.elapsed()>Duration::from_secs(300)||(!ready&&started.elapsed()>Duration::from_secs(30)){return Err("Monitor de entradas sem resposta; tentando reconectar".into())}
   },
   message=ws.next()=>{
    let message=message.ok_or("Monitor de entradas desconectado")?.map_err(|_|"Conexão de entradas interrompida")?;last=std::time::Instant::now();
    match message{
     Message::Ping(data)=>ws.send(Message::Pong(data)).await.map_err(|_|"Falha de conexão de entradas")?,
     Message::Close(_)=>return Err("Monitor de entradas desconectado".into()),
     Message::Text(text)=>for line in text.lines(){
      let (prefix,command,args)=parts(line.trim_end_matches('\r'));
      if command=="PING"{ws.send(Message::Text(format!("PONG {args}\r\n").into())).await.map_err(|_|"Falha ao responder Twitch")?;continue}
      if command=="NOTICE"&& (args.contains("authentication failed")||args.contains("Improperly formatted auth")){return Err("Twitch recusou o monitor de entradas. Autorize novamente o bot.".into())}
      if command=="RECONNECT"{return Err("Twitch solicitou reconexão do monitor de entradas".into())}
      if command=="366"&&args.split_whitespace().any(|s|s==room){ready=true;rt.log(&p.id,"presence","Monitor de entradas ativo. Aguardando novas conexões ao chat.","success");continue}
      if !ready||!matches!(command,"JOIN"|"PART")||args.trim_start_matches(':').split_whitespace().next()!=Some(room.as_str()){continue}
      let name=prefix.split('!').next().unwrap_or("").to_ascii_lowercase();
      if !login(&name)||name==nick{continue}
      if command=="PART"{present.remove(&name);continue}
      let c=chat_extras::config(rt,&p.id);
      // Only configured people need tracking or an identity lookup.
      if !c.people.iter().any(|a|a.enabled&&a.trigger!="message"&&a.name.trim().trim_start_matches('@').eq_ignore_ascii_case(&name)){continue}
      if !present.insert(name.clone()){continue}
      if present.len()>200{present.clear();present.insert(name.clone());}
      let mut id=String::new();
      if c.people.iter().any(|a|!a.user_id.is_empty()&&a.name.trim().trim_start_matches('@').eq_ignore_ascii_case(&name)){
       let user:Value=rt.http.get("https://api.twitch.tv/helix/users").header("Client-Id",&p.client_id).bearer_auth(&token).query(&[("login",&name)]).send().await.map_err(|_|"Falha ao identificar entrada Twitch")?.json().await.map_err(|_|"Identidade da entrada inválida")?;
       id=user["data"][0]["id"].as_str().unwrap_or("").into();
      }
      // Membership alone must not award points or trigger unrelated automations.
      chat_extras::sound(rt,p,&Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),kind:"join".into(),user:name,user_id:id,role:"everyone".into(),message:String::new(),data:json!({"source":"twitch-irc"}),simulated:false});
     },_=>{}
    }
   }
  }
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn parses_membership_and_ping(){assert_eq!(parts(":ana!ana@ana.tmi.twitch.tv JOIN #canal"),("ana!ana@ana.tmi.twitch.tv","JOIN","#canal"));assert_eq!(parts("PING :tmi.twitch.tv"),("","PING",":tmi.twitch.tv"));assert_eq!(parts("@a=b :server 366 bot #canal :End"),("server","366","bot #canal :End"));assert!(!login("a\r\nJOIN #outro"));assert!(login("ana_123"));}
}
