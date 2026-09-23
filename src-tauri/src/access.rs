use crate::{engine::Runtime,secrets,model::Profile};
use serde_json::{json,Value};
fn valid(name:&str)->bool{!name.is_empty()&&name.len()<=40&&name.chars().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||c=='_'||c=='-')}
pub fn current(rt:&Runtime)->String{rt.actor.lock().unwrap().clone()}
pub fn allowed(rt:&Runtime,p:&Profile)->bool{let user=current(rt);user=="owner"||p.editors.contains(&user)}
pub fn guard(rt:&Runtime,op:&str,args:&Value)->Result<(),String>{
 if op.starts_with("access."){return Ok(())}
 let actor=current(rt);if actor=="locked"{return Err("SESSION_LOCKED".into())}
 if actor=="owner"{return Ok(())}
 if ["snapshot","presets"].contains(&op){return Ok(())}
 if ["settings","settings.get","update.check","update.install","profile.delete","secret.save","oauth.start","oauth.finish"].contains(&op){return Err("Somente o proprietário pode realizar esta operação".into())}
 let id=args["profileId"].as_str().or(args["profile"]["id"].as_str()).or(args["flow"]["profileId"].as_str()).ok_or("Selecione um perfil autorizado")?;
 let p=rt.db.profile(id)?;if !p.editors.contains(&actor){return Err("Você não tem permissão para editar este perfil".into())}
 if op=="profile.save"{
 let proposed:Profile=serde_json::from_value(args["profile"].clone()).map_err(|_|"Perfil inválido")?;
 if proposed.editors!=p.editors||proposed.client_id!=p.client_id||proposed.channel_id!=p.channel_id||proposed.bot_id!=p.bot_id||proposed.platform!=p.platform{return Err("Somente o proprietário altera contas e permissões".into())}
 }
 Ok(())
}
pub fn operation(rt:&Runtime,op:&str,args:&Value)->Result<Value,String>{
 match op{
 "access.status"=>Ok(json!({"user":current(rt),"enabled":rt.db.get("accessEnabled")==true})),
 "access.login"=>{
 let name=args["name"].as_str().unwrap_or("owner");if !valid(name){return Err("Nome de usuário inválido".into())}
 {let mut times=rt.cooldowns.lock().unwrap();let key="login_attempt".to_string();if times.get(&key).is_some_and(|t|t.elapsed().as_secs()<2){return Err("Aguarde dois segundos e tente novamente".into())}times.insert(key,std::time::Instant::now());}
 let pin=secrets::get("local-access",&format!("{name}_pin")).map_err(|_|"Usuário ou senha incorretos")?;
 let entered=args["pin"].as_str().unwrap_or("");
 let mut different=pin.len()^entered.len();for(a,b)in pin.bytes().zip(entered.bytes()){different|=(a^b) as usize;}
 if different!=0{return Err("Usuário ou senha incorretos".into())}
 *rt.actor.lock().unwrap()=name.into();Ok(json!({"user":name}))
 },
 "access.lock"=>{if rt.db.get("accessEnabled")!=true{return Err("Configure primeiro a senha do proprietário".into())}*rt.actor.lock().unwrap()="locked".into();Ok(Value::Null)},
 "access.user"=>{if current(rt)!="owner"{return Err("Somente o proprietário gerencia acessos".into())}
 let name=args["name"].as_str().unwrap_or("");let pin=args["pin"].as_str().unwrap_or("");
 if !valid(name)||name=="locked"||pin.len()<6{return Err("Use nome em minúsculas, sem espaços, e senha de ao menos seis caracteres".into())}
 if name!="owner"&&rt.db.get("accessEnabled")!=true{return Err("Crie primeiro a senha de owner (proprietário)".into())}
 secrets::set("local-access",&format!("{name}_pin"),pin)?;
 if name=="owner"{rt.db.set("accessEnabled",&json!(true))?;}
 Ok(json!({"created":name}))
 },
 _=>Err("Operação de acesso desconhecida".into())
 }
}
