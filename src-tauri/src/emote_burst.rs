//! Explosão de emotes no overlay: sub, renovação, presente e meta completada.
//! Os ids vêm do cache de emotes da Twitch (emotes.rs); os totais de uso são
//! contados a partir da ativação e guardados por perfil, para a origem "mais
//! usados" sobreviver ao reinício do aplicativo.
use crate::{engine::Runtime,model::{Event,Profile}};
use serde_json::{json,Value};
use std::collections::HashMap;

/// Os quatro eventos que podem explodir — os mesmos nomes usados na tela.
pub const EVENTS:[&str;4]=["sub","resub","gift","meta"];

pub fn config(rt:&Runtime,p:&str)->Value{rt.db.module(p,"emoteBurst")}

pub fn valid_config(v:&Value)->bool{
 let count=v["count"].as_u64().unwrap_or(40);
 let size=v["size"].as_u64().unwrap_or(56);
 let top=v["topN"].as_u64().unwrap_or(10);
 let source_ok=matches!(v["source"].as_str().unwrap_or("random"),"all"|"top"|"random");
 let events_ok=v["events"].as_object().map(|o|o.len()<=EVENTS.len()&&o.iter().all(|(k,val)|EVENTS.contains(&k.as_str())&&val.is_boolean())).unwrap_or(true);
 (10..=80).contains(&count)&&(24..=120).contains(&size)&&(5..=30).contains(&top)&&source_ok&&events_ok
}

/// Conta o emote de cada mensagem para a origem "mais usados". O custo é uma
/// leitura de configuração e um contador em memória — nada de banco por mensagem.
pub fn count(rt:&Runtime,p:&str,e:&Event){
 if e.simulated||config(rt,p)["enabled"]!=Value::Bool(true){return}
 let frags=crate::chat_overlay::frags(&e.data,&e.message);
 let Some(frags)=frags.as_array()else{return};
 let mut maps=rt.emote_counts.lock().unwrap();
 let per=maps.entry(p.to_owned()).or_default();
 for f in frags{
  if f["t"]=="e"{if let Some(id)=f["id"].as_str(){if !id.is_empty(){*per.entry(id.to_owned()).or_default()+=1;}}}
 }
}

/// Totais do perfil: a memória da sessão, ou o que ficou guardado ao fechar.
pub(crate) fn totals(rt:&Runtime,p:&str)->HashMap<String,u64>{
 if let Some(mem)=rt.emote_counts.lock().unwrap().get(p){if !mem.is_empty(){return mem.clone()}}
 let v:Value=rt.db.module(p,"emoteBurstStats");
 v.as_object().map(|o|o.iter().filter_map(|(k,n)|Some((k.clone(),n.as_u64()?))).collect()).unwrap_or_default()
}

/// Escolhe os ids da explosão: todos em ordem, os mais usados primeiro ou uma
/// amostra aleatória — sempre contando só com a lista guardada do canal.
pub fn pick_ids(pool:&[String],source:&str,top_n:usize,counts:&HashMap<String,u64>,rng:&mut impl rand::Rng)->Vec<String>{
 if pool.is_empty(){return Vec::new()}
 match source{
  "top"=>{let mut by_use:Vec<String>=pool.to_vec();by_use.sort_by_key(|id|std::cmp::Reverse(counts.get(id).copied().unwrap_or(0)));by_use.truncate(top_n.max(1));by_use}
  "random"=>{use rand::seq::SliceRandom;let mut s=pool.to_vec();s.shuffle(rng);s}
  _=>pool.to_vec(),
 }
}

/// Repete a seleção até a quantidade pedida: a explosão enche o ecrã mesmo
/// quando a lista do canal é curta.
fn burst_ids(rt:&Runtime,p:&str,cfg:&Value)->Vec<String>{
 let pool=crate::emotes::ids(&rt.db,p);
 if pool.is_empty(){return Vec::new()}
 let count=cfg["count"].as_u64().unwrap_or(40).clamp(10,80) as usize;
 let base=pick_ids(&pool,cfg["source"].as_str().unwrap_or("random"),cfg["topN"].as_u64().unwrap_or(10).clamp(5,30) as usize,&totals(rt,p),&mut rand::thread_rng());
 if base.is_empty(){return Vec::new()}
 (0..count).map(|i|base[i%base.len()].clone()).collect()
}

fn emit(rt:&Runtime,p:&Profile,cfg:&Value,ids:Vec<String>,via:&str,rehearsal:bool){
 let mut payload=json!({"profileId":p.id,"ids":ids,"size":cfg["size"].as_u64().unwrap_or(56).clamp(24,120),"via":via,"alsoChat":cfg["alsoChat"]==Value::Bool(true)});
 if rehearsal{payload["rehearsal"]=json!(true);}
 let _=rt.broadcast.send(json!({"type":"emote.burst","payload":payload}));
}

