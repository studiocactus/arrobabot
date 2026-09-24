use crate::{engine::Runtime,model::*,variables};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{collections::{HashMap,HashSet},io::Read,path::{Path,PathBuf},time::{Duration,Instant}};
use base64::Engine;
use rand::Rng;
type R<T> = Result<T,String>;
#[derive(Clone,Default,Serialize,Deserialize)]
#[serde(default,rename_all="camelCase")]
pub struct Config {pub replies_enabled:bool,pub sounds_enabled:bool,pub replies:Vec<Reply>,pub people:Vec<Person>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Reply {pub id:String,pub enabled:bool,pub keyword:String,pub matching:String,pub selection:String,pub asset:String,pub cooldown:u64,pub user_cooldown:u64}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Person {pub id:String,pub enabled:bool,pub name:String,pub nickname:String,pub user_id:String,pub asset:String,pub mode:String,pub cooldown:u64,pub volume:f64}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Asset {pub id:String,pub kind:String,pub name:String,pub path:String}
#[derive(Default)]
pub struct State {last:HashMap<String,Instant>,seen:HashSet<String>,index:HashMap<String,usize>}
pub fn config(rt:&Runtime,p:&str)->Config {serde_json::from_value(rt.db.module(p,"chatExtras")).unwrap_or_default()}
fn assets(rt:&Runtime,p:&str)->Vec<Asset>{serde_json::from_value(rt.db.module(p,"chatAssets")).unwrap_or_default()}
fn asset(rt:&Runtime,p:&str,id:&str,kind:&str)->R<Asset>{assets(rt,p).into_iter().find(|a|a.id==id&&a.kind==kind).ok_or("Selecione um arquivo deste perfil".into())}
fn read_file(path:&Path,max:u64)->R<Vec<u8>>{
 let f=std::fs::File::open(path).map_err(|_|"Arquivo não encontrado ou sem permissão")?;
 let meta=f.metadata().map_err(|_|"Não foi possível ler o arquivo")?;
 if !meta.is_file()||meta.len()>max{return Err("Arquivo inválido ou maior que o limite".into())}
 let mut bytes=vec![];f.take(max+1).read_to_end(&mut bytes).map_err(|_|"Falha ao ler arquivo")?;
 if bytes.len() as u64>max{return Err("Arquivo maior que o limite".into())}Ok(bytes)
}
pub fn lines(path:&Path)->R<Vec<String>>{
 let bytes=read_file(path,256*1024)?;
 let text=std::str::from_utf8(&bytes).map_err(|_|"Salve o TXT como UTF-8")?;
 let rows:Vec<String>=text.trim_start_matches('\u{feff}').lines().map(str::trim).filter(|s|!s.is_empty()).map(str::to_owned).collect();
 if rows.is_empty()||rows.len()>2000||rows.iter().any(|s|s.chars().count()>450||s.contains('\0')){return Err("Use 1 a 2000 linhas não vazias, com até 450 caracteres cada".into())}Ok(rows)
}
fn audio_kind(path:&Path,bytes:&[u8])->R<&'static str>{
 let ext=path.extension().and_then(|s|s.to_str()).unwrap_or("").to_ascii_lowercase();
 match ext.as_str(){
 "wav" if bytes.starts_with(b"RIFF")&&bytes.get(8..12)==Some(b"WAVE")=>Ok("audio/wav"),
 "ogg" if bytes.starts_with(b"OggS")=>Ok("audio/ogg"),
 "mp3" if bytes.starts_with(b"ID3")||(bytes.len()>1&&bytes[0]==255&&bytes[1]&224==224)=>Ok("audio/mpeg"),
 _=>Err("Escolha um áudio WAV, MP3 ou OGG válido, até 5 MiB".into())
 }
}
fn audio_path(rt:&Runtime,p:&str,a:&Asset)->R<PathBuf>{
 let root=rt.base.join("media").join(p).canonicalize().map_err(|_|"Pasta de sons ausente")?;
 let path=Path::new(&a.path).canonicalize().map_err(|_|"Som não encontrado")?;
 if !path.starts_with(&root)||path==root{return Err("Caminho de som inválido".into())}Ok(path)
}
pub fn operation(rt:&Runtime,p:&str,op:&str,args:&Value)->R<Value>{
 rt.db.profile(p)?;
 match op {
 "chatExtras.get"=>Ok(json!({"config":config(rt,p),"assets":assets(rt,p).iter().map(|a|json!({"id":a.id,"name":a.name,"kind":a.kind})).collect::<Vec<_>>() })),
 "chatExtras.import"=>{
 let _guard=rt.module_lock.lock().unwrap();
 let kind=args["kind"].as_str().unwrap_or("");let source=Path::new(args["path"].as_str().ok_or("Escolha um arquivo")?).canonicalize().map_err(|_|"Arquivo não encontrado")?;
 let mut list=assets(rt,p);if list.len()>=300{return Err("Limite de 300 arquivos por perfil".into())}
 let id=uuid::Uuid::new_v4().to_string();let path=if kind=="txt"{
 if !source.extension().is_some_and(|e|e.eq_ignore_ascii_case("txt")){return Err("Escolha um arquivo .txt".into())}lines(&source)?;source.clone()
 }else if kind=="sound"{
 let bytes=read_file(&source,5*1024*1024)?;audio_kind(&source,&bytes)?;
 let dir=rt.base.join("media").join(p);std::fs::create_dir_all(&dir).map_err(|_|"Não foi possível guardar o som")?;
 let path=dir.join(format!("{}.{}",id,source.extension().unwrap().to_string_lossy().to_lowercase()));std::fs::write(&path,bytes).map_err(|_|"Não foi possível copiar o som")?;path
 }else{return Err("Tipo de arquivo inválido".into())};
 let a=Asset{id:id.clone(),kind:kind.into(),name:source.file_name().unwrap().to_string_lossy().into(),path:path.to_string_lossy().into()};list.push(a.clone());rt.db.set_module(p,"chatAssets",&json!(list))?;Ok(json!({"id":id,"kind":kind,"name":a.name}))
 },
 "chatExtras.save"=>{
 let mut c:Config=serde_json::from_value(args["config"].clone()).map_err(|_|"Configuração inválida")?;
 for r in &mut c.replies{r.keyword=r.keyword.trim().into();}
 for person in &mut c.people{person.name=person.name.trim().into();person.user_id=person.user_id.trim().into();person.nickname=person.nickname.trim().into();}
 if c.replies.len()>100||c.people.len()>200{return Err("Limite: 100 respostas e 200 pessoas por perfil".into())}
 let mut ids=HashSet::new();let mut identities=HashSet::new();
 for r in &c.replies{
 if !valid_id(&r.id)||!ids.insert(r.id.clone())||r.keyword.trim().is_empty()||r.keyword.len()>200||!["contains","word"].contains(&r.matching.as_str())||!["random","sequence"].contains(&r.selection.as_str())||!(1..=86400).contains(&r.cooldown)||r.user_cooldown>86400{return Err("Confira palavra, modo e intervalos das respostas".into())}
 asset(rt,p,&r.asset,"txt")?;
 }
 for person in &c.people{
 let identity=if person.user_id.trim().is_empty(){format!("name:{}",normalize(&person.name))}else{format!("id:{}",person.user_id.trim())};
 if !valid_id(&person.id)||!ids.insert(person.id.clone())||!identities.insert(identity)||person.name.trim().is_empty()||person.name.len()>200||person.nickname.len()>200||person.user_id.len()>200||!["first","interval"].contains(&person.mode.as_str())||!(5..=86400).contains(&person.cooldown)||!person.volume.is_finite()||!(0.0..=1.0).contains(&person.volume){return Err("Confira nomes únicos, volume e intervalo dos sons (mínimo 5s)".into())}
 asset(rt,p,&person.asset,"sound")?;
 }
 rt.db.set_module(p,"chatExtras",&json!(c))?;Ok(Value::Null)
 },
 "chatExtras.preview"=>{let a=asset(rt,p,args["asset"].as_str().unwrap_or(""),"txt")?;let rows=lines(Path::new(&a.path))?;Ok(json!({"count":rows.len(),"lines":rows.into_iter().take(10).collect::<Vec<_>>()}))},
 "chatExtras.audio"=>{
 let a=asset(rt,p,args["asset"].as_str().unwrap_or(""),"sound")?;let path=audio_path(rt,p,&a)?;let bytes=read_file(&path,5*1024*1024)?;let mime=audio_kind(&path,&bytes)?;
 Ok(json!(format!("data:{mime};base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes))))
 },
 "chatExtras.reset"=>{let prefix=format!("{p}:sound:");let mut s=rt.chat_extras.lock().unwrap();s.seen.retain(|k|!k.starts_with(&prefix));s.last.retain(|k,_|!k.starts_with(&prefix));Ok(Value::Null)},
 _=>Err("Operação desconhecida".into())
 }
}
fn normalize(s:&str)->String{s.trim().trim_start_matches('@').to_lowercase()}
pub fn keyword_matches(message:&str,keyword:&str,mode:&str)->bool{
 let message=message.to_lowercase();let keyword=keyword.trim().to_lowercase();
 if keyword.is_empty(){return false}
 if mode=="word" {message.match_indices(&keyword).any(|(i,_)|{
 let word=|c:char|c.is_alphanumeric()||c=='_';
 !message[..i].chars().next_back().is_some_and(word)&&!message[i+keyword.len()..].chars().next().is_some_and(word)
 })}else{message.contains(&keyword)}
}
fn available(s:&State,key:&str,seconds:u64)->bool{!s.last.get(key).is_some_and(|t|t.elapsed()<Duration::from_secs(seconds))}
#[cfg(test)] mod tests {
 use super::*;
 fn p()->Profile{serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal","blocklist":["bloqueado"]})).unwrap()}
 fn e(p:&Profile)->Event{serde_json::from_value(json!({"id":"1","profileId":p.id,"kind":"chat","user":"ANA","userId":"123","message":"Olá, café!"})).unwrap()}
 fn register(rt:&Runtime,p:&Profile,path:&Path,kind:&str)->String{operation(rt,&p.id,"chatExtras.import",&json!({"path":path,"kind":kind})).unwrap()["id"].as_str().unwrap().into()}
 fn rule(asset:String)->Reply{Reply{id:uuid::Uuid::new_v4().to_string(),enabled:true,keyword:"café".into(),matching:"word".into(),selection:"sequence".into(),asset,cooldown:30,user_cooldown:60}}
 fn save(rt:&Runtime,p:&Profile,c:&Config){operation(rt,&p.id,"chatExtras.save",&json!({"config":c})).unwrap();}
 #[test] fn matching_and_txt_validation(){
  assert!(keyword_matches("CAFÉ!","café","word"));assert!(!keyword_matches("cafés","café","word"));assert!(keyword_matches("cafés","café","contains"));assert!(keyword_matches("um bom dia!","bom dia","word"));assert!(!keyword_matches("bom diazão","bom dia","word"));
  let dir=tempfile::tempdir().unwrap();let file=dir.path().join("respostas.txt");std::fs::write(&file,"\u{feff}Olá\r\n\r\nTudo bem?\r\n").unwrap();assert_eq!(lines(&file).unwrap(),vec!["Olá","Tudo bem?"]);
  for data in [vec![255],b"\n  \n".to_vec(),vec![b'a';451],vec![b'a';256*1024+1]]{std::fs::write(&file,data).unwrap();assert!(lines(&file).is_err());}
 }
 #[tokio::test] async fn txt_reload_cooldown_simulation_sequence_random_and_isolation(){
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let p=p();let other=super::tests::p();rt.db.save_profile(&p).unwrap();rt.db.save_profile(&other).unwrap();let path=dir.path().join("respostas.txt");std::fs::write(&path,"Olá {{user}}\nSegunda\nTerceira").unwrap();
  let id=register(&rt,&p,&path,"txt");let mut c=Config{replies_enabled:true,replies:vec![rule(id.clone())],..Default::default()};save(&rt,&p,&c);let mut event=e(&p);
  event.simulated=true;assert_eq!(response(&rt,&p,&event).unwrap(),Some("Olá ANA".into()));event.simulated=false;assert_eq!(response(&rt,&p,&event).unwrap(),Some("Olá ANA".into()));assert!(response(&rt,&p,&event).unwrap().is_none());
  operation(&rt,&p.id,"chatExtras.reset",&json!({})).unwrap();assert!(response(&rt,&p,&event).unwrap().is_none());
  rt.chat_extras.lock().unwrap().last.clear();assert_eq!(response(&rt,&p,&event).unwrap(),Some("Segunda".into()));
  std::fs::write(&path,"Atualizado").unwrap();rt.chat_extras.lock().unwrap().last.clear();assert_eq!(response(&rt,&p,&event).unwrap(),Some("Atualizado".into()));
  assert!(operation(&rt,&other.id,"chatExtras.preview",&json!({"asset":id})).is_err());assert!(response(&rt,&other,&e(&other)).unwrap().is_none());
  std::fs::write(&path,"A\nB").unwrap();c.replies[0].selection="random".into();save(&rt,&p,&c);let mut last=String::new();for _ in 0..8{rt.chat_extras.lock().unwrap().last.clear();let next=response(&rt,&p,&event).unwrap().unwrap();assert_ne!(last,next);last=next;}
  std::fs::write(&path,"bloqueado").unwrap();rt.chat_extras.lock().unwrap().last.clear();assert!(response(&rt,&p,&event).is_err());
  std::fs::remove_file(&path).unwrap();assert!(response(&rt,&p,&event).is_err());c.replies_enabled=false;save(&rt,&p,&c);assert!(response(&rt,&p,&event).unwrap().is_none());
 }
 #[tokio::test] async fn sound_copy_identity_toggles_session_and_simulation(){
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let p=p();rt.db.save_profile(&p).unwrap();let path=dir.path().join("som.wav");std::fs::write(&path,b"RIFF0000WAVEfmt ").unwrap();let id=register(&rt,&p,&path,"sound");std::fs::remove_file(path).unwrap();
  assert!(operation(&rt,&p.id,"chatExtras.audio",&json!({"asset":id})).unwrap().as_str().unwrap().starts_with("data:audio/wav;base64,"));
  let mut c=Config{sounds_enabled:true,people:vec![Person{id:uuid::Uuid::new_v4().to_string(),enabled:true,name:"OutroNome".into(),nickname:"Aninha".into(),user_id:"123".into(),asset:id,mode:"first".into(),cooldown:60,volume:0.7}],..Default::default()};save(&rt,&p,&c);
  let mut event=e(&p);let mut rx=rt.broadcast.subscribe();event.simulated=true;sound(&rt,&p,&event);assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="viewer-sound"));event.simulated=false;
  sound(&rt,&p,&event);sound(&rt,&p,&event);let events:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="viewer-sound").collect();assert_eq!(events.len(),1);assert_eq!(events[0]["payload"]["nickname"],"Aninha");
  operation(&rt,&p.id,"chatExtras.reset",&json!({})).unwrap();c.people[0].enabled=false;save(&rt,&p,&c);sound(&rt,&p,&event);assert!(rx.try_recv().is_err());
  c.people[0].enabled=true;c.sounds_enabled=false;save(&rt,&p,&c);sound(&rt,&p,&event);assert!(rx.try_recv().is_err());
  c.sounds_enabled=true;c.people[0].user_id="".into();c.people[0].name="@ana".into();save(&rt,&p,&c);event.kind="join".into();sound(&rt,&p,&event);assert_eq!(rx.try_recv().unwrap()["type"],"viewer-sound");
  c.people.push(c.people[0].clone());assert!(operation(&rt,&p.id,"chatExtras.save",&json!({"config":c})).is_err());
 }
}
pub fn sound(rt:&Runtime,p:&Profile,e:&Event){
 if e.kind!="chat"&&e.kind!="join"{return}let c=config(rt,&p.id);if !c.sounds_enabled{return}
 let person=c.people.iter().find(|a|a.enabled&&!a.user_id.is_empty()&&a.user_id==e.user_id).or_else(||c.people.iter().find(|a|a.enabled&&a.user_id.is_empty()&&normalize(&a.name)==normalize(&e.user)));let Some(person)=person else{return};
 let key=format!("{}:sound:{}",p.id,person.id);let global=format!("{}:sound:global",p.id);
 let mut s=rt.chat_extras.lock().unwrap();
 if (person.mode=="first"&&s.seen.contains(&key))||!available(&s,&key,person.cooldown)||!available(&s,&global,5){return}
 if e.simulated{drop(s);rt.log(&p.id,"sound",&format!("[Simulação] Tocaria o som de {}",person.nickname),"info");return}
 s.seen.insert(key.clone());s.last.insert(key,Instant::now());s.last.insert(global,Instant::now());drop(s);
 rt.emit("viewer-sound",json!({"profileId":p.id,"asset":person.asset,"nickname":person.nickname,"volume":person.volume}));
 rt.log(&p.id,"sound",&format!("Som solicitado para {}",person.nickname),"info");
}
pub fn response(rt:&Runtime,p:&Profile,e:&Event)->R<Option<String>>{
 if e.kind!="chat"{return Ok(None)}let c=config(rt,&p.id);if !c.replies_enabled{return Ok(None)}
 let mut s=rt.chat_extras.lock().unwrap();s.last.retain(|_,t|t.elapsed()<Duration::from_secs(86400));
 for r in c.replies.iter().filter(|r|r.enabled&&keyword_matches(&e.message,&r.keyword,&r.matching)){
 let key=format!("{}:txt:{}",p.id,r.id);let user=format!("{key}:{}",if e.user_id.is_empty(){e.user.as_str()}else{e.user_id.as_str()});
 if !e.simulated&&(!available(&s,&key,r.cooldown)||!available(&s,&user,r.user_cooldown)){continue}
 let a=asset(rt,&p.id,&r.asset,"txt")?;let rows=lines(Path::new(&a.path))?;let previous=s.index.get(&key).copied();
 let index=if r.selection=="sequence"{previous.map_or(0,|i|(i+1)%rows.len())}else{
 let mut candidates:Vec<_>=(0..rows.len()).filter(|i|rows.len()==1||Some(*i)!=previous).collect();if candidates.is_empty(){candidates.push(0)}candidates[rand::thread_rng().gen_range(0..candidates.len())]
 };
 let text=variables::Context::new(&rt.db,p,e,None)?.render(&rows[index])?;
 if blocked(&text,p){return Err("Resposta TXT bloqueada pelas restrições do perfil".into())}
 if text.trim().is_empty()||text.chars().count()>450{return Err("Resposta TXT expandida vazia ou maior que 450 caracteres".into())}
 if !e.simulated{s.last.insert(key.clone(),Instant::now());s.last.insert(user,Instant::now());s.index.insert(key,index);}
 return Ok(Some(text))
 }Ok(None)
}
