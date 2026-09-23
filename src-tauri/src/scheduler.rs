use crate::{engine::Runtime,model::Event,modules};
use serde_json::{json,Value};
use std::{sync::Arc,time::Duration};
pub async fn run(rt:Arc<Runtime>){
 let mut tick=tokio::time::interval(Duration::from_secs(30));
 loop{
 tick.tick().await;
 for p in rt.db.profiles().unwrap_or_default(){
 if p.modules["games"]!=true||rt.statuses.lock().unwrap().get(&p.id).map(String::as_str)!=Some("online"){continue}
 let config=rt.db.module(&p.id,"games");
 if config["autoTrivia"]!=true{continue}
 let now=chrono::Utc::now().timestamp();
 let question={
 let _lock=rt.module_lock.lock().unwrap();let mut state=modules::load(&rt,&p.id);
 if state.trivia["open"]==true&&state.trivia["expires"].as_i64().is_some_and(|t|t<=now){state.trivia["open"]=json!(false);let _=rt.db.set_module(&p.id,"community",&serde_json::to_value(&state).unwrap());}
 let idle=config["idleSeconds"].as_i64().unwrap_or(300).clamp(60,3600);
 let interval=config["intervalSeconds"].as_i64().unwrap_or(900).clamp(300,86400);
 if state.trivia["open"]==true||state.last_chat==0||now-state.last_chat<idle||now-state.last_trivia<interval{None}else{
 let lines=config["questions"].as_str().unwrap_or("");
 let questions:Vec<_>=lines.lines().filter_map(|l|l.split_once('|')).filter(|(q,a)|!q.trim().is_empty()&&!a.trim().is_empty()).collect();
 if questions.is_empty(){None}else{
 use rand::Rng;let(q,a)=questions[rand::thread_rng().gen_range(0..questions.len())];
 state.trivia=json!({"open":true,"question":q.trim(),"answer":a.trim(),"reward":50,"expires":now+300});state.last_trivia=now;
 match rt.db.set_module(&p.id,"community",&serde_json::to_value(&state).unwrap()){Ok(())=>Some(q.trim().to_owned()),Err(_)=>None}
 }
 }
 };
 if let Some(question)=question{
 let event=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),kind:"timer".into(),user:String::new(),user_id:String::new(),role:"broadcaster".into(),message:String::new(),data:Value::Null,simulated:false};
 if let Err(err)=rt.send(&p,&event,&format!("Trivia: {question} — a primeira resposta correta vale 50 pontos!")).await{
 rt.log(&p.id,"trivia",&err,"error");
 let _lock=rt.module_lock.lock().unwrap();let mut state=modules::load(&rt,&p.id);state.trivia["open"]=json!(false);let _=rt.db.set_module(&p.id,"community",&serde_json::to_value(state).unwrap());
 }
 rt.emit("community",json!({"profileId":p.id}));
 }
 }
 }
}
