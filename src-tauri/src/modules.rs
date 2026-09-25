use crate::{engine::Runtime,model::*};
use serde::{Serialize,Deserialize};
use serde_json::{json,Value};
use std::collections::BTreeMap;
use rand::Rng;
#[derive(Default,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Community {
 #[serde(default)] pub last_chat:i64,
 #[serde(default)] pub last_trivia:i64,
 #[serde(default)] pub balances:BTreeMap<String,i64>,
 #[serde(default)] pub names:BTreeMap<String,String>,
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
pub fn load(rt:&Runtime,p:&str)->Community {serde_json::from_value(rt.db.module(p,"community")).unwrap_or_default()}
/// Sorteia um nome entre quem já falou no chat deste perfil. Sem ninguém no cadastro, não há valor.
pub fn random_chatter(db:&crate::db::Db,p:&str)->Option<String>{
 let state:Community=serde_json::from_value(db.module(p,"community")).unwrap_or_default();
 let names:Vec<&String>=state.names.values().filter(|n|!n.trim().is_empty()).collect();
 if names.is_empty(){None}else{Some(names[rand::thread_rng().gen_range(0..names.len())].clone())}
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
 let uid=&e.user_id;s.names.insert(uid.clone(),e.user.clone());
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
 rt.db.set_module(&p.id,"community",&serde_json::to_value(&s).unwrap())?;rt.emit("community",json!({"profileId":p.id}));if cmd=="!musica"||cmd=="!removermusica"{rt.emit("songs",json!({"profileId":p.id,"queue":s.songs}));}Ok(reply)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn no_negative_or_overflow(){let mut s=Community::default();assert_eq!(balance(&mut s,"a",20).unwrap(),20);assert!(balance(&mut s,"a",-21).is_err());assert_eq!(s.balances["a"],20);assert!(balance(&mut s,"a",i64::MAX).is_err());}
}
