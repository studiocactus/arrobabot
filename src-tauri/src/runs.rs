//! Execuções de fluxos com identidade, estado e cancelamento. O motor de
//! actions[] continua o mesmo; aqui vive só o acompanhamento por execução.
use std::{
 collections::HashMap,
 sync::Mutex,
 time::Instant,
};
#[derive(Clone)]
pub struct Step {
 pub index: usize,
 pub kind: String,
 pub status: String,
}
#[derive(Clone)]
pub struct Run {
 pub id: String,
 pub short: String,
 pub profile_id: String,
 pub flow_id: String,
 pub flow_name: String,
 pub started: Instant,
 pub status: String,
 pub steps: Vec<Step>,
 pub cancelled: bool,
 pub test: bool,
}
#[derive(Default)]
pub struct Runs(pub Mutex<HashMap<String, Run>>);
fn short(id: &str) -> String {
 id.chars().take(8).collect()
}
impl Runs {
 pub fn start(&self, profile: &str, flow_id: &str, flow_name: &str, kinds: &[String], test: bool) -> String {
  let id = uuid::Uuid::new_v4().to_string();
  let run = Run {
   id: id.clone(),
   short: short(&id),
   profile_id: profile.into(),
   flow_id: flow_id.into(),
   flow_name: flow_name.into(),
   started: Instant::now(),
   status: "RUNNING".into(),
   steps: kinds.iter().enumerate().map(|(i, k)| Step { index: i, kind: k.clone(), status: "PENDING".into() }).collect(),
   cancelled: false,
   test,
  };
  let mut all = self.0.lock().unwrap();
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
 pub fn step(&self, id: &str, index: usize, status: &str) {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   if let Some(step) = run.steps.get_mut(index) {
    step.status = status.into();
   }
   if status == "RUNNING" && run.status == "RUNNING" {
    run.status = "RUNNING".into();
   }
   if status == "WAITING" {
    run.status = "WAITING".into();
   }
  }
 }
 pub fn cancel(&self, id: &str) -> bool {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   if run.status == "RUNNING" || run.status == "WAITING" {
    run.cancelled = true;
    return true;
   }
  }
  false
 }
 pub fn cancelled(&self, id: &str) -> bool {
  self.0.lock().unwrap().get(id).is_some_and(|r| r.cancelled)
 }
 pub fn finish(&self, id: &str, status: &str) {
  if let Some(run) = self.0.lock().unwrap().get_mut(id) {
   run.status = status.into();
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
 fn lifecycle_tracks_steps_and_cancels() {
  let (runs, id) = run_with(&["obs", "wait", "obs"]);
  assert_eq!(runs.get(&id).unwrap().status, "RUNNING");
  runs.step(&id, 0, "SUCCESS");
  runs.step(&id, 1, "WAITING");
  assert_eq!(runs.get(&id).unwrap().status, "WAITING");
  assert!(runs.cancel(&id));
  assert!(runs.cancelled(&id));
  assert!(!runs.cancel(&id), "segundo cancelamento não vale");
  runs.finish(&id, "CANCELLED");
  assert_eq!(runs.get(&id).unwrap().status, "CANCELLED");
  assert!(!runs.cancel(&id), "finalizada não cancela");
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
