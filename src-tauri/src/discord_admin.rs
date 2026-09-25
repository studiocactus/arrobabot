use crate::{engine::Runtime,model::*,discord};
use serde_json::{json,Value};
use std::sync::Arc;

// ---------------------------------------------------------------- state

pub fn stop(rt:&Runtime,p:&str){
 if let Some(h)=rt.discord_tasks.lock().unwrap().remove(p){h.abort()}
 discord::clear_cache(rt,p);discord::status(rt,p,"offline");
}
pub async fn delete_message(rt:&Runtime,p:&Profile,channel:&str,message:&str)->Result<(),String>{
 if channel.trim().is_empty()||message.trim().is_empty(){return Err("Canal ou mensagem ausente".into())}
 discord::rest(rt,p,"DELETE",&format!("/channels/{channel}/messages/{message}"),None).await?;
 Ok(())
}
fn cut(s:&str,n:usize)->String{s.chars().take(n).collect()}
fn id_ok(s:&str)->bool{s.is_empty()||(s.len()<=24&&s.chars().all(|c|c.is_ascii_digit()))}
fn pick<'a>(v:&'a Value,path:&[&str])->&'a Value{let mut cur=v;for k in path{cur=&cur[*k]}cur}
pub fn valid_config(v:&Value)->bool{
 if !v.is_object(){return false}
 if !id_ok(v["guildId"].as_str().unwrap_or("")){return false}
 for key in ["logChannelId","autoroleId","appId","counterChannelId"]{if !id_ok(v[key].as_str().unwrap_or("")){return false}}
 for path in [["mirror","channelId"],["welcome","channelId"],["goodbye","channelId"],["notify","channelId"],["birthday","channelId"],["roleMap","moderator"],["roleMap","subscriber"]]{if !id_ok(pick(v,&path).as_str().unwrap_or("")){return false}}
 if v["enabled"]==true&&v["guildId"].as_str().unwrap_or("").is_empty(){return false}
 if v["mirror"]["enabled"]==true&&v["mirror"]["toDiscord"]==true&&v["mirror"]["channelId"].as_str().unwrap_or("").is_empty(){return false}
 for (key,limit) in [("welcome",600usize),("goodbye",600usize),("birthday",600usize)]{if v[key]["text"].as_str().unwrap_or("").chars().count()>limit{return false}}
 if v["slowmodeSeconds"].as_u64().is_some_and(|s|s>21600){return false}
 if v["giveaway"]["minutes"].as_u64().is_some_and(|m|m>10080){return false}
 true
}

// ---------------------------------------------------------------- persistence

fn audit_key(p:&str)->String{format!("discord_audit:{p}")}
fn warn_key(p:&str)->String{format!("discord_warns:{p}")}
fn read_list(rt:&Runtime,key:&str)->Vec<Value>{rt.db.get(key).as_array().cloned().unwrap_or_default()}
fn write_list(rt:&Runtime,key:&str,list:&[Value])->Result<(),String>{rt.db.set(key,&Value::Array(list.to_vec()))}
pub fn audit(rt:&Runtime,p:&str,entry:Value){
 let key=audit_key(p);
 let mut list=rt.db.get(&key).as_array().cloned().unwrap_or_default();
 let mut e=entry;e["id"]=json!(uuid::Uuid::new_v4().to_string());e["at"]=json!(chrono::Utc::now().to_rfc3339());e["undone"]=json!(false);
 list.insert(0,e);list.truncate(200);
 let _=rt.db.set(&key,&Value::Array(list));
 rt.emit("discord",json!({"profileId":p,"kind":"audit"}));
}

// ---------------------------------------------------------------- ops

