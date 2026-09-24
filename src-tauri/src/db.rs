use crate::model::*;
use rusqlite::{params, Connection};
use serde_json::{json,Value};
use std::{path::Path,sync::Mutex};
pub struct Db(pub Mutex<Connection>);
type R<T> = Result<T,String>;
impl Db {
 pub fn open(path:&Path)->R<Self> {
 let c=Connection::open(path).map_err(|e|e.to_string())?;
 c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
 CREATE TABLE IF NOT EXISTS profiles(id TEXT PRIMARY KEY,data TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS flows(id TEXT PRIMARY KEY,profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,data TEXT NOT NULL);
 CREATE INDEX IF NOT EXISTS flows_profile ON flows(profile_id);
 CREATE TABLE IF NOT EXISTS command_counters(flow_id TEXT PRIMARY KEY REFERENCES flows(id) ON DELETE CASCADE,value INTEGER NOT NULL DEFAULT 0 CHECK(value>=0));
 CREATE TABLE IF NOT EXISTS logs(id INTEGER PRIMARY KEY AUTOINCREMENT,profile_id TEXT NOT NULL,timestamp TEXT NOT NULL,kind TEXT NOT NULL,message TEXT NOT NULL,status TEXT NOT NULL);
 CREATE INDEX IF NOT EXISTS logs_profile ON logs(profile_id,id);
 CREATE TABLE IF NOT EXISTS presets(id TEXT PRIMARY KEY,data TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS kv(key TEXT PRIMARY KEY,data TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS module_state(profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,key TEXT NOT NULL,data TEXT NOT NULL,PRIMARY KEY(profile_id,key));
 CREATE TABLE IF NOT EXISTS points(profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,user_id TEXT NOT NULL,balance INTEGER NOT NULL CHECK(balance>=0),PRIMARY KEY(profile_id,user_id));
 CREATE TABLE IF NOT EXISTS variables(profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,scope TEXT NOT NULL,user_id TEXT NOT NULL,name TEXT NOT NULL,data TEXT NOT NULL,PRIMARY KEY(profile_id,scope,user_id,name));
 CREATE TEMP TABLE session_variables(profile_id TEXT NOT NULL,scope TEXT NOT NULL,user_id TEXT NOT NULL,name TEXT NOT NULL,data TEXT NOT NULL,PRIMARY KEY(profile_id,scope,user_id,name));
 PRAGMA user_version=3;").map_err(|e|e.to_string())?;
 Ok(Self(Mutex::new(c)))
 }
 pub fn profiles(&self)->R<Vec<Profile>> {
 let c=self.0.lock().unwrap();let mut s=c.prepare("SELECT data FROM profiles ORDER BY rowid").map_err(|e|e.to_string())?;
 let rows=s.query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?;
 rows.map(|r|serde_json::from_str(&r.map_err(|e|e.to_string())?).map_err(|e|e.to_string())).collect()
 }
 pub fn profile(&self,id:&str)->R<Profile> {
 let c=self.0.lock().unwrap();
 let s:String=c.query_row("SELECT data FROM profiles WHERE id=?",[id],|r|r.get(0)).map_err(|_|"Perfil não encontrado")?;
 serde_json::from_str(&s).map_err(|e|e.to_string())
 }
 pub fn save_profile(&self,p:&Profile)->R<()> {
 if !valid_id(&p.id)||p.name.trim().is_empty()||!["twitch","youtube","kick"].contains(&p.platform.as_str()) {return Err("Preencha um nome e uma plataforma válida".into())}
 self.0.lock().unwrap().execute("INSERT INTO profiles VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![p.id,serde_json::to_string(p).unwrap()]).map_err(|e|e.to_string())?;Ok(())
 }
 pub fn delete_profile(&self,id:&str)->R<()> {
 let mut c=self.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
 tx.execute("DELETE FROM logs WHERE profile_id=?",[id]).map_err(|e|e.to_string())?;
 tx.execute("DELETE FROM profiles WHERE id=?",[id]).map_err(|e|e.to_string())?;
 tx.execute("DELETE FROM session_variables WHERE profile_id=?",[id]).map_err(|e|e.to_string())?;
 tx.commit().map_err(|e|e.to_string())
 }
 pub fn flows(&self,id:&str)->R<Vec<Flow>> {
 let c=self.0.lock().unwrap(); let mut s=c.prepare("SELECT data FROM flows WHERE profile_id=? ORDER BY rowid").map_err(|e|e.to_string())?;
 let rows=s.query_map([id],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?;
 rows.map(|r|serde_json::from_str(&r.map_err(|e|e.to_string())?).map_err(|e|e.to_string())).collect()
 }
 pub fn save_flow(&self,f:&Flow)->R<()> {
 validate_flow(f)?;
 let c=self.0.lock().unwrap();
 let owner:Option<String>=c.query_row("SELECT profile_id FROM flows WHERE id=?",[&f.id],|r|r.get(0)).ok();
 if owner.is_some_and(|x| x!=f.profile_id) {return Err("Este fluxo pertence a outro perfil".into())}
 c.execute("INSERT INTO flows VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![f.id,f.profile_id,serde_json::to_string(f).unwrap()]).map_err(|e|e.to_string())?;Ok(())
 }
 pub fn delete_flow(&self,profile:&str,id:&str)->R<()> {self.0.lock().unwrap().execute("DELETE FROM flows WHERE id=? AND profile_id=?",params![id,profile]).map_err(|e|e.to_string())?;Ok(())}
 pub fn log(&self,p:&str,kind:&str,message:&str,status:&str)->Log {
 let time=chrono::Utc::now().to_rfc3339();let c=self.0.lock().unwrap();
 let message:String=message.chars().take(2000).collect();
 let _=c.execute("INSERT INTO logs(profile_id,timestamp,kind,message,status) VALUES(?,?,?,?,?)",params![p,time,kind,message,status]);
 let id=c.last_insert_rowid();
 if id%100==0 {let _=c.execute("DELETE FROM logs WHERE id < (SELECT MAX(id)-10000 FROM logs)",[]);}
 Log{id,profile_id:p.into(),timestamp:time,kind:kind.into(),message,status:status.into()}
 }
 pub fn logs(&self,p:&str)->R<Vec<Log>> {
 let c=self.0.lock().unwrap();let mut s=c.prepare("SELECT id,profile_id,timestamp,kind,message,status FROM logs WHERE (?1='' OR profile_id=?1) ORDER BY id DESC LIMIT 300").map_err(|e|e.to_string())?;
 let r=s.query_map([p],|r|Ok(Log{id:r.get(0)?,profile_id:r.get(1)?,timestamp:r.get(2)?,kind:r.get(3)?,message:r.get(4)?,status:r.get(5)?})).map_err(|e|e.to_string())?;
 r.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
 }
 pub fn get(&self,key:&str)->Value {self.0.lock().unwrap().query_row("SELECT data FROM kv WHERE key=?",[key],|r|r.get::<_,String>(0)).ok().and_then(|s|serde_json::from_str(&s).ok()).unwrap_or(Value::Null)}
 pub fn set(&self,key:&str,v:&Value)->R<()> {self.0.lock().unwrap().execute("INSERT INTO kv VALUES(?,?) ON CONFLICT(key) DO UPDATE SET data=excluded.data",params![key,v.to_string()]).map_err(|e|e.to_string())?;Ok(())}
 pub fn module(&self,p:&str,key:&str)->Value {self.0.lock().unwrap().query_row("SELECT data FROM module_state WHERE profile_id=? AND key=?",params![p,key],|r|r.get::<_,String>(0)).ok().and_then(|s|serde_json::from_str(&s).ok()).unwrap_or(Value::Null)}
 pub fn set_module(&self,p:&str,key:&str,v:&Value)->R<()> {self.0.lock().unwrap().execute("INSERT INTO module_state VALUES(?,?,?) ON CONFLICT(profile_id,key) DO UPDATE SET data=excluded.data",params![p,key,v.to_string()]).map_err(|e|e.to_string())?;Ok(())}
 pub fn points(&self,p:&str,user:&str,delta:i64)->R<i64> {
 let mut c=self.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
 let old:i64=tx.query_row("SELECT balance FROM points WHERE profile_id=? AND user_id=?",params![p,user],|r|r.get(0)).unwrap_or(0);
 let new=old.checked_add(delta).filter(|n|*n>=0).ok_or("Saldo insuficiente ou valor inválido")?;
 tx.execute("INSERT INTO points VALUES(?,?,?) ON CONFLICT(profile_id,user_id) DO UPDATE SET balance=excluded.balance",params![p,user,new]).map_err(|e|e.to_string())?;
 tx.commit().map_err(|e|e.to_string())?;Ok(new)
 }
 pub fn ranking(&self,p:&str)->R<Value> {
 let c=self.0.lock().unwrap();let mut s=c.prepare("SELECT user_id,balance FROM points WHERE profile_id=? ORDER BY balance DESC LIMIT 100").map_err(|e|e.to_string())?;
 let rows=s.query_map([p],|r|Ok(json!({"user":r.get::<_,String>(0)?,"balance":r.get::<_,i64>(1)?}))).map_err(|e|e.to_string())?;
 Ok(Value::Array(rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?))
 }
}
#[cfg(test)] mod tests {
 use super::*;
 fn p()->Profile {Profile{id:uuid::Uuid::new_v4().to_string(),name:"Teste".into(),platform:"twitch".into(),channel:"teste".into(),channel_id:"".into(),bot_id:"".into(),client_id:"".into(),blocklist:vec![],topics:vec![],editors:vec![],ai:AiConfig::default(),modules:Value::Null}}
 #[test] fn isolation_and_points() {
 let d=Db::open(Path::new(":memory:")).unwrap();let a=p();let b=p();d.save_profile(&a).unwrap();d.save_profile(&b).unwrap();
 assert_eq!(d.points(&a.id,"ana",10).unwrap(),10);assert_eq!(d.points(&b.id,"ana",0).unwrap(),0);assert!(d.points(&a.id,"ana",-11).is_err());
 d.delete_profile(&a.id).unwrap();assert_eq!(d.profiles().unwrap().len(),1);assert!(d.profile(&b.id).is_ok());
 }
}
