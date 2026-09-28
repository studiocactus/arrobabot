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
/// !setgame e !settitle digitados no chat: devolve (op, resto da mensagem).
/// Falar ou digitar é escolha de quem usa: os dois caminhos executam a mesma
/// operação pela conta do bot.
pub fn builtin_command(message:&str)->Option<(&str,&str)>{
 let first=message.split_whitespace().next()?;
 let op=if first.eq_ignore_ascii_case("!setgame"){"game"}else if first.eq_ignore_ascii_case("!settitle"){"title"}else{return None};
 let rest=message[first.len()..].trim_start_matches(|c:char|c.is_whitespace());
 Some((op,rest))
}
/// Primeiro número da fala: "timeout 300" ou "slow 5".
fn first_number(message:&str)->Option<i64>{message.split_whitespace().filter_map(|w|w.parse::<i64>().ok()).find(|n|*n>=0)}
async fn user_id(rt:&Runtime,p:&Profile,login:&str)->Result<String,String>{crate::moderation::twitch_user_id(rt,p,login).await}
/// Resolve quem sofre a ação: @menção, nome visto no chat, alvo fixo do editor.
async fn resolve_login(rt:&Runtime,p:&Profile,e:&Event,a:&Action)->Result<(String,String),String>{
 let words:Vec<&str>=e.message.split_whitespace().collect();
 for w in &words{
  if w.starts_with('@')&&w.len()>1{
   let login=clean_word(&w[1..]);
   if mention_ok(&login){let id=user_id(rt,p,&login).await?;return Ok((login,id))}
  }
 }
 let found={
  let map=rt.seen_chatters.lock().unwrap();
  let mut hit=None;
  for w in words.iter().rev(){
   let name=clean_word(w);
   if bare_ok(&name){if let Some((login,_))=map.get(&name){hit=Some(login.clone());break}}
  }
  hit
 };
 if let Some(login)=found{let id=user_id(rt,p,&login).await?;return Ok((login,id))}
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
async fn channel_creds(rt:&Runtime,p:&Profile)->Result<String,String>{
 if p.platform!="twitch"||p.channel_id.is_empty(){return Err("A ação na Twitch exige perfil Twitch com a conta do canal autorizada".into())}
 crate::oauth::token(&rt.http,p,"channel").await.map_err(|_|"Autorize novamente a conta do canal em Perfis para conceder as permissões novas".to_string())
}
/// Categoria, título e VIP mexem no canal e executam com o token do canal
/// (a Twitch só aceita o dono do canal nesse endpoint). O resto executa
/// com o token do bot, que precisa ser moderador.
fn uses_channel_account(op:&str)->bool{matches!(op,"game"|"title"|"vip"|"unvip")}
/// Escopos que cada operação exige do token do bot.
fn required_scopes(op:&str)->&'static [&'static str]{
 match op{
  "game"|"title"=>&["channel:manage:broadcast"],
  "timeout"|"ban"|"unban"=>&["moderator:manage:banned_users"],
  "warn"=>&["moderator:manage:warnings"],
  "vip"|"unvip"=>&["channel:manage:vips"],
  "slow"|"slowoff"|"followers"|"followersoff"|"subsonly"|"subsonlyoff"|"emoteonly"|"emoteonlyoff"=>&["moderator:manage:chat_settings"],
  _=>&[],
 }
}
/// Lê os escopos reais do token e cobra os que faltam, para a reautorização
/// não ser tentativa e erro: o Histórico diz exatamente o que reautorizar.
async fn require_scopes(rt:&Runtime,p:&Profile,account:&str,op:&str,token:&str)->Result<(),String>{
 let need=required_scopes(op);
 let expected_id=if account=="do canal"{p.channel_id.as_str()}else{p.bot_id.as_str()};
 let res=rt.http.get("https://id.twitch.tv/oauth2/validate").header("Authorization",format!("OAuth {token}")).send().await.map_err(|_|"Não foi possível conferir a autorização".to_string())?;
 if !res.status().is_success(){return Err(format!("Autorize novamente a conta {account} em Perfis"))}
 let v:Value=res.json().await.map_err(|_|"Resposta de autorização inválida".to_string())?;
 let uid=v["user_id"].as_str().unwrap_or("");
 let login=v["login"].as_str().unwrap_or("?");
 if !uid.is_empty()&&uid!=expected_id{return Err(format!("O token guardado é da conta {login}, diferente da conta registrada no perfil. Entre como a conta certa no navegador e reautorize a conta {account} em Perfis."))}
 let token_client=v["client_id"].as_str().unwrap_or("");
 if !token_client.is_empty()&&token_client!=p.client_id{return Err(format!("O token foi emitido para outro Client ID (o campo Client ID do perfil mudou depois da autorização). Confira o Client ID e reautorize a conta {account} em Perfis."))}
 if need.is_empty(){return Ok(())}
 let have:Vec<&str>=v["scopes"].as_array().map(|a|a.iter().filter_map(|s|s.as_str()).collect()).unwrap_or_default();
 let missing:Vec<&str>=need.iter().copied().filter(|s|!have.contains(s)).collect();
 if missing.is_empty(){return Ok(())}
 Err(format!("O token da conta {account} não tem os escopos: {}. Reautorize a conta {account} em Perfis usando a versão atual do programa.",missing.join(", ")))
}
fn refused(what:&str,status:u16)->String{
 match status{
  401=>format!("{what} recusado (HTTP 401). A Twitch recusou o chamador: categoria, título e VIP usam a conta do canal; o resto usa a conta do bot, que precisa ser moderadora. Confira a conta autorizada em Perfis e o ID do canal."),
  403=>format!("{what} recusado: a conta usada não tem o papel exigido (bot precisa ser moderador do canal para moderação e modos)."),
  429=>format!("{what} recusado: a Twitch limitou os pedidos agora. Tente em instantes."),
  _=>format!("{what} recusado: HTTP {status}."),
 }
}
/// Lê o motivo que a Twitch devolveu no corpo, para o Histórico mostrar a
/// causa exata em vez de só o HTTP.
async fn twitch_err(res:reqwest::Response,what:&str)->String{
 let status=res.status().as_u16();
 let detail=res.text().await.unwrap_or_default();
 let detail=detail.trim().chars().take(200).collect::<String>();
 let base=refused(what,status);
 if detail.is_empty(){base}else{format!("{base} Resposta da Twitch: {detail}")}
}
/// Nome do jogo vira o ID que a API de canal exige. Devolve None quando a Twitch
/// não conhece o nome, para o chamador tentar outra grafia antes de desistir.
async fn lookup_game(rt:&Runtime,p:&Profile,token:&str,name:&str)->Result<Option<String>,String>{
 let res=rt.http.get("https://api.twitch.tv/helix/games").query(&[("name",name)]).header("Client-Id",&p.client_id).bearer_auth(token).send().await.map_err(|_|"Não foi possível consultar o jogo na Twitch".to_string())?;
 if !res.status().is_success(){return Err(twitch_err(res,"Categoria").await)}
 let v:Value=res.json().await.map_err(|_|"Resposta de jogo inválida".to_string())?;
 Ok(Some(v["data"][0]["id"].as_str().unwrap_or("").to_owned()).filter(|s|!s.is_empty()))
}
async fn game_id(rt:&Runtime,p:&Profile,token:&str,name:&str)->Result<String,String>{
 if let Some(id)=lookup_game(rt,p,token,name).await?{return Ok(id)}
 let titled=title_case(name);
 if titled!=name{if let Some(id)=lookup_game(rt,p,token,&titled).await?{return Ok(id)}}
 Err(format!("A Twitch não encontrou o jogo {name}. Confira o nome ou configure o jogo fixo no conteúdo da ação."))
}
/// Palavras de ligação descartadas das bordas ao extrair o assunto da fala.
const FILLERS:&[&str]=&["para","pra","pro","o","a","os","as","um","uma","de","do","da","dos","das","no","na","nos","nas","por","favor","porfavor","jogo","categoria","titulo","título","jogar","troca","trocar","muda","mudar","coloca","colocar","bota","botar","ponha","poe","põe","agora","ai","aí","meu","minha"];
/// Remove a primeira ocorrência de `agulha` em `palheiro` sem diferenciar
/// maiúsculas de minúsculas, mantendo a caixa original do restante.
fn remove_first_case_insensitive(palheiro:&str,agulha:&str)->String{
 let lower=palheiro.to_lowercase();
 let Some(pos)=lower.find(&agulha.to_lowercase()) else{return palheiro.to_owned()};
 let before=lower[..pos].chars().count();
 let len=agulha.chars().count();
 let chars:Vec<char>=palheiro.chars().collect();
 let mut out:String=chars[..before].iter().collect();
 out.push_str(&chars[before+len..].iter().collect::<String>());
 out
}
/// Extrai o assunto da fala removendo a variação do gatilho que casou e as
/// palavras de ligação das bordas: "troca o jogo para minecraft" vira "minecraft".
/// A pontuação das bordas sai ("Minecraft." vira "Minecraft") mas a caixa
/// original é mantida para a busca na Twitch e para o título.
fn derive_subject(message:&str,pattern:&str)->String{
 let mut alts:Vec<String>=pattern.split(',').map(str::trim).filter(|s|!s.is_empty()).map(|s|s.to_owned()).collect();
 alts.sort_by_key(|s|std::cmp::Reverse(s.len()));
 let mut rest=message.to_owned();
 for alt in &alts{
  let next=remove_first_case_insensitive(&rest,alt);
  if next!=rest{rest=next;break}
 }
 let words:Vec<String>=rest.split_whitespace().map(|w|w.trim_matches(|c:char|!(c.is_alphanumeric()||c=='_'||c=='-')).to_owned()).filter(|w|!w.is_empty()).collect();
 let mut start=0;let mut end=words.len();
 while start<end&&FILLERS.contains(&words[start].to_lowercase().as_str()){start+=1}
 while end>start&&FILLERS.contains(&words[end-1].to_lowercase().as_str()){end-=1}
 words[start..end].join(" ").trim().to_owned()
}
/// Tira a palavra de ativação da fala ("Arroba. Troca o jogo..." vira
/// "Troca o jogo..."), para ela não vazar para o nome do jogo ou o alvo.
fn strip_activation(message:&str,activation:&str)->String{
 let mut stops:Vec<String>=activation.split(',').map(str::trim).filter(|s|!s.is_empty()).map(|s|s.to_owned()).collect();
 if stops.is_empty(){return message.to_owned()}
 stops.sort_by_key(|s|std::cmp::Reverse(s.len()));
 let mut rest=message.to_owned();
 for s in &stops{rest=remove_first_case_insensitive(&rest,s);}
 rest.trim_matches(|c:char|c.is_whitespace()|| matches!(c,'.'|','|'!'|'?'|':'|';'|'-')).to_owned().split_whitespace().collect::<Vec<_>>().join(" ")
}
fn title_case(s:&str)->String{
 s.split_whitespace().map(|w|{let mut c=w.chars();match c.next(){None=>String::new(),Some(f)=>f.to_uppercase().collect::<String>()+c.as_str()}}).collect::<Vec<_>>().join(" ")
}
async fn modify_channel(rt:&Runtime,p:&Profile,token:&str,body:Value,what:&str)->Result<(),String>{
 let res=rt.http.patch("https://api.twitch.tv/helix/channels").query(&[("broadcaster_id",p.channel_id.as_str())]).header("Client-Id",&p.client_id).bearer_auth(token).json(&body).send().await.map_err(|_|format!("Não foi possível alterar {what}"))?;
 if !res.status().is_success(){return Err(twitch_err(res,what).await)}
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
 if !res.status().is_success(){return Err(twitch_err(res,"Moderação").await)}
 Ok(())
}
async fn chat_settings(rt:&Runtime,p:&Profile,token:&str,moderator:&str,body:Value)->Result<(),String>{
 let res=rt.http.patch("https://api.twitch.tv/helix/chat/settings").query(&[("broadcaster_id",p.channel_id.as_str()),("moderator_id",moderator)]).header("Client-Id",&p.client_id).bearer_auth(token).json(&body).send().await.map_err(|_|"Não foi possível ajustar o chat".to_string())?;
 if !res.status().is_success(){return Err(twitch_err(res,"Modo do chat").await)}
 Ok(())
}
async fn vip(rt:&Runtime,p:&Profile,token:&str,add:bool,user_id:&str)->Result<(),String>{
 let query=[("broadcaster_id",p.channel_id.as_str()),("user_id",user_id)];
 let res=if add{rt.http.post("https://api.twitch.tv/helix/channels/vips").query(&query).header("Client-Id",&p.client_id).bearer_auth(token).send().await}
  else{rt.http.delete("https://api.twitch.tv/helix/channels/vips").query(&query).header("Client-Id",&p.client_id).bearer_auth(token).send().await};
 let res=res.map_err(|_|"Falha ao alterar VIP na Twitch".to_string())?;
 if res.status().as_u16()==409{return Err("Limite de VIPs do canal atingido".into())}
 if !res.status().is_success(){return Err(twitch_err(res,"VIP").await)}
 Ok(())
}
/// Executa a operação da ação "twitch" com a conta do bot. `text` já vem renderizado.
/// Devolve a mensagem de confirmação para o chat e o Histórico.
pub async fn run(rt:&Arc<Runtime>,p:&Profile,e:&Event,a:&Action,text:&str,trigger:&str)->Result<String,String>{
 if p.platform!="twitch"{return Err("A ação na Twitch só vale em perfil Twitch".into())}
 if crate::discord::from_discord(e).is_some(){return Err("A ação na Twitch não vale em mensagens vindas do Discord".into())}
 let (token,moderator)=if uses_channel_account(&a.tw_op){(channel_creds(rt,p).await?,String::new())}else{bot_creds(rt,p).await?};
 let account=if uses_channel_account(&a.tw_op){"do canal"}else{"do bot"};
 require_scopes(rt,p,account,&a.tw_op,&token).await?;
 match a.tw_op.as_str(){
  "game"=>{
   // Conteúdo vazio usa o que foi falado: tira a ativação, remove o gatilho
   // e as ligações.
   let mut name=text.trim().to_owned();
   if name.is_empty(){
    let cfg=crate::listen::Config::read(&rt.db,&p.id);
    name=derive_subject(&strip_activation(&e.message,&cfg.activation),trigger);
   }
   if name.is_empty(){return Err("Não entendi o nome do jogo. Fale como 'troca o jogo para Minecraft' ou configure o jogo fixo no conteúdo da ação.".into())}
   let id=game_id(rt,p,&token,&name).await?;
   modify_channel(rt,p,&token,json!({"game_id":id}),"Categoria").await?;
   let done=format!("Categoria alterada para {name} pelo bot");
   rt.log(&p.id,"twitch",&done,"success");Ok(done)
  }
  "title"=>{
   let mut title:String=text.trim().to_owned();
   if title.is_empty(){
    let cfg=crate::listen::Config::read(&rt.db,&p.id);
    title=derive_subject(&strip_activation(&e.message,&cfg.activation),trigger);
   }
   if title.is_empty(){return Err("Não entendi o novo título. Fale como 'troca o título para Ranked com viewers' ou configure o título fixo no conteúdo da ação.".into())}
   let title:String=title.chars().take(140).collect();
   modify_channel(rt,p,&token,json!({"title":title}),"Título").await?;
   let done=format!("Título alterado para {title} pelo bot");
   rt.log(&p.id,"twitch",&done,"success");Ok(done)
  }
  "timeout"|"ban"|"unban"|"warn"=>{
   let (login,user_id)=resolve_login(rt,p,e,a).await?;
   guard_streamer(p,&login,&user_id)?;
   let reason=if text.trim().is_empty(){"Regras da comunidade".to_string()}else{text.trim().chars().take(500).collect::<String>()};
   let seconds=if a.tw_op=="timeout"{first_number(&e.message).or(if a.value>0{Some(a.value)}else{None}).unwrap_or(60).clamp(1,1209600) as u64}else{0};
   punish_as(rt,p,&token,&moderator,&a.tw_op,&user_id,&reason,seconds).await?;
   let done=format!("{} aplicado a @{login} pelo bot",a.tw_op.as_str());
   rt.log(&p.id,"twitch",&done,"success");Ok(done)
  }
  "vip"|"unvip"=>{
   let (login,user_id)=resolve_login(rt,p,e,a).await?;
   guard_streamer(p,&login,&user_id)?;
   vip(rt,p,&token,a.tw_op=="vip",&user_id).await?;
   let done=format!("{} aplicado a @{login} pelo bot",if a.tw_op=="vip"{"VIP"}else{"remoção de VIP"});
   rt.log(&p.id,"twitch",&done,"success");Ok(done)
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
   let done=format!("Modo do chat ajustado pelo bot: {}",a.tw_op.as_str());
   rt.log(&p.id,"twitch",&done,"success");Ok(done)
  }
  "shoutout"=>{
   let (login,_)=resolve_login(rt,p,e,a).await?;
   crate::platforms::shoutout(rt,p,&login).await?;
   let done=format!("Destaque para @{login} pelo bot");
   rt.log(&p.id,"twitch",&done,"success");Ok(done)
  }
  "mention"=>{
   let (login,_)=resolve_login(rt,p,e,a).await?;
   let msg=text.trim();
   if msg.is_empty(){return Err("Escreva a mensagem no conteúdo da ação".into())}
   rt.send(p,e,&format!("@{login} {msg}")).await?;
   Ok(format!("Resposta enviada para @{login}"))
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
 #[test] fn categoria_titulo_e_vip_usam_a_conta_do_canal(){
  for op in ["game","title","vip","unvip"]{assert!(uses_channel_account(op),"{op} mexe no canal")}
  for op in ["timeout","ban","unban","warn","slow","slowoff","followers","shoutout","mention"]{assert!(!uses_channel_account(op),"{op} usa o bot")}
 }
 #[test] fn cada_operacao_cobra_seu_escopo(){
  assert_eq!(required_scopes("game"),&["channel:manage:broadcast"]);
  assert_eq!(required_scopes("title"),&["channel:manage:broadcast"]);
  assert_eq!(required_scopes("ban"),&["moderator:manage:banned_users"]);
  assert_eq!(required_scopes("warn"),&["moderator:manage:warnings"]);
  assert_eq!(required_scopes("vip"),&["channel:manage:vips"]);
  assert_eq!(required_scopes("slow"),&["moderator:manage:chat_settings"]);
  assert!(required_scopes("mention").is_empty());
  assert!(required_scopes("shoutout").is_empty());
 }
 #[test] fn comandos_nativos_separam_operacao_e_texto(){
  assert_eq!(builtin_command("!setgame Valorant"),Some(("game","Valorant")));
  assert_eq!(builtin_command("!SETTITLE Ranked hoje"),Some(("title","Ranked hoje")));
  assert_eq!(builtin_command("!setgame"),Some(("game","")));
  assert_eq!(builtin_command("!oi"),None);
 }
 #[test] fn o_primeiro_numero_vira_duracao(){
  assert_eq!(first_number("timeout 300 por favor"),Some(300));
  assert_eq!(first_number("sem numero"),None);
 }
 #[test] fn o_assunto_sai_da_fala_sem_o_gatilho(){
  assert_eq!(derive_subject("troca o jogo para minecraft","troca o jogo, muda o jogo"),"minecraft");
  assert_eq!(derive_subject("muda o título para ranked com viewers","muda o título"),"ranked com viewers");
  assert_eq!(derive_subject("minecraft",""),"minecraft");
  assert_eq!(derive_subject("troca o jogo","troca o jogo"),"");
  assert_eq!(title_case("counter strike"),"Counter Strike");
 }
 #[test] fn a_ativacao_e_a_pontuacao_nao_vazam_para_o_nome(){
  assert_eq!(strip_activation("Arroba. Troca o jogo para Minecraft.","Arroba"),"Troca o jogo para Minecraft");
  assert_eq!(derive_subject("Troca o jogo para Minecraft.","troca o jogo, muda o jogo"),"Minecraft");
  assert_eq!(
   derive_subject(&strip_activation("Arroba. Troca o jogo para Minecraft.","Arroba"),"troca o jogo, muda o jogo"),
   "Minecraft");
 }
}
