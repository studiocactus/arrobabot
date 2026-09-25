//! Bounded, single-pass templates. Values are data and are never parsed as templates.
use crate::{db::Db, model::{Event, Profile, Flow}};
use serde_json::{json, Value};
use rusqlite::{params, OptionalExtension};
use std::collections::BTreeMap;
type R<T> = Result<T, String>;
const LIMIT: usize = 65536;

pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 64 && name.bytes().enumerate().all(|(i,c)| c.is_ascii_alphabetic() || c == b'_' || (i > 0 && c.is_ascii_digit()))
}
pub fn target(target: &str) -> R<(&str, &str)> {
    let (scope, name) = target.split_once('.').ok_or("Use escopo.nome, por exemplo local.resposta ou global.contador")?;
    if !["local", "global", "user", "session", "sessionUser"].contains(&scope) || !valid_name(name) { return Err("Escopo ou nome de variável inválido".into()); }
    Ok((scope, name))
}
fn table(scope: &str) -> &'static str { if scope.starts_with("session") { "session_variables" } else { "variables" } }
fn owner<'a>(scope: &str, user: &'a str) -> R<&'a str> {
    if ["user", "sessionUser"].contains(&scope) { if user.is_empty() { Err("Este evento não possui ID de usuário".into()) } else { Ok(user) } } else { Ok("") }
}
pub fn list(db: &Db, profile: &str, user: &str) -> R<Value> {
    db.profile(profile)?;
    let c = db.0.lock().unwrap();
    let mut result = Vec::new();
    for table in ["variables", "session_variables"] {
        let mut stmt = c.prepare(&format!("SELECT scope,name,data,user_id FROM {table} WHERE profile_id=?1 AND (user_id='' OR user_id=?2) ORDER BY scope,name")).map_err(|e|e.to_string())?;
        let rows = stmt.query_map(params![profile,user], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).map_err(|e|e.to_string())?;
        for row in rows { let (scope,name,data,user_id) = row.map_err(|e|e.to_string())?; result.push(json!({"scope":scope,"name":name,"value":serde_json::from_str::<Value>(&data).map_err(|e|e.to_string())?,"userId":user_id})); }
    }
    Ok(json!(result))
}
pub fn mutate(db: &Db, profile: &str, platform: &str, user: &str, key: &str, operation: &str, value: Value) -> R<Value> {
    let (scope,name) = target(key)?;
    if scope == "local" { return Err("Variáveis locais existem somente dentro do fluxo".into()); }
    if !["set","increment","delete"].contains(&operation) { return Err("Operação de variável inválida".into()); }
    if value.to_string().len() > 16000 || user.len() > 200 { return Err("Valor ou ID muito longo".into()); }
    let user_id = owner(scope,user)?;
    let user_id = if user_id.is_empty() { String::new() } else { format!("{platform}:{user_id}") };
    let mut c = db.0.lock().unwrap();
    let tx = c.transaction().map_err(|e|e.to_string())?;
    let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM profiles WHERE id=?)", [profile], |r|r.get(0)).map_err(|e|e.to_string())?;
    if !exists { return Err("Perfil não encontrado".into()); }
    let table = table(scope);
    let old: Option<String> = tx.query_row(&format!("SELECT data FROM {table} WHERE profile_id=? AND scope=? AND user_id=? AND name=?"), params![profile,scope,user_id,name], |r|r.get(0)).optional().map_err(|e|e.to_string())?;
    let next = if operation == "increment" { add(old.as_deref().map(serde_json::from_str::<Value>).transpose().map_err(|e|e.to_string())?.unwrap_or(json!(0)), value)? } else { value };
    if operation == "delete" {
        tx.execute(&format!("DELETE FROM {table} WHERE profile_id=? AND scope=? AND user_id=? AND name=?"),params![profile,scope,user_id,name]).map_err(|e|e.to_string())?;
    } else {
        if old.is_none() { let count:i64=tx.query_row(&format!("SELECT COUNT(*) FROM {table} WHERE profile_id=?"),[profile],|r|r.get(0)).map_err(|e|e.to_string())?; if count >= 10000 { return Err("Limite de 10000 variáveis por perfil atingido".into()); } }
        tx.execute(&format!("INSERT INTO {table}(profile_id,scope,user_id,name,data) VALUES(?,?,?,?,?) ON CONFLICT(profile_id,scope,user_id,name) DO UPDATE SET data=excluded.data"),params![profile,scope,user_id,name,next.to_string()]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e|e.to_string())?;
    Ok(if operation == "delete" { Value::Null } else { next })
}
fn add(a: Value, b: Value) -> R<Value> {
    if let (Some(a),Some(b)) = (a.as_i64(),b.as_i64()) { return a.checked_add(b).map(|v|json!(v)).ok_or("Contador excedeu o limite numérico".into()); }
    let n=a.as_f64().ok_or("O contador precisa ser numérico")? + b.as_f64().ok_or("O incremento precisa ser numérico")?;
    if !n.is_finite() { return Err("Resultado numérico inválido".into()); } Ok(json!(n))
}
pub fn typed(text: &str) -> Value { serde_json::from_str(text).unwrap_or_else(|_|json!(text)) }
pub fn display(value: &Value) -> String { match value { Value::String(s)=>s.clone(), Value::Null=>String::new(), _=>value.to_string() } }

