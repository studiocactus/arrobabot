mod model;
mod variables;
mod db;
mod vault;
mod secrets;
mod ai;
mod conversation;
mod oauth;
mod engine;
mod platforms;
mod modules;
mod presets;
mod local_api;
mod update;
mod moderation;
mod voice;
mod access;
mod stats;
mod scheduler;
mod obsidian;
use engine::Runtime;
use model::*;
use serde_json::{json,Value};
use std::sync::Arc;
use tauri::{Manager,State};
type R<T>=Result<T,String>;
#[tauri::command]
async fn rpc(state:State<'_,Arc<Runtime>>,op:String,args:Value)->R<Value>{
 let rt=state.inner().clone();
 dispatch(rt,&op,args).await
}
pub async fn dispatch(rt:Arc<Runtime>,op:&str,args:Value)->R<Value>{
 access::guard(&rt,op,&args)?;
 if op.starts_with("access."){return access::operation(&rt,op,&args)}
 let p=args["profileId"].as_str().unwrap_or("").to_owned();
 match op{
 "snapshot"=>Ok(json!({"profiles":rt.db.profiles()?.into_iter().filter(|p|access::allowed(&rt,p)).collect::<Vec<_>>(),"logs":rt.db.logs("")?.into_iter().filter(|l|access::current(&rt)=="owner"||rt.db.profile(&l.profile_id).is_ok_and(|p|access::allowed(&rt,&p))).collect::<Vec<_>>(),"statuses":*rt.statuses.lock().unwrap(),"theme":rt.db.get("theme"),"accent":rt.db.get("accent"),"apiPort":*rt.api_port.lock().unwrap(),"dataDir":rt.base.to_string_lossy()})),
 "profile.save"=>{let mut profile:Profile=serde_json::from_value(args["profile"].clone()).map_err(|_|"Perfil inválido")?;if let Ok(previous)=rt.db.profile(&profile.id){if previous.channel!=profile.channel||previous.channel_id!=profile.channel_id||previous.platform!=profile.platform{rt.conversation.lock().unwrap().clear(&profile.id);}if previous.platform!=profile.platform||previous.client_id!=profile.client_id{rt.disconnect(&profile.id);for key in ["bot_token","bot_refresh","channel_token","channel_refresh","bot_expires","channel_expires"]{let _=secrets::set(&profile.id,key,"");}}}profile.blocklist=profile.blocklist.into_iter().map(|s|s.trim().to_owned()).filter(|s|!s.is_empty()).collect();profile.editors=profile.editors.into_iter().map(|s|s.trim().to_lowercase()).filter(|s|!s.is_empty()).collect();profile.topics=profile.topics.into_iter().map(|s|s.trim().to_owned()).filter(|s|!s.is_empty()).collect();vault::root(&rt.base,&profile.id)?;rt.db.save_profile(&profile)?;Ok(json!(profile))},
 "profile.delete"=>{
 let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p)?;rt.disconnect(&p);
 // Refuse paths that escape the app-data vault through junctions or symlinks.
 let root=vault::root(&rt.base,&p)?;
 let canonical=root.canonicalize().map_err(|e|e.to_string())?;
 let parent=rt.base.join("vaults").canonicalize().map_err(|e|e.to_string())?;
 if !canonical.starts_with(&parent)||canonical==parent{return Err("Caminho de vault inválido".into())}
 std::fs::remove_dir_all(&root).map_err(|e|e.to_string())?;
 rt.db.delete_profile(&p)?;rt.conversation.lock().unwrap().clear(&p);secrets::clear(&p);Ok(Value::Null)
 },
 "flows"=>Ok(json!(rt.db.flows(&p)?)),
 "variables.list"=>{let profile=rt.db.profile(&p)?;let user=args["userId"].as_str().unwrap_or("");let key=if user.is_empty(){String::new()}else{format!("{}:{user}",profile.platform)};variables::list(&rt.db,&p,&key)},
 "variables.change"=>{let profile=rt.db.profile(&p)?;variables::mutate(&rt.db,&p,&profile.platform,args["userId"].as_str().unwrap_or(""),args["target"].as_str().ok_or("Variável ausente")?,args["operation"].as_str().unwrap_or("set"),args["value"].clone())},
 "variables.preview"=>{
 let profile=rt.db.profile(&p)?;
 let mut e:Event=serde_json::from_value(args["event"].clone()).map_err(|_|"Evento inválido")?;
 e.profile_id=p.clone();e.simulated=true;
 let flow:Option<Flow>=args.get("flow").filter(|v|!v.is_null()).map(|v|serde_json::from_value(v.clone())).transpose().map_err(|_|"Fluxo inválido")?;
 let mut context=variables::Context::new(&rt.db,&profile,&e,flow.as_ref())?;
 let mut steps=vec![];
 if let Some(f)=flow {validate_flow(&f)?;for a in &f.actions {if !a.condition.is_empty()&&!e.message.to_lowercase().contains(&a.condition.to_lowercase()){continue}if a.kind=="script"{steps.push(json!({"kind":"script","text":"Script não executado na prévia"}));continue}let text=if a.kind=="variable.delete"{String::new()}else{context.render(&a.text)?};if a.kind.starts_with("variable."){context.change(&rt.db,&profile,&e,&a.target,a.kind.trim_start_matches("variable."),variables::typed(&text))?;}if matches!(a.kind.as_str(),"ai"|"ai.generate"){ai::save_response(&mut context,&rt,&profile,&e,a,"[Prévia: resposta contextual da IA]",false)?;}steps.push(json!({"kind":a.kind,"text":text}));}}
 let text=context.render(args["text"].as_str().unwrap_or(""))?;
 Ok(json!({"text":text,"steps":steps,"variables":context.inspect()}))
 },
 "flow.save"=>{let f:Flow=serde_json::from_value(args["flow"].clone()).map_err(|_|"Fluxo inválido")?;rt.db.save_flow(&f)?;Ok(json!(f))},
 "flow.delete"=>{rt.db.delete_flow(&p,args["id"].as_str().ok_or("Fluxo inválido")?)?;Ok(Value::Null)},
 "logs"=>Ok(json!(rt.db.logs(&p)?)),
 "stats"=>stats::read(&rt,&p),
 "connect"=>{rt.connect(&p).await?;Ok(Value::Null)},
 "disconnect"=>{rt.disconnect(&p);Ok(Value::Null)},
 "simulate"=>{let mut e:Event=serde_json::from_value(args["event"].clone()).map_err(|_|"Evento inválido")?;e.simulated=true;e.id=uuid::Uuid::new_v4().to_string();rt.submit(e).await?;Ok(Value::Null)},
 "chat.send"=>{let profile=rt.db.profile(&p)?;let e=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p,kind:"manual".into(),user:"".into(),user_id:"".into(),role:"broadcaster".into(),message:"".into(),data:Value::Null,simulated:false};rt.send(&profile,&e,args["text"].as_str().ok_or("Escreva uma mensagem")?).await?;Ok(Value::Null)},
 "secret.save"=>{rt.db.profile(&p)?;let key=args["key"].as_str().ok_or("Chave inválida")?;if !["ai_key","client_secret","discord_webhook","obsidian_key"].contains(&key){return Err("Tipo de credencial não permitido".into())}secrets::set(&p,key,args["value"].as_str().ok_or("Credencial inválida")?)?;
 if key=="ai_key"{let profile=rt.db.profile(&p)?;let origin=url::Url::parse(&profile.ai.endpoint).map_err(|_|"Endpoint inválido")?.origin().ascii_serialization();secrets::set(&p,"ai_origin",&origin)?;}
 Ok(Value::Null)},
 "secret.status"=>Ok(json!({"ai":secrets::get(&p,"ai_key").is_ok(),"bot":secrets::get(&p,"bot_token").is_ok(),"channel":secrets::get(&p,"channel_token").is_ok()})),
 "oauth.start"=>{let profile=rt.db.profile(&p)?;let account=args["account"].as_str().unwrap_or("bot");if !["bot","channel"].contains(&account){return Err("Conta inválida".into())}if profile.platform=="twitch"{oauth::device_start(&rt.http,&profile,account).await}else{oauth::browser_auth(&rt.http,&profile,account).await}},
 "oauth.finish"=>{
 let mut profile=rt.db.profile(&p)?;let account=args["account"].as_str().unwrap_or("bot");
 let identity=oauth::device_finish(&rt.http,&profile,account,args["code"].as_str().ok_or("Código ausente")?).await?;
 let id=identity["user_id"].as_str().ok_or("Conta sem identificador")?.to_owned();
 if account=="bot"{profile.bot_id=id;}else{profile.channel_id=id;profile.channel=identity["login"].as_str().unwrap_or(&profile.channel).to_owned();}
 rt.db.save_profile(&profile)?;Ok(json!(profile))
 },
 "url.open"=>{let url=args["url"].as_str().ok_or("URL inválida")?;ai::validate_url(url)?;open::that(url).map_err(|_|"Não foi possível abrir o navegador")?;Ok(Value::Null)},
 "ai.preview"=>{
 let profile=rt.db.profile(&p)?;
 let message=args["message"].as_str().unwrap_or("");let instruction=args["instruction"].as_str().unwrap_or("");let recent=args["recent"].as_str().unwrap_or("");
 if message.trim().is_empty()||message.len()>8000||instruction.len()>32768||recent.len()>24000{return Err("Preencha uma mensagem de teste dentro dos limites".into())}
 let e=Event{id:"ai-preview".into(),profile_id:p,kind:"chat".into(),user:"Espectador de teste".into(),user_id:"test-user".into(),role:"everyone".into(),message:message.into(),data:Value::Null,simulated:true};
 let c=variables::Context::new(&rt.db,&profile,&e,None)?;
 let history:Vec<Value>=recent.lines().filter(|s|!s.trim().is_empty()).rev().take(12).collect::<Vec<_>>().into_iter().rev().map(|s|json!({"message":s.chars().take(500).collect::<String>()})).collect();
 Ok(json!(ai::conversation(&rt.http,&rt.base,&profile,&e,&c.render(instruction)?,&history).await?))
 },
 "ai.test"=>{let profile=rt.db.profile(&p)?;Ok(json!(ai::generate(&rt.http,&rt.base,&profile,args["user"].as_str().unwrap_or("streamer"),args["prompt"].as_str().unwrap_or("Olá! Apresente-se brevemente.")).await?))},
 "ollama"=>{let profile=rt.db.profile(&p)?;ai::validate_url(&profile.ai.endpoint)?;let res=rt.http.get(format!("{}/api/tags",profile.ai.endpoint.trim_end_matches('/'))).send().await.map_err(|_|"Ollama não está disponível. Inicie o Ollama e tente novamente.")?;res.json().await.map_err(|_|"Resposta Ollama inválida".into())},
 "notes"=>{rt.db.profile(&p)?;Ok(json!(vault::list(&rt.base,&p)?))},
 "note.save"=>{let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p)?;vault::write(&rt.base,&p,args["path"].as_str().ok_or("Caminho ausente")?,args["content"].as_str().ok_or("Conteúdo ausente")?,false)?;Ok(Value::Null)},
 "note.delete"=>{let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p)?;vault::delete(&rt.base,&p,args["path"].as_str().ok_or("Caminho ausente")?)?;Ok(Value::Null)},
 "vault.open"=>{rt.db.profile(&p)?;open::that(vault::root(&rt.base,&p)?).map_err(|_|"Não foi possível abrir a pasta")?;Ok(Value::Null)},
 "community"=>{rt.db.profile(&p)?;Ok(json!(modules::load(&rt,&p)))},
 "community.action"=>modules::admin(&rt,&p,args["action"].as_str().ok_or("Ação ausente")?,args["data"].clone()),
 "preset.create"=>presets::export(&rt,&p,args["kind"].as_str().unwrap_or("profile"),args["name"].as_str().unwrap_or("Meu preset"),serde_json::from_value(args["ids"].clone()).unwrap_or_default()),
 "preset.preview"=>presets::preview(&rt,&p,&args["preset"]),
 "preset.apply"=>{presets::apply(&rt,&p,&args["preset"],args["policy"].as_str().unwrap_or("cancel"))?;Ok(Value::Null)},
 "preset.save"=>{let v=&args["preset"];presets::validate(v)?;let id=v["id"].as_str().ok_or("Preset sem identificador")?;rt.db.0.lock().unwrap().execute("INSERT INTO presets VALUES(?,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data",rusqlite::params![id,v.to_string()]).map_err(|e|e.to_string())?;Ok(Value::Null)},
 "presets"=>{let c=rt.db.0.lock().unwrap();let mut s=c.prepare("SELECT data FROM presets ORDER BY rowid DESC").map_err(|e|e.to_string())?;let rows=s.query_map([],|r|r.get::<_,String>(0)).map_err(|e|e.to_string())?;let values:Vec<Value>=rows.filter_map(|r|r.ok().and_then(|s|serde_json::from_str(&s).ok())).collect();Ok(json!(values))},
 "file.export"=>{let path=args["path"].as_str().ok_or("Escolha um arquivo")?;if !path.ends_with(".botlivepreset"){return Err("Use a extensão .botlivepreset".into())}presets::validate(&args["preset"])?;std::fs::write(path,serde_json::to_string_pretty(&args["preset"]).unwrap()).map_err(|e|e.to_string())?;Ok(Value::Null)},
 "file.import"=>{let path=args["path"].as_str().ok_or("Escolha um arquivo")?;let meta=std::fs::metadata(path).map_err(|_|"Arquivo não encontrado")?;if meta.len()>2_000_000{return Err("Preset muito grande".into())}let text=std::fs::read_to_string(path).map_err(|_|"Não foi possível ler o preset")?;let v:Value=serde_json::from_str(&text).map_err(|_|"JSON inválido")?;presets::validate(&v)?;Ok(v)},
 "obsidian.sync"=>obsidian::sync(&rt,&p,args["path"].as_str().ok_or("Selecione uma nota")?).await,
 "voice.transcribe"=>voice::transcribe(&rt,&p,args["audio"].as_str().ok_or("Áudio ausente")?).await,
 "update.check"=>update::check(&rt,false).await,
 "update.install"=>update::check(&rt,true).await,
 "module.config"=>{rt.db.profile(&p)?;let key=args["key"].as_str().ok_or("Configuração ausente")?;if !["moderation","tts","voice","obsidian","points","songs","discord","games"].contains(&key){return Err("Configuração inválida".into())}if key=="moderation"&&!moderation::valid_config(&args["value"]){return Err("Ação inválida".into())}rt.db.set_module(&p,key,&args["value"])?;Ok(Value::Null)},
 "module.config.get"=>{rt.db.profile(&p)?;Ok(rt.db.module(&p,args["key"].as_str().ok_or("Configuração ausente")?))},
 "settings"=>{let key=args["key"].as_str().ok_or("Configuração inválida")?;if !["theme","accent","apiPort","updateEndpoint","updatePublicKey","autoUpdate"].contains(&key){return Err("Configuração não permitida".into())}if key=="accent"&&!args["value"].as_str().is_some_and(|v|v.is_empty()||(v.len()==7&&v.starts_with('#')&&v[1..].chars().all(|c|c.is_ascii_hexdigit()))){return Err("Cor inválida".into())}rt.db.set(key,&args["value"])?;Ok(Value::Null)},
 "settings.get"=>Ok(json!({"theme":rt.db.get("theme"),"apiPort":rt.db.get("apiPort"),"apiToken":rt.api_token,"updateEndpoint":update::endpoint(&rt),"updatePublicKey":update::public_key(&rt),"autoUpdate":rt.db.get("autoUpdate")})),
 _=>Err("Operação desconhecida".into())
 }
}
pub fn run(){
 tauri::Builder::default().plugin(tauri_plugin_dialog::init()).plugin(tauri_plugin_updater::Builder::new().build())
 .setup(|app|{
 let base=app.path().app_data_dir()?;
 let rt=tauri::async_runtime::block_on(async{Runtime::new(base)})?;
 *rt.app.lock().unwrap()=Some(app.handle().clone());app.manage(rt.clone());
 let scheduler_rt=rt.clone();tauri::async_runtime::spawn(scheduler::run(scheduler_rt));
 let update_rt=rt.clone();tauri::async_runtime::spawn(async move{if update_rt.db.get("autoUpdate")==true {if let Ok(v)=update::check(&update_rt,false).await{if v["available"]==true{update_rt.log("","update","Há uma atualização disponível nas configurações.","info");}}}});
 tauri::async_runtime::spawn(async move{if let Err(e)=local_api::serve(rt.clone()).await{rt.log("","api",&e,"error");}});
 Ok(())
 }).invoke_handler(tauri::generate_handler![rpc]).run(tauri::generate_context!()).expect("Não foi possível iniciar o BotLive");
}

#[cfg(test)] mod integration_tests;
