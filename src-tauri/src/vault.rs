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
 let r=root(base,p)?;let mut notes=vec![];
 for dir in ["","usuarios","eventos","contexto-live"] {
 for item in fs::read_dir(r.join(dir)).map_err(|e|e.to_string())? {
 let item=item.map_err(|e|e.to_string())?;
 if item.file_type().map_err(|e|e.to_string())?.is_file()&&item.path().extension().is_some_and(|e|e=="md") {
 let path=if dir.is_empty(){item.file_name().to_string_lossy().into_owned()}else{format!("{dir}/{}",item.file_name().to_string_lossy())};
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
pub fn context(base:&Path,p:&str,user:&str,query:&str)->Result<String,String> {
 let words:Vec<_>=query.to_lowercase().split_whitespace().filter(|s|s.len()>2).take(20).map(str::to_owned).collect();
 let mut notes:Vec<_>=list(base,p)?.into_iter().filter(|n|n.path!="config-memoria.md").map(|n|{
 let text=n.content.to_lowercase();
 let score=words.iter().filter(|w|text.contains(w.as_str())).count()+ if !user.is_empty() && (text.contains(&user.to_lowercase())||n.path.contains(user)){10}else{0}+if n.path.starts_with("contexto-live/"){2}else{0};
 (score,n)
 }).filter(|(s,_)|*s>0).collect();
 notes.sort_by(|a,b|b.0.cmp(&a.0));
 Ok(notes.into_iter().take(4).map(|(_,n)|format!("{}\n{}",n.path,n.content)).collect::<Vec<_>>().join("\n\n").chars().take(6000).collect())
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn vault_isolation_and_traversal(){
 let d=tempfile::tempdir().unwrap();let a=uuid::Uuid::new_v4().to_string();let b=uuid::Uuid::new_v4().to_string();
 write(d.path(),&a,"usuarios/ana.md","Ana gosta de xadrez",true).unwrap();
 assert!(context(d.path(),&a,"ana","xadrez").unwrap().contains("xadrez"));
 assert!(context(d.path(),&b,"ana","xadrez").unwrap().is_empty());
 assert!(write(d.path(),&a,"../escape.md","x",false).is_err());
 assert!(write(d.path(),&a,"usuarios/C:escape.md","x",false).is_err());
 }
}
