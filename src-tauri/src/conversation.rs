use crate::model::Event;
use serde_json::{json, Value};
use std::{collections::{HashMap, VecDeque}, time::{Duration, Instant}};

#[derive(Default)]
pub struct History(HashMap<String, VecDeque<(Instant, Value)>>);
impl History {
    pub fn clear(&mut self,profile:&str) {self.0.remove(profile);}
    pub fn sent(&mut self,profile:&str,text:&str) {
        let rows=self.0.entry(profile.into()).or_default();
        rows.push_back((Instant::now(),json!({"person":"Bot","message":text.chars().take(500).collect::<String>()})));
        while rows.len()>12 {rows.pop_front();}
    }
    // Snapshot before recording the current message. Simulations never read or write live history.
    pub fn receive(&mut self, e: &Event) -> Vec<Value> {
        if e.simulated { return vec![]; }
        self.0.retain(|_, rows| {
            rows.retain(|(at,_)| at.elapsed() < Duration::from_secs(300));
            !rows.is_empty()
        });
        let rows = self.0.entry(e.profile_id.clone()).or_default();
        let previous = rows.iter().map(|(_,v)|v.clone()).collect();
        if e.kind == "chat" {
            rows.push_back((Instant::now(), json!({"person":e.user.chars().take(100).collect::<String>(),"message":e.message.chars().take(500).collect::<String>()})));
            while rows.len() > 12 { rows.pop_front(); }
        }
        previous
    }
}

pub fn prompt(e:&Event, history:&[Value]) -> String {
    json!({"currentMessage":{"person":e.user,"message":e.message},"recentChat":history}).to_string()
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn bounded_isolated_and_expiring() {
        let mut h=History::default();
        let mut e:Event=serde_json::from_value(json!({"id":"1","profileId":"a","kind":"chat","user":"Ana","message":"oi"})).unwrap();
        assert!(h.receive(&e).is_empty());
        assert_eq!(h.receive(&e).len(),1);
        e.profile_id="b".into();assert!(h.receive(&e).is_empty());
        e.profile_id="a".into();e.simulated=true;assert!(h.receive(&e).is_empty());
        e.simulated=false;assert_eq!(h.receive(&e).len(),2);
        for _ in 0..20 { h.receive(&e); }
        assert_eq!(h.receive(&e).len(),12);
        for (at,_) in h.0.get_mut("a").unwrap() { *at=Instant::now()-Duration::from_secs(301); }
        assert!(h.receive(&e).is_empty());
    }
}
