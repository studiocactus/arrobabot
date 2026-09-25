use crate::{engine::Runtime,model::*,moderation};
use serde_json::{json,Value};
use std::{sync::Arc,time::{Duration,Instant}};
use futures_util::{StreamExt,SinkExt};
use tokio_tungstenite::{connect_async,tungstenite::Message};

pub const API:&str="https://discord.com/api/v10";
pub const MODULE:&str="discordBot";

// Gateway intents. Guilds, moderation, reactions and guild messages are always on;
// members and message content are privileged and can be disabled per profile.
const I_GUILDS:i64=1<<0;
const I_MEMBERS:i64=1<<1;
const I_MODERATION:i64=1<<2;
const I_REACTIONS:i64=1<<6;
const I_MESSAGES:i64=1<<9;
const I_CONTENT:i64=1<<15;

// Permission bits used to build the bot invite.
const P_KICK:u64=1<<1;
const P_BAN:u64=1<<2;
const P_MANAGE_CHANNELS:u64=1<<4;
const P_VIEW_AUDIT:u64=1<<7;
const P_ADD_REACTIONS:u64=1<<6;
const P_VIEW:u64=1<<10;
const P_SEND:u64=1<<11;
const P_MANAGE_MESSAGES:u64=1<<13;
const P_EMBED:u64=1<<14;
const P_HISTORY:u64=1<<16;
const P_MANAGE_ROLES:u64=1<<28;
const P_MANAGE_WEBHOOKS:u64=1<<29;
const P_MODERATE:u64=1<<40;
// Administrator bit, used only to recognize moderators that already own it.
pub const P_ADMINISTRATOR:u64=1<<3;
pub const P_MANAGE_MESSAGES_BIT:u64=1<<13;

#[derive(Clone,Default)]
pub struct Cache{pub guilds:Vec<Value>,pub channels:Vec<Value>,pub roles:Vec<Value>,pub guild_id:String,pub member_count:u64}

enum Fail{Fatal(String),Transient(String)}
enum Next{Resume,Fresh}

#[derive(Default)]
struct State{seq:u64,session:String,resume:String,user_id:String,app_id:String}

pub fn config(rt:&Runtime,p:&str)->Value{rt.db.module(p,MODULE)}
pub fn intents(cfg:&Value)->i64{
 let mut i=I_GUILDS|I_MODERATION|I_REACTIONS|I_MESSAGES;
 if cfg["intents"]["members"].as_bool().unwrap_or(true){i|=I_MEMBERS}
 if cfg["intents"]["content"].as_bool().unwrap_or(true){i|=I_CONTENT}
 i
}
pub fn permissions()->u64{
 P_KICK|P_BAN|P_MANAGE_CHANNELS|P_VIEW_AUDIT|P_ADD_REACTIONS|P_VIEW|P_SEND|P_MANAGE_MESSAGES|P_EMBED|P_HISTORY|P_MANAGE_ROLES|P_MANAGE_WEBHOOKS|P_MODERATE
}
pub fn invite_url(app:&str,guild:&str)->String{
 let mut url=format!("https://discord.com/oauth2/authorize?client_id={}&scope=bot%20applications.commands&permissions={}",app,permissions());
 if !guild.is_empty(){url.push_str(&format!("&guild_id={guild}"))}
 url
}
pub fn cache(rt:&Runtime,p:&str)->Cache{rt.discord_cache.lock().unwrap().get(p).cloned().unwrap_or_default()}
pub fn with_cache<F:FnOnce(&mut Cache)>(rt:&Runtime,p:&str,f:F){let mut m=rt.discord_cache.lock().unwrap();let c=m.entry(p.to_owned()).or_default();f(c);}
pub fn clear_cache(rt:&Runtime,p:&str){rt.discord_cache.lock().unwrap().remove(p);}
pub fn status(rt:&Runtime,p:&str,s:&str){rt.discord_status.lock().unwrap().insert(p.into(),s.into());rt.emit("discord-connection",json!({"profileId":p,"status":s}));}
pub fn online(rt:&Runtime,p:&str)->bool{rt.discord_status.lock().unwrap().get(p).map(String::as_str)==Some("online")}
pub fn truncate(s:&str,n:usize)->String{s.chars().take(n).collect()}
pub fn from_discord(e:&Event)->Option<String>{
 if e.data["source"].as_str()!=Some("discord"){return None}
 Some(e.data["channelId"].as_str().unwrap_or("").to_owned())
}

