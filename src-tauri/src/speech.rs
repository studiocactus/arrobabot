//! As últimas falas reconhecidas do microfone do streamer.
//! Guardadas no banco porque o contexto de variáveis só enxerga o perfil: assim a IA
//! continua sabendo o que acabou de ser dito ao trocar de tela ou reabrir o app.
use crate::db::Db;
use serde_json::{json,Value};
use std::time::{SystemTime,UNIX_EPOCH};

/// Dez minutos cobrem uma conversa sem levar o assunto para a live seguinte.
const TTL:u64=600;
/// Janela curta: é contexto, não transcrição integral.
const MAX:usize=12;
/// Teto de uma linha: fala longa não deve ocupar o prompt inteiro.
const LINE:usize=400;
const KEY:&str="speech";
type R<T>=Result<T,String>;

fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).map(|d|d.as_secs()).unwrap_or(0)}
fn load(db:&Db,profile:&str)->Vec<(u64,String)>{
 let value=db.module(profile,KEY);
 let mut items:Vec<(u64,String)>=value["items"].as_array()
  .map(|rows|rows.iter().filter_map(|row|Some((row["at"].as_u64()?,row["text"].as_str()?.to_owned()))).collect())
  .unwrap_or_default();
 let cut=now().saturating_sub(TTL);
 items.retain(|(at,_)|*at>=cut);
 items
}
fn save(db:&Db,profile:&str,items:&[(u64,String)])->R<()>{
 let rows:Vec<Value>=items.iter().map(|(at,text)|json!({"at":at,"text":text})).collect();
 db.set_module(profile,KEY,&json!({"items":rows}))
}
/// Registra uma fala reconhecida e devolve a janela já atualizada.
pub fn push(db:&Db,profile:&str,text:&str)->R<Vec<String>>{
 let text:String=text.trim().chars().take(LINE).collect();
 if text.is_empty(){return Ok(recent(db,profile))}
 let mut items=load(db,profile);
 items.push((now(),text));
 while items.len()>MAX{items.remove(0);}
 save(db,profile,&items)?;
 Ok(items.into_iter().map(|(_,line)|line).collect())
}
/// As falas dentro do tempo útil, mais antiga primeiro.
pub fn recent(db:&Db,profile:&str)->Vec<String>{load(db,profile).into_iter().map(|(_,line)|line).collect()}
/// A última fala reconhecida; vazia quando o microfone ainda não ouviu nada.
pub fn last(db:&Db,profile:&str)->String{recent(db,profile).pop().unwrap_or_default()}
/// Linha curta para o prompt de sistema da IA.
pub fn summary(db:&Db,profile:&str)->String{
 let items=recent(db,profile);
 if items.is_empty(){return String::new()}
 let tail:Vec<String>=items.iter().rev().take(6).cloned().collect();
 let spoken=tail.into_iter().rev().collect::<Vec<_>>().join(" | ");
 format!("[Últimas falas do streamer na live: {}]",spoken).chars().take(600).collect()
}
pub fn clear(db:&Db,profile:&str)->R<()>{db.set_module(profile,KEY,&json!({"items":[]}))}
#[cfg(test)] mod tests {
 use super::*;
 use crate::model::Profile;
 /// module_state tem chave estrangeira para profiles e o id precisa ser UUID.
 const PID:&str="00000000-0000-4000-8000-000000000001";
 fn db()->(tempfile::TempDir,Db){
  let dir=tempfile::tempdir().unwrap();let db=Db::open(&dir.path().join("t.sqlite")).unwrap();
  let p:Profile=serde_json::from_value(json!({"id":PID,"name":"P","platform":"twitch","channel":"canal"})).unwrap();
  db.save_profile(&p).unwrap();
  (dir,db)
 }
 #[test] fn a_janela_guarda_as_falas_recentes_e_caiba_no_limite(){
  let (_dir,db)=db();
  assert_eq!(recent(&db,PID),Vec::<String>::new(),"microfone parado não tem fala");
  assert_eq!(summary(&db,PID),"","sem fala não gasta token de contexto");
  assert_eq!(last(&db,PID),"");
  push(&db,PID,"  vamos de ranked  ").unwrap();
  assert_eq!(last(&db,PID),"vamos de ranked","a fala é guardada limpa");
  for i in 0..20 {push(&db,PID,&format!("fala {i}")).unwrap();}
  let items=recent(&db,PID);
  assert_eq!(items.len(),MAX,"a janela não cresce sem limite");
  assert_eq!(items.last().unwrap(),"fala 19","a mais nova fica no fim");
  assert!(summary(&db,PID).contains("fala 19")&&summary(&db,PID).len()<=600);
  clear(&db,PID).unwrap();assert!(recent(&db,PID).is_empty());
 }
 #[test] fn fala_antiga_e_linha_longa_sao_tratadas(){
  let (_dir,db)=db();
  save(&db,PID,&[(now()-TTL-1,"já esquecida".into()),(now(),"ainda vale".into())]).unwrap();
  assert_eq!(recent(&db,PID),vec!["ainda vale"],"fala fora do tempo sai da janela");
  push(&db,PID,&"x".repeat(5000)).unwrap();
  assert_eq!(last(&db,PID).chars().count(),LINE,"linha longa é cortada");
  push(&db,PID,"   ").unwrap();
  assert_eq!(last(&db,PID).chars().count(),LINE,"fala vazia não entra");
 }
}