pub async fn operation(rt:&Arc<Runtime>,p:&str,op:&str,args:&Value)->Result<Value,String>{
 match op{
 "discord.status"=>status_payload(rt,p).await,
 "discord.connect"=>connect(rt,p).await,
 "discord.disconnect"=>{stop(rt,p);Ok(Value::Null)},
 "discord.save"=>{rt.db.profile(p)?;if !valid_config(&args["config"]){return Err("Configuração do Discord inválida. Confira o servidor e os canais informados.".into())}rt.db.set_module(p,discord::MODULE,&args["config"])?;Ok(Value::Null)},
 "discord.discover"=>discover(rt,p).await,
 "discord.action"=>action(rt,p,args).await,
 _=>Err("Operação do Discord desconhecida".into())
 }
}
async fn status_payload(rt:&Arc<Runtime>,p:&str)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 let cfg=discord::config(rt,p);
 let running=rt.discord_tasks.lock().unwrap().contains_key(p);
 let live=rt.discord_status.lock().unwrap().get(p).cloned().unwrap_or_else(||"offline".into());
 let cache=discord::cache(rt,p);
 Ok(json!({
  "status":if running{live}else{"offline".to_owned()},
  "running":running,
  "enabled":cfg["enabled"]==true,
  "hasToken":crate::secrets::get(p,"discord_token").is_ok(),
  "config":cfg,
  "guilds":cache.guilds,
  "guildId":cache.guild_id,
  "channels":cache.channels,
  "roles":cache.roles,
  "moduleEnabled":profile.modules["discord"]==true,
  "platform":profile.platform,
  "auditCount":read_list(rt,&audit_key(p)).len()
 }))
}
async fn connect(rt:&Arc<Runtime>,p:&str)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 stop(rt,p);
 let cfg=discord::config(rt,p);
 if cfg["enabled"]!=true{return Err("Ative o bot do Discord antes de conectar".into())}
 if cfg["guildId"].as_str().unwrap_or("").is_empty(){return Err("Informe o ID do servidor (guild) do Discord".into())}
 crate::secrets::get(p,"discord_token").map_err(|_|"Salve o token do bot antes de conectar".to_string())?;
 let me=discord::identity(rt,&profile).await.map_err(|e|format!("Não foi possível validar o token: {e}"))?;
 discord::status(rt,p,"connecting");
 let rt2=rt.clone();let p2=profile.clone();
 let task=tokio::spawn(async move{
  if let Err(err)=discord::run(rt2.clone(),p2.clone()).await{
   rt2.log(&p2.id,"discord",&err,"error");discord::status(&rt2,&p2.id,"offline");
  }
 });
 rt.discord_tasks.lock().unwrap().insert(p.to_owned(),task);
 Ok(json!({"identity":me,"status":"connecting"}))
}
async fn discover(rt:&Arc<Runtime>,p:&str)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 let mut guilds=vec![];
 if let Ok(list)=discord::rest(rt,&profile,"GET","/users/@me/guilds",None).await{if let Some(a)=list.as_array(){guilds=a.clone()}}
 let cfg=discord::config(rt,p);
 let guild=cfg["guildId"].as_str().unwrap_or("");
 let mut channels=vec![];let mut roles=vec![];let mut server=Value::Null;
 if !guild.is_empty(){
  server=discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}"),None).await.unwrap_or(Value::Null);
  if let Ok(v)=discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/channels"),None).await{if let Some(a)=v.as_array(){channels=a.clone()}}
  if let Ok(v)=discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/roles"),None).await{if let Some(a)=v.as_array(){roles=a.clone()}}
 }
 let identity=discord::identity(rt,&profile).await.unwrap_or(Value::Null);
 discord::with_cache(rt,p,|c|{c.guilds=guilds.clone();c.channels=channels.clone();c.roles=roles.clone();c.guild_id=guild.to_owned();});
 Ok(json!({"guilds":guilds,"guild":server,"channels":channels,"roles":roles,"identity":identity}))
}

// ---------------------------------------------------------------- action router

