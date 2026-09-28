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

/// Sorteio dentro de uma faixa: `{{random:1,50}}` sai inteiro e `{{random:1,50.00}}`
/// sai com centavos em vírgula (6,65), como se escreve no Brasil. As casas decimais
/// são as do limite da faixa que tem mais casas, e a vírgula é o separador da lista,
/// por isso os decimais se escrevem com ponto na entrada.
const FAIXA: &str = "Use random:min,max, por exemplo random:1,50 ou random:1,50.00";
fn decimals(text: &str) -> usize { text.split_once('.').map(|(_, d)| d.len()).unwrap_or(0) }
fn draw(key: &str) -> R<String> {
    use rand::Rng;
    let spec = key.strip_prefix("random:").ok_or_else(|| FAIXA.to_owned())?;
    let mut parts = spec.split(',').map(str::trim);
    let (Some(min), Some(max), None) = (parts.next(), parts.next(), parts.next()) else { return Err(FAIXA.into()); };
    let a: f64 = min.parse().map_err(|_| FAIXA.to_owned())?;
    let b: f64 = max.parse().map_err(|_| FAIXA.to_owned())?;
    if !a.is_finite() || !b.is_finite() { return Err(FAIXA.into()); }
    if a > b { return Err("A faixa começa no maior valor: use random:min,max".into()); }
    if a.abs() > 1e9 || b.abs() > 1e9 { return Err("Faixa de random muito grande: use valores até 1000000000".into()); }
    let digits = decimals(min).max(decimals(max));
    if digits > 4 { return Err("Use no máximo quatro casas decimais na faixa".into()); }
    if digits == 0 { return Ok(rand::thread_rng().gen_range(a as i64..=b as i64).to_string()); }
    let n = if a == b { a } else { rand::thread_rng().gen_range(a..b) };
    Ok(format!("{:.1$}", n, digits).replace('.', ","))
}