// ---------------------------------------------------------------- REST

fn discord_error(status:u16,code:i64,msg:&str)->String{
 match code{
 50013=>return "Permissão negada no Discord. Confira os cargos e permissões do bot no servidor.".into(),
 50007=>return "O Discord recusou o envio neste canal ou para este membro.".into(),
 30001|10004=>return "Servidor do Discord não encontrado pelo bot.".into(),
 50001=>return "O bot não enxerga esse canal. Verifique as permissões de visibilidade.".into(),
 _=>{}
 }
 match status{
 401=>format!("Token do Discord inválido ou expirado. {msg}"),
 403=>format!("O Discord negou a permissão necessária. {msg}"),
 404=>format!("Recurso do Discord não encontrado. {msg}"),
 400=>format!("Pedido recusado pelo Discord: {msg}"),
 _=>format!("Discord respondeu HTTP {status}: {msg}")
 }
}
pub async fn rest(rt:&Runtime,p:&Profile,method:&str,path:&str,body:Option<Value>)->Result<Value,String>{
 let token=crate::secrets::get(&p.id,"discord_token")?;
 let url=format!("{API}{path}");
 for _ in 0..3{
  let req=match method{"GET"=>rt.http.get(&url),"POST"=>rt.http.post(&url),"PATCH"=>rt.http.patch(&url),"PUT"=>rt.http.put(&url),"DELETE"=>rt.http.delete(&url),_=>return Err("Método inválido".into())};
  let req=req.header("Authorization",format!("Bot {token}"));
  let req=if let Some(b)=body.clone(){req.json(&b)}else{req};
  let res=req.send().await.map_err(|_|"Não foi possível alcançar a API do Discord".to_string())?;
  let code=res.status().as_u16();let text=res.text().await.unwrap_or_default();
  if code==429{
   let v:Value=serde_json::from_str(&text).unwrap_or(Value::Null);
   let wait=v["retry_after"].as_f64().unwrap_or(1.0).clamp(0.1,8.0);
   tokio::time::sleep(Duration::from_secs_f64(wait)).await;continue;
  }
  if !(200..300).contains(&code){
   let v:Value=serde_json::from_str(&text).unwrap_or(Value::Null);
   return Err(discord_error(code,v["code"].as_i64().unwrap_or(0),v["message"].as_str().unwrap_or("")));
  }
  if text.trim().is_empty(){return Ok(Value::Null)}
  return serde_json::from_str(&text).map_err(|_|"Resposta inválida do Discord".into());
 }
 Err("O Discord aplicou limite de requisições. Aguarde alguns segundos e tente novamente.".into())
}
pub async fn post(rt:&Runtime,p:&Profile,channel:&str,text:&str)->Result<Value,String>{
 if channel.trim().is_empty(){return Err("Canal do Discord não configurado".into())}
 let body=json!({"content":truncate(text,2000),"allowed_mentions":{"parse":[]}});
 rest(rt,p,"POST",&format!("/channels/{channel}/messages"),Some(body)).await
}
pub async fn post_embed(rt:&Runtime,p:&Profile,channel:&str,title:&str,description:&str,color:i64,fields:Vec<Value>)->Result<Value,String>{
 if channel.trim().is_empty(){return Err("Canal do Discord não configurado".into())}
 let embed=json!({"title":truncate(title,256),"description":truncate(description,4096),"color":color.clamp(0,0xFFFFFF),"fields":fields,"timestamp":chrono::Utc::now().to_rfc3339()});
 let body=json!({"embeds":[embed],"allowed_mentions":{"parse":[]}});
 rest(rt,p,"POST",&format!("/channels/{channel}/messages"),Some(body)).await
}
pub async fn identity(rt:&Runtime,p:&Profile)->Result<Value,String>{
 let me=rest(rt,p,"GET","/users/@me",None).await?;
 Ok(json!({"id":me["id"].as_str().unwrap_or("").to_owned(),"name":me["global_name"].as_str().or(me["username"].as_str()).unwrap_or("").to_owned(),"avatar":me["avatar"].as_str().unwrap_or("").to_owned()}))
}

