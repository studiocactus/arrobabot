use crate::{engine::Runtime,model::*,oauth};
use rand::Rng;
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
/// Primeiro alvo escrito na mensagem: quando o gatilho é comando, o salto pula o "!comando".
fn first_login(message:&str)->Result<String,String>{
 let mut words=message.split_whitespace().map(|w|w.trim_start_matches('@')).filter(|w|!w.is_empty());
 let first=words.next().unwrap_or("");
 let login=if first.starts_with('!'){words.next().unwrap_or("")}else{first};
 if login.is_empty(){return Err("Escreva quem leva a ação depois do comando, como !silenciar @alvo".into())}
 Ok(login.to_lowercase())
}
/// Converte nome de usuário no ID da Twitch, o identificador que a API de moderação usa.
pub async fn twitch_user_id(rt:&Runtime,p:&Profile,login:&str)->Result<String,String>{
 let login=login.trim_start_matches('@').to_lowercase();
 if login.is_empty()||login.len()>40||!login.chars().all(|c|c.is_ascii_alphanumeric()||c=='_'||c=='-'){return Err("Informe um nome de usuário válido da Twitch".into())}
 let token=oauth::token(&rt.http,p,"channel").await?;
 let res=rt.http.get("https://api.twitch.tv/helix/users").query(&[("login",login.as_str())]).header("Client-Id",&p.client_id).bearer_auth(token).send().await.map_err(|_|"Não foi possível consultar o usuário na Twitch".to_string())?;
 if !res.status().is_success(){return Err(format!("Consulta de usuário recusada pela Twitch: HTTP {}",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Resposta inválida da Twitch".to_string())?;
 let id=v["data"][0]["id"].as_str().unwrap_or("").to_owned();
 if id.is_empty(){return Err(format!("A Twitch não encontrou o usuário {login}"))}
 Ok(id)
}
/// Sorteia quem leva a punição entre quem já falou no chat, sem o streamer.
/// Devolve o login sorteado ou explica por que não houve sorteio.
pub fn draw_punished(names:&[String],channel:&str)->Result<String,String>{
 if names.is_empty(){return Err("Ninguém no cadastro para sortear: use {{randomViewer|default:alguém}} na mensagem e aguarde o chat falar".into())}
 let eligible:Vec<&String>=names.iter().filter(|n|!n.trim().is_empty()&&!n.eq_ignore_ascii_case(channel)).collect();
 if eligible.is_empty(){return Err("Só o streamer no cadastro: aguarde mais gente falar".into())}
 Ok(eligible[rand::thread_rng().gen_range(0..eligible.len())].clone())
}
/// Punição pedida por uma automação: silencia, bane ou avisa na Twitch.
/// A prévia nunca executa aqui e mensagens vindas do Discord ficam de fora: o alvo é da Twitch.
pub async fn punish(rt:&Runtime,p:&Profile,e:&Event,a:&Action,reason:&str,variables:Option<&mut crate::variables::Context>)->Result<(),String>{
 if e.simulated{return Ok(())}
 if crate::discord::from_discord(e).is_some(){return Err("Esta ação pune na Twitch e não vale em mensagens vindas do Discord".into())}
 if p.platform!="twitch"||p.channel_id.is_empty(){return Err("A punição usa a Twitch: autorize a conta do canal no perfil".into())}
 if !PUNISH_MODES.contains(&a.punish.as_str()){return Err("Escolha silenciar, banir ou avisar".into())}
 if !PUNISH_TARGETS.contains(&a.target.as_str()){return Err("Escolha quem leva a ação: quem enviou, o primeiro argumento ou um sorteado".into())}
 let login;
 let user_id=match a.target.as_str() {
  "first"=>{login=first_login(&e.message)?;twitch_user_id(rt,p,&login).await?},
  "random"=>{
   let names=crate::modules::chatter_names(&rt.db,&p.id);
   // Tentativas curtas: pula streamer e a conta do próprio bot, que a API recusa.
   let mut picked=String::new();
   for _ in 0..names.len().max(1).min(5){
    let candidate=draw_punished(&names,&p.channel)?;
    let id=twitch_user_id(rt,p,&candidate).await?;
    if id!=p.channel_id&&id!=p.bot_id{picked=candidate;break}
   }
   if picked.is_empty(){return Err("O sorteio só caiu em contas impuníveis (streamer ou bot)".into())}
   login=picked.to_lowercase();twitch_user_id(rt,p,&login).await?
  },
  _=>{
   if e.user_id.is_empty()||!e.user_id.chars().all(|c|c.is_ascii_digit()){return Err("Este evento não traz uma conta da Twitch para punir".into())}
   login=e.user.to_lowercase();e.user_id.clone()
  }
 };
 if let Some(v)=variables{let _=v.change(&rt.db,p,e,"local.punished","set",json!(login.clone()));}
 if user_id==p.channel_id||login.eq_ignore_ascii_case(&p.channel){return Err("O próprio streamer não pode levar esta punição".into())}
 let reason=if reason.trim().is_empty(){"Regras da comunidade".to_string()}else{reason.chars().take(500).collect::<String>()};
 let seconds=if a.punish=="timeout"{a.value.clamp(1,1209600) as u64}else{0};
 twitch_punish(rt,p,&a.punish,&user_id,&reason,seconds).await?;
 rt.log(&p.id,"moderation",&format!("{} aplicado a @{login} pela automação: {reason}",a.punish.as_str()),"success");
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
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn draw_punished_skips_streamer_and_needs_a_crowd(){
  assert!(draw_punished(&[],"thenees").is_err());
  assert!(draw_punished(&["Thenees".into()], "thenees").is_err());
  let pool=vec!["Thenees".into(),"Ana".into(),"Bia".into()];
  for _ in 0..50{
   let pick=draw_punished(&pool,"thenees").unwrap();
   assert!(["Ana","Bia"].contains(&pick.as_str()),"streamer nunca cai: {pick}");
  }
 }
}
