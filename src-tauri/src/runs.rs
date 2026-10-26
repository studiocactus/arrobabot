//! Execuções de fluxos com identidade, estado e cancelamento. O motor de
//! actions[] continua o mesmo; aqui vive só o acompanhamento por execução.
use std::{
 collections::HashMap,
 sync::Mutex,
 time::{Instant, SystemTime, UNIX_EPOCH},
};
use sha2::{Digest,Sha256};
use crate::model::Event;
/// Somente metadados permitidos. Nunca copia mensagem, pessoa ou payload.
#[derive(Clone,serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct RunContext {
 pub event_kind: String,
 pub event_ref: Option<String>,
 pub mode: String,
}
impl RunContext {
 fn from_event(e:&Event,test:bool)->Self {
  let kind=match e.kind.as_str(){"chat"|"timer"|"follow"|"subscription"|"resub"|"gift"|"cheer"|"raid"|"redemption"|"custom"|"voice"|"manual"|"live"|"unlive"=>e.kind.as_str(),_=>"other"};
  let mode=if test{"test"}else if e.rehearsal{"rehearsal"}else if e.simulated{"simulation"}else{"live"};
  let event_ref=if e.id.is_empty(){None}else{
   let mut hash=Sha256::new();hash.update((e.profile_id.len() as u64).to_le_bytes());hash.update(e.profile_id.as_bytes());hash.update(e.id.as_bytes());
   Some(format!("evt-{:x}",hash.finalize())[..20].to_owned())
  };
  Self{event_kind:kind.into(),event_ref,mode:mode.into()}
 }
}
#[derive(Clone)]
pub struct Step {
 pub index: usize,
 pub kind: String,
 pub status: String,
 pub started: Option<Instant>,
 pub finished: Option<Instant>,
}
#[derive(Clone)]
pub struct Run {
 pub id: String,
 pub short: String,
 pub profile_id: String,
 pub flow_id: String,
 pub flow_name: String,
 pub started: Instant,
 pub started_at: u64,
 pub finished: Option<Instant>,
 pub status: String,
 pub steps: Vec<Step>,
 pub cancelled: bool,
 pub test: bool,
 /// Motivo da falha, para a tela mostrar junto da etapa ou da execução.
 pub error: Option<String>,
 pub context: Option<RunContext>,
}
#[derive(Default)]
pub struct Runs(pub Mutex<HashMap<String, Run>>,Option<std::sync::Arc<crate::db::Db>>);
fn short(id: &str) -> String {
 id.chars().take(8).collect()
}
impl Run {
 fn snapshot(&self)->serde_json::Value {
  let now=Instant::now();
  serde_json::json!({"id":self.id,"flow":self.flow_name,"status":self.status,"test":self.test,"startedAt":self.started_at,"durationMs":self.finished.unwrap_or(now).saturating_duration_since(self.started).as_millis() as u64,"cancelRequested":self.cancelled,"context":self.context,"steps":self.steps.iter().map(|s|serde_json::json!({"index":s.index,"kind":s.kind,"status":s.status,"durationMs":s.started.map(|start|s.finished.unwrap_or(now).saturating_duration_since(start).as_millis() as u64)})).collect::<Vec<_>>(),"error":self.error})
 }
}
impl Runs {
 pub fn open(db:std::sync::Arc<crate::db::Db>)->Result<Self,String>{
  {
   let mut c=db.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
   let mut records=Vec::new();
   {let mut stmt=tx.prepare("SELECT id,data FROM execution_history WHERE status IN ('RUNNING','WAITING')").map_err(|e|e.to_string())?;
    let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?;
    for row in rows{records.push(row.map_err(|e|e.to_string())?);}
   }
   for (id,data) in records {
    let mut v:serde_json::Value=serde_json::from_str(&data).map_err(|e|e.to_string())?;
    v["status"]=serde_json::json!("INTERRUPTED");v["cancelRequested"]=serde_json::json!(false);
    v["interruptionNote"]=serde_json::json!("O aplicativo encerrou sem confirmar o resultado final. Efeitos externos podem ter ocorrido. Nenhuma ação foi repetida.");
    if let Some(steps)=v["steps"].as_array_mut(){for step in steps{match step["status"].as_str(){Some("RUNNING"|"WAITING")=>step["status"]=serde_json::json!("INTERRUPTED"),Some("PENDING")=>step["status"]=serde_json::json!("NOT_EXECUTED"),_=>{}}}}
    tx.execute("UPDATE execution_history SET status='INTERRUPTED',data=? WHERE id=?",rusqlite::params![v.to_string(),id]).map_err(|e|e.to_string())?;
   }
   tx.execute("DELETE FROM execution_history WHERE id IN (SELECT id FROM (SELECT id,ROW_NUMBER() OVER(PARTITION BY profile_id ORDER BY started_at DESC,rowid DESC) AS n FROM execution_history WHERE status NOT IN ('RUNNING','WAITING')) WHERE n>30)",[]).map_err(|e|e.to_string())?;
   tx.commit().map_err(|e|e.to_string())?;
  }
  Ok(Self(Mutex::new(HashMap::new()),Some(db)))
 }
 fn persist(&self,run:&Run)->Result<(),String>{
  let Some(db)=&self.1 else{return Ok(())};
  let mut c=db.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
  tx.execute("INSERT INTO execution_history(id,profile_id,started_at,status,data) VALUES(?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET status=excluded.status,data=excluded.data",rusqlite::params![run.id,run.profile_id,run.started_at as i64,run.status,run.snapshot().to_string()]).map_err(|e|e.to_string())?;
  tx.execute("DELETE FROM execution_history WHERE profile_id=?1 AND status NOT IN ('RUNNING','WAITING') AND id NOT IN (SELECT id FROM execution_history WHERE profile_id=?1 AND status NOT IN ('RUNNING','WAITING') ORDER BY started_at DESC,rowid DESC LIMIT 30)",[&run.profile_id]).map_err(|e|e.to_string())?;
  tx.commit().map_err(|e|e.to_string())
 }
 pub fn history(&self,profile:&str)->Result<Vec<serde_json::Value>,String>{
  let all=self.0.lock().unwrap();
  // Repetir a gravação aqui recupera falhas transitórias e nunca executa ações.
  for run in all.values().filter(|r|r.profile_id==profile){self.persist(run)?;}
  let Some(db)=&self.1 else{return Ok(all.values().filter(|r|r.profile_id==profile).map(Run::snapshot).collect())};
  let c=db.0.lock().unwrap();
  let mut stmt=c.prepare("SELECT id,data FROM execution_history WHERE profile_id=? ORDER BY started_at DESC,rowid DESC").map_err(|e|e.to_string())?;
  let rows=stmt.query_map([profile],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?;
  let mut out=Vec::new();for row in rows{let (id,data)=row.map_err(|e|e.to_string())?;out.push(if let Some(run)=all.get(&id){run.snapshot()}else{serde_json::from_str(&data).map_err(|e|e.to_string())?});}Ok(out)
 }

 pub fn start(&self, profile: &str, flow_id: &str, flow_name: &str, kinds: &[String], test: bool) -> String {
  let id = uuid::Uuid::new_v4().to_string();
  let run = Run {
   id: id.clone(),
   short: short(&id),
   profile_id: profile.into(),
   flow_id: flow_id.into(),
   flow_name: flow_name.into(),
   started: Instant::now(),
   started_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
   finished: None,
   status: "RUNNING".into(),
   steps: kinds.iter().enumerate().map(|(i, k)| Step { index: i, kind: k.clone(), status: "PENDING".into(), started: None, finished: None }).collect(),
   cancelled: false,
   test,
   error: None,
   context: None,
  };
  let mut all = self.0.lock().unwrap();
  let _=self.persist(&run);
  all.insert(id.clone(), run);
  let over = all.values().filter(|r| r.profile_id == profile).count().saturating_sub(30);
  let mut finished: Vec<(Instant, String)> = all
   .values()
   .filter(|r| r.profile_id == profile && r.status != "RUNNING" && r.status != "WAITING")
   .map(|r| (r.started, r.id.clone()))
   .collect();
  finished.sort();
  for (_, drop) in finished.into_iter().take(over) {
   all.remove(&drop);
  }
  id
 }
 pub fn prepared_steps(&self,id:&str,kinds:&[String]){
  if let Some(run)=self.0.lock().unwrap().get_mut(id){for (step,kind) in run.steps.iter_mut().zip(kinds){step.kind=kind.clone();}let _=self.persist(run);}
 }
 pub fn attach_event(&self,id:&str,e:&Event,test:bool){
  if let Some(run)=self.0.lock().unwrap().get_mut(id){
   if run.profile_id==e.profile_id && run.context.is_none(){
    let context=RunContext::from_event(e,test);
    run.test=context.mode!="live";
    run.context=Some(context);
    let _=self.persist(run);
   }
  }
 }
 pub fn step(&self, id: &str, index: usize, status: &str) {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   if let Some(step) = run.steps.get_mut(index) {
    if matches!(status,"RUNNING"|"WAITING") {step.started.get_or_insert_with(Instant::now);}
    else if step.started.is_some() {step.finished.get_or_insert_with(Instant::now);}
    step.status = status.into();
   }
   if status == "RUNNING" {
    run.status = "RUNNING".into();
   }
   if status == "WAITING" {
    run.status = "WAITING".into();
   }
   let _=self.persist(run);
  }
 }
 /// Marca a etapa como concluída, mas preserva um resultado que a própria ação já
 /// definiu (a condição grava TRUE ou FALSE antes de voltar ao motor).
 pub fn succeed(&self, id: &str, index: usize) {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   if let Some(step) = run.steps.get_mut(index) {
    if step.status == "RUNNING" || step.status == "WAITING" || step.status == "PENDING" {
     step.status = "SUCCESS".into();
     if step.started.is_some(){step.finished.get_or_insert_with(Instant::now);}
    }
   }
   let _=self.persist(run);
  }
 }
 pub fn fail(&self, id: &str, index: Option<usize>, error: &str) {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   if let Some(i) = index {
    if let Some(step) = run.steps.get_mut(i) {
     step.status = "FAILED".into();
     if step.started.is_some(){step.finished.get_or_insert_with(Instant::now);}
    }
   }
   if run.error.is_none() {
    run.error = Some(error.into());
   }
   let _=self.persist(run);
  }
 }
 pub fn cancel(&self, profile: &str, id: &str) -> bool {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   if run.profile_id == profile && (run.status == "RUNNING" || run.status == "WAITING") {
    run.cancelled = true;
    let _=self.persist(run);
    return true;
   }
  }
  false
 }
 pub fn cancelled(&self, id: &str) -> bool {
  self.0.lock().unwrap().get(id).map_or(true,|r| r.cancelled)
 }
 pub fn finish(&self, id: &str, status: &str) {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   run.status = status.into();
   let end=*run.finished.get_or_insert_with(Instant::now);
   for step in &mut run.steps {
    if step.started.is_some() && step.finished.is_none(){step.finished=Some(end);if status=="CANCELLED"{step.status="CANCELLED".into();}}
   }
   let _=self.persist(run);
  }
 }
 pub fn get(&self, id: &str) -> Option<Run> {
  self.0.lock().unwrap().get(id).cloned()
 }
 pub fn list(&self, profile: &str) -> Vec<Run> {
  let mut out: Vec<Run> = self.0.lock().unwrap().values().filter(|r| r.profile_id == profile).cloned().collect();
  out.sort_by_key(|r| std::cmp::Reverse(r.started));
  out.truncate(30);
  out
 }
 /// Importação não pode repersistir registros do conjunto de dados anterior.
 pub fn reset_after<T>(&self,operation:impl FnOnce()->Result<T,String>)->Result<T,String>{
  let mut all=self.0.lock().unwrap();let result=operation()?;all.clear();Ok(result)
 }
 pub fn short_of(&self, id: &str) -> String {
  self.0.lock().unwrap().get(id).map(|r| r.short.clone()).unwrap_or_else(|| short(id))
 }
}
#[cfg(test)]
mod tests {
 use super::*;
 fn run_with(steps: &[&str]) -> (Runs, String) {
  let runs = Runs::default();
  let kinds = steps.iter().map(|s| s.to_string()).collect::<Vec<_>>();
  let id = runs.start("p", "f", "Fluxo", &kinds, true);
  (runs, id)
 }
 #[test]
 fn persisted_history_recovers_without_replaying_or_faking_completion(){
  let dir=tempfile::tempdir().unwrap();let path=dir.path().join("history.sqlite");
  let db=std::sync::Arc::new(crate::db::Db::open(&path).unwrap());
  db.0.lock().unwrap().execute("INSERT INTO profiles VALUES('p','{}')",[]).unwrap();
  db.0.lock().unwrap().execute("INSERT INTO profiles VALUES('q','{}')",[]).unwrap();
  let runs=Runs::open(db.clone()).unwrap();
  let done=runs.start("p","f","Concluído",&["chat".into()],false);runs.step(&done,0,"RUNNING");runs.succeed(&done,0);runs.finish(&done,"COMPLETED");
  let active=runs.start("p","g","Sem confirmação",&["wait".into(),"chat".into()],false);runs.step(&active,0,"WAITING");
  let completed=runs.history("p").unwrap().into_iter().find(|r|r["id"]==done).unwrap();
  assert!(runs.history("q").unwrap().is_empty());drop(runs);drop(db);
  let db=std::sync::Arc::new(crate::db::Db::open(&path).unwrap());let runs=Runs::open(db.clone()).unwrap();
  let history=runs.history("p").unwrap();assert_eq!(history.len(),2);
  assert_eq!(history.iter().find(|r|r["id"]==done).unwrap(),&completed);
  let interrupted=history.iter().find(|r|r["id"]==active).unwrap();
  assert_eq!(interrupted["status"],"INTERRUPTED");assert_eq!(interrupted["steps"][0]["status"],"INTERRUPTED");assert_eq!(interrupted["steps"][1]["status"],"NOT_EXECUTED");
  assert!(interrupted["interruptionNote"].as_str().unwrap().contains("Nenhuma ação foi repetida"));
  assert!(runs.list("p").is_empty(),"histórico nunca volta ao motor");assert!(!runs.cancel("p",&active));
  let snapshot=interrupted.clone();drop(runs);let reopened=Runs::open(db).unwrap();
  assert_eq!(reopened.history("p").unwrap().into_iter().find(|r|r["id"]==active).unwrap(),snapshot);
 }
 #[test]
 fn reset_is_atomic_and_recovery_enforces_retention(){
  let dir=tempfile::tempdir().unwrap();let db=std::sync::Arc::new(crate::db::Db::open(&dir.path().join("history.sqlite")).unwrap());
  db.0.lock().unwrap().execute("INSERT INTO profiles VALUES('p','{}')",[]).unwrap();let runs=Runs::open(db.clone()).unwrap();
  let id=runs.start("p","f","Ainda ativa",&[],false);
  let failed:Result<(),String>=runs.reset_after(||Err("falhou".into()));assert!(failed.is_err());assert!(runs.get(&id).is_some());
  for _ in 0..34{runs.start("p","f","Em andamento",&[],false);}
  assert_eq!(runs.history("p").unwrap().len(),35);
  runs.reset_after(||Ok(())).unwrap();assert!(runs.get(&id).is_none());assert!(runs.cancelled(&id));drop(runs);
  let runs=Runs::open(db).unwrap();let history=runs.history("p").unwrap();assert_eq!(history.len(),30);assert!(history.iter().all(|r|r["status"]=="INTERRUPTED"));
 }
 #[test]
 fn retention_profile_deletion_and_storage_failure_are_explicit(){
  let dir=tempfile::tempdir().unwrap();let db=std::sync::Arc::new(crate::db::Db::open(&dir.path().join("history.sqlite")).unwrap());
  db.0.lock().unwrap().execute("INSERT INTO profiles VALUES('p','{}')",[]).unwrap();
  let runs=Runs::open(db.clone()).unwrap();let active=runs.start("p","f","Ativa",&[],false);
  for _ in 0..40{let id=runs.start("p","f","Fim",&[],false);runs.finish(&id,"COMPLETED");}
  let history=runs.history("p").unwrap();assert_eq!(history.len(),31);assert!(history.iter().any(|r|r["id"]==active));
  db.0.lock().unwrap().execute_batch("CREATE TRIGGER fail_history BEFORE INSERT ON execution_history BEGIN SELECT RAISE(ABORT,'disco indisponivel'); END;").unwrap();
  assert!(runs.history("p").unwrap_err().contains("disco indisponivel"));
  db.0.lock().unwrap().execute_batch("DROP TRIGGER fail_history;").unwrap();assert_eq!(runs.history("p").unwrap().len(),31);
  db.delete_profile("p").unwrap();drop(runs);let runs=Runs::open(db).unwrap();assert!(runs.history("p").unwrap().is_empty());
 }
 #[test]
 fn lifecycle_tracks_steps_and_cancels() {
  let (runs, id) = run_with(&["obs", "wait", "obs"]);
  assert_eq!(runs.get(&id).unwrap().status, "RUNNING");
  runs.step(&id, 0, "SUCCESS");
  runs.step(&id, 1, "WAITING");
  assert_eq!(runs.get(&id).unwrap().status, "WAITING");
  assert!(runs.cancel("p", &id));
  assert!(runs.cancelled(&id));
  assert!(runs.cancel("p", &id), "cancelar de novo continua valendo");
  runs.finish(&id, "CANCELLED");
  assert_eq!(runs.get(&id).unwrap().status, "CANCELLED");
  assert!(!runs.cancel("p", &id), "finalizada não cancela");
 }
 #[test]
 fn cancellation_is_scoped_and_only_requested_until_finish() {
  let (runs,id)=run_with(&["wait"]);
  assert!(!runs.cancel("other",&id));
  assert!(!runs.cancelled(&id));
  assert!(runs.cancel("p",&id));
  assert_eq!(runs.get(&id).unwrap().status,"RUNNING");
  runs.finish(&id,"CANCELLED");
  assert!(!runs.cancel("p",&id));
 }
 #[test]
 fn timings_freeze_and_waiting_returns_to_running() {
  let (runs,id)=run_with(&["wait","chat"]);
  runs.step(&id,0,"RUNNING");runs.step(&id,0,"WAITING");runs.succeed(&id,0);
  let end=runs.get(&id).unwrap().steps[0].finished.unwrap();
  runs.succeed(&id,0);assert_eq!(runs.get(&id).unwrap().steps[0].finished,Some(end));
  runs.step(&id,1,"RUNNING");assert_eq!(runs.get(&id).unwrap().status,"RUNNING");
  runs.finish(&id,"COMPLETED");let run=runs.get(&id).unwrap();
  assert!(run.started_at>0);assert!(run.finished.is_some());assert!(run.steps[1].finished.is_some());
  runs.finish(&id,"COMPLETED");assert_eq!(runs.get(&id).unwrap().finished,run.finished);
 }
 #[test]
 fn failure_records_step_and_reason() {
  let (runs, id) = run_with(&["chat", "wait"]);
  runs.fail(&id, Some(0), "Sem permissão para publicar");
  let run = runs.get(&id).unwrap();
  assert_eq!(run.steps[0].status, "FAILED");
  assert_eq!(run.steps[1].status, "PENDING");
  assert_eq!(run.error.as_deref(), Some("Sem permissão para publicar"));
  runs.fail(&id, None, "Outro motivo");
  assert_eq!(
   runs.get(&id).unwrap().error.as_deref(),
   Some("Sem permissão para publicar"),
   "o primeiro motivo é o que fica"
  );
  runs.finish(&id, "FAILED");
  assert_eq!(runs.get(&id).unwrap().status, "FAILED");
 }
 #[test]
 fn succeed_keeps_the_condition_result() {
  let (runs, id) = run_with(&["condition", "wait"]);
  runs.step(&id, 0, "TRUE");
  runs.succeed(&id, 0);
  assert_eq!(
   runs.get(&id).unwrap().steps[0].status, "TRUE",
   "a condição grava o resultado dela e o motor não sobrescreve"
  );
  runs.succeed(&id, 1);
  assert_eq!(runs.get(&id).unwrap().steps[1].status, "SUCCESS");
 }
 #[test]
 fn event_context_is_allowlisted_profile_scoped_and_redacted(){
  let event=|profile:&str|Event{id:"secret-token-in-id".into(),profile_id:profile.into(),kind:"untrusted-secret-kind".into(),user:"private-name".into(),user_id:"private-id".into(),role:"secret-role".into(),message:"oauth:secret".into(),data:serde_json::json!({"token":"secret-password"}),simulated:false,rehearsal:true};
  let (runs,id)=run_with(&[]);
  runs.attach_event(&id,&event("other"),false);assert!(runs.get(&id).unwrap().context.is_none());
  runs.attach_event(&id,&event("p"),false);let run=runs.get(&id).unwrap();let context=run.context.unwrap();
  assert_eq!(context.event_kind,"other");assert_eq!(context.mode,"rehearsal");assert!(run.test);
  let serialized=serde_json::to_string(&context).unwrap();
  for secret in ["secret","private","oauth","token","password"]{assert!(!serialized.contains(secret));}
  assert_eq!(context.event_ref,RunContext::from_event(&event("p"),false).event_ref);
  assert_ne!(context.event_ref,RunContext::from_event(&event("other"),false).event_ref);
  runs.attach_event(&id,&event("p"),true);assert_eq!(runs.get(&id).unwrap().context.unwrap().mode,"rehearsal");
  let mut e=event("p");e.kind="raid".into();e.id.clear();e.rehearsal=false;
  assert_eq!(RunContext::from_event(&e,false).mode,"live");assert_eq!(RunContext::from_event(&e,false).event_ref,None);
  e.simulated=true;assert_eq!(RunContext::from_event(&e,false).mode,"simulation");assert_eq!(RunContext::from_event(&e,true).mode,"test");
 }
 #[test]
 fn prune_keeps_the_newest_runs() {
  let runs = Runs::default();
  for _ in 0..35 {
   let kinds = vec!["chat".to_string()];
   let id = runs.start("p", "f", "Fluxo", &kinds, false);
   runs.finish(&id, "COMPLETED");
  }
  assert!(runs.list("p").len() <= 30);
  assert_eq!(runs.short_of("inexistente").len(), 8);
 }
}