// ---------------------------------------------------------------- Gateway

pub async fn run(rt:Arc<Runtime>,p:Profile)->Result<(),String>{
 let token=crate::secrets::get(&p.id,"discord_token").map_err(|e|e)?;
 let mut st=State::default();
 let mut delay=2u64;
 loop {
  let url=if st.resume.is_empty(){"wss://gateway.discord.gg/?v=10&encoding=json".to_owned()}else{format!("{}/?v=10&encoding=json",st.resume.trim_end_matches('/'))};
  match session(&rt,&p,&token,&mut st,&url).await{
  Ok(Next::Resume)=>{delay=2},
  Ok(Next::Fresh)=>{st=State::default();delay=2},
  Err(Fail::Fatal(msg))=>{status(&rt,&p.id,"offline");return Err(msg)},
  Err(Fail::Transient(msg))=>{
   status(&rt,&p.id,"reconnecting");rt.log(&p.id,"discord",&msg,"error");
   tokio::time::sleep(Duration::from_secs(delay)).await;delay=(delay*2).min(60);
  }
  }
 }
}
async fn session(rt:&Arc<Runtime>,p:&Profile,token:&str,st:&mut State,url:&str)->Result<Next,Fail>{
 let (mut ws,_)=connect_async(url).await.map_err(|_|Fail::Transient("Não foi possível conectar ao Discord".into()))?;
 let mut interval=Duration::from_millis(41250);
 let mut next_heartbeat=Instant::now()+interval;
 let mut acked=true;
 loop {
  let wait=next_heartbeat.saturating_duration_since(Instant::now());
  let step=tokio::time::timeout(wait,ws.next()).await;
  if step.is_err(){
   if !acked{return Err(Fail::Transient("O Discord não confirmou o heartbeat".into()))}
   ws.send(Message::Text(heartbeat(st).to_string().into())).await.map_err(|_|Fail::Transient("Falha ao manter a conexão com o Discord".into()))?;
   next_heartbeat=Instant::now()+interval;continue;
  }
  let msg=step.map_err(|_|Fail::Transient("Conexão com o Discord interrompida".into()))?.ok_or_else(||Fail::Transient("Conexão com o Discord encerrada".into()))?.map_err(|_|Fail::Transient("Conexão com o Discord interrompida".into()))?;
  match msg{
  Message::Ping(v)=>{ws.send(Message::Pong(v)).await.map_err(|_|Fail::Transient("Falha de conexão".into()))?;},
  Message::Close(cf)=>{
   let code:u16=cf.as_ref().map(|f|f.code.into()).unwrap_or(1006);
   return Err(match code{
   4004=>Fail::Fatal("Token do Discord inválido. Gere um novo token no portal do desenvolvedor.".into()),
   4013=>Fail::Fatal("Intents inválidos. Revise as intenções configuradas para o perfil.".into()),
   4014=>Fail::Fatal("Privileged Intents desativados. Ative \"Server Members Intent\" e \"Message Content Intent\" no portal do Discord.".into()),
   _=>Fail::Transient(format!("Discord encerrou a conexão (código {code})"))
   });
  },
  Message::Text(text)=>{
   let v:Value=serde_json::from_str(&text).map_err(|_|Fail::Transient("Evento Discord inválido".into()))?;
   match v["op"].as_i64().unwrap_or(-1){
   10=>{
    interval=Duration::from_millis(v["d"]["heartbeat_interval"].as_u64().unwrap_or(41250).max(1000));
    next_heartbeat=Instant::now()+interval;
    let payload=if st.session.is_empty(){identify(&config(rt,&p.id),token)}else{resume(token,st)};
    ws.send(Message::Text(payload.to_string().into())).await.map_err(|_|Fail::Transient("Não foi possível se identificar no Discord".into()))?;
   },
   11=>{acked=true},
   1=>{ws.send(Message::Text(heartbeat(st).to_string().into())).await.map_err(|_|Fail::Transient("Falha de conexão".into()))?;},
   0=>{
    if let Some(s)=v["s"].as_u64(){st.seq=s}
    let kind=v["t"].as_str().unwrap_or("").to_owned();
    let data=v["d"].clone();
    dispatch(rt,p,st,&kind,&data).await;
   },
   7=>return Ok(Next::Resume),
   9=>{st.session.clear();return Ok(Next::Fresh)},
   _=>{}
   }
  },
  _=>{}
  }
 }
}
fn heartbeat(st:&State)->Value{json!({"op":1,"d":if st.seq==0{Value::Null}else{json!(st.seq)}})}
fn identify(cfg:&Value,token:&str)->Value{
 json!({"op":2,"d":{"token":token,"intents":intents(cfg),"properties":{"os":std::env::consts::OS,"browser":"BotLive","device":"BotLive"}}})
}
fn resume(token:&str,st:&State)->Value{
 json!({"op":6,"d":{"token":token,"session_id":st.session,"seq":st.seq}})
}
// ---------------------------------------------------------------- Dispatch

