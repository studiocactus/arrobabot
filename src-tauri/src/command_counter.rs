use crate::{db::Db,model::Flow};
use rusqlite::{params,OptionalExtension};
use serde_json::{json,Value};
const MAX:i64=9_007_199_254_740_991;
pub fn get(db:&Db,profile:&str,id:&str)->Result<i64,String>{db.0.lock().unwrap().query_row("SELECT c.value FROM command_counters c JOIN flows f ON f.id=c.flow_id WHERE f.profile_id=? AND f.id=?",params![profile,id],|r|r.get(0)).optional().map(|n|n.unwrap_or(0)).map_err(|e|e.to_string())}
pub fn change(db:&Db,profile:&str,id:&str,set:Option<i64>)->Result<i64,String>{
 let mut c=db.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
 let data:String=tx.query_row("SELECT data FROM flows WHERE id=? AND profile_id=?",params![id,profile],|r|r.get(0)).map_err(|_|"Comando não encontrado neste perfil")?;
 let flow:Flow=serde_json::from_str(&data).map_err(|_|"Comando inválido")?;
 if flow.trigger.kind!="command"||!flow.counter{return Err("Ative o contador deste comando antes de alterar".into())}
 let old:i64=tx.query_row("SELECT value FROM command_counters WHERE flow_id=?",[id],|r|r.get(0)).optional().map_err(|e|e.to_string())?.unwrap_or(0);
 let next=set.unwrap_or_else(||old.saturating_add(1));if !(0..=MAX).contains(&next){return Err("Contador fora do intervalo permitido".into())}
 tx.execute("INSERT INTO command_counters VALUES(?,?) ON CONFLICT(flow_id) DO UPDATE SET value=excluded.value",params![id,next]).map_err(|e|e.to_string())?;tx.commit().map_err(|e|e.to_string())?;Ok(next)
}
pub fn list(db:&Db,profile:&str)->Result<Value,String>{let mut out=serde_json::Map::new();for f in db.flows(profile)?{if f.trigger.kind=="command"{out.insert(f.id.clone(),json!(get(db,profile,&f.id)?));}}Ok(Value::Object(out))}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn persistent_atomic_isolated_and_deleted_with_command(){
 let dir=tempfile::tempdir().unwrap();let path=dir.path().join("test.sqlite");let db=std::sync::Arc::new(Db::open(&path).unwrap());
 let p:crate::model::Profile=serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal"})).unwrap();db.save_profile(&p).unwrap();
 let f:Flow=serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"profileId":p.id,"name":"Mortes","enabled":true,"counter":true,"trigger":{"kind":"command","pattern":"!mortes"},"actions":[{"kind":"chat"}]})).unwrap();db.save_flow(&f).unwrap();
 let workers:Vec<_>=(0..8).map(|_|{let db=db.clone();let f=f.clone();std::thread::spawn(move||{for _ in 0..20{change(&db,&f.profile_id,&f.id,None).unwrap();}})}).collect();for worker in workers{worker.join().unwrap();}assert_eq!(get(&db,&p.id,&f.id).unwrap(),160);
 assert!(change(&db,"outro",&f.id,Some(0)).is_err());assert!(change(&db,&p.id,&f.id,Some(-1)).is_err());assert_eq!(change(&db,&p.id,&f.id,Some(12)).unwrap(),12);drop(db);
 let db=Db::open(&path).unwrap();assert_eq!(get(&db,&p.id,&f.id).unwrap(),12);db.delete_flow(&p.id,&f.id).unwrap();assert_eq!(get(&db,&p.id,&f.id).unwrap(),0);
 }
}
