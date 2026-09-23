use crate::engine::Runtime;
use serde_json::{Value,json};
pub fn read(rt:&Runtime,p:&str)->Result<Value,String>{
 rt.db.profile(p)?;let c=rt.db.0.lock().unwrap();
 let count=|kind:&str,status:&str|->i64{c.query_row("SELECT COUNT(*) FROM logs WHERE profile_id=?1 AND kind=?2 AND status=?3",rusqlite::params![p,kind,status],|r|r.get(0)).unwrap_or(0)};
 let mut s=c.prepare("SELECT substr(timestamp,1,13) || ':00', COUNT(*) FROM logs WHERE profile_id=?1 AND kind='chat' AND status='info' GROUP BY substr(timestamp,1,13) ORDER BY 1 DESC LIMIT 24").map_err(|e|e.to_string())?;
 let hours=s.query_map([p],|r|Ok(json!({"hour":r.get::<_,String>(0)?,"count":r.get::<_,i64>(1)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
 let mut s=c.prepare("SELECT message,COUNT(*) FROM logs WHERE profile_id=?1 AND kind='flow' GROUP BY message ORDER BY 2 DESC LIMIT 10").map_err(|e|e.to_string())?;
 let commands=s.query_map([p],|r|Ok(json!({"name":r.get::<_,String>(0)?.trim_start_matches("Iniciando "),"count":r.get::<_,i64>(1)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
 Ok(json!({"messages":count("chat","info"),"actions":count("action","success"),"followers":count("follow","info"),"hours":hours,"commands":commands}))
}