async fn dispatch(rt:&Arc<Runtime>,p:&Profile,st:&mut State,kind:&str,data:&Value){
 let cfg=config(rt,&p.id);
 match kind{
 "READY"=>{
  st.session=data["session_id"].as_str().unwrap_or("").to_owned();
  st.resume=data["resume_gateway_url"].as_str().unwrap_or("").to_owned();
  st.user_id=data["user"]["id"].as_str().unwrap_or("").to_owned();
  st.app_id=st.user_id.clone();
  status(rt,&p.id,"online");
  rt.log(&p.id,"discord","Bot do Discord conectado","success");
  rt.emit("discord-connection",json!({"profileId":p.id,"status":"online","identity":json!({"id":st.user_id,"name":data["user"]["username"]})}));
 },
 "RESUMED"=>{status(rt,&p.id,"online");rt.log(&p.id,"discord","Sessão do Discord retomada","info")},
 "GUILD_CREATE"=>on_guild(rt,p,data),
 "MESSAGE_CREATE"=>on_message(rt,p,st,data).await,
 "GUILD_MEMBER_ADD"=>on_member_join(rt,p,data),
 "GUILD_MEMBER_REMOVE"=>on_member_leave(rt,p,data),
 "GUILD_BAN_ADD"|"GUILD_BAN_REMOVE"=>on_ban(rt,p,kind,data),
 "INTERACTION_CREATE"=>crate::discord_engage::interaction(rt,p,data).await,
 "MESSAGE_REACTION_ADD"=>crate::discord_engage::reaction(rt,p,&cfg,data),
 _=>{}
 }
}
fn on_guild(rt:&Arc<Runtime>,p:&Profile,d:&Value){
 let cfg=config(rt,&p.id);
 let id=d["id"].as_str().unwrap_or("").to_owned();
 if id.is_empty(){return}
 let target=cfg["guildId"].as_str().unwrap_or("");
 let entry=json!({"id":id,"name":d["name"].as_str().unwrap_or("Servidor"),"icon":d["icon"].as_str().unwrap_or("")});
 with_cache(rt,&p.id,|c|{
  c.guilds.retain(|g|g["id"].as_str()!=Some(id.as_str()));
  c.guilds.push(entry);
  if !target.is_empty()&&target==id||c.guild_id.is_empty(){
   c.guild_id=id.clone();
   c.channels=d["channels"].as_array().cloned().unwrap_or_default();
   c.roles=d["roles"].as_array().cloned().unwrap_or_default();
   c.member_count=d["member_count"].as_u64().unwrap_or(c.member_count);
  }
 });
 if !cfg["counterChannelId"].as_str().unwrap_or("").is_empty()&&cfg["guildId"].as_str()==Some(id.as_str()){
  let rt2=rt.clone();let p2=p.clone();let cfg2=cfg.clone();
  tokio::spawn(async move{update_counter(&rt2,&p2,&cfg2,0).await});
 }
}
fn map_role(cfg:&Value,member:&Value)->String{
 let roles=member["roles"].as_array().cloned().unwrap_or_default();
 let map=&cfg["roleMap"];
 let has=|want:&str|!want.is_empty()&&roles.iter().any(|r|r.as_str()==Some(want));
 if has(map["moderator"].as_str().unwrap_or("")){return "moderator".into()}
 if has(map["subscriber"].as_str().unwrap_or("")){return "subscriber".into()}
 let perms=member["permissions"].as_str().and_then(|s|s.parse::<u64>().ok()).unwrap_or(0);
 if perms&P_ADMINISTRATOR!=0||perms&P_MANAGE_MESSAGES_BIT!=0{return "moderator".into()}
 "everyone".into()
}
async fn on_message(rt:&Arc<Runtime>,p:&Profile,st:&State,d:&Value){
 let cfg=config(rt,&p.id);
 if cfg["enabled"]!=true{return}
 let author=&d["author"];
 if author["bot"]==true||d["webhook_id"].is_string(){return}
 let uid=author["id"].as_str().unwrap_or("");
 if uid.is_empty()||uid==st.user_id{return}
 let channel=d["channel_id"].as_str().unwrap_or("");
 let guild=d["guild_id"].as_str().unwrap_or("");
 let wanted=cfg["guildId"].as_str().unwrap_or("");
 if !wanted.is_empty()&&guild!=wanted{return}
 let content=d["content"].as_str().unwrap_or("").to_owned();
 let member=&d["member"];
 let role=map_role(&cfg,member);
 let user=member["nick"].as_str().or_else(||author["global_name"].as_str()).or_else(||author["username"].as_str()).unwrap_or("espectador").to_owned();
 // Auto-moderation reuses the same blocklist, link and spam rules as Twitch.
 if cfg["automod"]["enabled"]==true {
 if let Some(reason)=moderation::reasons(rt,p,&content,uid,&role) {
  if let Err(err)=crate::discord_admin::delete_message(rt,p,channel,d["id"].as_str().unwrap_or("")).await{
   rt.log(&p.id,"discord",&err,"error");
  } else {
   rt.log(&p.id,"moderation",&format!("Discord · mensagem de {user} removida ({reason})"),"info");
   crate::discord_admin::audit(rt,&p.id,json!({"action":"clear","target":uid,"targetName":user,"reason":reason,"undoable":false,"platforms":["discord"]}));
  }
  return;
 }
 }
 crate::discord_engage::on_chat(rt,p,&cfg,uid,&user,&content);
 // Mirror: what is said on Discord is forwarded to the Twitch chat.
 if cfg["mirror"]["enabled"]==true&&cfg["mirror"]["toTwitch"]==true&&cfg["mirror"]["channelId"].as_str()==Some(channel)&&!content.is_empty(){
  let text=format!("{user}: {content}");
  if let Err(err)=crate::platforms::send(rt,p,&text).await{rt.log(&p.id,"discord",&err,"error")}
 }
 let event=Event{
  id:d["id"].as_str().unwrap_or("").into(),profile_id:p.id.clone(),kind:"chat".into(),
  user,user_id:uid.into(),role,message:content,
  data:json!({"source":"discord","channelId":channel,"guildId":guild,"discord":d}),simulated:false
 };
 if let Err(err)=rt.submit(event).await{rt.log(&p.id,"discord",&err,"error")}
}
/// `#membros-1284`: Discord only accepts lowercase letters, digits and dashes.
fn counter_name(label:&str,count:u64)->String{
 let mut base=String::new();
 for c in label.to_lowercase().chars(){
  if c.is_ascii_alphanumeric(){base.push(c)}else if !base.ends_with('-'){base.push('-')}
 }
 let base=base.trim_matches('-');
 let base=if base.is_empty(){"membros"}else{base};
 format!("{base}-{count}")
}
/// Keeps the member counter channel renamed to the live number of members.
async fn update_counter(rt:&Arc<Runtime>,p:&Profile,cfg:&Value,delta:i64){
 let channel=cfg["counterChannelId"].as_str().unwrap_or("");
 if channel.is_empty(){return}
 let label=cfg["counterLabel"].as_str().unwrap_or("membros").to_owned();
 let guild=cfg["guildId"].as_str().unwrap_or("").to_owned();
 let mut count=cache(rt,&p.id).member_count as i64;
 if count<=0&&!guild.is_empty(){
  if let Ok(v)=rest(rt,p,"GET",&format!("/guilds/{guild}?with_counts=true"),None).await{count=v["member_count"].as_i64().unwrap_or(0)}
 }
 count=(count+delta).max(0);
 if count<=0{return}
 with_cache(rt,&p.id,|c|c.member_count=count as u64);
 if let Err(err)=rest(rt,p,"PATCH",&format!("/channels/{channel}"),Some(json!({"name":counter_name(&label,count as u64)}))).await{
  rt.log(&p.id,"discord",&err,"error");
 }
}
fn schedule_counter(rt:&Arc<Runtime>,p:&Profile,cfg:&Value,delta:i64){
 if cfg["counterChannelId"].as_str().unwrap_or("").is_empty(){return}
 let rt2=rt.clone();let p2=p.clone();let cfg2=cfg.clone();
 tokio::spawn(async move{update_counter(&rt2,&p2,&cfg2,delta).await});
}
fn on_member_join(rt:&Arc<Runtime>,p:&Profile,d:&Value){
 let cfg=config(rt,&p.id);if cfg["enabled"]!=true{return}
 let uid=d["id"].as_str().unwrap_or("");if uid.is_empty(){return}
 let name=d["user"]["global_name"].as_str().or(d["user"]["username"].as_str()).unwrap_or(uid).to_owned();
 let guild=cfg["guildId"].as_str().unwrap_or("");
 if !guild.is_empty()&&d["guild_id"].as_str()!=Some(guild){return}
 let log_channel=cfg["logChannelId"].as_str().unwrap_or("");
 let autorole=cfg["autoroleId"].as_str().unwrap_or("");
 if !autorole.is_empty(){
  let path=format!("/guilds/{guild}/members/{uid}/roles/{autorole}");
  let rt2=rt.clone();let p2=p.clone();
  tokio::spawn(async move{if let Err(err)=rest(&rt2,&p2,"PUT",&path,None).await{rt2.log(&p2.id,"discord",&err,"error")}});
 }
 if cfg["welcome"]["enabled"]==true{
  let channel=cfg["welcome"]["channelId"].as_str().unwrap_or("");
  let template=cfg["welcome"]["text"].as_str().unwrap_or("");
  let text=if template.trim().is_empty(){format!("Bem-vindo(a) ao servidor, {name}!")}else{template.replace("{user}",&format!("<@{uid}>")).replace("{name}",&name)};
  let rt2=rt.clone();let p2=p.clone();let ch=channel.to_owned();
  tokio::spawn(async move{if let Err(err)=post(&rt2,&p2,&ch,&text).await{rt2.log(&p2.id,"discord",&err,"error")}});
 }
 if !log_channel.is_empty(){
  let rt2=rt.clone();let p2=p.clone();let ch=log_channel.to_owned();let n=name.clone();
  tokio::spawn(async move{let _=post_embed(&rt2,&p2,&ch,"Novo membro",&n,0x57F287,vec![]).await;});
 }
 schedule_counter(rt,p,&cfg,1);
 rt.log(&p.id,"discord",&format!("{name} entrou no servidor"),"info");
}
fn on_member_leave(rt:&Arc<Runtime>,p:&Profile,d:&Value){
 let cfg=config(rt,&p.id);if cfg["enabled"]!=true{return}
 let uid=d["id"].as_str().unwrap_or("");if uid.is_empty(){return}
 let name=d["user"]["global_name"].as_str().or(d["user"]["username"].as_str()).unwrap_or(uid).to_owned();
 if cfg["goodbye"]["enabled"]==true{
  let channel=cfg["goodbye"]["channelId"].as_str().unwrap_or("");
  let template=cfg["goodbye"]["text"].as_str().unwrap_or("");
  let text=if template.trim().is_empty(){format!("Até logo, {name}.")}else{template.replace("{user}",&format!("<@{uid}>")).replace("{name}",&name)};
  let rt2=rt.clone();let p2=p.clone();let ch=channel.to_owned();
  tokio::spawn(async move{if let Err(err)=post(&rt2,&p2,&ch,&text).await{rt2.log(&p2.id,"discord",&err,"error")}});
 }
 let log_channel=cfg["logChannelId"].as_str().unwrap_or("");
 if !log_channel.is_empty(){
  let rt2=rt.clone();let p2=p.clone();let ch=log_channel.to_owned();let n=name.clone();
  tokio::spawn(async move{let _=post_embed(&rt2,&p2,&ch,"Saiu do servidor",&n,0xED4245,vec![]).await;});
 }
 schedule_counter(rt,p,&cfg,-1);
 rt.log(&p.id,"discord",&format!("{name} saiu do servidor"),"info");
}
fn on_ban(rt:&Arc<Runtime>,p:&Profile,kind:&str,d:&Value){
 let cfg=config(rt,&p.id);if cfg["enabled"]!=true{return}
 let channel=cfg["logChannelId"].as_str().unwrap_or("");if channel.is_empty(){return}
 let uid=d["user"]["id"].as_str().unwrap_or("");
 let name=d["user"]["global_name"].as_str().or(d["user"]["username"].as_str()).unwrap_or(uid);
 let (title,color)=(if kind=="GUILD_BAN_ADD"{"Membro banido"}else{"Banimento removido"},if kind=="GUILD_BAN_ADD"{0xED4245}else{0x57F287});
 let rt2=rt.clone();let p2=p.clone();let ch=channel.to_owned();
 let title=title.to_owned();let name=name.to_owned();
 tokio::spawn(async move{let _=post_embed(&rt2,&p2,&ch,&title,&name,color,vec![]).await;});
}

