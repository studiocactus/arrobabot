use crate::{engine::Runtime,model::{Event,Flow}};
use serde_json::json;
use std::{collections::HashMap,sync::Arc,time::{Duration,Instant}};
#[derive(Default)]pub struct Schedule{entries:HashMap<String,(String,Instant)>}
impl Schedule{
 pub fn due(&mut self,flows:&[Flow],now:Instant)->Vec<Flow>{
  self.entries.retain(|id,_|flows.iter().any(|f|f.id==*id));let mut ready=vec![];
  for f in flows{
   let signature=serde_json::to_string(f).unwrap();let period=Duration::from_secs(f.timer_seconds.clamp(30,86400));
   let entry=self.entries.entry(f.id.clone()).or_insert((signature.clone(),now+period));
   if entry.0!=signature{*entry=(signature,now+period)}
   if now>=entry.1{entry.1=now+period;ready.push(f.clone());}
  }ready
 }
}
pub fn active(rt:&Runtime,f:&Flow)->bool{f.enabled&&f.trigger.kind=="timer"&&rt.statuses.lock().unwrap().get(&f.profile_id).is_some_and(|s|s=="online")}
pub fn revision(f:&Flow)->String{use sha2::Digest;format!("{:x}",sha2::Sha256::digest(serde_json::to_vec(f).unwrap()))}
pub fn event_active(rt:&Runtime,e:&Event)->bool{rt.db.flows(&e.profile_id).unwrap_or_default().iter().any(|f|e.data["timerId"]==f.id&&e.data["timerRevision"]==revision(f)&&active(rt,f))}
pub async fn run(rt:Arc<Runtime>){
 let mut schedule=Schedule::default();let mut tick=tokio::time::interval(Duration::from_secs(1));
 loop{tick.tick().await;
  let flows:Vec<_>=rt.db.profiles().unwrap_or_default().iter().flat_map(|p|rt.db.flows(&p.id).unwrap_or_default()).filter(|f|active(&rt,f)).collect();
  for f in schedule.due(&flows,Instant::now()){
   if !rt.timer_pending.lock().unwrap().insert(f.id.clone()){continue}
   let event=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:f.profile_id.clone(),kind:"timer".into(),user:"BotLive".into(),user_id:String::new(),role:"broadcaster".into(),message:String::new(),data:json!({"timerId":f.id,"timerRevision":revision(&f)}),simulated:false};
   if rt.tx.try_send(event).is_err(){rt.timer_pending.lock().unwrap().remove(&f.id);rt.log(&f.profile_id,"timer","Fila ocupada; este disparo foi pulado. O próximo respeitará o intervalo.","info");}
  }
 }
}
pub struct Pending(pub Arc<Runtime>,pub Option<String>);
impl Drop for Pending{fn drop(&mut self){if let Some(id)=&self.1{self.0.timer_pending.lock().unwrap().remove(id);}}}
#[cfg(test)]mod tests{
 use super::*;
 fn flow(id:&str,seconds:u64)->Flow{serde_json::from_value(json!({"id":id,"profileId":"p","name":"Lembrete","enabled":true,"timerSeconds":seconds,"trigger":{"kind":"timer"},"actions":[{"kind":"chat","text":"Olá"}]})).unwrap()}
 #[test]fn independent_intervals_edits_pause_and_no_catchup(){
  let now=Instant::now();let mut s=Schedule::default();let mut f=vec![flow("a",30),flow("b",60)];assert!(s.due(&f,now).is_empty());assert!(s.due(&f,now+Duration::from_secs(29)).is_empty());assert_eq!(s.due(&f,now+Duration::from_secs(30))[0].id,"a");assert_eq!(s.due(&f,now+Duration::from_secs(90)).len(),2);assert!(s.due(&f,now+Duration::from_secs(91)).is_empty());
  f[0].timer_seconds=120;assert!(s.due(&f,now+Duration::from_secs(92)).is_empty());s.due(&[],now+Duration::from_secs(100));assert!(s.due(&f,now+Duration::from_secs(200)).is_empty());assert_eq!(s.due(&f,now+Duration::from_secs(260))[0].id,"b");
 }
}
