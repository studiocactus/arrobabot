//! Base de conhecimento importada: arquivos `.md` que a IA lê antes de responder.
//! A pasta escolhida pelo usuário é copiada para os dados do perfil e daí em diante
//! o aplicativo é autônomo: mover ou apagar a pasta original não quebra nada.
use crate::model::{Event,Profile,valid_id};
use serde_json::{json,Value};
use std::{collections::BTreeMap,fs,io::Read,path::{Path,PathBuf}};
type R<T>=Result<T,String>;
const MAX_FILES:usize=400;
const MAX_FILE:u64=200_000;
const MAX_TOTAL:u64=12_000_000;
const MAX_WALK:usize=3;
/// Orçamento de caracteres do bloco `<conhecimento>` por profundidade.
pub const DEPTHS:[(&str,usize);3]=[("light",5000),("standard",13000),("full",20000)];

#[derive(Clone)]
pub struct Entry { pub path:String, pub file:String, pub category:String, pub title:String, pub kind:String, pub size:usize }
pub fn depth_budget(name:&str)->usize { DEPTHS.iter().find(|(n,_)|*n==name).map(|(_,b)|*b).unwrap_or(DEPTHS[1].1) }

pub fn root(base:&Path,p:&str)->R<PathBuf> {
 if !valid_id(p) {return Err("Perfil inválido".into());}
 let holder=base.join("knowledge");fs::create_dir_all(&holder).map_err(|e|e.to_string())?;
 let base=base.canonicalize().map_err(|e|e.to_string())?;
 let holder=holder.canonicalize().map_err(|e|e.to_string())?;
 if !holder.starts_with(&base) {return Err("Pasta de conhecimento fora dos dados do aplicativo".into());}
 let root=holder.join(p);fs::create_dir_all(&root).map_err(|e|e.to_string())?;
 if !root.canonicalize().map_err(|e|e.to_string())?.starts_with(&holder) {return Err("Perfil aponta para fora do conhecimento".into());}
 Ok(root)
}
fn safe(root:&Path,rel:&str)->R<PathBuf> {
 if rel.contains('\\')||rel.split('/').any(|s|s.is_empty()||s=="."||s==".."||s.contains(':')) {return Err("Caminho de arquivo inválido".into());}
 Ok(root.join(rel))
}
fn category(rel:&str)->String { rel.rsplit_once('/').map(|(c,_)|c).unwrap_or("geral").to_owned() }
fn file_of(rel:&str)->String { rel.rsplit_once('/').map(|(_,f)|f).unwrap_or(rel).trim_end_matches(".md").to_owned() }
/// Percorre `dir` anotando o caminho relativo a `root`; ignora links e nada de `.md`.
fn walk(root:&Path,dir:&Path,depth:usize,out:&mut Vec<(String,PathBuf)>) {
 if depth>MAX_WALK {return;}
 let Ok(items)=fs::read_dir(dir) else {return;};
 let mut items:Vec<_>=items.filter_map(|i|i.ok()).collect();items.sort_by_key(|i|i.file_name());
 for item in items {
  let path=item.path();let Ok(kind)=item.file_type() else {continue};
  if kind.is_symlink() {continue;}
  if kind.is_dir() {walk(root,&path,depth+1,out);continue;}
  if !matches!(path.extension(),Some(e) if e.eq_ignore_ascii_case("md")) {continue;}
  let Ok(rel)=path.strip_prefix(root) else {continue};
  let rel=rel.to_string_lossy().replace('\\',"/");
  if rel.split('/').any(|s|s.is_empty()||s=="."||s==".."||s.contains(':')) {continue;}
  out.push((rel,path));
 }
}
fn scan(dir:&Path)->Vec<(String,PathBuf)> {let mut out=vec![];walk(dir,dir,0,&mut out);out}
/// Frontmatter simples + primeiro título: suficiente para exibir e para filtrar.
fn meta(text:&str)->(String,String) {
 let mut kind=String::new();let mut title=String::new();
 let mut lines=text.lines();
 if lines.next().map(str::trim)==Some("---") {
  for line in lines {
   if line.trim()=="---" {break;}
   if let Some((key,value))=line.split_once(':') {if key.trim()=="tipo" {kind=value.trim().to_owned();}}
  }
 }
 for line in text.lines() {if let Some(rest)=line.strip_prefix("# ") {title=rest.trim().to_owned();break;}}
 (kind,title.chars().take(120).collect())
}
fn body(text:&str)->&str {
 let t=text.trim_start();
 if let Some(rest)=t.strip_prefix("---") {
  if let Some(end)=rest.find("\n---") {return rest[end+4..].trim_start();}
 }
 text
}
pub fn list(base:&Path,p:&str)->R<Vec<Entry>> {
 let root=root(base,p)?;let mut out=vec![];
 for (rel,path) in scan(&root) {
  let Ok(data)=fs::metadata(&path) else {continue};
  let mut head=[0u8;4096];
  let text=match fs::File::open(&path).and_then(|mut f|f.read(&mut head)) {Ok(n)=>String::from_utf8_lossy(&head[..n]).into_owned(),Err(_)=>String::new()};
  let (kind,title)=meta(&text);
  out.push(Entry{category:category(&rel),file:file_of(&rel),path:rel,title,kind,size:data.len() as usize});
 }
 out.sort_by(|a,b|a.path.cmp(&b.path));Ok(out)
}
pub fn remove(base:&Path,p:&str,rel:&str)->R<()> {
 if !rel.ends_with(".md") {return Err("Arquivo inválido".into());}
 fs::remove_file(safe(&root(base,p)?,rel)?).map_err(|_|"Arquivo não encontrado na base".into())
}
/// Copia a pasta escolhida para os dados do perfil, substituindo a importação anterior.
pub fn import(base:&Path,p:&str,source:&str)->R<Value> {
 let src=PathBuf::from(source);
 if !src.is_dir() {return Err("Escolha uma pasta existente com arquivos .md".into());}
 let src=src.canonicalize().map_err(|e|e.to_string())?;
 let root=root(base,p)?;
 let dest=root.canonicalize().map_err(|e|e.to_string())?;
 if dest.starts_with(&src)||src.starts_with(&dest) {return Err("Escolha uma pasta fora dos dados do aplicativo".into());}
 let found=scan(&src);
 if found.is_empty() {return Err("A pasta escolhida não tem arquivos .md".into());}
 if found.len()>MAX_FILES {return Err(format!("A pasta tem {} arquivos .md; o limite é {MAX_FILES}.",found.len()));}
 let mut total=0u64;
 for (_,path) in &found {
  let data=fs::metadata(path).map_err(|e|e.to_string())?;
  if data.len()>MAX_FILE {return Err(format!("Arquivo grande demais: {}",path.file_name().map(|n|n.to_string_lossy().into_owned()).unwrap_or_default()));}
  total+=data.len();
 }
 if total>MAX_TOTAL {return Err("A pasta soma mais de 12 MB; divida em pastas menores".into());}
 let count=found.len();
 fs::remove_dir_all(&root).map_err(|e|e.to_string())?;fs::create_dir_all(&root).map_err(|e|e.to_string())?;
 let mut categories:BTreeMap<String,usize>=BTreeMap::new();
 for (rel,path) in found {
  let text=fs::read_to_string(&path).map_err(|_|format!("Arquivo que não é texto UTF-8: {rel}"))?;
  let dest=safe(&root,&rel)?;
  if let Some(parent)=dest.parent() {fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
  fs::write(&dest,text.as_bytes()).map_err(|e|e.to_string())?;
  *categories.entry(category(&rel)).or_insert(0)+=1;
 }
 Ok(json!({"files":count,"categories":categories,"source":source}))
}
/// Evento → nome do arquivo de reação da base.
fn hint(e:Option<&Event>)->&'static str {
 match e.map(|e|e.kind.as_str()) {
  Some("subscription")|Some("cheer")|Some("redemption")|Some("gift")|Some("donation") => "sub-doacao",
  Some("clip")|Some("clip_created") => "clip-highlight",
  Some("category") => "troca-de-jogo",
  _ => "",
 }
}
/// Monta o bloco de conhecimento: regras fixas, nicho escolhido, evento da hora,
/// gírias e exemplos do canal, sempre dentro do orçamento da profundidade escolhida.
pub fn context(base:&Path,p:&Profile,e:Option<&Event>,query:&str)->String {
 if !p.ai.knowledge {return String::new();}
 context_enabled(base,p,e,query)
}
/// O chamador já resolveu a escolha do perfil/bloco; preserva os arquivos desligados.
pub fn context_enabled(base:&Path,p:&Profile,e:Option<&Event>,query:&str)->String {
 let root=match root(base,&p.id) {Ok(r)=>r,Err(_)=>return String::new()};
 let found=scan(&root);
 if found.is_empty() {return String::new();}
 let niche=p.ai.knowledge_nicho.trim().to_lowercase();
 let channel=p.channel.trim().to_lowercase();
 let event=hint(e);
 let mut loaded=vec![];
 for (rel,path) in found {
  if p.ai.knowledge_off.iter().any(|o|*o==rel) {continue;}
  // Não abrir arquivos de outros nichos/canais/eventos a cada mensagem.
  let cat=category(&rel);let file=file_of(&rel).to_lowercase();
  if (cat=="nichos"&&(niche.is_empty()||file!=niche))
   ||(cat=="canais"&&(channel.is_empty()||(file!=channel&&file!=format!("{channel}-exemplos-reais"))))
   ||(cat=="eventos-de-live"&&(event.is_empty()||!file.contains(event))){continue}
  let Ok(text)=fs::read_to_string(&path) else {continue};
  let (kind,title)=meta(&text);
  if kind=="documento_de_logica" {continue;}
  loaded.push((Entry{category:category(&rel),file:file_of(&rel),path:rel,title,kind,size:0},body(&text).to_owned()));
 }
 let (mut tom,mut nicho,mut evento,mut girias,mut canal,mut outros)=(vec![],vec![],vec![],vec![],vec![],vec![]);
 for item in loaded {
  match item.0.category.as_str() {
   "tom-e-comportamento"=>tom.push(item),
   "girias"=>girias.push(item),
   "nichos" if !niche.is_empty()&&item.0.file.to_lowercase()==niche=>nicho.push(item),
   "nichos"=>{},
   "eventos-de-live" if !event.is_empty()&&item.0.file.contains(event)=>evento.push(item),
   "eventos-de-live"=>{},
   "canais" if !channel.is_empty()&&(item.0.file.to_lowercase()==channel||item.0.file.to_lowercase()==format!("{channel}-exemplos-reais"))=>canal.push(item),
   "canais"=>{},
   _=>outros.push(item),
  }
 }
 let words:Vec<String>=query.to_lowercase().split_whitespace().filter(|s|s.len()>2).take(24).map(str::to_owned).collect();
 outros.sort_by(|a,b|score(&b.1,&words).cmp(&score(&a.1,&words)));outros.truncate(4);
 let mut order=tom;order.extend(nicho);order.extend(evento);order.extend(canal);order.extend(girias);order.extend(outros);
 let budget=depth_budget(&p.ai.knowledge_depth);
 let mut out=Vec::new();let mut used=0usize;
 let count=order.len().max(1);
 let allowance=(budget/count).min(2200);
 for (entry,text) in order {
  let header=format!("{}\n",entry.path);
  let room=allowance.saturating_sub(header.chars().count()+2);
  let mut paragraphs:Vec<_>=text.split("\n\n").enumerate().collect();
  paragraphs.sort_by(|a,b|score(b.1,&words).cmp(&score(a.1,&words)).then_with(||a.0.cmp(&b.0)));
  let mut excerpt=String::new();
  for (_,paragraph) in paragraphs {
   let remaining=room.saturating_sub(excerpt.chars().count());if remaining<20{break}
   excerpt.push_str(&paragraph.chars().take(remaining.saturating_sub(2)).collect::<String>());excerpt.push_str("\n\n");
  }
  let piece=format!("{header}{}",excerpt.trim());
  let size=piece.chars().count()+2;
  if used+size<=budget {used+=size;out.push(piece);}
 }
 out.join("\n\n")
}
fn score(text:&str,words:&[String])->usize {let t=text.to_lowercase();words.iter().filter(|w|t.contains(w.as_str())).count()}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn corpus_and_each_category_fit_without_loading_whole_files(){
  let d=tempfile::tempdir().unwrap();let src=d.path().join("origem");
  for name in ["tom-e-comportamento/tom.md","nichos/fps.md","canais/canal.md","canais/canal-exemplos-reais.md","girias/geral.md"]{write(&src,name,&format!("# Exemplo\n\n{}","mira boa áéíóú ".repeat(1500)));}
  let id=uuid::Uuid::new_v4().to_string();import(d.path(),&id,src.to_str().unwrap()).unwrap();
  let mut p=profile(&id,"canal");p.ai.knowledge_depth="light".into();p.ai.knowledge_nicho="fps".into();
  let out=context(d.path(),&p,None,"mira");assert!(out.chars().count()<=5000);
  for name in ["tom-e-comportamento/tom.md","nichos/fps.md","canais/canal.md","canais/canal-exemplos-reais.md","girias/geral.md"]{assert!(out.contains(name));}
  p.ai.knowledge=false;assert!(context(d.path(),&p,None,"").is_empty());assert!(!context_enabled(d.path(),&p,None,"").is_empty());
 }
 fn write(dir:&Path,rel:&str,text:&str){let path=dir.join(rel);fs::create_dir_all(path.parent().unwrap()).unwrap();fs::write(path,text).unwrap();}
 fn profile(id:&str,channel:&str)->Profile{serde_json::from_value(json!({"id":id,"name":"P","platform":"twitch","channel":channel})).unwrap()}
 #[test]
 fn import_list_and_select_by_priority(){
  let d=tempfile::tempdir().unwrap();let src=d.path().join("origem");
  write(&src,"tom-e-comportamento/anti-padroes.md","---\ntipo: exemplo\n---\n# Anti-padrões\nnunca diga olá mundo");
  write(&src,"nichos/fps.md","# FPS\ncsgo e valorant");
  write(&src,"nichos/terror.md","# Terror\nsusto");
  write(&src,"eventos-de-live/reacao-a-sub-doacao.md","# Sub\nvlw pela sub");
  write(&src,"sistema/regras.md","---\ntipo: documento_de_logica\n---\n# Regras\ndocumento de lógica, não de estilo");
  write(&src,"leia-me.txt","não é markdown");
  let id=uuid::Uuid::new_v4().to_string();
  let made=import(d.path(),&id,src.to_str().unwrap()).unwrap();
  assert_eq!(made["files"],5);
  let items=list(d.path(),&id).unwrap();
  assert_eq!(items.len(),5);
  assert!(items.iter().any(|e|e.category=="tom-e-comportamento"&&e.title=="Anti-padrões"));
  let mut p=profile(&id,"thenees");
  p.ai.knowledge_nicho="fps".into();
  let out=context(d.path(),&p,None,"");
  assert!(out.contains("nunca diga olá mundo"),"regras fixas entram sempre");
  assert!(out.contains("csgo e valorant"),"nicho escolhido entra");
  assert!(!out.contains("susto"),"nicho não escolhido fica de fora");
  assert!(!out.contains("documento de lógica"),"documento de lógica nunca vira prompt");
  assert!(!out.contains("vlw pela sub"),"evento entra só quando o evento combina");
  assert!(context(d.path(),&p,None,"").len()<=depth_budget("standard"));
  p.ai.knowledge=false;assert!(context(d.path(),&p,None,"").is_empty());
  p.ai.knowledge=true;p.ai.knowledge_off=vec!["nichos/fps.md".into()];
  assert!(!context(d.path(),&p,None,"").contains("csgo e valorant"));
  remove(d.path(),&id,"nichos/fps.md").unwrap();
  assert_eq!(list(d.path(),&id).unwrap().len(),4);
  assert!(remove(d.path(),&id,"../escape.md").is_err());
 }
 #[test]
 fn event_hint_matches_the_matching_reaction_file(){
  let d=tempfile::tempdir().unwrap();let src=d.path().join("base");
  write(&src,"eventos-de-live/reacao-a-sub-doacao.md","# Sub\ndoacao celebrada");
  write(&src,"eventos-de-live/reacao-a-clip-highlight.md","# Clip\nclip comentado");
  let id=uuid::Uuid::new_v4().to_string();import(d.path(),&id,src.to_str().unwrap()).unwrap();
  let p=profile(&id,"canal");
  let sub=Event{id:"1".into(),profile_id:id.clone(),kind:"subscription".into(),user:"Ana".into(),user_id:"ana".into(),role:"subscriber".into(),message:String::new(),data:json!({}),simulated:false};
  assert!(context(d.path(),&p,Some(&sub),"").contains("doacao celebrada"));
  let clip=Event{kind:"clip".into(),..sub.clone()};
  assert!(context(d.path(),&p,Some(&clip),"").contains("clip comentado"));
  assert!(context(d.path(),&p,None,"").is_empty());
 }
 #[test]
 fn shallow_budget_still_delivers_the_first_rules(){
  let d=tempfile::tempdir().unwrap();let src=d.path().join("regras");
  write(&src,"tom-e-comportamento/grande.md",&format!("# Tom\n{}", "y".repeat(9000)));
  let id=uuid::Uuid::new_v4().to_string();import(d.path(),&id,src.to_str().unwrap()).unwrap();
  let mut p=profile(&id,"canal");p.ai.knowledge_depth="light".into();
  let out=context(d.path(),&p,None,"");
  assert!(out.len()<=5000);assert!(out.starts_with("tom-e-comportamento/grande.md"));
 }
}
