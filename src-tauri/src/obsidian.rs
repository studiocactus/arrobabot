use crate::{engine::Runtime,secrets,vault,ai};
use serde_json::Value;
pub async fn sync(rt:&Runtime,p:&str,path:&str)->Result<Value,String>{
 rt.db.profile(p)?;let config=rt.db.module(p,"obsidian");
 let endpoint=config["endpoint"].as_str().unwrap_or("http://127.0.0.1:27123");
 ai::validate_url(endpoint)?;let mut url=url::Url::parse(endpoint).map_err(|_|"Endereço do Obsidian inválido")?;
 if !matches!(url.host_str(),Some("127.0.0.1"|"localhost"|"[::1]")){return Err("A ponte Obsidian deve estar neste computador".into())}
 let note=vault::list(&rt.base,p)?.into_iter().find(|n|n.path==path).ok_or("Nota não encontrada")?;
 {let mut segments=url.path_segments_mut().map_err(|_|"Endereço inválido")?;segments.pop_if_empty();segments.push("vault").push("BotLive").push(p);for segment in path.split('/'){segments.push(segment);}}
 let res=rt.http.put(url).bearer_auth(secrets::get(p,"obsidian_key")?).header("Content-Type","text/markdown").body(note.content).send().await.map_err(|_|"Abra o Obsidian e confira o plugin Local REST API")?;
 if !res.status().is_success(){return Err(format!("Obsidian retornou HTTP {}",res.status().as_u16()))}
 Ok(serde_json::json!({"path":format!("BotLive/{p}/{path}")}))
}