pub async fn action(rt:&Arc<Runtime>,p:&str,args:&Value)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 let cfg=discord::config(rt,p);
 let act=args["action"].as_str().ok_or("Ação ausente")?;
 let guild=cfg["guildId"].as_str().unwrap_or("");
 let target=args["target"].as_str().unwrap_or("");
 let reason:String=args["reason"].as_str().unwrap_or("").chars().take(512).collect();
 let channel=args["channel"].as_str().unwrap_or("");
 match act{
 // ---- discovery
 "identity"=>discord::identity(rt,&profile).await,
 "guilds"=>discord::rest(rt,&profile,"GET","/users/@me/guilds",None).await,
 "invite"=>{let id=discord::identity(rt,&profile).await?;Ok(json!({"url":discord::invite_url(id["id"].as_str().unwrap_or(""),guild)}))},
 "channels"=>{
  if !guild.is_empty(){if let Ok(v)=discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/channels"),None).await{return Ok(v)}}
  Ok(json!(discord::cache(rt,p).channels))
 },
 "roles"=>{if guild.is_empty(){return Err("Informe o ID do servidor do Discord".into())}discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/roles"),None).await},
 "members"=>{
  if guild.is_empty(){return Err("Informe o ID do servidor do Discord".into())}
  let list=discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/members?limit=1000"),None).await?;
  let q=args["q"].as_str().unwrap_or("").to_lowercase();
  let out:Vec<Value>=list.as_array().cloned().unwrap_or_default().into_iter().filter(|m|{
   if q.is_empty(){return true}
   m["user"]["username"].as_str().unwrap_or("").to_lowercase().contains(&q)
   ||m["nick"].as_str().unwrap_or("").to_lowercase().contains(&q)
   ||m["user"]["id"].as_str().unwrap_or("")==q
  }).take(50).collect();
  Ok(Value::Array(out))
 },
 "member"=>{if guild.is_empty()||target.is_empty(){return Err("Informe o servidor e o membro".into())}discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/members/{target}"),None).await},
 // ---- messaging
 "message"=>{if channel.is_empty(){return Err("Escolha um canal".into())}let text=args["text"].as_str().ok_or("Escreva uma mensagem")?;discord::post(rt,&profile,channel,text).await},
 "history"=>{if channel.is_empty(){return Err("Escolha um canal".into())}let n=args["limit"].as_u64().unwrap_or(50).clamp(1,100);discord::rest(rt,&profile,"GET",&format!("/channels/{channel}/messages?limit={n}"),None).await},
 "delete"=>{
  if channel.is_empty()||target.is_empty(){return Err("Escolha a mensagem".into())}
  let found=discord::rest(rt,&profile,"GET",&format!("/channels/{channel}/messages/{target}"),None).await.unwrap_or(Value::Null);
  let messages=vec![json!({"id":target,"content":found["content"].as_str().unwrap_or("").to_owned(),"author":found["author"]["username"].as_str().unwrap_or("").to_owned()})];
  delete_message(rt,&profile,channel,target).await?;
  audit(rt,&p,json!({"action":"delete","target":target,"targetName":found["author"]["username"].as_str().unwrap_or(target).to_owned(),"reason":reason,"undoable":true,"platforms":["discord"],"undo":{"kind":"restoreMessages","channel":channel,"messages":messages}}));
  Ok(Value::Null)
 },
 "clear"=>{
  if channel.is_empty(){return Err("Escolha um canal".into())}
  let n=args["limit"].as_u64().unwrap_or(50).clamp(1,100);
  let list=discord::rest(rt,&profile,"GET",&format!("/channels/{channel}/messages?limit={n}"),None).await?;
  let now=chrono::Utc::now().timestamp();
  let mut ids=vec![];let mut captured=vec![];
  for m in list.as_array().cloned().unwrap_or_default(){
   let id=m["id"].as_str().unwrap_or("").to_owned();if id.is_empty(){continue}
   let ts=m["timestamp"].as_str().and_then(|t|chrono::DateTime::parse_from_rfc3339(t).ok()).map(|t|t.timestamp()).unwrap_or(now);
   if now-ts>1209600{continue}
   ids.push(Value::String(id.clone()));
   captured.push(json!({"id":id,"content":m["content"].as_str().unwrap_or("").to_owned(),"author":m["author"]["username"].as_str().unwrap_or("").to_owned()}));
  }
  if ids.is_empty(){return Err("Nada para apagar nos últimos 14 dias".into())}
  discord::rest(rt,&profile,"POST",&format!("/channels/{channel}/messages/bulk-delete"),Some(json!({"messages":ids}))).await?;
  audit(rt,&p,json!({"action":"clear","target":channel,"targetName":args["channelName"].as_str().unwrap_or(channel),"reason":reason,"undoable":true,"platforms":["discord"],"undo":{"kind":"restoreMessages","channel":channel,"messages":captured}}));
  Ok(json!({"removed":ids.len()}))
 },
 // ---- channel state
 "lock"|"unlock"=>{
  if channel.is_empty()||guild.is_empty(){return Err("Escolha um canal".into())}
  let before=discord::rest(rt,&profile,"GET",&format!("/channels/{channel}"),None).await.unwrap_or(Value::Null);
  let previous=before["permission_overwrites"].as_array().cloned().unwrap_or_default().into_iter().find(|o|o["id"].as_str()==Some(guild)).unwrap_or(Value::Null);
  let (allow,deny)=if act=="lock"{("0","2048")}else{("2048","0")};
  discord::rest(rt,&profile,"PATCH",&format!("/channels/{channel}/permissions/{guild}"),Some(json!({"type":0,"allow":allow,"deny":deny,"reason":reason}))).await?;
  audit(rt,&p,json!({"action":act,"target":channel,"targetName":args["channelName"].as_str().unwrap_or(channel),"reason":reason,"undoable":true,"platforms":["discord"],"undo":{"kind":"restorePermissions","channel":channel,"guild":guild,"previous":previous}}));
  Ok(Value::Null)
 },
 "slowmode"=>{
  if channel.is_empty(){return Err("Escolha um canal".into())}
  let before=discord::rest(rt,&profile,"GET",&format!("/channels/{channel}"),None).await.unwrap_or(Value::Null);
  let seconds=args["seconds"].as_u64().unwrap_or(0).clamp(0,21600);
  discord::rest(rt,&profile,"PATCH",&format!("/channels/{channel}"),Some(json!({"rate_limit_per_user":seconds,"reason":reason}))).await?;
  audit(rt,&p,json!({"action":"slowmode","target":channel,"targetName":args["channelName"].as_str().unwrap_or(channel),"reason":reason,"undoable":true,"platforms":["discord"],"undo":{"kind":"restoreSlowmode","channel":channel,"previous":before["rate_limit_per_user"].as_u64().unwrap_or(0)}}));
  Ok(json!({"seconds":seconds}))
 },
 "channelCreate"=>{
  if guild.is_empty(){return Err("Informe o ID do servidor do Discord".into())}
  let name=args["name"].as_str().unwrap_or("").trim().to_lowercase().replace(' ', "-");
  if name.is_empty()||name.len()>100{return Err("Dê um nome ao canal".into())}
  let mut body=json!({"name":name,"type":0});
  let topic=args["topic"].as_str().unwrap_or("");
  if !topic.is_empty(){body["topic"]=json!(cut(topic,1024))}
  if !args["parent"].as_str().unwrap_or("").is_empty(){body["parent_id"]=json!(args["parent"])}
  discord::rest(rt,&profile,"POST",&format!("/guilds/{guild}/channels"),Some(body)).await
 },
 "channelEdit"=>{if channel.is_empty(){return Err("Escolha um canal".into())}let mut body=json!({});if let Some(t)=args["name"].as_str(){body["name"]=json!(t)}if let Some(t)=args["topic"].as_str(){body["topic"]=json!(cut(t,1024))}discord::rest(rt,&profile,"PATCH",&format!("/channels/{channel}"),Some(body)).await},
 "channelDelete"=>{if channel.is_empty(){return Err("Escolha um canal".into())}discord::rest(rt,&profile,"DELETE",&format!("/channels/{channel}"),None).await?;Ok(Value::Null)},
 // ---- moderation
 "ban"|"unban"|"kick"|"timeout"|"untimeout"=>punish_one(rt,&profile,guild,act,target,&reason,args,true).await,
 "warn"=>warn_member(rt,p,target,args["targetName"].as_str().unwrap_or(""),&reason,&["discord","twitch"]).await,
 "unwarn"=>{
  let id=args["id"].as_str().ok_or("Selecione o aviso")?;
  let key=warn_key(p);let mut list=read_list(rt,&key);
  let before=list.len();list.retain(|w|w["id"].as_str()!=Some(id));
  if list.len()==before{return Err("Aviso não encontrado".into())}
  write_list(rt,&key,&list)?;
  audit(rt,&p,json!({"action":"unwarn","target":target,"targetName":args["targetName"].as_str().unwrap_or(target),"reason":reason,"undoable":false,"platforms":["discord","twitch"]}));
  Ok(Value::Null)
 },
 "warnlist"=>{let list=read_list(rt,&warn_key(p));Ok(Value::Array(list.into_iter().filter(|w|target.is_empty()||w["user"].as_str()==Some(target)).collect()))},
 "modlog"=>{
  let warns:Vec<Value>=read_list(rt,&warn_key(p)).into_iter().filter(|w|w["user"].as_str()==Some(target)).collect();
  let actions:Vec<Value>=read_list(rt,&audit_key(p)).into_iter().filter(|a|a["target"].as_str()==Some(target)).collect();
  Ok(json!({"warns":warns,"actions":actions}))
 },
 // ---- roles
 "roleAdd"|"roleRemove"=>{
  if guild.is_empty()||target.is_empty()||args["role"].as_str().unwrap_or("").is_empty(){return Err("Escolha o membro e o cargo".into())}
  let role=args["role"].as_str().unwrap_or("");
  if act=="roleRemove"{
   let before=discord::rest(rt,&profile,"GET",&format!("/guilds/{guild}/members/{target}"),None).await.unwrap_or(Value::Null);
   let had=before["roles"].as_array().is_some_and(|r|r.iter().any(|x|x.as_str()==Some(role)));
   if !had{return Err("Este membro não tem o cargo informado".into())}
   discord::rest(rt,&profile,"DELETE",&format!("/guilds/{guild}/members/{target}/roles/{role}"),None).await?;
  }else{
   discord::rest(rt,&profile,"PUT",&format!("/guilds/{guild}/members/{target}/roles/{role}"),None).await?;
  }
  let undo=if act=="roleAdd"{json!({"kind":"roleRemove","guild":guild,"user":target,"role":role})}else{json!({"kind":"roleAdd","guild":guild,"user":target,"role":role})};
  audit(rt,&p,json!({"action":act,"target":target,"targetName":args["targetName"].as_str().unwrap_or(target),"reason":reason,"undoable":true,"platforms":["discord"],"undo":undo,"role":role}));
  Ok(Value::Null)
 },
 // ---- cross platform
 "punish"=>punish(rt,&profile,&cfg,args).await,
 // ---- audit
 "audit"=>{
  let mut list=read_list(rt,&audit_key(p));
  if let Some(a)=args["target"].as_str(){if !a.is_empty(){list=list.into_iter().filter(|e|e["target"].as_str()==Some(a)).collect()}}
  if let Some(n)=args["limit"].as_u64(){list.truncate((n as usize).max(1))}
  Ok(Value::Array(list))
 },
 "auditExport"=>Ok(Value::String(csv(&read_list(rt,&audit_key(p))))),
 "undo"=>undo(rt,p,&profile,args["id"].as_str().ok_or("Selecione o registro")?).await,
 // ---- engagement: XP, birthdays, giveaways, identity links, slash commands
 "slash"=>crate::discord_engage::register(rt,&profile,guild,args).await,
 "xpRank"|"xpAdd"|"xpEdit"|"xpTransfer"|"birthdayAdd"|"birthdayRemove"|"birthdayList"|"birthdayToday"
 |"giveawayList"|"giveawayStart"|"giveawayEdit"|"giveawayEnd"|"giveawayReroll"
 |"linkCreate"|"linkList"|"linkRemove"|"linkIdentity"=>crate::discord_engage::action(rt,p,&profile,args).await,
 _=>Err("Ação do Discord desconhecida".into())
 }
}

// ---------------------------------------------------------------- punishment

async fn punish_one(rt:&Arc<Runtime>,p:&Profile,guild:&str,act:&str,target:&str,reason:&str,args:&Value,write_audit:bool)->Result<Value,String>{
 if target.is_empty(){return Err("Escolha o membro".into())}
 if guild.is_empty(){return Err("Informe o ID do servidor do Discord".into())}
 let name=args["targetName"].as_str().unwrap_or(target);
 match act{
 "ban"|"unban"=>{
  let seconds=args["deleteMessageSeconds"].as_u64().unwrap_or(0).clamp(0,604800);
  let body=if act=="ban"{Some(json!({"delete_message_seconds":seconds,"reason":reason}))}else{None};
  discord::rest(rt,p,if act=="ban"{"PUT"}else{"DELETE"},&format!("/guilds/{guild}/bans/{target}"),body).await?;
  if write_audit{
   let undo=if act=="ban"{json!({"kind":"unban","guild":guild,"user":target})}else{json!({"kind":"ban","guild":guild,"user":target,"seconds":seconds,"reason":reason})};
   audit(rt,&p.id,json!({"action":act,"target":target,"targetName":name,"reason":reason,"undoable":true,"platforms":["discord"],"undo":undo}));
  }
 },
 "kick"=>{
  discord::rest(rt,p,"DELETE",&format!("/guilds/{guild}/members/{target}"),Some(json!({"reason":reason}))).await?;
  if write_audit{audit(rt,&p.id,json!({"action":"kick","target":target,"targetName":name,"reason":reason,"undoable":false,"platforms":["discord"]}));}
 },
 "timeout"|"untimeout"=>{
  let before=discord::rest(rt,p,"GET",&format!("/guilds/{guild}/members/{target}"),None).await.unwrap_or(Value::Null);
  let until=if act=="timeout"{
   let seconds=args["seconds"].as_u64().unwrap_or(600).clamp(60,2419200);
   json!((chrono::Utc::now()+chrono::Duration::seconds(seconds as i64)).to_rfc3339_opts(chrono::SecondsFormat::Millis,true))
  }else{Value::Null};
  discord::rest(rt,p,"PATCH",&format!("/guilds/{guild}/members/{target}"),Some(json!({"communication_disabled_until":until,"reason":reason}))).await?;
  if write_audit{
   let undo=if act=="timeout"{json!({"kind":"untimeout","guild":guild,"user":target})}else{json!({"kind":"timeout","guild":guild,"user":target,"until":before["communication_disabled_until"]})};
   audit(rt,&p.id,json!({"action":act,"target":target,"targetName":name,"reason":reason,"undoable":true,"platforms":["discord"],"undo":undo}));
  }
 },
 _=>return Err("Ação de moderação inválida".into())
 }
 Ok(Value::Null)
}
/// One request, both chats: applies the same verdict on Twitch and on Discord.
async fn punish(rt:&Arc<Runtime>,p:&Profile,cfg:&Value,args:&Value)->Result<Value,String>{
 let kind=args["kind"].as_str().ok_or("Escolha a punição")?;
 let target=args["target"].as_str().unwrap_or("");
 let reason:String=args["reason"].as_str().unwrap_or("").chars().take(512).collect();
 let where_to=args["platform"].as_str().unwrap_or("both");
 if target.is_empty(){return Err("Escolha o membro".into())}
 if !["ban","unban","timeout","warn","kick"].contains(&kind){return Err("Punição inválida".into())}
 let on_twitch=where_to=="both"||where_to=="twitch";
 let on_discord=where_to=="both"||where_to=="discord";
 let mut applied=vec![];let mut errors=vec![];
 if kind=="warn"{
  let mut platforms:Vec<&str>=vec![];
  if on_twitch&&p.platform=="twitch"{
   match crate::moderation::twitch_punish(rt,p,"warn",target,&reason,0).await{
   Ok(())=>platforms.push("twitch"),
   Err(err)=>errors.push(format!("Twitch: {err}"))
   }
  }
  if on_discord{platforms.push("discord")}
  if platforms.is_empty(){if !errors.is_empty(){return Err(errors.join(" · "))}platforms.push("ledger")}
  let entry=warn_member(rt,&p.id,target,args["targetName"].as_str().unwrap_or(""),&reason,&platforms).await?;
  rt.log(&p.id,"moderation",&format!("warn · {target} · {}",platforms.join("+")),"success");
  return Ok(json!({"applied":platforms,"errors":errors,"warn":entry}));
 }
 if on_twitch{
  if kind=="kick"{errors.push("A Twitch não oferece expulsão; use banimento ou timeout.".to_owned())}
  else{
   let seconds=args["seconds"].as_u64().unwrap_or(600).clamp(60,1209600);
   let t=if kind=="timeout"{"timeout"}else{kind};
   match crate::moderation::twitch_punish(rt,p,t,target,&reason,seconds).await{
   Ok(())=>applied.push("twitch"),
   Err(err)=>errors.push(format!("Twitch: {err}"))
   }
  }
 }
 if on_discord{
  let guild=cfg["guildId"].as_str().unwrap_or("");
  if guild.is_empty(){errors.push("Discord: servidor não informado".to_owned())}
  else{
   match punish_one(rt,p,guild,kind,target,&reason,args,false).await{
   Ok(_)=>applied.push("discord"),
   Err(err)=>errors.push(format!("Discord: {err}"))
   }
  }
 }
 if applied.is_empty(){return Err(if errors.is_empty(){"Nenhuma punição aplicada".into()}else{errors.join(" · ")})}
 audit(rt,&p.id,json!({"action":kind,"target":target,"targetName":args["targetName"].as_str().unwrap_or(target),"reason":reason,"undoable":kind=="unban","platforms":applied.clone(),"undo":json!({"kind":"unban","guild":cfg["guildId"],"user":target})}));
 rt.log(&p.id,"moderation",&format!("{kind} · {target} · {}",applied.join("+")),"success");
 Ok(json!({"applied":applied,"errors":errors}))
}
async fn warn_member(rt:&Arc<Runtime>,p:&str,target:&str,name:&str,reason:&str,platforms:&[&str])->Result<Value,String>{
 if target.is_empty(){return Err("Escolha o membro".into())}
 let key=warn_key(p);let mut list=read_list(rt,&key);
 let entry=json!({"id":uuid::Uuid::new_v4().to_string(),"at":chrono::Utc::now().to_rfc3339(),"user":target,"name":name.to_owned(),"reason":reason.to_owned()});
 list.insert(0,entry.clone());list.truncate(300);write_list(rt,&key,&list)?;
 audit(rt,p,json!({"action":"warn","target":target,"targetName":name,"reason":reason,"undoable":true,"platforms":platforms,"undo":{"kind":"unwarn","user":target,"warnId":entry["id"]}}));
 Ok(entry)
}

// ---------------------------------------------------------------- undo

async fn undo(rt:&Arc<Runtime>,p:&str,profile:&Profile,id:&str)->Result<Value,String>{
 let key=audit_key(p);
 let mut list=read_list(rt,&key);
 let idx=list.iter().position(|e|e["id"].as_str()==Some(id)).ok_or("Registro não encontrado")?;
 if list[idx]["undoable"]!=true{return Err("Este registro não pode ser desfeito".into())}
 if list[idx]["undone"]==true{return Err("Este registro já foi desfeito".into())}
 let u=list[idx]["undo"].clone();
 match u["kind"].as_str().unwrap_or(""){
 "restorePermissions"=>{
  let ch=u["channel"].as_str().unwrap_or("");let g=u["guild"].as_str().unwrap_or("");
  if ch.is_empty()||g.is_empty(){return Err("Registro sem canal ou servidor".into())}
  let previous=&u["previous"];
  if !previous.is_object(){discord::rest(rt,profile,"DELETE",&format!("/channels/{ch}/permissions/{g}"),None).await?;}
  else{let mut body=json!({"type":0});body["allow"]=previous["allow"].clone();body["deny"]=previous["deny"].clone();discord::rest(rt,profile,"PATCH",&format!("/channels/{ch}/permissions/{g}"),Some(body)).await?;}
 },
 "restoreSlowmode"=>{let ch=u["channel"].as_str().unwrap_or("");if ch.is_empty(){return Err("Registro sem canal".into())}discord::rest(rt,profile,"PATCH",&format!("/channels/{ch}"),Some(json!({"rate_limit_per_user":u["previous"].as_u64().unwrap_or(0)}))).await?;},
 "restoreMessages"=>{
  let ch=u["channel"].as_str().unwrap_or("");if ch.is_empty(){return Err("Registro sem canal".into())}
  let mut restored=0;
  for m in u["messages"].as_array().cloned().unwrap_or_default().into_iter().rev(){
   let author=m["author"].as_str().unwrap_or("").to_owned();
   let content=m["content"].as_str().unwrap_or("");
   if content.trim().is_empty(){continue}
   if discord::post(rt,profile,ch,&format!("**{author}** · {}",cut(content,1900))).await.is_ok(){restored+=1}
  }
  if restored==0{return Err("Nenhuma mensagem pôde ser restaurada".into())}
 },
 "roleAdd"|"roleRemove"=>{
  let g=u["guild"].as_str().unwrap_or("");let who=u["user"].as_str().unwrap_or("");let role=u["role"].as_str().unwrap_or("");
  if g.is_empty()||who.is_empty()||role.is_empty(){return Err("Registro sem membro ou cargo".into())}
  let path=format!("/guilds/{g}/members/{who}/roles/{role}");
  if u["kind"].as_str()==Some("roleAdd"){discord::rest(rt,profile,"PUT",&path,None).await?;}else{discord::rest(rt,profile,"DELETE",&path,None).await?;}
 },
 "unban"=>{let g=u["guild"].as_str().unwrap_or("");let who=u["user"].as_str().unwrap_or("");if g.is_empty()||who.is_empty(){return Err("Registro sem servidor ou membro".into())}discord::rest(rt,profile,"DELETE",&format!("/guilds/{g}/bans/{who}"),None).await?;},
 "ban"=>{let g=u["guild"].as_str().unwrap_or("");let who=u["user"].as_str().unwrap_or("");if g.is_empty()||who.is_empty(){return Err("Registro sem servidor ou membro".into())}let mut body=json!({"reason":"Desfeito pelo BotLive"});body["delete_message_seconds"]=json!(u["seconds"].as_u64().unwrap_or(0).min(604800));discord::rest(rt,profile,"PUT",&format!("/guilds/{g}/bans/{who}"),Some(body)).await?;},
 "timeout"|"untimeout"=>{
  let g=u["guild"].as_str().unwrap_or("");let who=u["user"].as_str().unwrap_or("");if g.is_empty()||who.is_empty(){return Err("Registro sem servidor ou membro".into())}
  let until=if u["kind"].as_str()==Some("timeout"){u["until"].clone()}else{Value::Null};
  discord::rest(rt,profile,"PATCH",&format!("/guilds/{g}/members/{who}"),Some(json!({"communication_disabled_until":until}))).await?;
 },
 "unwarn"=>{
  let who=u["user"].as_str().unwrap_or("");
  let kw=warn_key(p);let mut warns=read_list(rt,&kw);
  let before=warns.len();
  if let Some(id)=u["warnId"].as_str(){warns.retain(|w|w["id"].as_str()!=Some(id));}
  else{warns.retain(|w|w["user"].as_str()!=Some(who));}
  if warns.len()==before{return Err("Não há avisos para remover".into())}
  write_list(rt,&kw,&warns)?;
 },
 _=>return Err("Este registro não pode ser desfeito por aqui".into())
 }
 list[idx]["undone"]=json!(true);
 write_list(rt,&key,&list)?;
 rt.log(p,"discord",&format!("Ação desfeita: {}",list[idx]["action"]),"success");
 rt.emit("discord",json!({"profileId":p,"kind":"audit"}));
 Ok(Value::Null)
}

// ---------------------------------------------------------------- export

fn csv(rows:&[Value])->String{
 let mut out=String::from("quando;acao;alvo;nome;motivo;desfeita;plataformas\n");
 for r in rows{
  let platforms=r["platforms"].as_array().map(|a|a.iter().filter_map(|v|v.as_str()).collect::<Vec<_>>().join("+")).unwrap_or_default();
  let cells=[r["at"].as_str().unwrap_or(""),r["action"].as_str().unwrap_or(""),r["target"].as_str().unwrap_or(""),r["targetName"].as_str().unwrap_or(""),r["reason"].as_str().unwrap_or(""),if r["undone"]==true{"sim"}else{"nao"},platforms.as_str()];
  out.push_str(&cells.iter().map(|c|format!("\"{}\"",c.replace('"',"\"\""))).collect::<Vec<_>>().join(";"));
  out.push('\n');
 }
 out
}

#[cfg(test)]
mod tests {
 use super::*;
 fn base()->Value{json!({"enabled":true,"guildId":"123456789012345678"})}
 #[test]
 fn config_validation_accepts_a_minimal_enabled_setup() {
 assert!(valid_config(&base()));
 let mut v=base();v["enabled"]=json!(false);assert!(valid_config(&v));
 }
 #[test]
 fn config_validation_rejects_missing_guild_and_bad_ids() {
 let mut v=base();v["guildId"]=json!("");assert!(!valid_config(&v));
 let mut v=base();v["logChannelId"]=json!("abc");assert!(!valid_config(&v));
 let mut v=base();v["welcome"]=json!({"channelId":"not-a-snowflake"});assert!(!valid_config(&v));
 assert!(!valid_config(&json!("x")));
 }
 #[test]
 fn config_validation_rejects_mirror_without_channel_and_long_templates() {
 let mut v=base();v["mirror"]=json!({"enabled":true,"toDiscord":true,"channelId":""});assert!(!valid_config(&v));
 let mut v=base();v["mirror"]=json!({"enabled":false,"toDiscord":true,"channelId":""});assert!(valid_config(&v));
 let mut v=base();v["welcome"]=json!({"channelId":"1","text":"x"});assert!(valid_config(&v));
 v["welcome"]["text"]=json!("x".repeat(700));assert!(!valid_config(&v));
 let mut v=base();v["slowmodeSeconds"]=json!(999999);assert!(!valid_config(&v));
 let mut v=base();v["giveaway"]=json!({"minutes":20000});assert!(!valid_config(&v));
 }
 #[test]
 fn csv_escapes_quotes_and_joins_platforms() {
 let rows=vec![json!({"at":"t","action":"ban","target":"1","targetName":"ana","reason":"\"x;y\"","undone":false,"platforms":["discord","twitch"]})];
 let out=csv(&rows);
 assert!(out.starts_with("quando;acao"));
 assert!(out.contains("\"\"x;y\"\""));
 assert!(out.contains("discord+twitch"));
 assert!(out.contains("nao"));
 }
 #[test]
 fn storage_keys_are_profile_scoped() {
 assert_eq!(audit_key("a"),"discord_audit:a");
 assert_eq!(warn_key("b"),"discord_warns:b");
 assert_ne!(warn_key("a"),warn_key("b"));
 }
}
