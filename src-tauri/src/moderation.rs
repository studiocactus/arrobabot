use crate::{engine::Runtime,model::*,oauth};
use serde_json::{json,Value};
use std::time::{Instant,Duration};
/// Content rules shared by Twitch and Discord: blocklist, links and repeated messages.
/// Returns the reason when the message must be acted on, or `None` for moderators and clean text.
pub fn reasons(rt:&Runtime,p:&Profile,text:&str,user_id:&str,role:&str)->Option<&'static str>{
 if permitted("moderator",role){return None}
 if blocked(text,p){return Some("expressão proibida")}
 if ["http://","https://","www."].iter().any(|s|text.to_lowercase().contains(s)){return Some("link não autorizado")}
 let mut seen=rt.cooldowns.lock().unwrap();
 let key=format!("spam:{}:{}:{}",p.id,user_id,text.to_lowercase());
 let repeated=seen.get(&key).is_some_and(|t|t.elapsed()<Duration::from_secs(15));
 seen.insert(key,Instant::now());
 if repeated{return Some("mensagem repetida")}
 None
}
pub fn detect(rt:&Runtime,p:&Profile,e:&Event)->Option<&'static str>{
 if !p.modules["moderation"].as_bool().unwrap_or(false)||e.simulated{return None}
 reasons(rt,p,&e.message,&e.user_id,&e.role)
}
/// Applies a Twitch moderation action directly, without needing an incoming event.
/// Used both by the automatic moderation pipeline and by the cross-platform "punish" action.
pub async fn twitch_punish(rt:&Runtime,p:&Profile,action:&str,user_id:&str,reason:&str,seconds:u64)->Result<(),String>{
 if p.platform!="twitch"||p.channel_id.is_empty()||user_id.is_empty(){return Err("A moderação da Twitch exige um perfil Twitch com ID de canal e um alvo".into())}
 let token=oauth::token(&rt.http,p,"channel").await?;
 let req=match action{
 "warn"=>rt.http.post("https://api.twitch.tv/helix/moderation/warnings").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",p.channel_id.as_str())]).json(&json!({"data":{"user_id":user_id,"reason":reason}})),
 "timeout"|"ban"=>{
  let mut data=json!({"user_id":user_id,"reason":reason});
  if action=="timeout"{data["duration"]=json!(seconds.clamp(1,1209600));}
  rt.http.post("https://api.twitch.tv/helix/moderation/bans").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",p.channel_id.as_str())]).json(&json!({"data":data}))
 },
 "unban"=>rt.http.delete("https://api.twitch.tv/helix/moderation/bans").query(&[("broadcaster_id",p.channel_id.as_str()),("user_id",user_id)]),
 _=>return Err("Ação de moderação inválida".into())
 };
 let res=req.header("Client-Id",&p.client_id).bearer_auth(token).send().await.map_err(|_|"Falha ao moderar na Twitch".to_string())?;
 if !res.status().is_success(){return Err(format!("Moderação recusada pela Twitch: HTTP {}",res.status().as_u16()))}
 Ok(())
}
pub async fn act(rt:&Runtime,p:&Profile,e:&Event,reason:&str)->Result<(),String>{
 let config=rt.db.module(&p.id,"moderation");
 let action=config["action"].as_str().unwrap_or("ignore");
 if action=="ignore"{return Ok(())}
 let seconds=if action=="timeout"{config["seconds"].as_u64().unwrap_or(60).clamp(1,1209600)}else{60};
 twitch_punish(rt,p,action,&e.user_id,reason,seconds).await
}
pub fn valid_config(v:&Value)->bool{["ignore","warn","timeout","ban"].contains(&v["action"].as_str().unwrap_or(""))}
