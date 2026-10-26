//! Etapas compartilhadas por perfil, resolvidas antes de qualquer efeito.
use crate::{db::Db,model::{Action,Flow,Trigger,validate_flow,valid_id}};
use serde::{Serialize,Deserialize};use serde_json::json;
const KEY:&str="actionLibrary";
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Entry {pub id:String,pub name:String,pub action:Action}
pub fn list(db:&Db,p:&str)->Result<Vec<Entry>,String>{
 db.profile(p)?;let value=db.module(p,KEY);if value.is_null(){return Ok(vec![])}serde_json::from_value(value).map_err(|_|"Biblioteca de etapas inválida".into())
}
fn valid(entry:&Entry,p:&str)->Result<(),String>{
 if !valid_id(&entry.id)||entry.name.trim().is_empty()||entry.name.chars().count()>80{return Err("Dê um nome de até 80 caracteres à etapa".into())}
 if entry.action.kind=="action.ref"{return Err("Uma etapa reutilizável não pode chamar outra referência".into())}
 let flow=Flow{id:uuid::Uuid::new_v4().to_string(),profile_id:p.into(),name:entry.name.clone(),trigger:Trigger{kind:"chat".into(),..Default::default()},actions:vec![entry.action.clone()],..Default::default()};validate_flow(&flow)
}
pub fn save(db:&Db,p:&str,entry:Entry)->Result<Entry,String>{
 db.profile(p)?;valid(&entry,p)?;let mut c=db.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
 let raw:Option<String>=tx.query_row("SELECT data FROM module_state WHERE profile_id=? AND key=?",[p,KEY],|r|r.get(0)).ok();
 let mut entries:Vec<Entry>=match raw{Some(raw)=>serde_json::from_str(&raw).map_err(|_|"Biblioteca de etapas inválida")?,None=>vec![]};
 entries.retain(|e|e.id!=entry.id);if entries.len()>=64{return Err("Limite de 64 etapas reutilizáveis por perfil".into())}entries.push(entry.clone());
 tx.execute("INSERT INTO module_state VALUES(?,?,?) ON CONFLICT(profile_id,key) DO UPDATE SET data=excluded.data",rusqlite::params![p,KEY,serde_json::to_string(&entries).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;tx.commit().map_err(|e|e.to_string())?;Ok(entry)
}
pub fn remove(db:&Db,p:&str,id:&str)->Result<(),String>{
 db.profile(p)?;let mut c=db.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
 {let mut stmt=tx.prepare("SELECT data FROM flows WHERE profile_id=?").map_err(|e|e.to_string())?;let rows=stmt.query_map([p],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?;
  for row in rows{let flow:Flow=serde_json::from_str(&row.map_err(|e|e.to_string())?).map_err(|_|"Fluxo inválido")?;if flow.actions.iter().any(|a|a.kind=="action.ref"&&a.target==id){return Err("Esta etapa é usada por um fluxo salvo; remova as referências antes de excluir".into())}}
 }
 let raw:String=tx.query_row("SELECT data FROM module_state WHERE profile_id=? AND key=?",[p,KEY],|r|r.get(0)).map_err(|_|"Etapa não encontrada")?;
 let mut entries:Vec<Entry>=serde_json::from_str(&raw).map_err(|_|"Biblioteca inválida")?;entries.retain(|e|e.id!=id);
 tx.execute("UPDATE module_state SET data=? WHERE profile_id=? AND key=?",rusqlite::params![serde_json::to_string(&entries).map_err(|e|e.to_string())?,p,KEY]).map_err(|e|e.to_string())?;tx.commit().map_err(|e|e.to_string())
}
fn resolve_action(action:&Action,entries:&[Entry],p:&str)->Result<Action,String>{
 if action.kind!="action.ref"{return Ok(action.clone())}
 if !action.enabled{return Ok(Action{kind:"delay".into(),enabled:false,..Default::default()})}
 let entry=entries.iter().find(|e|e.id==action.target).ok_or("Etapa reutilizável não encontrada neste perfil; confira a biblioteca antes de executar")?;
 valid(entry,p)?;let mut resolved=entry.action.clone();resolved.enabled=action.enabled&&resolved.enabled;Ok(resolved)
}
pub fn resolve(db:&Db,flow:&Flow)->Result<Flow,String>{
 if !flow.actions.iter().any(|a|a.kind=="action.ref"){return Ok(flow.clone())}
 let entries=list(db,&flow.profile_id)?;let mut out=flow.clone();out.actions=flow.actions.iter().map(|a|resolve_action(a,&entries,&flow.profile_id)).collect::<Result<Vec<_>,_>>()?;
 if let Some(nodes)=out.layout["nodes"].as_array_mut(){for node in nodes {if node["data"]["action"]["kind"]=="action.ref"{let a:Action=serde_json::from_value(node["data"]["action"].clone()).map_err(|_|"Referência inválida no editor")?;node["data"]["action"]=json!(resolve_action(&a,&entries,&flow.profile_id)?);}}}
 Ok(out)
}

#[cfg(test)]mod tests{
 use super::*;
 fn setup()->(tempfile::TempDir,Db,String,String){let dir=tempfile::tempdir().unwrap();let db=Db::open(&dir.path().join("qa.sqlite")).unwrap();let make=|name:&str|serde_json::from_value::<crate::model::Profile>(json!({"id":uuid::Uuid::new_v4().to_string(),"name":name,"platform":"twitch","channel":"qa"})).unwrap();let p=make("A");let q=make("B");db.save_profile(&p).unwrap();db.save_profile(&q).unwrap();(dir,db,p.id,q.id)}
 #[test]fn references_are_scoped_frozen_and_never_nested(){
  let (_dir,db,p,q)=setup();let mut entry=Entry{id:uuid::Uuid::new_v4().to_string(),name:"Saudação".into(),action:Action{kind:"overlay".into(),text:"Antes {{user}}".into(),..Default::default()}};save(&db,&p,entry.clone()).unwrap();
  let flow=Flow{id:uuid::Uuid::new_v4().to_string(),profile_id:p.clone(),name:"Principal".into(),actions:vec![Action{kind:"action.ref".into(),target:entry.id.clone(),..Default::default()}],..Default::default()};
  let prepared=resolve(&db,&flow).unwrap();entry.action.text="Depois".into();save(&db,&p,entry.clone()).unwrap();assert_eq!(prepared.actions[0].text,"Antes {{user}}");assert_eq!(resolve(&db,&flow).unwrap().actions[0].text,"Depois");
  let mut other=flow.clone();other.profile_id=q;assert!(resolve(&db,&other).is_err());
  entry.action=flow.actions[0].clone();assert!(save(&db,&p,entry).unwrap_err().contains("outra referência"));
  other.actions[0].enabled=false;assert!(!resolve(&db,&other).unwrap().actions[0].enabled);
  let mut saved=flow.clone();saved.trigger=Trigger{kind:"chat".into(),..Default::default()};db.save_flow(&saved).unwrap();assert!(remove(&db,&p,&saved.actions[0].target).unwrap_err().contains("usada"));db.delete_flow(&p,&saved.id).unwrap();remove(&db,&p,&saved.actions[0].target).unwrap();assert!(list(&db,&p).unwrap().is_empty());
 }
 #[test]fn invalid_reference_conditions_are_rejected(){let (_dir,_db,p,_q)=setup();let flow=Flow{id:uuid::Uuid::new_v4().to_string(),profile_id:p,name:"A".into(),actions:vec![Action{kind:"action.ref".into(),target:uuid::Uuid::new_v4().to_string(),condition:"duplicada".into(),..Default::default()}],..Default::default()};assert!(validate_flow(&flow).unwrap_err().contains("condição no original"));}
}