// ---------------------------------------------------------------- Twitch → Discord

pub fn notify(rt:&Arc<Runtime>,p:&Profile,kind:&str,e:&Event){
 let cfg=config(rt,&p.id);
 if cfg["enabled"]!=true{return}
 let channel=cfg["notify"]["channelId"].as_str().unwrap_or("");
 if channel.is_empty()||cfg["notify"]["enabled"]!=true{return}
 let (title,color,description)=match kind{
 "follow"=>("Novo seguidor",0x9146FF,format!("{} começou a seguir o canal.",e.user)),
 "subscription"=>("Nova inscrição",0x00F593,format!("{} assinou o canal.",e.user)),
 "cheer"=>("Bits",0xFFB300,format!("{} enviou {} bits.",e.user,e.message)),
 "raid"=>("Raid",0xE91E63,format!("{} invadiu a transmissão.",e.user)),
 _=>return
 };
 let rt2=rt.clone();let p2=p.clone();let ch=channel.to_owned();
 tokio::spawn(async move{if let Err(err)=post_embed(&rt2,&p2,&ch,&title,&description,color,vec![]).await{rt2.log(&p2.id,"discord",&err,"error")}});
}
pub fn mirror_chat(rt:&Arc<Runtime>,p:&Profile,e:&Event){
 let cfg=config(rt,&p.id);
 if cfg["enabled"]!=true||cfg["mirror"]["enabled"]!=true||cfg["mirror"]["toDiscord"]!=true{return}
 if from_discord(e).is_some(){return}
 let channel=cfg["mirror"]["channelId"].as_str().unwrap_or("");
 if channel.is_empty(){return}
 let text=format!("**{}** · {}",e.user,e.message);
 let rt2=rt.clone();let p2=p.clone();let ch=channel.to_owned();
 tokio::spawn(async move{if let Err(err)=post(&rt2,&p2,&ch,&text).await{rt2.log(&p2.id,"discord",&err,"error")}});
}

