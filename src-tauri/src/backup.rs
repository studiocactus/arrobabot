//! Backup de todas as áreas do BotLive em um arquivo único com extensão .botlivebak.
//! O cofre de credenciais fica de fora por construção: nenhum caminho daqui lê o cofre.
use crate::{db::Db,engine::Runtime};
use base64::Engine as _;
use chrono::Datelike;
use serde_json::{json,Value};
use std::collections::HashSet;
use std::path::{Path,PathBuf};
pub const EXT:&str="botlivebak";
const FORMAT:&str="botlive-backup";
const FORMAT_VERSION:u64=1;
const KEY:&str="backup";
/// Áreas que entram no arquivo. O Histórico entra junto: sem ele, estatísticas e
/// diagnóstico da live começariam do zero depois de uma importação.
const TABLES:[&str;9]=["profiles","flows","command_counters","logs","presets","kv","module_state","points","variables"];
/// Pastas de conteúdo que acompanham o arquivo, no mesmo caminho relativo de sempre.
const DIRS:[&str;3]=["knowledge","media","vaults"];
const MAX_BYTES:u64=256*1024*1024;
const MAX_FILE_BYTES:u64=64*1024*1024;
const MAX_FILES:usize=5000;
fn defaults()->Value{json!({"folder":"","auto":false,"days":7,"weeks":4,"months":12,"lastSuccess":0,"lastTry":0})}
/// Configuração salva no banco. Valores conhecidos herdam o padrão quando faltam.
pub fn config(db:&Db)->Value{
 let base=defaults();
 let mut out=base.clone();
 if let Some(old)=db.get(KEY).as_object(){for (k,v) in old{if base.get(k).is_some(){out[k.to_string()]=v.clone()}}}
 out
}
/// Guarda a pasta escolhida e as quantidades de retenção, com validação na entrada.
pub fn save_config(db:&Db,input:&Value)->Result<Value,String>{
 let mut cfg=config(db);
 if let Some(v)=input["folder"].as_str(){if v.len()>600{return Err("Caminho da pasta longo demais".into())}cfg["folder"]=json!(v.trim())}
 if let Some(v)=input["auto"].as_bool(){cfg["auto"]=json!(v)}
 for (key,min,max,label) in [("days",1i64,365i64,"dias"),("weeks",1,52,"semanas"),("months",1,60,"meses")] {
  if let Some(v)=input[key].as_i64(){if !(min..=max).contains(&v){return Err(format!("Retenção de {label}: escolha de {min} a {max}"))}cfg[key]=json!(v)}
 }
 let folder=cfg["folder"].as_str().unwrap_or("").to_owned();
 if !folder.is_empty()&&!Path::new(&folder).is_dir(){return Err("A pasta escolhida para os backups não existe".into())}
 if cfg["auto"]==true&&folder.is_empty(){return Err("Escolha a pasta antes de ligar o backup automático".into())}
 db.set(KEY,&cfg)?;Ok(cfg)
}
/// Nome do arquivo a data de hoje. O sufixo evita sobrescrever uma cópia manual.
fn stamp_name(folder:&Path,date:&str,seq:usize)->PathBuf{
 if seq==0{folder.join(format!("botlive-{date}.{EXT}"))}else{folder.join(format!("botlive-{date}-{seq}.{EXT}"))}
}
fn date_of(name:&str)->Option<chrono::NaiveDate>{
 let rest=name.strip_prefix("botlive-")?;
 chrono::NaiveDate::parse_from_str(rest.get(0..10)?,"%Y-%m-%d").ok()
}
/// Arquivos do BotLive na pasta, do mais recente para o mais antigo.
fn scan(folder:&str)->Result<Vec<(chrono::NaiveDate,PathBuf,u64)>,String>{
 let dir=Path::new(folder);
 if !dir.is_dir(){return Err("A pasta escolhida para os backups não existe mais".into())}
 let mut out=Vec::new();
 for entry in std::fs::read_dir(dir).map_err(|e|format!("Não foi possível ler a pasta: {e}"))? {
  let entry=entry.map_err(|e|e.to_string())?;
  let name=entry.file_name().to_string_lossy().into_owned();
  if let Some(date)=date_of(&name){
   let size=entry.metadata().map(|m|m.len()).unwrap_or(0);
   out.push((date,entry.path(),size));
  }
 }
 out.sort_by(|a,b|b.0.cmp(&a.0).then_with(||b.2.cmp(&a.2)));
 Ok(out)
}
fn week_start(d:chrono::NaiveDate)->chrono::NaiveDate{d-chrono::Duration::days(d.weekday().num_days_from_monday() as i64)}
fn month_index(d:chrono::NaiveDate)->i64{d.year() as i64*12+d.month() as i64}
/// Mantém as últimas N cópias por dia, por semana e por mês. Só apaga arquivos com o
/// nosso nome, na pasta escolhida: nada fora disso é tocado.
fn prune(rt:&Runtime,folder:&str)->Result<usize,String>{prune_since(rt,folder,chrono::Local::now().date_naive())}
/// Miolo da retenção com a data fixada: tudo é contado a partir dela.
fn prune_since(rt:&Runtime,folder:&str,today:chrono::NaiveDate)->Result<usize,String>{
 let cfg=config(&rt.db);
 let days=cfg["days"].as_i64().unwrap_or(7).clamp(1,365);
 let weeks=cfg["weeks"].as_i64().unwrap_or(4).clamp(1,52);
 let months=cfg["months"].as_i64().unwrap_or(12).clamp(1,60);
 let files=scan(folder)?;
 let daily_from=today-chrono::Duration::days(days-1);
 let week_from=week_start(today)-chrono::Duration::days(7*(weeks-1));
 let month_from=month_index(today)-(months-1);
 let mut keep:HashSet<&PathBuf>=HashSet::new();
 let mut week_seen:HashSet<chrono::NaiveDate>=HashSet::new();
 let mut month_seen:HashSet<(i32,u32)>=HashSet::new();
 for (date,path,_) in &files {
  if *date>=daily_from {keep.insert(path);continue}
  let ws=week_start(*date);
  if ws>=week_from&&week_seen.insert(ws){keep.insert(path);continue}
  if month_index(*date)>=month_from&&month_seen.insert((date.year(),date.month())){keep.insert(path);}
 }
 let mut removed=0;
 for (_,path,_) in &files {
  if keep.contains(path){continue}
  if std::fs::remove_file(path).is_ok(){removed+=1}
 }
 Ok(removed)
}
fn sql_value(v:&Value)->rusqlite::types::Value{
 match v {
  Value::Null=>rusqlite::types::Value::Null,
  Value::Bool(b)=>rusqlite::types::Value::Integer(*b as i64),
  Value::Number(n)=>n.as_i64().map(rusqlite::types::Value::Integer)
   .or_else(||n.as_f64().map(rusqlite::types::Value::Real))
   .unwrap_or(rusqlite::types::Value::Null),
  Value::String(s)=>rusqlite::types::Value::Text(s.clone()),
  Value::Array(_)|Value::Object(_)=>rusqlite::types::Value::Text(v.to_string()),
 }
}
fn read_tables(db:&Db)->Result<Value,String>{
 use rusqlite::types::ValueRef;
 let conn=db.0.lock().unwrap();
 let mut tables=serde_json::Map::new();
 for table in TABLES {
  let sql=format!("SELECT * FROM {table}");
  let mut stmt=conn.prepare(&sql).map_err(|e|e.to_string())?;
  let cols:Vec<String>=stmt.column_names().into_iter().map(|c|c.to_owned()).collect();
  let mut rows=Vec::new();
  let mut query=stmt.query([]).map_err(|e|e.to_string())?;
  loop {
   let Some(row)=query.next().map_err(|e|e.to_string())? else {break};
   let mut values=Vec::with_capacity(cols.len());
   for i in 0..cols.len() {
    let v=row.get_ref(i).map_err(|e|e.to_string())?;
    values.push(match v {
     ValueRef::Null=>Value::Null,
     ValueRef::Integer(n)=>json!(n),
     ValueRef::Real(f)=>json!(f),
     ValueRef::Text(t)=>Value::String(String::from_utf8_lossy(t).into_owned()),
     ValueRef::Blob(_)=>return Err(format!("A área {table} guarda um dado binário que o backup não transporta")),
    });
   }
   rows.push(Value::Array(values));
  }
  tables.insert(table.to_owned(),json!({"columns":cols,"rows":rows}));
 }
 Ok(Value::Object(tables))
}
fn rel_ok(path:&str)->bool{
 if path.is_empty()||path.len()>600||path.contains("..")||path.contains('\\')||path.contains(':')||path.starts_with('/')||path.starts_with('~'){return false}
 if path.chars().any(|c|c.is_control()){return false}
 matches!(path.split('/').next(),Some(head)if DIRS.contains(&head))&&path.contains('/')
}
fn read_files(base:&Path)->Result<Vec<Value>,String>{
 let mut out=Vec::new();
 for dir in DIRS {
  let root=base.join(dir);
  if !root.exists(){continue}
  walk(base,&root,&mut out)?
 }
 if out.len()>MAX_FILES{return Err("Há arquivos demais para um backup único".into())}
 Ok(out)
}
fn walk(base:&Path,dir:&Path,out:&mut Vec<Value>)->Result<(),String>{
 for entry in std::fs::read_dir(dir).map_err(|e|format!("Não foi possível ler {e}"))? {
  let entry=entry.map_err(|e|e.to_string())?;
  let path=entry.path();
  if path.is_dir(){walk(base,&path,out)?;continue}
  let meta=entry.metadata().map_err(|e|e.to_string())?;
  if !meta.is_file(){continue}
  if meta.len()>MAX_FILE_BYTES{return Err("Há um arquivo grande demais para o backup".into())}
  let rel=path.strip_prefix(base).map_err(|_|"Arquivo fora da pasta de dados".to_string())?.to_string_lossy().replace('\\',"/");
  if !rel_ok(&rel){return Err(format!("Arquivo em caminho inesperado: {rel}"))}
  let data=std::fs::read(&path).map_err(|e|format!("Não foi possível ler {rel}: {e}"))?;
  out.push(json!({"path":rel,"data":base64::engine::general_purpose::STANDARD.encode(data)}));
 }
 Ok(())
}
fn bundle(rt:&Runtime)->Result<Value,String>{
 let files=read_files(&rt.base)?;
 let tables=read_tables(&rt.db)?;
 Ok(json!({"format":FORMAT,"formatVersion":FORMAT_VERSION,"appVersion":env!("CARGO_PKG_VERSION"),"createdAt":chrono::Utc::now().to_rfc3339(),"tables":tables,"files":files}))
}
/// Cria um arquivo de backup na pasta e aplica a retenção em seguida.
pub fn create(rt:&Runtime,folder:&str)->Result<Value,String>{
 let folder=folder.trim();
 if folder.is_empty(){return Err("Escolha uma pasta para guardar os backups".into())}
 let dir=Path::new(folder);
 if !dir.is_dir(){return Err("A pasta escolhida para os backups não existe".into())}
 let payload=bundle(rt)?;
 let bytes=serde_json::to_string(&payload).map_err(|e|e.to_string())?.into_bytes();
 if bytes.len() as u64>MAX_BYTES{return Err("Backup muito grande para um arquivo só".into())}
 let date=chrono::Local::now().format("%Y-%m-%d").to_string();
 let mut path=stamp_name(dir,&date,0);
 let mut seq=0;
 while path.exists(){seq+=1;path=stamp_name(dir,&date,seq)}
 std::fs::write(&path,&bytes).map_err(|e|format!("Não foi possível gravar o backup: {e}"))?;
 let pruned=prune(rt,folder)?;
 if pruned>0{rt.log("","backup",&format!("{pruned} arquivo(s) antigo(s) removidos pela retenção"),"info")}
 Ok(json!({"path":path.to_string_lossy(),"name":path.file_name().unwrap_or_default().to_string_lossy(),"date":date,"size":bytes.len(),"pruned":pruned}))
}
/// Arquivos presentes na pasta de backups, do mais recente para o mais antigo.
pub fn list(db:&Db)->Result<Value,String>{
 let folder=config(db)["folder"].as_str().unwrap_or("").to_owned();
 if folder.is_empty(){return Ok(json!([]))}
 let files=scan(&folder)?;
 Ok(json!(files.into_iter().take(30).map(|(date,path,size)|json!({"name":path.file_name().unwrap_or_default().to_string_lossy(),"path":path.to_string_lossy(),"date":date.to_string(),"size":size})).collect::<Vec<_>>()))
}
fn validate(v:&Value)->Result<(&serde_json::Map<String,Value>,usize),String>{
 if v["format"].as_str()!=Some(FORMAT){return Err("Este arquivo não é um backup do BotLive".into())}
 let version=v["formatVersion"].as_u64().unwrap_or(0);
 if version==0||version>FORMAT_VERSION{return Err("Este backup veio de outra versão do BotLive. Importe com a versão que o criou.".into())}
 let tables=v["tables"].as_object().ok_or("Backup sem as áreas do bot")?;
 let mut rows_total=0usize;
 for table in TABLES {
  let spec=tables.get(table).ok_or(format!("Backup sem a área {table}"))?;
  let cols=spec["columns"].as_array().ok_or("Backup sem lista de colunas")?;
  if cols.is_empty(){return Err("Backup com colunas vazias".into())}
  for c in cols {
   let c=c.as_str().ok_or("Coluna inválida no backup")?;
   if c.is_empty()||c.len()>40||!c.chars().all(|ch|ch.is_ascii_lowercase()||ch.is_ascii_digit()||ch=='_'){return Err("Nome de coluna inválido no backup".into())}
  }
  let rows=spec["rows"].as_array().ok_or("Backup sem linhas")?;
  rows_total=rows_total.checked_add(rows.len()).ok_or("Backup grande demais")?;
  if rows_total>500_000{return Err("Backup com linhas demais".into())}
  for r in rows {if r.as_array().map_or(true,|row|row.len()!=cols.len()){return Err("Linha fora do formato no backup".into())}}
 }
 for (name,table) in tables {
  if !TABLES.contains(&name.as_str()){return Err("O backup tem uma área que este BotLive não conhece".into())}
  let _=table;
 }
 let files=v["files"].as_array().ok_or("Backup sem arquivos")?;
 if files.len()>MAX_FILES{return Err("Backup com arquivos demais".into())}
 let mut total=0usize;
 for f in files {
  let path=f["path"].as_str().ok_or("Arquivo sem caminho no backup")?;
  let data=f["data"].as_str().ok_or("Arquivo sem conteúdo no backup")?;
  if !rel_ok(path){return Err("Caminho de arquivo inválido no backup".into())}
  base64::engine::general_purpose::STANDARD.decode(data).map_err(|_|"Arquivo corrompido no backup".to_string())?;
  total=total.checked_add(data.len()).ok_or("Backup grande demais")?;
  if total as u64>MAX_BYTES{return Err("Backup com arquivos grandes demais".into())}
 }
 Ok((tables,rows_total))
}
fn write_tables(db:&Db,tables:&serde_json::Map<String,Value>)->Result<usize,String>{
 let mut conn=db.0.lock().unwrap();
 let tx=conn.transaction().map_err(|e|e.to_string())?;
 for table in TABLES.iter().rev(){tx.execute(&format!("DELETE FROM {table}"),[]).map_err(|e|e.to_string())?;}
 let mut total=0usize;
 for table in TABLES {
  let spec=&tables[table];
  let cols:Vec<String>=match spec["columns"].as_array(){Some(c)=>c.iter().filter_map(|c|c.as_str().map(str::to_owned)).collect(),None=>continue};
  if cols.is_empty(){continue}
  let sql=format!("INSERT INTO {table} ({}) VALUES ({})",cols.join(","),vec!["?";cols.len()].join(","));
  let rows=spec["rows"].as_array().ok_or("Backup sem linhas")?;
  for row in rows {
   let values:Vec<rusqlite::types::Value>=row.as_array().ok_or("Linha fora do formato no backup")?.iter().map(sql_value).collect();
   tx.execute(&sql,rusqlite::params_from_iter(values.iter())).map_err(|e|e.to_string())?;
   total+=1;
  }
 }
 tx.commit().map_err(|e|e.to_string())?;
 Ok(total)
}
fn write_files(base:&Path,files:&[Value])->Result<usize,String>{
 for dir in DIRS {
  let root=base.join(dir);
  if root.exists(){std::fs::remove_dir_all(&root).map_err(|e|format!("Não foi possível substituir a pasta {dir}: {e}"))?}
 }
 let mut total=0usize;
 for f in files {
  let rel=f["path"].as_str().unwrap_or("");
  if !rel_ok(rel){return Err("Caminho de arquivo inválido no backup".into())}
  let data=base64::engine::general_purpose::STANDARD.decode(f["data"].as_str().unwrap_or("")).map_err(|_|"Arquivo corrompido no backup".to_string())?;
  let dest=base.join(rel);
  if let Some(parent)=dest.parent(){std::fs::create_dir_all(parent).map_err(|e|e.to_string())?}
  std::fs::write(&dest,&data).map_err(|e|format!("Não foi possível gravar {rel}: {e}"))?;
  total+=1;
 }
 Ok(total)
}
/// Cópia de segurança gravada na pasta de dados antes de qualquer importação.
fn safety_copy(rt:&Runtime)->Option<PathBuf>{
 let bytes=match bundle(rt).and_then(|v|serde_json::to_string(&v).map_err(|e|e.to_string())){Ok(b)=>b,Err(_)=>return None};
 if bytes.len() as u64>MAX_BYTES{return None}
 let path=rt.base.join(format!("backup-antes-da-importacao-{}.{}",chrono::Local::now().format("%Y-%m-%d-%H%M%S"),EXT));
 std::fs::write(&path,&bytes).ok().map(|_|path)
}
/// Troca todas as áreas e arquivos pelos do arquivo escolhido.
pub fn restore(rt:&Runtime,path:&str)->Result<Value,String>{
 let meta=std::fs::metadata(path).map_err(|_|"Arquivo de backup não encontrado".to_string())?;
 if meta.len()>MAX_BYTES{return Err("Este arquivo é grande demais para um backup".into())}
 let text=std::fs::read_to_string(path).map_err(|_|"Não foi possível ler o arquivo de backup".to_string())?;
 let v:Value=serde_json::from_str(&text).map_err(|_|"Este arquivo não é um backup do BotLive".to_string())?;
 let (tables,_rows)=validate(&v)?;
 let files=v["files"].as_array().cloned().unwrap_or_default();
 let safety=safety_copy(rt);
 let old:Vec<String>=rt.db.profiles().unwrap_or_default().into_iter().map(|p|p.id).collect();
 let written=write_tables(&rt.db,tables)?;
 let file_count=write_files(&rt.base,&files)?;
 for id in &old{rt.disconnect(id);rt.conversation.lock().unwrap().clear(id);rt.live.clear(id);}
 let tasks:Vec<String>={rt.discord_tasks.lock().unwrap().keys().cloned().collect()};
 for id in tasks{if let Some(h)=rt.discord_tasks.lock().unwrap().remove(&id){h.abort()}}
 let mut flows=0;
 for p in rt.db.profiles().unwrap_or_default(){flows+=rt.db.flows(&p.id).map(|f|f.len()).unwrap_or(0)}
 Ok(json!({"profiles":rt.db.profiles().map(|p|p.len()).unwrap_or(0),"flows":flows,"rows":written,"files":file_count,"safety":safety.map(|s|Value::String(s.to_string_lossy().into_owned()))}))
}
/// Chamado pelo agendador: grava um arquivo por dia quando o backup automático está ligado.
pub fn tick(rt:&Runtime){
 let cfg=config(&rt.db);
 if cfg["auto"]!=true{return}
 let folder=cfg["folder"].as_str().unwrap_or("").to_owned();
 if folder.is_empty(){return}
 let now=chrono::Utc::now().timestamp();
 if now.saturating_sub(cfg["lastSuccess"].as_i64().unwrap_or(0))<86400{return}
 if now.saturating_sub(cfg["lastTry"].as_i64().unwrap_or(0))<3600{return}
 let mut attempt=cfg.clone();attempt["lastTry"]=json!(now);let _=rt.db.set(KEY,&attempt);
 match create(rt,&folder){
  Ok(v)=>{
   let mut done=config(&rt.db);done["lastSuccess"]=json!(now);done["lastTry"]=json!(now);let _=rt.db.set(KEY,&done);
   rt.log("","backup",&format!("Backup diário salvo: {}",v["name"]),"success")
  },
  Err(e)=>rt.log("","backup",&format!("Backup diário não foi salvo: {e}"),"error"),
 }
}
#[cfg(test)] mod tests {
 use super::*;
 use crate::engine::Runtime;
 fn profile()->crate::model::Profile{serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Backup","platform":"twitch","channel":"canal"})).unwrap()}
 #[test] fn only_relative_paths_inside_the_content_folders_are_accepted() {
  assert!(rel_ok("knowledge/tom.md"));
  assert!(rel_ok("media/som com espaço.wav"));
  assert!(rel_ok("vaults/usuarios/ana.md"));
  assert!(!rel_ok(""));
  assert!(!rel_ok("knowledge"));
  assert!(!rel_ok("knowledge/../botlive.sqlite"),"fora da pasta de conteúdo não pode");
  assert!(!rel_ok("../knowledge/a.md"));
  assert!(!rel_ok("/knowledge/a.md"));
  assert!(!rel_ok("C:/knowledge/a.md"));
  assert!(!rel_ok("knowledge\\a.md"));
  assert!(!rel_ok("cofre/segredo.md"),"só as três pastas conhecidas");
  assert!(!rel_ok("knowledge/a\u{0}b"));
  assert_eq!(date_of("botlive-2026-09-27.botlivebak").map(|d|d.to_string()),Some("2026-09-27".to_string()));
  assert_eq!(date_of("botlive-2026-09-27-3.botlivebak").map(|d|d.to_string()),Some("2026-09-27".to_string()));
  assert!(date_of("backup-antes-da-importacao-x.botlivebak").is_none());
  assert!(date_of("notas.txt").is_none());
 }
 #[tokio::test] async fn backup_round_trip_covers_every_area_and_keeps_credentials_out() {
  let dir=tempfile::tempdir().unwrap();
  let rt=Runtime::new(dir.path().join("app")).unwrap();
  let p=profile();rt.db.save_profile(&p).unwrap();
  let flow:crate::model::Flow=serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"profileId":p.id,"name":"Ifood","enabled":true,"trigger":{"kind":"command","pattern":"!ifood"},"actions":[{"kind":"chat","text":"{{commandCount}}º"}]})).unwrap();
  rt.db.save_flow(&flow).unwrap();
  std::fs::create_dir_all(rt.base.join("knowledge")).unwrap();
  std::fs::write(rt.base.join("knowledge").join("tom.md"),"conteúdo original").unwrap();
  let folder=dir.path().join("backups");
  std::fs::create_dir_all(&folder).unwrap();
  let folder=folder.to_string_lossy().into_owned();
  let saved=save_config(&rt.db,&json!({"folder":folder,"days":7,"weeks":4,"months":12})).unwrap();
  assert_eq!(saved["folder"].as_str(),Some(folder.as_str()));
  assert!(save_config(&rt.db,&json!({"days":0})).is_err(),"retenção fora do intervalo");
  assert!(save_config(&rt.db,&json!({"weeks":99})).is_err());
  assert!(save_config(&rt.db,&json!({"folder":dir.path().join("não existe").to_string_lossy()})).is_err());
  assert!(save_config(&rt.db,&json!({"auto":true,"folder":""})).is_err(),"automático exige pasta");
  let made=create(&rt,&folder).unwrap();
  let path=made["path"].as_str().unwrap().to_owned();
  assert!(std::path::Path::new(&path).is_file());
  assert_eq!(list(&rt.db).unwrap().as_array().map(|v|v.len()),Some(1));
  let text=std::fs::read_to_string(&path).unwrap();
  assert!(text.contains(FORMAT),"é o formato do BotLive");
  let lower=text.to_lowercase();
  assert!(!lower.contains("bot_token")&&!lower.contains("channel_token")&&!lower.contains("refresh_token"),"o cofre de credenciais não entra no arquivo");
  // Apaga tudo e importa de volta.
  rt.db.delete_flow(&p.id,&flow.id).unwrap();
  std::fs::write(rt.base.join("knowledge").join("tom.md"),"alterado").unwrap();
  let back=restore(&rt,&path).unwrap();
  assert_eq!(back["files"].as_u64(),Some(1));
  assert_eq!(back["profiles"].as_u64(),Some(1));
  assert!(rt.db.flows(&p.id).unwrap().iter().any(|f|f.id==flow.id),"o fluxo volta");
  assert_eq!(std::fs::read_to_string(rt.base.join("knowledge").join("tom.md")).unwrap(),"conteúdo original");
  let safety=back["safety"].as_str().expect("cópia de segurança antes de importar");
  assert!(std::path::Path::new(safety).is_file());
  let estranho=dir.path().join("estranho.botlivebak");
  std::fs::write(&estranho,"{\"format\":\"de outra coisa\"}").unwrap();
  assert!(restore(&rt,&estranho.to_string_lossy()).is_err(),"arquivo de outra origem é recusado");
  assert!(restore(&rt,&dir.path().join("sumiu.botlivebak").to_string_lossy()).is_err());
  // Um segundo arquivo do mesmo dia não sobrescreve o primeiro.
  let second=create(&rt,&folder).unwrap();
  assert_ne!(second["path"].as_str().unwrap(),path);
  assert!(std::path::Path::new(&path).is_file());
 }
 #[tokio::test] async fn retention_keeps_recent_days_and_a_sample_of_weeks_and_months() {
  let dir=tempfile::tempdir().unwrap();
  let rt=Runtime::new(dir.path().join("app")).unwrap();
  let folder=dir.path().join("bk");std::fs::create_dir_all(&folder).unwrap();
  let folder=folder.to_string_lossy().into_owned();
  save_config(&rt.db,&json!({"folder":folder,"days":2,"weeks":1,"months":1})).unwrap();
  // Domingo: segunda dessa semana é 2026-09-21.
  let today=chrono::NaiveDate::from_ymd_opt(2026,9,27).unwrap();
  for date in ["2026-09-27","2026-09-26","2026-09-25","2026-09-20","2026-08-30","2025-11-01"] {
   std::fs::write(Path::new(&folder).join(format!("botlive-{date}.botlivebak")),vec![b'x';64]).unwrap();
  }
  std::fs::write(Path::new(&folder).join("notas-do-streamer.txt"),"não é nosso").unwrap();
  let pruned=prune_since(&rt,&folder,today).unwrap();
  assert_eq!(pruned,2,"sai o do mês anterior e o do ano passado");
  let mut restantes=std::fs::read_dir(&folder).unwrap().filter_map(|e|e.ok()).map(|e|e.file_name().to_string_lossy().into_owned()).filter(|n|n.starts_with("botlive-")).collect::<Vec<_>>();
  restantes.sort();
  assert_eq!(restantes,vec!["botlive-2026-09-20.botlivebak","botlive-2026-09-25.botlivebak","botlive-2026-09-26.botlivebak","botlive-2026-09-27.botlivebak"]);
  assert!(Path::new(&folder).join("notas-do-streamer.txt").is_file(),"arquivo que não é nosso nunca é tocado");
  assert_eq!(prune_since(&rt,&folder,today).unwrap(),0,"a segunda passagem não muda mais nada");
 }
}
