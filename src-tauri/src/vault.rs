use crate::model::{Note,valid_id};
use std::{fs,path::{Path,PathBuf}};
pub fn root(base:&Path,p:&str)->Result<PathBuf,String> {
 if !valid_id(p) {return Err("Perfil inválido".into())}
 let vaults=base.join("vaults");fs::create_dir_all(&vaults).map_err(|e|e.to_string())?;
 let base_canonical=base.canonicalize().map_err(|e|e.to_string())?;
 let parent=vaults.canonicalize().map_err(|e|e.to_string())?;
 if !parent.starts_with(&base_canonical){return Err("Pasta de vault fora dos dados do aplicativo".into())}
 let root=vaults.join(p);fs::create_dir_all(&root).map_err(|e|e.to_string())?;
 if !root.canonicalize().map_err(|e|e.to_string())?.starts_with(&parent){return Err("Perfil aponta para fora dos vaults".into())}
 for d in ["usuarios","eventos","contexto-live"] {fs::create_dir_all(root.join(d)).map_err(|e|e.to_string())?;}
 let config=root.join("config-memoria.md");
 if !config.exists(){fs::write(config,"---\ntags: [config]\n---\n# Regras de memória\nRegistre apenas fatos úteis à comunidade. Não registre segredos ou dados sensíveis.\n").map_err(|e|e.to_string())?}
 Ok(root)
}
fn safe(root:&Path,name:&str)->Result<PathBuf,String> {
 if name.contains('\\')||name.split('/').any(|s| s.is_empty()||s=="."||s==".."||s.contains(':'))||!name.ends_with(".md") {return Err("Use um caminho Markdown dentro deste vault".into())}
 let parts:Vec<_>=name.split('/').collect();
 if parts.len()>2|| (parts.len()==2&&!["usuarios","eventos","contexto-live"].contains(&parts[0])) {return Err("Pasta de memória inválida".into())}
 let file=root.join(name);
 let canonical=root.canonicalize().map_err(|e|e.to_string())?;
 let parent=file.parent().unwrap().canonicalize().map_err(|e|e.to_string())?;
 if !parent.starts_with(&canonical) {return Err("Caminho fora do vault".into())}
 if file.exists() && !file.canonicalize().map_err(|e|e.to_string())?.starts_with(&canonical) {return Err("Link fora do vault".into())}
 Ok(file)
}
pub fn list(base:&Path,p:&str)->Result<Vec<Note>,String> {
 read_notes(base,p,|_|true)
}
fn read_notes(base:&Path,p:&str,include:impl Fn(&str)->bool)->Result<Vec<Note>,String> {
 let r=root(base,p)?;let mut notes=vec![];
 for dir in ["","usuarios","eventos","contexto-live"] {
 for item in fs::read_dir(r.join(dir)).map_err(|e|e.to_string())? {
 let item=item.map_err(|e|e.to_string())?;
 if item.file_type().map_err(|e|e.to_string())?.is_file()&&item.path().extension().is_some_and(|e|e=="md") {
 let path=if dir.is_empty(){item.file_name().to_string_lossy().into_owned()}else{format!("{dir}/{}",item.file_name().to_string_lossy())};
 if !include(&path){continue}
 let content=fs::read_to_string(safe(&r,&path)?).map_err(|e|e.to_string())?;
 notes.push(Note{path,content});
 }
 }
 }
 notes.sort_by(|a,b|a.path.cmp(&b.path));Ok(notes)
}
pub fn write(base:&Path,p:&str,name:&str,content:&str,append:bool)->Result<(),String> {
 if content.len()>1_000_000 {return Err("Nota deve ter menos de 1 MB".into())}
 let r=root(base,p)?;let path=safe(&r,name)?;
 let next=if append {
 let existing=fs::read_to_string(&path).unwrap_or_else(|_|"---\ntags: [botlive]\n---\n".into());
 format!("{existing}\n- {} — {content}\n",chrono::Utc::now().to_rfc3339())
 } else {content.to_owned()};
 let temp=path.with_extension("md.tmp");fs::write(&temp,next).map_err(|e|e.to_string())?;
 fs::rename(&temp,&path).map_err(|e|e.to_string())
}
pub fn delete(base:&Path,p:&str,name:&str)->Result<(),String> {fs::remove_file(safe(&root(base,p)?,name)?).map_err(|e|e.to_string())}
fn auto_path(platform:&str,id:&str)->Option<String> {
 if id.is_empty()||id.len()>80||platform.len()>16{return None}
 let key=format!("{platform}:{id}");
 Some(format!("usuarios/auto-{}.md",key.as_bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()))
}
/// Registro separado das notas manuais, limitado e sem chamadas extras ao provedor.
pub fn record_interaction(base:&Path,p:&str,platform:&str,id:&str,user:&str,message:&str)->Result<(),String>{
 let Some(name)=auto_path(platform,id) else{return Ok(())};
 let message=message.split_whitespace().collect::<Vec<_>>().join(" ");
 if message.chars().count()<8||message.starts_with('!'){return Ok(())}
 let message:String=message.chars().take(500).collect();
 let r=root(base,p)?;let path=safe(&r,&name)?;
 let existing=if path.exists(){fs::read_to_string(path).map_err(|e|e.to_string())?}else{String::new()};
 let mut rows:Vec<String>=existing.lines().filter(|l|l.starts_with("- ")).map(str::to_owned).collect();
 // JSON guarda a fala sem permitir que quebras de linha injetem novos registros.
 let encoded=serde_json::to_string(&message).map_err(|e|e.to_string())?;
 rows.retain(|line|!line.ends_with(&format!(" | {encoded}")));
 rows.push(format!("- {} | {}",chrono::Utc::now().to_rfc3339(),encoded));
 if rows.len()>100{rows.drain(..rows.len()-100);}
 let user=serde_json::to_string(&user.chars().take(100).collect::<String>()).map_err(|e|e.to_string())?;
 write(base,p,&name,&format!("# Interações automáticas\nPessoa: {user}\nPlataforma: {platform}\nFalas declaradas, não fatos verificados.\n\n{}\n",rows.join("\n")),false)
}
fn words(text:&str)->Vec<String>{
 let mut words:Vec<String>=text.to_lowercase().split(|c:char|!c.is_alphanumeric()).filter(|s|s.chars().count()>2&&!matches!(*s,"que"|"para"|"com"|"uma"|"não"|"hoje"|"como"|"isso"|"essa"|"esse")).map(str::to_owned).collect();
 words.sort();words.dedup();words.truncate(32);words
}
pub fn context(base:&Path,p:&str,user:&str,query:&str)->Result<String,String> {context_for(base,p,user,"","",query)}
pub fn context_for(base:&Path,p:&str,user:&str,id:&str,platform:&str,query:&str)->Result<String,String> {
 let query=words(query);let own=auto_path(platform,id);let user=user.to_lowercase();
 let mut notes=vec![];
 let selected=read_notes(base,p,|path|path!="config-memoria.md"&&(!path.starts_with("usuarios/auto-")||own.as_deref()==Some(path)))?;
 for n in selected {
  let is_auto=n.path.starts_with("usuarios/auto-");
  // Históricos automáticos são pessoais: nunca recuperar o de outro espectador.
  if is_auto&&own.as_deref()!=Some(n.path.as_str()){continue}
  let legacy=!id.is_empty()&&n.path==format!("usuarios/{id}.md");
  let named=!user.is_empty()&&(n.path.to_lowercase()==format!("usuarios/{user}.md")||n.content.split(|c:char|!c.is_alphanumeric()).any(|w|w.to_lowercase()==user));
  let bonus=if is_auto||legacy{20}else if named{10}else if n.path.starts_with("contexto-live/"){2}else{0};
  let mut lines:Vec<_>=n.content.lines().enumerate().filter(|(_,l)|!l.trim().is_empty()).map(|(index,line)|{
   let tokens=words(line);let relevance=query.iter().filter(|w|tokens.contains(w)).count();(relevance,index,line)
  }).collect();
  let relevance=lines.iter().map(|r|r.0).max().unwrap_or(0);
  if relevance+bonus==0{continue}
  lines.sort_by(|a,b|b.0.cmp(&a.0).then_with(||b.1.cmp(&a.1)));
  let mut excerpt=String::new();
  for (_,_,line) in lines {let remaining=1400usize.saturating_sub(excerpt.chars().count());if remaining<40{break}excerpt.push_str(&line.chars().take(remaining-1).collect::<String>());excerpt.push('\n');}
  notes.push((bonus+relevance,n.path,excerpt));
 }
 notes.sort_by(|a,b|b.0.cmp(&a.0).then_with(||a.1.cmp(&b.1)));
 Ok(notes.into_iter().take(4).map(|(_,path,text)|format!("{path}\n{text}")).collect::<Vec<_>>().join("\n\n").chars().take(6000).collect())
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn automatic_memory_is_bounded_deduplicated_and_identity_scoped(){
  let d=tempfile::tempdir().unwrap();let p=uuid::Uuid::new_v4().to_string();
  for i in 0..105{record_interaction(d.path(),&p,"twitch","42","Ana",&format!("Prefiro xadrez variante {i}")).unwrap();}
  record_interaction(d.path(),&p,"twitch","42","Ana","Prefiro xadrez variante 104").unwrap();
  let notes=list(d.path(),&p).unwrap();let note=notes.iter().find(|n|n.path.starts_with("usuarios/auto-")).unwrap();
  assert_eq!(note.content.lines().filter(|l|l.starts_with("- ")).count(),100);
  assert_eq!(note.content.matches("variante 104").count(),1);
  assert!(!note.content.contains("variante 0\""));
  assert!(context_for(d.path(),&p,"Novo apelido","42","twitch","xadrez").unwrap().contains("variante 104"));
  assert!(context_for(d.path(),&p,"Ana","42","discord","xadrez").unwrap().is_empty());
  assert!(context_for(d.path(),&p,"Ana","43","twitch","xadrez").unwrap().is_empty());
  write(d.path(),&p,"eventos/final.md","Torneio de xadrez vencido por Bia",false).unwrap();
  assert!(context_for(d.path(),&p,"Ana","43","twitch","xadrez!").unwrap().contains("Torneio"));
 }
 #[test] fn vault_isolation_and_traversal(){
 let d=tempfile::tempdir().unwrap();let a=uuid::Uuid::new_v4().to_string();let b=uuid::Uuid::new_v4().to_string();
 write(d.path(),&a,"usuarios/ana.md","Ana gosta de xadrez",true).unwrap();
 assert!(context(d.path(),&a,"ana","xadrez").unwrap().contains("xadrez"));
 assert!(context(d.path(),&b,"ana","xadrez").unwrap().is_empty());
 assert!(write(d.path(),&a,"../escape.md","x",false).is_err());
 assert!(write(d.path(),&a,"usuarios/C:escape.md","x",false).is_err());
 }
}