#[cfg(test)]
mod tests {
 use super::*;
 fn cfg(v:Value)->Value{v}
 #[test]
 fn intents_and_privileged_toggle() {
 let on=intents(&Value::Null);
 assert_eq!(on&(1<<1),1<<1);assert_eq!(on&(1<<15),1<<15);
 let off=intents(&cfg(json!({"intents":{"members":false,"content":false}})));
 assert_eq!(off&(1<<1),0);assert_eq!(off&(1<<15),0);
 assert_eq!(on&off,on&(1<<0|1<<2|1<<6|1<<9));
 }
 #[test]
 fn invite_carries_scope_and_permissions() {
 let url=invite_url("1234567890","");
 assert!(url.contains("client_id=1234567890"));
 assert!(url.contains("scope=bot%20applications.commands"));
 assert!(url.contains(&format!("permissions={}",permissions())));
 assert!(permissions()&P_MODERATE==P_MODERATE,"o bot precisa de Moderate Members");
 assert!(permissions()&P_MANAGE_ROLES==P_MANAGE_ROLES);
 let guild=invite_url("1","9");assert!(guild.contains("&guild_id=9"));
 }
 #[test]
 fn role_mapping_prefers_configured_roles() {
 let cfg=json!({"roleMap":{"moderator":"111","subscriber":"222"}});
 assert_eq!(map_role(&cfg,&json!({"roles":["111"]})),"moderator");
 assert_eq!(map_role(&cfg,&json!({"roles":["222"]})),"subscriber");
 assert_eq!(map_role(&cfg,&json!({"roles":["222","111"]})),  "moderator");
 assert_eq!(map_role(&cfg,&json!({"roles":[],"permissions":"8"})),"moderator");
 assert_eq!(map_role(&cfg,&json!({"roles":[]})),"everyone");
 }
 #[test]
 fn truncate_counts_graphemes_by_char() {
 assert_eq!(truncate("abc",2),"ab");
 assert_eq!(truncate("áé",10),"áé");
 assert_eq!(truncate("mensagem longa",9),"mensagem ");
 }
 #[test]
 fn member_counter_channel_name_is_valid_for_discord() {
  assert_eq!(counter_name("membros",1284),"membros-1284");
  assert_eq!(counter_name("Membros Online",0),"membros-online-0");
  assert_eq!(counter_name("",7),"membros-7");
  assert_eq!(counter_name("###",9),"membros-9");
 }
 #[test]
 fn discord_source_is_recognized_only_on_discord_events() {
 let mut e=Event{id:"1".into(),profile_id:"p".into(),kind:"chat".into(),user:"u".into(),user_id:"1".into(),role:"everyone".into(),message:"hi".into(),data:json!({"source":"discord","channelId":"77"}),simulated:false};
 assert_eq!(from_discord(&e).as_deref(),Some("77"));
 e.data=json!({"source":"twitch"});assert!(from_discord(&e).is_none());
 e.data=Value::Null;assert!(from_discord(&e).is_none());
 }
}