/// Nomes simples que existiram com cifrão antes da unificação em `{{...}}`.
const LEGACY:[&str;28]=["user","userName","userId","role","message","channel","channelId","platform","profileId","profileName","eventId","eventType","isModerator","isBroadcaster","isSubscriber","simulated","date","time","unixtime","lf","randomViewer","command","rawInput","args","argCount","commandCount","actionId","actionName"];
fn legacy(name:&str) -> bool { LEGACY.contains(&name) || name.strip_prefix("arg").is_some_and(|n|!n.is_empty()&&n.bytes().all(|b|b.is_ascii_digit())) }
/// Reescreve o marcador legado `$nome` como `{{nome}}`. Só nomes conhecidos viram modelo:
/// `$5`, `R$100`, `$desconhecido` e o escape `\$user` ficam exatamente como estavam.
pub fn migrate(text: &str) -> String {
    if !text.contains('$') { return text.to_owned(); }
    let b=text.as_bytes();let mut out=String::with_capacity(text.len());let mut i=0;
    while i<b.len() {
        if b[i]!=b'$' { let c=text[i..].chars().next().unwrap();out.push(c);i+=c.len_utf8();continue; }
        if i>0&&b[i-1]==b'\\' { out.push('$');i+=1;continue; }
        let mut end=i+1;while end<b.len()&&(b[end].is_ascii_alphanumeric()||b[end]==b'_') {end+=1;}
        let name=&text[i+1..end];
        if legacy(name) { out.push_str("{{");out.push_str(name);out.push_str("}}"); } else { out.push('$');out.push_str(name); }
        i=end;
    }
    out
}
/// Converte modelos legados no conteúdo já gravado. Não toca em gatilho, condição
/// nem script: esses textos são comparados ou executados, nunca interpretados como modelo.
pub fn migrate_flow(f: &mut Flow) -> bool {
    let mut changed=false;
    for a in &mut f.actions {
        if a.kind=="script" { continue; }
        let text=migrate(&a.text);let target=migrate(&a.target);
        changed|=text!=a.text||target!=a.target;a.text=text;a.target=target;
    }
    changed
}
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
     // Alertas da live: contadores do canal, detalhes de subs e dados da raid.
     // followerCount e subCount chegam depois, com os totais da Twitch; aqui
     // ficam vazios para o modelo nunca quebrar por evento sem contagem.
     values.insert("followerCount".into(),json!("")); values.insert("subCount".into(),json!(""));
     let tier_short=match e.data["tier"].as_str().unwrap_or(""){"1000"=>"1","2000"=>"2","3000"=>"3",other=>other};
     values.insert("subTier".into(),json!(tier_short));
     values.insert("subMonths".into(),json!(e.data["cumulative_months"].as_u64().unwrap_or(0)));
     values.insert("subStreak".into(),json!(e.data["streak_months"].as_u64().unwrap_or(0)));
     values.insert("subMessage".into(),json!(e.data["message"]["text"].as_str().unwrap_or("")));
     values.insert("isGift".into(),json!(e.data["is_gift"]==true));
     values.insert("gifterName".into(),json!(e.data["user_name"].as_str().filter(|s|!s.is_empty()).unwrap_or("Anônimo")));
     values.insert("giftTotal".into(),json!(e.data["total"].as_u64().unwrap_or(0)));
     values.insert("giftTier".into(),json!(tier_short));
     values.insert("raidViewers".into(),json!(e.data["viewers"].as_u64().unwrap_or(0)));
     values.insert("raiderLogin".into(),json!(e.data["from_broadcaster_user_login"].as_str().unwrap_or("")));
        if let Some(f)=flow { values.insert("commandCount".into(),json!(crate::command_counter::get(db,&p.id,&f.id)?)); values.insert("actionId".into(),json!(f.id)); values.insert("actionName".into(),json!(f.name)); }
        // O que o microfone acabou de ouvir. Sem fala ainda as duas existem vazias:
        // uma automação que usa {{lastSpeech}} não pode quebrar por o streamer estar calado.
        let spoken=crate::speech::recent(db,&p.id);
        values.insert("lastSpeech".into(),json!(spoken.last().cloned().unwrap_or_default()));
        values.insert("liveSpeech".into(),json!(spoken.join(" | ")));
        let uid=if e.user_id.is_empty(){String::new()}else{format!("{}:{}",p.platform,e.user_id)};
        for item in list(db,&p.id,&uid)?.as_array().unwrap() { values.insert(format!("{}.{}",item["scope"].as_str().unwrap(),item["name"].as_str().unwrap()),item["value"].clone()); }
        Ok(Self{values,data:e.data.clone(),simulated:e.simulated})
    }
    pub fn set_command_count(&mut self,n:i64){self.values.insert("commandCount".into(),json!(n));}
     pub fn set(&mut self,key:&str,value:Value){self.values.insert(key.into(),value);}
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
        // `random:min,max` é sorteado a cada uso, por isso não vem do cadastro de valores.
        let mut value=if key.starts_with("random:"){Some(json!(draw(key)?))}else{self.lookup(key)};
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
            } else {let c=rest.chars().next().unwrap();out.push(c);i+=c.len_utf8();}
            if out.len()>LIMIT {return Err("Resultado ultrapassa 64 KiB".into());}
        }
        Ok(out)
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn random_stays_inside_the_range_and_writes_decimals_with_a_comma(){
        assert_eq!(draw("random:7,7").unwrap(),"7","faixa de um valor só entrega o próprio valor");
        assert_eq!(draw("random:7.00,7.00").unwrap(),"7,00","as casas da faixa definem o formato");
        assert_eq!(draw("random:7.0,7.0").unwrap(),"7,0","uma casa na faixa, uma casa na saída");
        assert_eq!(draw("random:-5,-5").unwrap(),"-5");
        for _ in 0..300 {
            let n:u32=draw("random:1,50").unwrap().parse().expect("inteiro sem casa decimal");
            assert!((1..=50).contains(&n));
            let money=draw("random:1,50.00").unwrap();
            let (whole,cents)=money.split_once(',').expect("vírgula decimal na saída");
            assert_eq!(cents.len(),2,"duas casas: {money}");
            let value=whole.parse::<i32>().unwrap()*100+cents.parse::<i32>().unwrap();
            assert!((100..=5000).contains(&value),"fora da faixa: {money}");
        }
    }
    #[test] fn random_explains_a_bad_range_instead_of_sending_a_hole(){
        for bad in ["random","random:","random:50,1","random:a,b","random:1,50,2","random:1,,50","random:1,1e12","random:1,50.00000"] {
            assert!(draw(bad).is_err(),"aceitaria {bad}");
        }
        assert!(draw("random:1,50").is_ok());
    }
    #[test] fn alertas_da_live_expoem_contadores_e_detalhes(){
        use crate::model::{Event, Profile};
        let dir=tempfile::tempdir().unwrap();let db=Db::open(&dir.path().join("t.sqlite")).unwrap();
        let p:Profile=serde_json::from_value(serde_json::json!({"id":"00000000-0000-4000-8000-000000000003","name":"P","platform":"twitch","channel":"canal"})).unwrap();
        db.save_profile(&p).unwrap();
        let e:Event=serde_json::from_value(serde_json::json!({"id":"e1","profileId":p.id,"kind":"resub","user":"Bia","user_id":"99","role":"subscriber","message":"amo aqui","data":{"tier":"2000","cumulative_months":5,"streak_months":3,"message":{"text":"amo aqui"}}})).unwrap();
        let mut c=Context::new(&db,&p,&e,None).unwrap();
        assert_eq!(c.render("{{subTier}}").unwrap(),"2","mil vira nível 1, dois mil vira 2");
        assert_eq!(c.render("{{subMonths}}").unwrap(),"5");
        assert_eq!(c.render("{{subStreak}}").unwrap(),"3");
        assert_eq!(c.render("{{subMessage}}").unwrap(),"amo aqui");
        assert_eq!(c.render("{{followerCount}}").unwrap(),"","sem contagem o modelo sai vazio, sem erro");
        c.set("followerCount",serde_json::json!(1022));c.set("subCount",serde_json::json!(42));
        assert_eq!(c.render("Obrigado por seguir a gente {{user}}! Agora estamos em {{followerCount}} seguidores e {{subCount}} subs!").unwrap(),"Obrigado por seguir a gente Bia! Agora estamos em 1022 seguidores e 42 subs!");
        let r:Event=serde_json::from_value(serde_json::json!({"id":"e2","profileId":p.id,"kind":"raid","user":"Mia","user_id":"98","role":"everyone","message":"","data":{"viewers":17,"from_broadcaster_user_login":"miazinha"}})).unwrap();
        let rc=Context::new(&db,&p,&r,None).unwrap();
        assert_eq!(rc.render("{{raidViewers}}").unwrap(),"17");
        assert_eq!(rc.render("{{raiderLogin}}").unwrap(),"miazinha");
    }
}
