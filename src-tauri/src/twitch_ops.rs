//! Ações na Twitch executadas pela conta do bot: categoria, título, moderação,
//! VIP, modos do chat, destaque e menção.
//! O bot precisa ser moderador do canal (moderação, VIP, modos do chat) e editor
//! do canal (categoria e título). Sem isso a Twitch responde 401/403 e o erro
//! explica o que falta. Reautorizar o bot em Perfis concede os escopos novos.
use crate::{engine::Runtime, model::*};
use serde_json::{json, Value};
use std::{sync::Arc, time::{Duration, Instant}};
/// Operações aceitas pela ação "twitch" (campo tw_op).
pub const OPS:&[&str]=&["game","title","timeout","ban","unban","warn","vip","unvip","slow","slowoff","followers","followersoff","subsonly","subsonlyoff","emoteonly","emoteonlyoff","shoutout","mention"];
/// Quem passou pelo chat fica aqui por 24 h para a fala livre achar nomes sem @.
const SEEN_TTL:Duration=Duration::from_secs(86400);
const SEEN_CAP:usize=2000;
fn login_chars(s:&str)->bool{!s.is_empty()&&s.len()<=25&&s.chars().all(|c|c.is_ascii_alphanumeric()||c=='_'||c=='-')}
fn mention_ok(s:&str)->bool{s.len()>=2&&login_chars(s)}
fn bare_ok(s:&str)->bool{s.len()>=4&&login_chars(s)}
/// Tira pontuação das bordas e minúsculas: "Maria," vira "maria".
fn clean_word(w:&str)->String{w.trim_matches(|c:char|!(c.is_ascii_alphanumeric()||c=='_'||c=='-')).to_lowercase()}
/// Anota quem falou no chat para a resolução de nomes sem @.
pub fn note_chatter(rt:&Arc<Runtime>,e:&Event){
 if e.kind!="chat"||e.simulated{return}
 let login=e.data["chatter_user_login"].as_str().map(str::to_owned).filter(|s|login_chars(&s.to_lowercase())).or_else(||{
  let u=e.user.to_lowercase();
  if !u.contains(char::is_whitespace)&&login_chars(&u){Some(u)}else{None}
 });
 let Some(login)=login else{return};
 let display=e.user.to_lowercase();
 let now=Instant::now();
 let mut map=rt.seen_chatters.lock().unwrap();
 map.retain(|_,(_,at)|now.saturating_duration_since(*at)<SEEN_TTL);
 if map.len()>=SEEN_CAP{
  let mut oldest:Vec<(String,Instant)>=map.iter().map(|(k,(_,at))|(k.clone(),*at)).collect();
  oldest.sort_by_key(|(_,at)|*at);
  for (k,_) in oldest.into_iter().take(map.len()-SEEN_CAP+1){map.remove(&k);}
 }
 map.insert(login.clone(),(login.clone(),now));
 if display!=login&&login_chars(&display){map.insert(display,(login,now));}
}
/// Primeiro número da fala: "timeout 300" ou "slow 5".
fn first_number(message:&str)->Option<i64>{message.split_whitespace().filter_map(|w|w.parse::<i64>().ok()).find(|n|*n>=0)}
async fn user_id(rt:&Runtime,p:&Profile,login:&str)->Result<String,String>{crate::moderation::twitch_user_id(rt,p,login).await}
/// Resolve quem sofre a ação: @menção, nome visto no chat, alvo fixo do editor.
fn resolve_login(rt:&Runtime,p:&Profile,e:&Event,a:&Action)->Result<(String,String),String>{
 let words:Vec<&str>=e.message.split_whitespace().collect();
 for w in &words{
  if w.starts_with('@')&&w.len()>1{
   let login=clean_word(&w[1..]);
   if mention_ok(&login){let id=user_id(rt,p,&login).await?;return Ok((login,id))}
  }
 }
 {
  let map=rt.seen_chatters.lock().unwrap();
  for w in words.iter().rev(){
   let name=clean_word(w);
   if bare_ok(&name){if let Some((login,_))=map.get(&name){let login=login.clone();drop(map);let id=user_id(rt,p,&login).await?;return Ok((login,id))}}
  }
 }
 let fixed=a.target.trim().trim_start_matches('@').to_lowercase();
 if login_chars(&fixed)&&fixed.len()>=2{let id=user_id(rt,p,&fixed).await?;return Ok((fixed,id))}
 Err("Não entendi quem. Fale @nome, use o nome de alguém do chat ou configure o alvo fixo na automação.".into())
}
fn guard_streamer(p:&Profile,login:&str,user_id:&str)->Result<(),String>{
 if user_id==p.channel_id||login.eq_ignore_ascii_case(&p.channel){return Err("O próprio streamer não pode levar esta ação".into())}
 Ok(())
}
async fn bot_creds(rt:&Runtime,p:&Profile)->Result<(String,String),String>{
 if p.platform!="twitch"||p.channel_id.is_empty()||p.bot_id.is_empty(){return Err("A ação na Twitch exige perfil Twitch com canal e bot autorizados".into())}
 let token=crate::oauth::token(&rt.http,p,"bot").await.map_err(|_|"Autorize novamente a conta do bot em Perfis para conceder as permissões novas".to_string())?;
 Ok((token,p.bot_id.clone()))
}
fn refused(what:&str,status:u16)->String{
 match status{
  401=>format!("{what} recusado. Autorize novamente a conta do bot em Perfis."),
  403=>format!("{what} recusado: marque o bot como moderador (e editor, para categoria e título) do canal e autorize novamente."),
  429=>format!("{what} recusado: a Twitch limitou os pedidos agora. Tente em instantes."),
  _=>format!("{what} recusado: HTTP {status}."),
 }
}
/// Nome do jogo vira o ID que a API de canal exige.
async fn game_id(rt:&Runtime,p:&Profile,token:&str,name:&str)->Result<String,String>{
 let res=rt.http.get("https://api.twitch.tv/helix/games").query(&[("name",name)]).header("Client-Id",&p.client_id).bearer_auth(token).send().await.map_err(|_|"Não foi possível consultar o jogo na Twitch".to_string())?;
 if !res.status().is_success(){return Err(refused("Categoria",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Resposta de jogo inválida".to_string())?;
 let id=v["data"][0]["id"].as_str().unwrap_or("").to_owned();
 if id.is_empty(){return Err(format!("A Twitch não encontrou o jogo {name}"))}
 Ok(id)
}
async fn modify_channel(rt:&Runtime,p:&Profile,token:&str,body:Value,what:&str)->Result<(),String>{
 let res=rt.http.patch("https://api.twitch.tv/helix/channels").query(&[("broadcaster_id",p.channel_id.as_str())]).header("Client-Id",&p.client_id).bearer_auth(token).json(&body).send().await.map_err(|_|format!("Não foi possível alterar {what}"))?;
 if !res.status().is_success(){return Err(refused(what,res.status().as_u16()))}
 Ok(())
}
async fn punish_as(rt:&Runtime,p:&Profile,token:&str,moderator:&str,action:&str,user_id:&str,reason:&str,seconds:u64)->Result<(),String>{
 let req=match action{
  "warn"=>rt.http.post("https://api.twitch.tv/helix/moderation/warnings").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",moderator)]).json(&json!({"data":{"user_id":user_id,"reason":reason}})),
  "timeout"|"ban"=>{
   let mut data=json!({"user_id":user_id,"reason":reason});
   if action=="timeout"{data["duration"]=json!(seconds.clamp(1,1209600));}
   rt.http.post("https://api.twitch.tv/helix/moderation/bans").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",moderator)]).json(&json!({"data":data}))
  },
  "unban"=>rt.http.delete("https://api.twitch.tv/helix/moderation/bans").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",moderator),("user_id",user_id)]),
  _=>return Err("Operação de moderação inválida".into())
 };
 let res=req.header("Client-Id",&p.client_id).bearer_auth(token).send().await.map_err(|_|"Falha ao moderar na Twitch".to_string())?;
 if !res.status().is_success(){return Err(refused("Moderação",res.status().as_u16()))}
 Ok(())
}
async fn chat_settings(rt:&Runtime,p:&Profile,token:&str,moderator:&str,body:Value)->Result<(),String>{
 let res=rt.http.patch("https://api.twitch.tv/helix/chat/settings").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",moderator)]).header("Client-Id",&p.client_id).bearer_auth(token).json(&body).send().await.map_err(|_|"Não foi possível ajustar o chat".to_string())?;
 if !res.status().is_success(){return Err(refused("Modo do chat",res.status().as_u16()))}
 Ok(())
}
async fn vip(rt:&Runtime,p:&Profile,token:&str,add:bool,user_id:&str)->Result<(),String>{
 let query=[("broadcaster_id",p.channel_id.as_str()),("user_id",user_id)];
 let res=if add{rt.http.post("https://api.twitch.tv/helix/channels/vips").query(&query).header("Client-Id",&p.client_id).bearer_auth(token).send().await}
  else{rt.http.delete("https://api.twitch.tv/helix/channels/vips").query(&query).header("Client-Id",&p.client_id).bearer_auth(token).send().await};
 let res=res.map_err(|_|"Falha ao alterar VIP na Twitch".to_string())?;
 if res.status().as_u16()==409{return Err("Limite de VIPs do canal atingido".into())}
 if !res.status().is_success(){return Err(refused("VIP",res.status().as_u16()))}
 Ok(())
}
/// Executa a operação da ação "twitch" com a conta do bot. `text` já vem renderizado.
pub async fn run(rt:&Arc<Runtime>,p:&Profile,e:&Event,a:&Action,text:&str)->Result<(),String>{
 if p.platform!="twitch"{return Err("A ação na Twitch só vale em perfil Twitch".into())}
 if crate::discord::from_discord(e).is_some(){return Err("A ação na Twitch não vale em mensagens vindas do Discord".into())}
 let (token,moderator)=bot_creds(rt,p).await?;
 match a.tw_op.as_str(){
  "game"=>{
   let name=text.trim().to_owned();
   if name.is_empty(){return Err("Escreva o nome do jogo no conteúdo da ação".into())}
   let id=game_id(rt,p,&token,&name).await?;
   modify_channel(rt,p,&token,json!({"game_id":id}),"Categoria").await?;
   rt.log(&p.id,"twitch",&format!("Categoria alterada para {name} pelo bot"),"success");Ok(())
  }
  "title"=>{
   let title:text.trim().chars().take(140).collect();
   if title.is_empty(){return Err("Escreva o novo título no conteúdo da ação".into())}
   modify_channel(rt,p,&token,json!({"title":title}),"Título").await?;
   rt.log(&p.id,"twitch",&format!("Título alterado para {title} pelo bot"),"success");Ok(())
  }
  "timeout"|"ban"|"unban"|"warn"=>{
   let (login,user_id)=resolve_login(rt,p,e,a)?;
   guard_streamer(p,&login,&user_id)?;
   let reason=if text.trim().is_empty(){"Regras da comunidade".to_string()}else{text.trim().chars().take(500).collect::<String>()};
   let seconds=if a.tw_op=="timeout"{first_number(&e.message).or(if a.value>0{Some(a.value)}else{None}).unwrap_or(60).clamp(1,1209600) as u64}else{0};
   punish_as(rt,p,&token,&moderator,&a.tw_op,&user_id,&reason,seconds).await?;
   rt.log(&p.id,"twitch",&format!("{} aplicado a @{login} pelo bot",a.tw_op.as_str()),"success");Ok(())
  }
  "vip"|"unvip"=>{
   let (login,user_id)=resolve_login(rt,p,e,a)?;
   guard_streamer(p,&login,&user_id)?;
   vip(rt,p,&token,a.tw_op=="vip",&user_id).await?;
   rt.log(&p.id,"twitch",&format!("{} aplicado a @{login} pelo bot",if a.tw_op=="vip"{"VIP"}else{"remoção de VIP"}),"success");Ok(())
  }
  "slow"|"slowoff"|"followers"|"followersoff"|"subsonly"|"subsonlyoff"|"emoteonly"|"emoteonlyoff"=>{
   let body=match a.tw_op.as_str(){
    "slow"=>{let s=first_number(&e.message).or(if a.value>0{Some(a.value)}else{None}).unwrap_or(5).clamp(3,120);json!({"slow_mode":true,"slow_mode_wait_time":s})}
    "slowoff"=>json!({"slow_mode":false}),
    "followers"=>{let m=first_number(&e.message).or(if a.value>0{Some(a.value)}else{None}).unwrap_or(30).clamp(0,129600);json!({"follower_mode":true,"follower_mode_duration":m})}
    "followersoff"=>json!({"follower_mode":false}),
    "subsonly"=>json!({"subscriber_mode":true}),
    "subsonlyoff"=>json!({"subscriber_mode":false}),
    "emoteonly"=>json!({"emote_mode":true}),
    _=>json!({"emote_mode":false}),
   };
   chat_settings(rt,p,&token,&moderator,body).await?;
   rt.log(&p.id,"twitch",&format!("Modo do chat ajustado pelo bot: {}",a.tw_op.as_str()),"success");Ok(())
  }
  "shoutout"=>{
   let (login,_)=resolve_login(rt,p,e,a)?;
   crate::platforms::shoutout(rt,p,&login).await?;
   rt.log(&p.id,"twitch",&format!("Destaque para @{login} pelo bot"),"success");Ok(())
  }
  "mention"=>{
   let (login,_)=resolve_login(rt,p,e,a)?;
   let msg=text.trim();
   if msg.is_empty(){return Err("Escreva a mensagem no conteúdo da ação".into())}
   rt.send(p,e,&format!("@{login} {msg}")).await?;
   Ok(())
  }
  _=>Err("Ação na Twitch: escolha uma operação válida".into())
 }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn a_limpeza_acha_nomes_na_fala(){
  assert_eq!(clean_word("Maria,"),"maria");
  assert_eq!(clean_word("@troll123"),"troll123");
  assert!(mention_ok("troll123"));
  assert!(!mention_ok("a"));
  assert!(bare_ok("maria"));
  assert!(!bare_ok("no"));
 }
 #[test] fn o_primeiro_numero_vira_duracao(){
  assert_eq!(first_number("timeout 300 por favor"),Some(300));
  assert_eq!(first_number("sem numero"),None);
 }
}
