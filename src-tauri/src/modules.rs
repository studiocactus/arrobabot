use crate::{engine::Runtime,model::*};
use serde::{Serialize,Deserialize};
use serde_json::{json,Value};
use std::collections::BTreeMap;
use std::time::{Duration,Instant};
use rand::Rng;
#[derive(Default,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Community {
 #[serde(default)] pub last_chat:i64,
 #[serde(default)] pub last_trivia:i64,
 #[serde(default)] pub balances:BTreeMap<String,i64>,
 #[serde(default)] pub names:BTreeMap<String,String>,
 #[serde(default)] pub logins:BTreeMap<String,String>,
 #[serde(default)] pub queue:Vec<String>,
 #[serde(default)] pub songs:Vec<Song>,
 #[serde(default)] pub raffle:Raffle,
 #[serde(default)] pub prediction:Prediction,
 #[serde(default)] pub shop:Vec<Item>,
 #[serde(default)] pub redemptions:Vec<Value>,
 #[serde(default)] pub last_earned:BTreeMap<String,i64>,
 #[serde(default)] pub game_at:BTreeMap<String,i64>,
 #[serde(default)] pub trivia:Value,
 #[serde(default)] pub duel:Value,
 #[serde(default)] pub bingo:Value,
}
#[derive(Default,Clone,Serialize,Deserialize)] pub struct Song {pub user:String,pub url:String}
#[derive(Default,Clone,Serialize,Deserialize)] pub struct Item {pub id:String,pub name:String,pub cost:i64,pub stock:i64}
#[derive(Default,Clone,Serialize,Deserialize)] pub struct Raffle {pub open:bool,pub keyword:String,pub price:i64,pub entries:BTreeMap<String,i64>,#[serde(default)] pub weights:BTreeMap<String,i64>,pub winner:String}
#[derive(Default,Clone,Serialize,Deserialize)] pub struct Prediction {pub open:bool,pub title:String,pub options:Vec<String>,pub bets:BTreeMap<String,(usize,i64)>,pub result:Option<usize>}
pub fn load(rt:&Runtime,p:&str)->Community {rt.db.module_cfg(p,"community")}
/// Nomes de quem já falou no chat deste perfil, para sorteios.
pub fn chatter_names(db:&crate::db::Db,p:&str)->Vec<String>{
 let state:Community=db.module_cfg(p,"community");
 state.names.values().filter(|n|!n.trim().is_empty()).cloned().collect()
}
/// Sorteia um nome entre quem já falou no chat deste perfil. Sem ninguém no cadastro, não há valor.
pub fn random_chatter(db:&crate::db::Db,p:&str)->Option<String>{
 let names=chatter_names(db,p);
 if names.is_empty(){None}else{Some(names[rand::thread_rng().gen_range(0..names.len())].clone())}
}
/// Match do chat: quem cita uma palavra romântica leva o nome de um par
/// sorteado entre quem falou no chat nos últimos dez minutos, e a % de
/// compatibilidade sai de um hash dos dois nomes — determinística.
pub const AFFINITY_TERMS:&str="amor, paixão, crush, casal, beijo, namorar";
pub const AFFINITY_MESSAGES:&str="💔 MATCH NEGADO: {{user1}} x {{user2}} = {{pct}}%. Nem o algoritmo do Tinder salva. Sigam como amigos.\n😬 {{user1}} x {{user2}} = {{pct}}%. Dá pra tentar, mas leva um café e muita paciência.\n🤔 {{user1}} x {{user2}} = {{pct}}%. Tem potencial. Falta só um de vocês tomar coragem.\n😍 DEU MATCH: {{user1}} x {{user2}} = {{pct}}%. O chat já pode ir escolhendo o nome do casal.\n🔥 ALMA GÊMEA: {{user1}} x {{user2}} = {{pct}}%. Marquem o casamento e me chamem de padrinho.";
const AFFINITY_WINDOW:u64=600;
fn nick_of(e:&Event)->String{
 e.data["chatter_user_login"].as_str().map(|s|s.to_lowercase()).unwrap_or_else(||{
  let u=e.user.to_lowercase();
  if !u.contains(char::is_whitespace)&&crate::twitch_ops::login_chars(&u){u}else{String::new()}
 })
}
/// Palavra inteira: "amor" casa em "seu amor!", nunca em "amortecedor".
fn affinity_hit(terms:&str,message:&str)->bool{
 let terms:Vec<String>=terms.split(',').map(|t|t.trim().to_lowercase()).filter(|t|!t.is_empty()).collect();
 !terms.is_empty()&&message.split_whitespace().any(|w|{
  let w=w.trim_matches(|c:char|!c.is_alphanumeric());
  !w.is_empty()&&terms.iter().any(|t|t==w)
 })
}
/// Hash FNV-1a do par em ordem canônica: o mesmo casal dá sempre o mesmo número.
fn pair_pct(a:&str,b:&str)->u8{
 let x=a.to_lowercase();let y=b.to_lowercase();let (first,second)=if x<=y{(&x,&y)}else{(&y,&x)};
 let mut key=first.to_owned();key.push('\n');key.push_str(second);
 let mut h:u64=0xcbf29ce484222325;
 for byte in key.bytes(){h^=byte as u64;h=h.wrapping_mul(0x100000001b3);}
 (h%101) as u8
}
fn band(pct:u8)->usize{if pct<=20{0}else if pct<=40{1}else if pct<=60{2}else if pct<=80{3}else{4}}
/// Mensagem da faixa: a linha do painel, ou o texto padrão quando a linha está vazia.
fn affinity_message(cfg:&Value,u1:&str,u2:&str,pct:u8)->String{
 let band=band(pct);
 let template=cfg["messages"].as_str().unwrap_or("").lines().nth(band).map(str::trim).filter(|l|!l.is_empty())
  .or_else(||AFFINITY_MESSAGES.lines().nth(band)).unwrap_or("");
 template.replace("{{user1}}",u1).replace("{{user2}}",u2).replace("{{pct}}",&pct.to_string())
}
pub fn valid_config(v:&Value)->bool{
 let terms=v["terms"].as_str().unwrap_or(AFFINITY_TERMS).trim();
 let command=v["command"].as_str().unwrap_or("!match").trim();
 let messages=v["messages"].as_str().unwrap_or("");
 !terms.is_empty()&&terms.len()<=300
  &&!command.is_empty()&&command.len()<=30&&command.starts_with('!')&&!command.contains(char::is_whitespace)
  &&messages.len()<=3200&&messages.lines().all(|l|l.len()<=500)
}
/// Primeiro salvamento materializa as faixas padrão, para a edição mostrar o que existe.
pub fn with_defaults(v:&Value)->Value{
 let mut v=v.clone();
 if v["messages"].as_str().map(|s|s.trim().is_empty()).unwrap_or(true){v["messages"]=json!(AFFINITY_MESSAGES);}
 v
}
/// Quem falou no chat na janela do Match: (login, nome na tela). O autor da
/// mensagem e a live ficam de fora — a live só entra quando a config mandar.
fn affinity_pool(rt:&Runtime,s:&Community,author:&str,include_streamer:bool,channel:&str)->Vec<(String,String)>{
 let window=Duration::from_secs(AFFINITY_WINDOW);let now=Instant::now();
 let map=rt.seen_chatters.lock().unwrap();
 let mut pool:Vec<(String,String)>=map.iter()
  .filter(|(key,(login,at))|*key==login&&now.saturating_duration_since(*at)<window)
  .filter(|(_,(login,_))|!login.eq_ignore_ascii_case(author))
  .filter(|(_,(login,_))|include_streamer||!login.eq_ignore_ascii_case(channel))
  .map(|(_, (login,_))|(login.clone(),s.logins.get(login).cloned().unwrap_or_else(||login.clone())))
  .collect();
 pool.sort();pool.dedup_by(|a,b|a.0==b.0);pool
}
fn affinity_ready(rt:&Runtime,p:&str,uid:&str,global:u64,user:u64)->bool{
 if global==0&&user==0{return true}
 let mut cd=rt.cooldowns.lock().unwrap();cd.retain(|_,t|t.elapsed()<Duration::from_secs(86400));
 let g=format!("{p}:affinity");let u=format!("{g}:{uid}");
 let blocked=cd.get(&g).is_some_and(|t|t.elapsed().as_secs()<global)||cd.get(&u).is_some_and(|t|t.elapsed().as_secs()<user);
 if !blocked{let now=Instant::now();cd.insert(g,now);cd.insert(u,now);}
 !blocked
}
fn affinity_reply(rt:&Runtime,p:&Profile,e:&Event,s:&mut Community)->Result<Option<String>,String>{
 let cfg=rt.db.module(&p.id,"affinity");
 let command=cfg["command"].as_str().unwrap_or("!match").trim().to_lowercase();
 let args:Vec<&str>=e.message.split_whitespace().collect();
 let global=cfg["cooldown"].as_u64().unwrap_or(120).clamp(0,86400);
 let user=cfg["userCooldown"].as_u64().unwrap_or(300).clamp(0,86400);
 let display=|login:&str|s.logins.values().find(|n|n.eq_ignore_ascii_case(login)).cloned().unwrap_or_else(||login.to_string());
 // !match nome1 nome2: a mesma conta do hash para um par que o chat escolher.
 if args.first().is_some_and(|c|c.to_lowercase()==command){
  let clean=|w:&str|w.trim_start_matches('@').to_lowercase();
  let (Some(a),Some(b))=(args.get(1).map(|w|clean(w)),args.get(2).map(|w|clean(w)))else{return Err(format!("Use {command} nome1 nome2"))};
  if a.is_empty()||b.is_empty(){return Err(format!("Use {command} nome1 nome2"))}
  if a==b{return Err("Escolha duas pessoas diferentes".into())}
  if !affinity_ready(rt,&p.id,&e.user_id,global,user){return Ok(None)}
  let pct=pair_pct(&a,&b);
  return Ok(Some(affinity_message(&cfg,&display(&a),&display(&b),pct)));
 }
 // Palavra-gatilho: mensagem que começa com ! é comando, não dispara.
 if e.message.trim().starts_with('!'){return Ok(None)}
 if !affinity_hit(cfg["terms"].as_str().unwrap_or(AFFINITY_TERMS),&e.message){return Ok(None)}
 let pool=affinity_pool(rt,s,&nick_of(e),cfg["includeStreamer"].as_bool().unwrap_or(false),&p.channel);
 // Sem um segundo par no chat, não há match a anunciar.
 if pool.is_empty(){return Ok(None)}
 let mentioned:Vec<String>=e.message.split_whitespace().filter(|w|w.starts_with('@')).map(|w|w.trim_start_matches('@').to_lowercase()).collect();
 let partner=mentioned.iter().find_map(|m|pool.iter().find(|(l,_)|l==m)).cloned()
  .or_else(||{use rand::seq::SliceRandom;pool.choose(&mut rand::thread_rng()).cloned()});
 let Some(partner)=partner else{return Ok(None)};
 if !affinity_ready(rt,&p.id,&e.user_id,global,user){return Ok(None)}
 let pct=pair_pct(&nick_of(e),&partner.0);
 Ok(Some(affinity_message(&cfg,&e.user,&partner.1,pct)))
}
fn balance(s:&mut Community,u:&str,delta:i64)->Result<i64,String>{
 let v=s.balances.entry(u.into()).or_default();let next=v.checked_add(delta).filter(|n|*n>=0).ok_or("Saldo insuficiente ou valor inválido")?;*v=next;Ok(next)
}
fn positive(s:Option<&str>)->Result<i64,String>{s.and_then(|s|s.parse::<i64>().ok()).filter(|n|*n>0&&*n<=1_000_000).ok_or("Use um valor entre 1 e 1.000.000".into())}
pub fn change_points(rt:&Runtime,p:&str,user:&str,amount:i64)->Result<i64,String>{
 let _lock=rt.module_lock.lock().unwrap();let mut s=load(rt,p);let n=balance(&mut s,user,amount)?;rt.db.set_module(p,"community",&serde_json::to_value(s).unwrap())?;Ok(n)
}
pub fn admin(rt:&Runtime,p:&str,op:&str,args:Value)->Result<Value,String>{
 rt.db.profile(p)?;let _lock=rt.module_lock.lock().unwrap();let mut s=load(rt,p);
 match op{
 "raffle_open"=>{if s.raffle.open{return Err("Encerre ou cancele o sorteio atual".into())}s.raffle=Raffle{open:true,keyword:args["keyword"].as_str().unwrap_or("!sorteio").into(),price:args["price"].as_i64().unwrap_or(0).clamp(0,1_000_000),..Default::default()};},
 "raffle_cancel"=>{if !s.raffle.open{return Err("Não há sorteio aberto para cancelar".into())}let entries=s.raffle.entries.clone();let price=s.raffle.price;for(u,n)in entries{balance(&mut s,&u,n*price)?;}s.raffle=Raffle::default();},
 "raffle_draw"=>{
 let total:i64=s.raffle.weights.values().sum();if !s.raffle.open||total==0{return Err("Abra um sorteio e aguarde participantes".into())}
 let mut pick=rand::thread_rng().gen_range(0..total);let mut winner=String::new();
 for(u,n)in &s.raffle.weights{if pick<*n{winner=u.clone();break}pick-=n;}
 s.raffle.winner=winner;s.raffle.open=false;
 },
 "prediction_open"=>{if s.prediction.open{return Err("Finalize a previsão atual".into())}let options:Vec<String>=serde_json::from_value(args["options"].clone()).map_err(|_|"Informe opções")?;if options.len()<2||options.len()>6{return Err("Use de 2 a 6 opções".into())}s.prediction=Prediction{open:true,title:args["title"].as_str().unwrap_or("Previsão").into(),options,..Default::default()};},
 "prediction_cancel"=>{if !s.prediction.open{return Err("Não há previsão aberta para cancelar".into())}let bets=s.prediction.bets.clone();for(u,(_,n))in bets{balance(&mut s,&u,n)?;}s.prediction=Prediction::default();},
 "prediction_settle"=>{
 if !s.prediction.open{return Err("Não há previsão aberta".into())}
 let result=args["result"].as_u64().ok_or("Escolha o resultado")? as usize;if result>=s.prediction.options.len(){return Err("Resultado inválido".into())}
 let total:i64=s.prediction.bets.values().map(|(_,n)|*n).sum();let winners:i64=s.prediction.bets.values().filter(|(i,_)|*i==result).map(|(_,n)|*n).sum();
 let bets=s.prediction.bets.clone();
 if winners==0{for(u,(_,n))in bets{balance(&mut s,&u,n)?;}}else{
 let mut remaining=total;let winning:Vec<_>=bets.into_iter().filter(|(_, (i,_))|*i==result).collect();
 for (index,(u,(_,n))) in winning.iter().enumerate(){let amount=if index==winning.len()-1{remaining}else{((total as i128 * *n as i128)/winners as i128) as i64};remaining-=amount;balance(&mut s,u,amount)?;}
 }
 s.prediction.open=false;s.prediction.result=Some(result);
 },
 "queue_next"=>{if !s.queue.is_empty(){s.queue.remove(0);}},
 "queue_clear"=>s.queue.clear(),
 "song_next"=>{if !s.songs.is_empty(){s.songs.remove(0);}},
 "shop_add"=>{let name=args["name"].as_str().unwrap_or("").trim();if name.is_empty(){return Err("Dê um nome ao item".into())}let cost=args["cost"].as_i64().unwrap_or(100);let stock=args["stock"].as_i64().unwrap_or(10);if cost<0||stock<0{return Err("Custo e estoque não podem ser negativos".into())}s.shop.push(Item{id:uuid::Uuid::new_v4().to_string(),name:name.into(),cost,stock});},
 "shop_defaults"=>{if s.shop.is_empty(){for(name,cost)in [("Escolher o próximo desafio",100),("Mensagem em destaque",50),("Jogar com o streamer",200)]{s.shop.push(Item{id:uuid::Uuid::new_v4().to_string(),name:name.into(),cost,stock:10});}}},
 "bingo_open"=>{let emotes:Vec<String>=serde_json::from_value(args["emotes"].clone()).map_err(|_|"Informe os emotes")?;if emotes.len()<5||emotes.len()>30{return Err("Use de 5 a 30 emotes".into())}s.bingo=json!({"open":true,"emotes":emotes,"drawn":[],"cards":{},"winner":null,"reward":50});},
 "trivia_open"=>{if args["question"].as_str().unwrap_or("").is_empty()||args["answer"].as_str().unwrap_or("").is_empty(){return Err("Preencha pergunta e resposta".into())}s.trivia=json!({"question":args["question"],"answer":args["answer"],"reward":args["reward"].as_i64().unwrap_or(50).clamp(0,10000),"open":true});},
 "points"=>{balance(&mut s,args["user"].as_str().ok_or("Informe o usuário")?,args["amount"].as_i64().ok_or("Informe os pontos")?)?;},
 _=>return Err("Operação desconhecida".into())
 }
 let value=serde_json::to_value(&s).unwrap();rt.db.set_module(p,"community",&value)?;rt.emit("community",json!({"profileId":p}));rt.emit("songs",json!({"profileId":p,"queue":s.songs}));Ok(value)
}
pub fn chat(rt:&Runtime,p:&Profile,e:&Event)->Result<Option<String>,String>{
 if e.simulated{return Ok(None)}
 let _lock=rt.module_lock.lock().unwrap();let mut s=load(rt,&p.id);
 let enabled=|key:&str|p.modules[key].as_bool().unwrap_or(false);
 let uid=&e.user_id;s.names.insert(uid.clone(),e.user.clone());let nick=nick_of(e);if !nick.is_empty(){s.logins.insert(nick,e.user.clone());}
 let now=chrono::Utc::now().timestamp();s.last_chat=now;
 let points=rt.db.module(&p.id,"points");let songs=rt.db.module(&p.id,"songs");
 let currency=points["currency"].as_str().filter(|s|!s.trim().is_empty()).unwrap_or("pontos");
 if s.duel["open"]==true&&s.duel["expires"].as_i64().unwrap_or(0)<now{s.duel=Value::Null;}
 if enabled("points") && now-s.last_earned.get(uid).copied().unwrap_or(0)>=points["interval"].as_i64().unwrap_or(60).clamp(10,86400){let multiplier=if permitted("subscriber",&e.role){points["subscriberMultiplier"].as_i64().unwrap_or(2).clamp(1,10)}else{1};balance(&mut s,uid,points["amount"].as_i64().unwrap_or(5).clamp(0,10000)*multiplier)?;s.last_earned.insert(uid.clone(),now);}
 let args:Vec<_>=e.message.split_whitespace().collect();let cmd=args.first().copied().unwrap_or("").to_lowercase();
 let reply=match cmd.as_str(){
 "!pontos" if enabled("points")=>Some(format!("{}, você tem {} {currency}.",e.user,s.balances.get(uid).unwrap_or(&0))),
 "!ranking" if enabled("points")=>{let mut b:Vec<_>=s.balances.iter().collect();b.sort_by(|a,b|b.1.cmp(a.1));Some(b.iter().take(5).map(|(u,n)|format!("{}: {n}",s.names.get(*u).unwrap_or(u))).collect::<Vec<_>>().join(" · "))},
 "!transferir" if enabled("points")=>{let name=args.get(1).ok_or("Use !transferir usuário pontos")?.trim_start_matches('@');let target=s.names.iter().find(|(_,n)|n.eq_ignore_ascii_case(name)).map(|(u,_)|u.clone()).ok_or("Esse usuário ainda não interagiu nesta sessão")?;let n=positive(args.get(2).copied())?;balance(&mut s,uid,-n)?;balance(&mut s,&target,n)?;Some(format!("Transferidos {n} pontos para {name}."))},
 "!loja" if enabled("points")=>Some(s.shop.iter().enumerate().map(|(i,x)|format!("{}: {} ({} pontos, {} disponíveis)",i+1,x.name,x.cost,x.stock)).collect::<Vec<_>>().join(" · ")),
 "!resgatar" if enabled("points")=>{let index=positive(args.get(1).copied())? as usize-1;let item=s.shop.get(index).ok_or("Item não encontrado. Use !loja")?.clone();if item.stock<1{return Err("Item esgotado".into())}balance(&mut s,uid,-item.cost)?;s.shop[index].stock-=1;s.redemptions.push(json!({"user":e.user,"item":item.name,"at":now}));Some(format!("{} resgatou {}!",e.user,item.name))},
 "!fila" if enabled("queue")=>Some(format!("Fila: {}",s.queue.iter().map(|u|s.names.get(u).unwrap_or(u).clone()).collect::<Vec<_>>().join(", "))),
 "!entrar" if enabled("queue")=>{if !s.queue.contains(uid){s.queue.push(uid.clone())}Some(format!("{}, posição {} na fila.",e.user,s.queue.iter().position(|u|u==uid).unwrap()+1))},
 "!sair" if enabled("queue")=>{s.queue.retain(|u|u!=uid);Some(format!("{} saiu da fila.",e.user))},
 "!musica" if enabled("songs")=>{
 let link=args.get(1).ok_or("Use !musica com um link do YouTube ou Spotify")?;
 let url=url::Url::parse(link).map_err(|_|"Link inválido")?;
 if url.scheme()!="https"||!["youtube.com","www.youtube.com","youtu.be","open.spotify.com"].contains(&url.host_str().unwrap_or("")){return Err("Use um link HTTPS do YouTube ou Spotify".into())}
 if s.songs.len()>=songs["limit"].as_u64().unwrap_or(50).clamp(1,500) as usize||s.songs.iter().filter(|x|x.user==*uid).count()>=songs["perUser"].as_u64().unwrap_or(2).clamp(1,20) as usize{return Err("Limite da fila de músicas atingido".into())}
 s.songs.push(Song{user:uid.clone(),url:url.to_string()});Some(format!("Música adicionada na posição {}.",s.songs.len()))
 },
 "!discord" if enabled("discord")=>{let cfg=rt.db.module(&p.id,"discord");let link=cfg["invite"].as_str().ok_or("O convite do Discord ainda não foi configurado")?;let parsed=url::Url::parse(link).map_err(|_|"Convite inválido")?;if parsed.scheme()!="https"||!["discord.gg","discord.com"].contains(&parsed.host_str().unwrap_or("")){return Err("Use um convite HTTPS oficial do Discord".into())}Some(format!("Entre na comunidade: {link}"))},
 "!musicas" if enabled("songs")=>Some(if s.songs.is_empty(){"A fila de músicas está vazia.".into()}else{s.songs.iter().take(3).enumerate().map(|(i,s)|format!("{}. {}",i+1,s.url)).collect::<Vec<_>>().join(" · ")}),
 "!minhamusica" if enabled("songs")=>Some(s.songs.iter().position(|s|s.user==*uid).map(|n|format!("Sua música está na posição {}.",n+1)).unwrap_or("Você não tem música na fila.".into())),
 "!removermusica" if enabled("songs")=>{s.songs.retain(|x|x.user!=*uid);Some("Seus pedidos foram removidos.".into())},
 "!bet" if enabled("predictions")=>{
 if !s.prediction.open{return Err("Não há previsão aberta".into())}let option=positive(args.get(1).copied())? as usize-1;let n=positive(args.get(2).copied())?;
 if option>=s.prediction.options.len()||s.prediction.bets.contains_key(uid){return Err("Opção inválida ou aposta já registrada".into())}
 balance(&mut s,uid,-n)?;s.prediction.bets.insert(uid.clone(),(option,n));Some("Previsão registrada!".into())
 },
 "!roleta" if enabled("games")=>{
 if now-s.game_at.get(uid).copied().unwrap_or(0)<30{return Err("Espere 30 segundos antes de jogar novamente".into())}
 let n=positive(args.get(1).copied())?;balance(&mut s,uid,-n)?;
 let win=rand::thread_rng().gen_bool(0.5);if win{balance(&mut s,uid,n*2)?;}s.game_at.insert(uid.clone(),now);Some(if win{format!("{} ganhou {n} pontos!",e.user)}else{format!("{} perdeu {n} pontos.",e.user)})
 },
 "!bingo" if enabled("games")=>{
 if s.bingo["open"]!=true{return Err("Não há bingo aberto".into())}
 if s.bingo["cards"].get(uid).is_none(){use rand::seq::SliceRandom;let mut emotes:Vec<String>=serde_json::from_value(s.bingo["emotes"].clone()).map_err(|_|"Bingo inválido")?;emotes.shuffle(&mut rand::thread_rng());s.bingo["cards"][uid]=json!(emotes.into_iter().take(3).collect::<Vec<_>>());}
 let card=s.bingo["cards"][uid].as_array().unwrap().iter().filter_map(|v|v.as_str()).collect::<Vec<_>>().join(" · ");
 Some(format!("{}, sua cartela: {card}. Os emotes são marcados quando aparecem no chat!",e.user))
 },
 "!duelo" if enabled("games")=>{
 let name=args.get(1).ok_or("Use !duelo usuário pontos")?.trim_start_matches('@');
 let target=s.names.iter().find(|(_,n)|n.eq_ignore_ascii_case(name)).map(|(u,_)|u.clone()).ok_or("Usuário não encontrado")?;
 if target==*uid||s.duel["open"]==true{return Err("Já existe um duelo aberto ou o alvo é você".into())}
 let n=positive(args.get(2).copied())?;if s.balances.get(uid).copied().unwrap_or(0)<n{return Err("Saldo insuficiente".into())}
 s.duel=json!({"from":uid,"to":target,"amount":n,"open":true,"expires":now+60});Some(format!("{name}, digite !aceitar em até 60 segundos para o duelo de {n} pontos."))
 },
 "!aceitar" if enabled("games")=>{
 if s.duel["open"]!=true||s.duel["to"]!=*uid||s.duel["expires"].as_i64().unwrap_or(0)<now{return Err("Nenhum desafio válido".into())}
 let from=s.duel["from"].as_str().unwrap().to_owned();let n=s.duel["amount"].as_i64().unwrap();balance(&mut s,&from,-n)?;balance(&mut s,uid,-n)?;
 let winner=if rand::thread_rng().gen_bool(0.5){from}else{uid.clone()};balance(&mut s,&winner,n*2)?;s.duel=json!(null);Some(format!("{} venceu o duelo!",s.names.get(&winner).unwrap_or(&winner)))
 },
 _=>None
 };
 let mut reply=reply;
 if enabled("raffles")&&s.raffle.open && (cmd==s.raffle.keyword||cmd=="!ticket"){
 let n=if s.raffle.price==0{1}else{positive(args.get(1).copied().or(Some("1")))?};
 if s.raffle.price==0&&s.raffle.entries.contains_key(uid){return Ok(None)}
 let cost=n*s.raffle.price;balance(&mut s,uid,-cost)?;*s.raffle.entries.entry(uid.clone()).or_default()+=n;*s.raffle.weights.entry(uid.clone()).or_default()+=n*if permitted("subscriber",&e.role){2}else{1};reply=Some(format!("{} entrou no sorteio!",e.user));
 }
 if enabled("games")&&s.trivia["open"]==true&&e.message.trim().eq_ignore_ascii_case(s.trivia["answer"].as_str().unwrap_or("")){
 let reward=s.trivia["reward"].as_i64().unwrap_or(50);balance(&mut s,uid,reward)?;s.trivia["open"]=json!(false);reply=Some(format!("{} acertou e ganhou {reward} pontos!",e.user));
 }
 if enabled("games")&&s.bingo["open"]==true {
 let emotes:Vec<String>=serde_json::from_value(s.bingo["emotes"].clone()).unwrap_or_default();
 let mut drawn:Vec<String>=serde_json::from_value(s.bingo["drawn"].clone()).unwrap_or_default();
 for emote in emotes{if e.message.split_whitespace().any(|word|word==emote)&&!drawn.contains(&emote){drawn.push(emote);}}
 s.bingo["drawn"]=json!(drawn);
 let cards:BTreeMap<String,Vec<String>>=serde_json::from_value(s.bingo["cards"].clone()).unwrap_or_default();
 for(user,card)in cards{if card.iter().all(|x|drawn.contains(x)){
 balance(&mut s,&user,50)?;s.bingo["open"]=json!(false);s.bingo["winner"]=json!(user);
 reply=Some(format!("Bingo! {} completou a cartela e ganhou 50 pontos.",s.names.get(&user).unwrap_or(&user)));break;
 }}
 }
 if enabled("affinity")&&reply.is_none(){reply=affinity_reply(rt,&p,&e,&mut s)?;}
 rt.db.set_module(&p.id,"community",&serde_json::to_value(&s).unwrap())?;rt.emit("community",json!({"profileId":p.id}));if cmd=="!musica"||cmd=="!removermusica"{rt.emit("songs",json!({"profileId":p.id,"queue":s.songs}));}Ok(reply)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn no_negative_or_overflow(){let mut s=Community::default();assert_eq!(balance(&mut s,"a",20).unwrap(),20);assert!(balance(&mut s,"a",-21).is_err());assert_eq!(s.balances["a"],20);assert!(balance(&mut s,"a",i64::MAX).is_err());}
 #[test]fn match_palavra_inteira_nao_pega_o_meio_da_palavra(){
  assert!(affinity_hit(AFFINITY_TERMS,"você é meu amor!"));
  assert!(affinity_hit(AFFINITY_TERMS,"AMOR e paixão"));
  assert!(affinity_hit("paixão","que paixão, gente"));
  assert!(!affinity_hit("amor","vamos de amortecedor"));
  assert!(!affinity_hit(" , ","qualquer frase"));
 }
 #[test]fn match_do_casal_e_deterministico_e_simetrico(){
  assert_eq!(pair_pct("ana","bruno"),pair_pct("Bruno","ANA"),"o resultado não depende da ordem");
  assert_eq!(pair_pct("ana","bruno"),pair_pct("ana","bruno"));
  // Trava o valor conhecido: trocar o hash mudaria o resultado de cada casal.
  assert_eq!(pair_pct("ana","bruno"),24,"valor de regressa do par ana/bruno");
  assert_eq!(pair_pct("a","b"),42,"valor de regressa do par a/b");
 }
 #[test]fn match_faixas_cobrem_todo_o_intervalo(){
  for (pct,expected)in [(0,0),(20,0),(21,1),(40,1),(41,2),(60,2),(61,3),(80,3),(81,4),(100,4)]{assert_eq!(band(pct),expected,"pct {pct}");}
 }
 #[test]fn match_mensagem_usa_padrao_quando_a_linha_esta_vazia(){
  let padrao=|n:usize,pct:u8|AFFINITY_MESSAGES.lines().nth(n).unwrap().replace("{{user1}}","Ana").replace("{{user2}}","Bruno").replace("{{pct}}",&pct.to_string());
  assert_eq!(affinity_message(&json!({}),"Ana","Bruno",50),padrao(2,50));
  let cfg=json!({"messages":"Primeira {{pct}}\n\nTerceira {{user1}} e {{user2}}"});
  assert_eq!(affinity_message(&cfg,"Ana","Bruno",10),"Primeira 10");
  assert_eq!(affinity_message(&cfg,"Ana","Bruno",30),padrao(1,30),"linha vazia cai no padrão");
  assert_eq!(affinity_message(&cfg,"Ana","Bruno",50),"Terceira Ana e Bruno");
 }
 #[test]fn match_config_valida_confere_termos_comando_e_faixas(){
  assert!(valid_config(&json!({})),"sem nada, valem os padrões");
  assert!(valid_config(&json!({"terms":"amor, casal","command":"!casal"})));
  assert!(!valid_config(&json!({"terms":" "})));
  assert!(!valid_config(&json!({"terms":"amor","command":"casal"})),"comando precisa do !");
  assert!(!valid_config(&json!({"terms":"amor","command":"!dois nomes"})),"comando não tem espaço");
  assert!(!valid_config(&json!({"terms":"amor","messages":"x".repeat(501)})),"faixa de 500 caracteres");
  assert_eq!(with_defaults(&json!({}))["messages"].as_str().unwrap(),AFFINITY_MESSAGES);
 }
}