fn fire(rt:&Runtime,p:&Profile,cfg:&Value,via:&str,rehearsal:bool){
 let ids=burst_ids(rt,&p.id,cfg);
 if ids.is_empty(){
  rt.log(&p.id,"emotes","Explosão de emotes sem emotes carregados: conecte o canal ao Twitch para a lista chegar.","info");
  return;
 }
 emit(rt,p,cfg,ids,via,rehearsal);
}

/// Explosão de um evento ligado na configuração.
pub fn maybe(rt:&Runtime,p:&Profile,via:&str){
 maybe_rehearsed(rt,p,via,false)
}

/// Mesma explosão, marcada como ensaio (chegou por teste, não por evento real).
pub fn maybe_rehearsed(rt:&Runtime,p:&Profile,via:&str,rehearsal:bool){
 let cfg=config(rt,&p.id);
 if cfg["enabled"]!=Value::Bool(true)||!EVENTS.contains(&via)||cfg["events"][via]!=Value::Bool(true){return}
 fire(rt,p,&cfg,via,rehearsal);
}

/// O evento da Twitch que virou explosão: sub, renovação e presente.
pub fn from_event(rt:&Runtime,p:&Profile,e:&Event){
 let via=match e.kind.as_str(){"subscription"=>"sub","resub"=>"resub","gift"=>"gift",_=>return};
 maybe_rehearsed(rt,p,via,e.rehearsal||e.simulated);
}

/// Botão de teste da tela: dispara mesmo com os interruptores de evento
/// desligados — é para conferir o efeito antes de confiar na live.
pub fn test(rt:&Runtime,p:&Profile)->Result<(),String>{
 let cfg=config(rt,&p.id);
 let ids=burst_ids(rt,&p.id,&cfg);
 if ids.is_empty(){return Err("Sem emotes carregados: conecte o canal ao Twitch e tente de novo.".into())}
 emit(rt,p,&cfg,ids,"sub",true);Ok(())
}

/// Mesmo teste, com uma tentativa de buscar os emotes do canal na hora: a lista
/// vazia costuma ser cache frio, não canal desconectado.
pub async fn test_fresh(rt:&crate::engine::Runtime,p:&Profile)->Result<(),String>{
 match test(rt,p){
  Err(err) if err.contains("Sem emotes")=>{
   crate::emotes::refresh(rt,p).await;
   match test(rt,p){Ok(())=>Ok(()),Err(_)=>Err("Sem emotes carregados: conecte o canal ao Twitch e tente de novo.".into())}
  },
  other=>other,
 }
}

/// Guarda os totais da sessão no banco a cada tique do agendador.
pub fn flush(rt:&Runtime){
 let snapshot:Vec<(String,HashMap<String,u64>)>={
  let maps=rt.emote_counts.lock().unwrap();
  maps.iter().filter(|(_,m)|!m.is_empty()).map(|(k,m)|(k.clone(),m.clone())).collect()
 };
 for(p,map)in snapshot{
  let value:Value=map.iter().map(|(id,n)|(id.clone(),json!(n))).collect::<serde_json::Map<String,Value>>().into();
  let _=rt.db.set_module(&p,"emoteBurstStats",&value);
 }
}

#[cfg(test)]mod tests{
 use super::*;
 use rand::SeedableRng;
 #[test]fn config_valida_confere_quantidade_tamanho_e_origem(){
  assert!(valid_config(&json!({})),"sem nada valem os padrões");
  assert!(valid_config(&json!({"count":40,"size":56,"source":"top","topN":10,"events":{"sub":true,"meta":false}})));
  assert!(!valid_config(&json!({"count":9})),"quantidade mínima é 10");
  assert!(!valid_config(&json!({"count":81})),"quantidade máxima é 80");
  assert!(!valid_config(&json!({"size":23})),"tamanho mínimo é 24");
  assert!(!valid_config(&json!({"size":121})),"tamanho máximo é 120");
  assert!(!valid_config(&json!({"source":"tudo"})),"origem é all, top ou random");
  assert!(!valid_config(&json!({"events":{"raid":true}})),"só os quatro eventos entram");
  assert!(!valid_config(&json!({"events":{"sub":"sim"}})),"interruptor é booleano");
 }
 #[test]fn selecao_pega_todos_os_mais_usados_ou_aleatorio(){
  let pool:Vec<String>=["a","b","c","d"].iter().map(|s|s.to_string()).collect();
  let mut counts:HashMap<String,u64>=HashMap::new();counts.insert("c".into(),9);counts.insert("b".into(),3);
  let mut rng=rand::rngs::StdRng::seed_from_u64(7);
  assert_eq!(pick_ids(&pool,"all",10,&counts,&mut rng),pool,"todos vêm na ordem do canal");
  assert_eq!(pick_ids(&pool,"top",2,&counts,&mut rng),vec!["c".to_string(),"b".to_string()],"os mais usados na frente");
  let random=pick_ids(&pool,"random",10,&counts,&mut rng);
  assert_eq!(random.len(),4,"aleatório embaralha sem perder ninguém");
  assert!(pool.iter().all(|id|random.contains(id)));
  assert!(pick_ids(&[],"all",10,&counts,&mut rng).is_empty(),"sem emotes, sem explosão");
 }
}
