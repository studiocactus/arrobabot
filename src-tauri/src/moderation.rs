use crate::{engine::Runtime,model::*,oauth};
use serde_json::{json,Value};
use std::time::{Instant,Duration};
pub fn detect(rt:&Runtime,p:&Profile,e:&Event)->Option<&'static str>{
 if !p.modules["moderation"].as_bool().unwrap_or(false)||permitted("moderator",&e.role)||e.simulated{return None}
 if blocked(&e.message,p){return Some("expressão proibida")}
 if ["http://","https://","www."].iter().any(|s|e.message.to_lowercase().contains(s)){return Some("link não autorizado")}
 let mut seen=rt.cooldowns.lock().unwrap();
 let key=format!("spam:{}:{}:{}",p.id,e.user_id,e.message.to_lowercase());
 let repeated=seen.get(&key).is_some_and(|t|t.elapsed()<Duration::from_secs(15));
 seen.insert(key,Instant::now());if repeated{Some("mensagem repetida")}else{None}
}
pub async fn act(rt:&Runtime,p:&Profile,e:&Event,reason:&str)->Result<(),String>{
 let config=rt.db.module(&p.id,"moderation");
 let action=config["action"].as_str().unwrap_or("ignore");
 if action=="ignore"{return Ok(())}
 if p.platform!="twitch"{return Err("Ação nativa de moderação configurada somente para Twitch".into())}
 let token=oauth::token(&rt.http,p,"channel").await?;
 let req=match action{
 "warn"=>rt.http.post("https://api.twitch.tv/helix/moderation/warnings").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",p.channel_id.as_str())]).json(&json!({"data":{"user_id":e.user_id,"reason":reason}})),
 "timeout"|"ban"=>{
 let mut data=json!({"user_id":e.user_id,"reason":reason});
 if action=="timeout"{data["duration"]=json!(config["seconds"].as_u64().unwrap_or(60).clamp(1,1209600));}
 rt.http.post("https://api.twitch.tv/helix/moderation/bans").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",p.channel_id.as_str())]).json(&json!({"data":data}))
 },
 _=>return Err("Ação de moderação inválida".into())
 };
 let res=req.header("Client-Id",&p.client_id).bearer_auth(token).send().await.map_err(|_|"Falha ao moderar mensagem")?;
 if !res.status().is_success(){return Err(format!("Moderação recusada: HTTP {}",res.status().as_u16()))}Ok(())
}
pub fn valid_config(v:&Value)->bool{["ignore","warn","timeout","ban"].contains(&v["action"].as_str().unwrap_or(""))}