pub struct Context { values: BTreeMap<String,Value>, data: Value, pub simulated: bool }
impl Context {
    pub fn new(db: &Db, p: &Profile, e: &Event, flow: Option<&Flow>) -> R<Self> {
        let now=chrono::Local::now(); let mut values=BTreeMap::new();
         for (key,value) in [("user",json!(e.user)),("userName",json!(e.user)),("userId",json!(e.user_id)),("role",json!(e.role)),("message",json!(e.message)),("channel",json!(p.channel)),("channelId",json!(p.channel_id)),("platform",json!(p.platform)),("profileId",json!(p.id)),("profileName",json!(p.name)),("eventId",json!(e.id)),("eventType",json!(e.kind)),("isModerator",json!(matches!(e.role.as_str(),"moderator"|"broadcaster"))),("isBroadcaster",json!(e.role=="broadcaster")),("isSubscriber",json!(e.role=="subscriber")),("simulated",json!(e.simulated)),("date",json!(now.format("%Y-%m-%d").to_string())),("time",json!(now.format("%H:%M:%S").to_string())),("unixtime",json!(now.timestamp())),("lf",json!("\n"))] { values.insert(key.into(),value); }
        // Sem pessoa no evento, como em um timer, o sorteio é o único caminho para citar um espectador.
        if let Some(name)=crate::modules::random_chatter(db,&p.id){values.insert("randomViewer".into(),json!(name));}
        let mut tokens=e.message.split_whitespace();
        let command=tokens.next().unwrap_or("");
        let raw=e.message.trim_start().strip_prefix(command).unwrap_or("").trim_start();
        let args:Vec<_>=tokens.collect();
        values.insert("command".into(),json!(command)); values.insert("rawInput".into(),json!(raw)); values.insert("args".into(),json!(args)); values.insert("argCount".into(),json!(args.len()));
        for (i,arg) in args.iter().enumerate() { values.insert(format!("arg{i}"),json!(arg)); }
        if let Some(f)=flow { values.insert("commandCount".into(),json!(crate::command_counter::get(db,&p.id,&f.id)?)); values.insert("actionId".into(),json!(f.id)); values.insert("actionName".into(),json!(f.name)); }
        let uid=if e.user_id.is_empty(){String::new()}else{format!("{}:{}",p.platform,e.user_id)};
        for item in list(db,&p.id,&uid)?.as_array().unwrap() { values.insert(format!("{}.{}",item["scope"].as_str().unwrap(),item["name"].as_str().unwrap()),item["value"].clone()); }
        Ok(Self{values,data:e.data.clone(),simulated:e.simulated})
    }
    pub fn set_command_count(&mut self,n:i64){self.values.insert("commandCount".into(),json!(n));}
    fn lookup(&self, key: &str) -> Option<Value> {
        if let Some(path)=key.strip_prefix("data.") { let mut v=&self.data; for part in path.split('.') { v=if let Some(a)=v.as_array(){a.get(part.parse::<usize>().ok()?)?}else{v.get(part)?}; } Some(v.clone()) } else { self.values.get(key).cloned() }
    }
    pub fn inspect(&self) -> Value { let mut v=json!(self.values);v["data"]=self.data.clone();v }
    pub fn change(&mut self,db:&Db,p:&Profile,e:&Event,key:&str,op:&str,value:Value)->R<Value> {
        let (scope,_)=target(key)?; owner(scope,&e.user_id)?;
        let next=if scope=="local" || self.simulated {
            if op=="increment" {add(self.lookup(key).unwrap_or(json!(0)),value)?}else{value}
        }else{mutate(db,&p.id,&p.platform,&e.user_id,key,op,value)?};
        if next.to_string().len()>16000 {return Err("Valor de variável muito longo".into());}
        if op=="delete" {self.values.remove(key);}else{self.values.insert(key.into(),next.clone());}
        Ok(next)
    }
    fn expression(&self, expr: &str) -> R<String> {
        let mut parts=expr.split('|').map(str::trim); let key=parts.next().unwrap_or("");
        let mut value=self.lookup(key);
        for filter in parts {
            let (name,param)=filter.split_once(':').unwrap_or((filter,""));
            if name=="default" {if value.as_ref().map_or(true,|v|v.is_null()||v.as_str()==Some("")){value=Some(json!(param));}continue;}
            let v=value.as_ref().ok_or_else(||format!("Variável ausente: {key}. Defina-a antes ou use |default:texto"))?;
            let s=display(v);
            value=Some(match name {
                "upper"=>json!(s.to_uppercase()), "lower"=>json!(s.to_lowercase()), "trim"=>json!(s.trim()),
                "length"=>json!(v.as_array().map(|a|a.len()).or_else(||v.as_object().map(|m|m.len())).unwrap_or_else(||s.chars().count())),
                "json"=>json!(v.to_string()), "url"=>json!(url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()),
                "number"=>{let digits:usize=param.parse().map_err(|_|"Use number:0 até number:6")?; if digits>6{return Err("Use no máximo seis casas decimais".into())} let n:f64=s.parse().map_err(|_|"Valor não numérico")?;if !n.is_finite(){return Err("Valor não finito".into())}json!(format!("{n:.digits$}"))},
                "add"=>add(typed(&s),typed(param))?,
                "multiply"=>{let a:f64=s.parse().map_err(|_|"Valor não numérico")?;let b:f64=param.parse().map_err(|_|"Fator inválido")?;if !(a*b).is_finite(){return Err("Resultado numérico inválido".into())}json!(a*b)},
                _=>return Err(format!("Filtro desconhecido: {name}"))
            });
        }
        value.map(|v|display(&v)).ok_or_else(||format!("Variável ausente: {key}. Defina-a antes ou use |default:texto"))
    }
    pub fn render(&self, text: &str) -> R<String> {
        if text.len()>LIMIT {return Err("Modelo muito longo".into());}
        let mut out=String::new();let mut i=0;
        while i<text.len() {
            let rest=&text[i..];
            if let Some(escaped)=rest.strip_prefix('\\') {if escaped.starts_with("{{")||escaped.starts_with('$')||escaped.starts_with('%'){let n=if escaped.starts_with("{{"){2}else{1};out.push_str(&escaped[..n]);i+=n+1;continue;}}
            if rest.starts_with("{{") {
                let end=rest.find("}}").ok_or("Variável sem fechamento }}")?;
                out.push_str(&self.expression(rest[2..end].trim())?);i+=end+2;
            } else if rest.starts_with('%') && rest[1..].find('%').is_some_and(|end|valid_name(&rest[1..end+1])) {
                let end=rest[1..].find('%').unwrap()+1;out.push_str(&self.expression(&rest[1..end])?);i+=end+1;
            } else if rest.starts_with('$') {
                let end=rest[1..].find(|c:char|!c.is_ascii_alphanumeric()&&c!='_').map(|n|n+1).unwrap_or(rest.len());
                if let Some(value)=self.lookup(&rest[1..end]) {out.push_str(&display(&value));i+=end;}else{out.push('$');i+=1;}
            } else {let c=rest.chars().next().unwrap();out.push(c);i+=c.len_utf8();}
            if out.len()>LIMIT {return Err("Resultado ultrapassa 64 KiB".into());}
        }
        Ok(out)
    }
}
