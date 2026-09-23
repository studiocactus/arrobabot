use crate::{engine::Runtime,model::*};
use serde_json::{json,Value};
use rusqlite::params;
pub fn export(rt:&Runtime,p:&str,kind:&str,name:&str,ids:Vec<String>)->Result<Value,String>{
 let mut profile=rt.db.profile(p)?;profile.channel.clear();profile.channel_id.clear();profile.bot_id.clear();profile.client_id.clear();profile.editors.clear();
 let flows:Vec<_>=rt.db.flows(p)?.into_iter().filter(|f|(ids.is_empty()||ids.contains(&f.id))&&(kind!="command"||f.trigger.kind=="command")).collect();
 let data=match kind{
 "profile"=>json!({"profile":profile,"flows":flows,"theme":rt.db.get("theme").as_str().unwrap_or("dark"),"accent":rt.db.get("accent").as_str().unwrap_or(""),"moduleConfigs":{"points":rt.db.module(p,"points"),"songs":rt.db.module(p,"songs"),"tts":rt.db.module(p,"tts"),"moderation":rt.db.module(p,"moderation"),"discord":rt.db.module(p,"discord"),"games":rt.db.module(p,"games")}}),
 "flow"|"command"=>json!({"flows":flows}),
 "personality"=>json!({"ai":profile.ai,"blocklist":profile.blocklist,"topics":profile.topics}),
 "theme"=>json!({"theme":rt.db.get("theme").as_str().unwrap_or("dark")}),
 _=>return Err("Tipo de preset desconhecido".into())
 };
 let preset=json!({"format":"botlivepreset","version":1,"id":uuid::Uuid::new_v4().to_string(),"name":name,"kind":kind,"tags":[],"data":data});
 validate(&preset)?;Ok(preset)
}
fn inspect(v:&Value)->Result<(),String>{
 match v{
 Value::Object(m)=>{for(k,v)in m{let key=k.to_lowercase();if ["token","secret","password","apikey","api_key","authorization","credential"].iter().any(|s|key.contains(s)){return Err("O preset contém campos de credenciais".into())}if k=="target"&&v.as_str().is_some_and(|x|x.starts_with("http")){return Err("Remova endereços de webhook antes de exportar; eles podem conter credenciais".into())}inspect(v)?;}},
 Value::Array(a)=>for x in a{inspect(x)?},
 Value::String(s)=>{if s.contains("sk-")||s.contains("discord.com/api/webhooks/"){return Err("O preset parece conter uma credencial em texto. Remova-a antes de compartilhar.".into())}},
 _=>{}
 }Ok(())
}
pub fn validate(v:&Value)->Result<(),String>{
 if v["format"]!="botlivepreset"||v["version"]!=1{return Err("Arquivo não é um preset BotLive de versão compatível".into())}
 if v.to_string().len()>2_000_000{return Err("Preset excede 2 MB".into())}
 if !["profile","flow","command","personality","theme"].contains(&v["kind"].as_str().unwrap_or("")){return Err("Tipo de preset inválido".into())}
 inspect(&v["data"])?;
 if let Some(flows)=v["data"]["flows"].as_array(){for f in flows{validate_flow(&serde_json::from_value(f.clone()).map_err(|_|"Fluxo inválido no preset")?)?;}}
 Ok(())
}
pub fn preview(rt:&Runtime,p:&str,v:&Value)->Result<Value,String>{
 validate(v)?;let existing=rt.db.flows(p)?;
 let imported:Vec<Flow>=serde_json::from_value(v["data"]["flows"].clone()).unwrap_or_default();
 let conflicts:Vec<_>=imported.iter().filter(|f|existing.iter().any(|e|e.name==f.name||(f.trigger.kind=="command"&&e.trigger.pattern==f.trigger.pattern))).map(|f|f.name.clone()).collect();
 Ok(json!({"name":v["name"],"kind":v["kind"],"flows":imported.len(),"conflicts":conflicts}))
}
pub fn apply(rt:&Runtime,p:&str,v:&Value,policy:&str)->Result<(),String>{
 validate(v)?;if !["cancel","skip","replace"].contains(&policy){return Err("Política de conflito inválida".into())}
 let summary=preview(rt,p,v)?;if policy=="cancel"&&summary["conflicts"].as_array().is_some_and(|a|!a.is_empty()){return Err("Há conflitos. Escolha manter os atuais ou substituir explicitamente.".into())}
 let mut profile=rt.db.profile(p)?;
 let existing=rt.db.flows(p)?;
 let imported:Vec<Flow>=serde_json::from_value(v["data"]["flows"].clone()).unwrap_or_default();
 // Validate everything before the single database transaction.
 let mut changes=vec![];
 for mut f in imported{
 let conflict=existing.iter().find(|e|e.name==f.name||(f.trigger.kind=="command"&&e.trigger.pattern==f.trigger.pattern));
 if conflict.is_some()&&policy=="skip"{continue}
 f.id=conflict.map(|e|e.id.clone()).unwrap_or_else(||uuid::Uuid::new_v4().to_string());f.profile_id=p.into();
 // Imported automation always requires deliberate enabling after inspection.
 f.enabled=false;validate_flow(&f)?;changes.push(f);
 }
 if v["kind"]=="profile"{
 let source:Profile=serde_json::from_value(v["data"]["profile"].clone()).map_err(|_|"Perfil inválido no preset")?;
 profile.ai=source.ai;profile.blocklist=source.blocklist;profile.topics=source.topics;profile.modules=source.modules;
 }
 if v["kind"]=="personality"{
 profile.ai=serde_json::from_value(v["data"]["ai"].clone()).map_err(|_|"Personalidade inválida")?;
 profile.blocklist=serde_json::from_value(v["data"]["blocklist"].clone()).map_err(|_|"Restrições inválidas")?;
 profile.topics=serde_json::from_value(v["data"]["topics"].clone()).map_err(|_|"Assuntos inválidos")?;
 }
 let mut c=rt.db.0.lock().unwrap();let tx=c.transaction().map_err(|e|e.to_string())?;
 tx.execute("UPDATE profiles SET data=? WHERE id=?",params![serde_json::to_string(&profile).unwrap(),p]).map_err(|e|e.to_string())?;
 for f in changes{tx.execute("INSERT INTO flows VALUES(?,?,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![f.id,p,serde_json::to_string(&f).unwrap()]).map_err(|e|e.to_string())?;}
 if v["kind"]=="profile"{if let Some(configs)=v["data"]["moduleConfigs"].as_object(){for(key,value)in configs{if !["points","songs","tts","moderation","discord","games"].contains(&key.as_str()){return Err("Configuração de módulo não suportada no preset".into())}if key=="moderation"&&!value.is_null()&&!crate::moderation::valid_config(value){return Err("Configuração de moderação inválida".into())}tx.execute("INSERT INTO module_state VALUES(?,?,?) ON CONFLICT(profile_id,key) DO UPDATE SET data=excluded.data",params![p,key,value.to_string()]).map_err(|e|e.to_string())?;}}}
 if v["kind"]=="theme"||(v["kind"]=="profile"&&v["data"]["theme"].is_string()){let theme=v["data"]["theme"].as_str().filter(|t|["dark","light"].contains(t)).ok_or("Tema inválido")?;tx.execute("INSERT INTO kv VALUES('theme',?) ON CONFLICT(key) DO UPDATE SET data=excluded.data",[json!(theme).to_string()]).map_err(|e|e.to_string())?;}
 if v["kind"]=="theme"||v["kind"]=="profile"{if let Some(accent)=v["data"]["accent"].as_str(){if !accent.is_empty()&&!(accent.len()==7&&accent.starts_with('#')&&accent[1..].chars().all(|c|c.is_ascii_hexdigit())){return Err("Cor inválida no preset".into())}tx.execute("INSERT INTO kv VALUES('accent',?) ON CONFLICT(key) DO UPDATE SET data=excluded.data",[json!(accent).to_string()]).map_err(|e|e.to_string())?;}}
 tx.commit().map_err(|e|e.to_string())?;Ok(())
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn rejects_secrets_and_versions(){
 let mut p=json!({"format":"botlivepreset","version":1,"kind":"personality","data":{"apiKey":"hidden"}});
 assert!(validate(&p).is_err());p["data"]=json!({});assert!(validate(&p).is_ok());p["version"]=json!(2);assert!(validate(&p).is_err());
 }
}
