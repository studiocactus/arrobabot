mod command_counter;
mod backup;
mod timers;
mod presence;
mod model;
mod variables;
mod db;
mod vault;
mod knowledge;
mod live_state;
mod dashboard;
mod secrets;
mod ai;
mod conversation;
mod chat_extras;
mod links;
mod oauth;
mod engine;
mod platforms;
mod emotes;
mod emote_burst;
mod helix;
mod modules;
mod presets;
mod local_api;
mod update;
mod moderation;
mod cmd_manager;
mod twitch_ops;
mod voice;
mod listen;
mod natural;
mod speech;
mod access;
mod stats;
mod labels;
mod obs;
mod voice_fx;
mod voicemod;
mod voice_redemptions;
mod intents;
mod runs;
mod action_library;
mod scheduler;
mod obsidian;
mod discord;
mod discord_admin;
mod discord_engage;
mod livepix;
mod alerts;
mod chat_overlay;
mod audio;
mod tts;
mod preflight;
mod spotify;
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
 if op.starts_with("chatExtras."){return chat_extras::operation(&rt,&p,op,&args)}
 if op.starts_with("labels."){return labels::operation(&rt,&p,op,&args).await}
 if op.starts_with("obs."){return obs::operation(&rt,&p,op,&args).await}
 if op.starts_with("discord."){return discord_admin::operation(&rt,&p,op,&args).await}
 if op.starts_with("livepix."){return livepix::operation(&rt,&p,op,&args).await}
 if op.starts_with("voicemod."){return voicemod::operation(&rt,&p,op,&args).await}
 match op{
 "snapshot"=>Ok(json!({"profiles":rt.db.profiles()?.into_iter().filter(|p|access::allowed(&rt,p)).collect::<Vec<_>>(),"logs":rt.db.logs("")?.into_iter().filter(|l|access::current(&rt)=="owner"||rt.db.profile(&l.profile_id).is_ok_and(|p|access::allowed(&rt,&p))).collect::<Vec<_>>(),"statuses":*rt.statuses.lock().unwrap(),"theme":rt.db.get("theme"),"accent":rt.db.get("accent"),"apiPort":*rt.api_port.lock().unwrap(),"dataDir":rt.base.to_string_lossy()})),
 "profile.save"=>{let mut profile:Profile=serde_json::from_value(args["profile"].clone()).map_err(|_|"Perfil inválido")?;
 if !profile.ai.anchor.is_empty()&&!ANCHORS.contains(&profile.ai.anchor.as_str()){return Err("Ancoragem da IA inválida".into())}
 if !profile.ai.answer_length.is_empty()&&!LENGTHS.contains(&profile.ai.answer_length.as_str()){return Err("Tamanho padrão da resposta da IA inválido".into())}
 if !KNOWLEDGE_DEPTHS.contains(&profile.ai.knowledge_depth.as_str()){return Err("Profundidade da base de conhecimento inválida".into())}
 if profile.ai.knowledge_off.len()>500||profile.ai.knowledge_off.iter().any(|o|o.len()>260){return Err("Lista da base de conhecimento inválida".into())}
 if let Ok(previous)=rt.db.profile(&profile.id){if previous.channel!=profile.channel||previous.channel_id!=profile.channel_id||previous.platform!=profile.platform{rt.conversation.lock().unwrap().clear(&profile.id);rt.live.clear(&profile.id);rt.dashboard.clear_profile(&profile.id);}if previous.platform!=profile.platform||previous.client_id!=profile.client_id{rt.disconnect(&profile.id);for key in ["bot_token","bot_refresh","channel_token","channel_refresh","bot_expires","channel_expires"]{let _=secrets::set(&profile.id,key,"");}}}profile.blocklist=profile.blocklist.into_iter().map(|s|s.trim().to_owned()).filter(|s|!s.is_empty()).collect();profile.editors=profile.editors.into_iter().map(|s|s.trim().to_lowercase()).filter(|s|!s.is_empty()).collect();profile.topics=profile.topics.into_iter().map(|s|s.trim().to_owned()).filter(|s|!s.is_empty()).collect();vault::root(&rt.base,&profile.id)?;rt.db.save_profile(&profile)?;Ok(json!(profile))},
 "profile.delete"=>{
 let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p)?;rt.disconnect(&p);
 // Refuse paths that escape the app-data vault through junctions or symlinks.
 let root=vault::root(&rt.base,&p)?;
 let canonical=root.canonicalize().map_err(|e|e.to_string())?;
 let parent=rt.base.join("vaults").canonicalize().map_err(|e|e.to_string())?;
 if !canonical.starts_with(&parent)||canonical==parent{return Err("Caminho de vault inválido".into())}
 std::fs::remove_dir_all(&root).map_err(|e|e.to_string())?;
 // A base de conhecimento acompanha o perfil; falha aqui não impede o apagamento.
 if let (Ok(kroot),Ok(kparent))=(knowledge::root(&rt.base,&p),rt.base.join("knowledge").canonicalize()) {
  if let Ok(k)=kroot.canonicalize() {if k.starts_with(&kparent)&&k!=kparent {let _=std::fs::remove_dir_all(&k);}}
 }
 rt.db.delete_profile(&p)?;rt.dashboard.clear_profile(&p);rt.conversation.lock().unwrap().clear(&p);rt.live.clear(&p);rt.listen.reset(&p);secrets::clear(&p);rt.spotify.bump(&p);rt.spotify.set(&p,None);crate::discord_admin::stop(&rt,&p);crate::discord::clear_cache(&rt,&p);Ok(Value::Null)
 },
 "command.counters"=>command_counter::list(&rt.db,&p),
 "command.counter.set"=>{let id=args["id"].as_str().ok_or("Comando ausente")?;let value=args["value"].as_i64().ok_or("Informe um número inteiro")?;let count=command_counter::change(&rt.db,&p,id,Some(value))?;rt.emit("command-counter",json!({"profileId":p,"id":id,"value":count}));Ok(json!(count))},
 "timer.preview"=>{let f=rt.db.flows(&p)?.into_iter().find(|f|f.id==args["id"].as_str().unwrap_or("")&&f.trigger.kind=="timer").ok_or("Timer não encontrado")?;rt.submit(timers::event(&f,true)).await?;Ok(Value::Null)},
 "flows"=>Ok(json!(rt.db.flows(&p)?)),
 "dashboard.get"=>{let profile=rt.db.profile(&p)?;Ok(dashboard::get(&rt,&profile).await)},
 "dashboard.obs"=>{let profile=rt.db.profile(&p)?;Ok(dashboard::obs(&rt,&profile).await)},
 "liveConsole.get"=>{
  let profile=rt.db.profile(&p)?;let runs=rt.runs.history(&p)?;let voice=voice_redemptions::view(&rt,&profile);
  Ok(json!({"channel":rt.statuses.lock().unwrap().get(&p).cloned().unwrap_or_else(||"offline".into()),"audio":audio::state(&rt,&p),"voice":voicemod::health(&rt),"voiceQueue":{"activeName":voice["queue"]["active"]["voiceName"],"waiting":voice["queue"]["upcoming"].as_array().map_or(0,Vec::len),"pendingRestore":voice["queue"]["pendingRestore"]},"obs":obs::health(&rt,&p),"running":runs.iter().filter(|r|matches!(r["status"].as_str(),Some("RUNNING"|"WAITING"))).count(),"issues":runs.iter().filter(|r|matches!(r["status"].as_str(),Some("FAILED"|"INTERRUPTED"))).count()}))
 },
 "actionLibrary.list"=>Ok(json!(action_library::list(&rt.db,&p)?)),
 "actionLibrary.delete"=>{action_library::remove(&rt.db,&p,args["id"].as_str().ok_or("Etapa ausente")?)?;Ok(Value::Null)},
 "actionLibrary.save"=>{let entry=serde_json::from_value(args["entry"].clone()).map_err(|_|"Etapa reutilizável inválida")?;Ok(json!(action_library::save(&rt.db,&p,entry)?))},
 "variables.list"=>{let profile=rt.db.profile(&p)?;let user=args["userId"].as_str().unwrap_or("");let key=if user.is_empty(){String::new()}else{format!("{}:{user}",profile.platform)};variables::list(&rt.db,&p,&key)},
 "variables.change"=>{let profile=rt.db.profile(&p)?;variables::mutate(&rt.db,&p,&profile.platform,args["userId"].as_str().unwrap_or(""),args["target"].as_str().ok_or("Variável ausente")?,args["operation"].as_str().unwrap_or("set"),args["value"].clone())},
 "variables.preview"=>{
 let profile=rt.db.profile(&p)?;
 let mut e:Event=serde_json::from_value(args["event"].clone()).map_err(|_|"Evento inválido")?;
 e.profile_id=p.clone();e.simulated=true;
 let flow:Option<Flow>=args.get("flow").filter(|v|!v.is_null()).map(|v|serde_json::from_value(v.clone())).transpose().map_err(|_|"Fluxo inválido")?;
 // A prévia trata conteúdo legado com cifrão exatamente como o salvamento vai tratar.
 let mut flow=flow;if let Some(f)=flow.as_mut(){variables::migrate_flow(f);}
 // A prévia entrega o evento que o gatilho produz na prática, não o dado de teste.
 if let Some(f)=flow.as_ref(){e=model::preview_event(f,e);}
 let mut context=variables::Context::new(&rt.db,&profile,&e,flow.as_ref(),rt.spotify.now(&profile.id))?;
 let mut steps=vec![];
 if let Some(f)=flow {validate_flow(&f)?;let f=action_library::resolve(&rt.db,&f)?;for a in &f.actions {if !a.condition.is_empty()&&!e.message.to_lowercase().contains(&a.condition.to_lowercase()){continue}if a.kind=="script"{steps.push(json!({"kind":"script","text":"Script não executado na prévia"}));continue}let text=if a.kind=="variable.delete"{String::new()}else{context.render(&a.text)?};if a.kind.starts_with("variable."){context.change(&rt.db,&profile,&e,&a.target,a.kind.trim_start_matches("variable."),variables::typed(&text))?;}if matches!(a.kind.as_str(),"ai"|"ai.generate"){ai::save_response(&mut context,&rt,&profile,&e,a,"[Prévia: resposta contextual da IA]",false)?;}steps.push(json!({"kind":a.kind,"text":text}));}}
 let text=context.render(&variables::migrate(args["text"].as_str().unwrap_or("")))?;
 Ok(json!({"text":text,"steps":steps,"variables":context.inspect()}))
 },
 "flow.save"=>{let mut f:Flow=serde_json::from_value(args["flow"].clone()).map_err(|_|"Fluxo inválido")?;if !f.audio.is_empty(){chat_extras::asset(&rt,&f.profile_id,&f.audio,"sound")?;}if command_counter::auto_enable(&mut f){rt.log(&f.profile_id,"counter",&format!("Contagem ligada em \"{}\": a resposta usa {{{{commandCount}}}}",f.name),"info");}action_library::resolve(&rt.db,&f)?;rt.db.save_flow(&f)?;variables::migrate_flow(&mut f);Ok(json!(f))},
 "flow.delete"=>{rt.db.delete_flow(&p,args["id"].as_str().ok_or("Fluxo inválido")?)?;Ok(Value::Null)},
  "flow.test"=>{let f:Flow=serde_json::from_value(args["flow"].clone()).map_err(|_|"Fluxo inválido")?;let profile=rt.db.profile(&f.profile_id)?;validate_flow(&f)?;let e=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:f.profile_id.clone(),kind:"manual".into(),user:"Teste".into(),user_id:String::new(),role:"broadcaster".into(),message:String::new(),data:json!({"botliveTest":true}),simulated:false,rehearsal:true};let kinds=f.actions.iter().map(|a|a.kind.clone()).collect::<Vec<_>>();let exec=rt.runs.start(&profile.id,&f.id,&f.name,&kinds,true);let (rt2,p2)=(rt.clone(),profile.clone());let exec2=exec.clone();tokio::spawn(async move{engine::execute_flow_with_run(&rt2,&p2,&e,&f,true,&[],None,true,Some(args["testVars"].clone()),exec2).await;});Ok(json!({"executionId":exec}))},
  // Comando ao vivo em ensaio: a mensagem de verdade entra pelo funil da live —
  // matches (com a permissão de quem usa), disputa entre automações e intervalo do
  // bucket de ensaio, nunca o da live. O que não responderia na live também não
  // responde aqui, e o motivo volta para o Estúdio avisar (decisões 2 e 3).
  "flow.rehearse"=>{let message=args["message"].as_str().unwrap_or("").trim();if message.is_empty(){return Err("Escreva o comando de teste".into())}let role=args["role"].as_str().filter(|r|["everyone","subscriber","moderator","broadcaster"].contains(r)).unwrap_or("everyone");let e=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.clone(),kind:"chat".into(),user:"Teste".into(),user_id:"ensaio".into(),role:role.into(),message:message.into(),data:json!({"botliveTest":true}),simulated:true,rehearsal:true};let matched:Vec<Flow>=rt.db.flows(&p)?.into_iter().filter(|f|f.enabled&&model::matches(&f.trigger,&e)).collect();let matched=matched.iter().map(|f|action_library::resolve(&rt.db,f)).collect::<Result<Vec<_>,_>>()?;let winner=match engine::responder(&matched){Some(id)=>matched.iter().find(|f|f.id==id).cloned(),None=>matched.first().cloned()};let Some(winner)=winner else{return Ok(json!({"fired":false,"reason":"Nenhuma automação ativa responderia esta mensagem: confira se está salva, ativa e se a permissão de quem usa confere."}))};if !engine::cooldown_ready(&rt,&p,&winner,&e){return Ok(json!({"fired":false,"reason":format!("Em live o comando não responderia agora: {} está no intervalo.",winner.name)}))}rt.submit(e).await?;Ok(json!({"fired":true,"flow":winner.name,"flowId":winner.id}))},
  "flow.runs"=>{rt.db.profile(&p)?;Ok(json!(rt.runs.history(&p)?))},
  // Pré-voo: varredura estática das configs que só quebrariam ao vivo. Só lê —
  // nada aqui liga, grava ou chama serviço externo.
  "preflight.check"=>{let profile=rt.db.profile(&p)?;Ok(preflight::check(&rt,&profile))},
  "flow.cancel"=>{if rt.runs.cancel(&p,args["executionId"].as_str().unwrap_or("")){rt.log(&p,"run","Cancelamento solicitado; aguardando confirmação do motor","info");Ok(Value::Null)}else{Err("Execução não encontrada ou já terminada".into())}},
 "logs"=>Ok(json!(rt.db.logs(&p)?)),
 "stats"=>stats::read(&rt,&p),
 "connect"=>{rt.connect(&p).await?;Ok(Value::Null)},
 "disconnect"=>{rt.disconnect(&p);Ok(Value::Null)},
 "simulate"=>{let mut e:Event=serde_json::from_value(args["event"].clone()).map_err(|_|"Evento inválido")?;e.simulated=true;e.id=uuid::Uuid::new_v4().to_string();rt.submit(e).await?;Ok(Value::Null)},
 "chat.send"=>{let profile=rt.db.profile(&p)?;let e=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p,kind:"manual".into(),user:"".into(),user_id:"".into(),role:"broadcaster".into(),message:"".into(),data:Value::Null,simulated:false,rehearsal:false};rt.send(&profile,&e,args["text"].as_str().ok_or("Escreva uma mensagem")?).await?;Ok(Value::Null)},
 "secret.save"=>{rt.db.profile(&p)?;let key=args["key"].as_str().ok_or("Chave inválida")?;if !["ai_key","client_secret","discord_webhook","discord_token","obsidian_key","obs_password","vm_key"].contains(&key){return Err("Tipo de credencial não permitido".into())}secrets::set(&p,key,args["value"].as_str().ok_or("Credencial inválida")?)?;
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
 let e=Event{id:"ai-preview".into(),profile_id:p,kind:"chat".into(),user:"Espectador de teste".into(),user_id:"test-user".into(),role:"everyone".into(),message:message.into(),data:Value::Null,simulated:true,rehearsal:false};
 let c=variables::Context::new(&rt.db,&profile,&e,None,rt.spotify.now(&profile.id))?;
 let history:Vec<Value>=recent.lines().filter(|s|!s.trim().is_empty()).rev().take(12).collect::<Vec<_>>().into_iter().rev().map(|s|json!({"message":s.chars().take(500).collect::<String>()})).collect();
 let opts=ai::Options{knowledge:crate::knowledge::context(&rt.base,&profile,Some(&e),message),live:rt.live.summary(&profile.id),speech:crate::speech::summary(&rt.db,&profile.id),emotes:crate::emotes::list(&rt.db,&profile.id),..ai::Options::defaults(&profile)};
 Ok(json!(ai::conversation(&rt.http,&rt.base,&profile,&e,&c.render(instruction)?,&history,&opts).await?))
 },
 "ai.test"=>{let profile=rt.db.profile(&p)?;Ok(json!(ai::generate(&rt.http,&rt.base,&profile,args["user"].as_str().unwrap_or("streamer"),args["prompt"].as_str().unwrap_or("Olá! Apresente-se brevemente.")).await?))},
 "ollama"=>{let profile=rt.db.profile(&p)?;ai::validate_url(&profile.ai.endpoint)?;let res=rt.http.get(format!("{}/api/tags",profile.ai.endpoint.trim_end_matches('/'))).send().await.map_err(|_|"Ollama não está disponível. Inicie o Ollama e tente novamente.")?;res.json().await.map_err(|_|"Resposta Ollama inválida".into())},
 "notes"=>{rt.db.profile(&p)?;Ok(json!(vault::list(&rt.base,&p)?))},
 "note.save"=>{let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p)?;vault::write(&rt.base,&p,args["path"].as_str().ok_or("Caminho ausente")?,args["content"].as_str().ok_or("Conteúdo ausente")?,false)?;Ok(Value::Null)},
 "note.delete"=>{let _lock=rt.vault_lock.lock().unwrap();rt.db.profile(&p)?;vault::delete(&rt.base,&p,args["path"].as_str().ok_or("Caminho ausente")?)?;Ok(Value::Null)},
 "vault.open"=>{rt.db.profile(&p)?;open::that(vault::root(&rt.base,&p)?).map_err(|_|"Não foi possível abrir a pasta")?;Ok(Value::Null)},
 "knowledge.list"=>{rt.db.profile(&p)?;Ok(json!(knowledge::list(&rt.base,&p)?.into_iter().map(|e|json!({"path":e.path,"category":e.category,"file":e.file,"title":e.title,"kind":e.kind,"size":e.size})).collect::<Vec<_>>()))},
 "knowledge.import"=>{let mut profile=rt.db.profile(&p)?;let source=args["path"].as_str().ok_or("Escolha uma pasta")?.to_owned();let made=knowledge::import(&rt.base,&p,&source)?;profile.ai.knowledge_source=source;rt.db.save_profile(&profile)?;Ok(made)},
 "knowledge.remove"=>{rt.db.profile(&p)?;knowledge::remove(&rt.base,&p,args["path"].as_str().ok_or("Arquivo ausente")?)?;Ok(Value::Null)},
 "community"=>{rt.db.profile(&p)?;Ok(json!(modules::load(&rt,&p)))},
 "twitch.rewards"=>{let profile=rt.db.profile(&p)?;crate::twitch_ops::custom_rewards(&rt,&profile).await},
 "twitch.reward.create"=>{let profile=rt.db.profile(&p)?;crate::twitch_ops::create_reward(&rt,&profile,args["title"].as_str().unwrap_or(""),args["cost"].as_i64().unwrap_or(0),args["userInput"].as_bool().unwrap_or(false)).await},
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
 "voice.frame"=>listen::frame(&rt,&p,args["pcm"].as_str().ok_or("Áudio ausente")?),
 "voice.status"=>Ok(listen::status(&rt,&p)),
 "update.check"=>update::check(&rt,false).await,
 "update.install"=>update::check(&rt,true).await,
 "module.config"=>{rt.db.profile(&p)?;let key=args["key"].as_str().ok_or("Configuração ausente")?;if !["moderation","tts","voice","obsidian","points","songs","discord","discordBot","games","alerts","chatOverlay","audio","affinity","emoteBurst","spotify","voiceRedeem"].contains(&key){return Err("Configuração inválida".into())}if key=="moderation"&&!moderation::valid_config(&args["value"]){return Err("Ação inválida".into())}if key=="voice"&&!listen::valid_config(&args["value"]){return Err("Configuração de voz inválida: o servidor de transcrição precisa ser local (127.0.0.1) e os limites do painel têm faixas fixas".into())}if key=="discordBot"&&!discord_admin::valid_config(&args["value"]){return Err("Configuração do Discord inválida. Confira o servidor e os canais informados.".into())}if key=="alerts"&&!alerts::valid_config(&args["value"]){return Err("Configuração de alertas inválida: confira duração (3 a 20 s), velocidade, cores, fontes e textos de cada evento".into())}if key=="chatOverlay"&&!chat_overlay::valid_config(&args["value"]){return Err("Configuração do chat inválida: confira faixas, cores, fontes, badges e filtros".into())}if key=="spotify"&&!spotify::valid_config(&args["value"]){return Err("Configuração do Spotify inválida: confira o interruptor, o Client ID e o design do overlay".into())}if key=="voiceRedeem"&&!voice_redemptions::valid_config(&args["value"]){return Err("Configuração de resgates de voz inválida: cada vínculo precisa de reward e voz escolhidos, duração e política de concorrência; \"Efeito desligado\" não é voz".into())}if key=="tts"&&!tts::valid_config(&args["value"]){return Err("Configuração do texto para voz inválida: escolha uma voz do Windows, velocidade de 0,5 a 2 e limite de 1 a 500 caracteres".into())}if key=="audio"&&!audio::valid_config(&args["value"]){return Err("Configuração de áudio inválida: confira o aparelho de saída".into())}if key=="affinity"&&!modules::valid_config(&args["value"]){return Err("Configuração do Match inválida: confira palavras-gatilho, comando começando com ! e as cinco faixas (uma por linha, até 500 caracteres)".into())}if key=="emoteBurst"&&!emote_burst::valid_config(&args["value"]){return Err("Configuração da explosão inválida: confira quantidade (10 a 80), tamanho (24 a 120) e a origem dos emotes".into())}let value=if key=="affinity"{modules::with_defaults(&args["value"])}else{args["value"].clone()};rt.db.set_module(&p,key,&value)?;if key=="alerts"{rt.emit("alerts",json!({"profileId":p,"config":args["value"]}));}if key=="chatOverlay"{rt.emit("chat.config",json!({"profileId":p,"config":args["value"]}));}if key=="spotify"{rt.emit("spotify.config",json!({"profileId":p,"config":args["value"]}));}Ok(Value::Null)},
 "module.config.get"=>{rt.db.profile(&p)?;Ok(rt.db.module(&p,args["key"].as_str().ok_or("Configuração ausente")?))},
 "alerts.get"=>{let profile=rt.db.profile(&p)?;Ok(json!({"config":alerts::config(&rt,&p),"sounds":alerts::sounds(&rt,&p),"overlay":livepix::overlay_info(&rt,&profile.id)?}))},
 "alerts.test"=>{let profile=rt.db.profile(&p)?;alerts::test(&rt,&profile,args["kind"].as_str().ok_or("Evento ausente")?).await},
  "emoteBurst.test"=>{let profile=rt.db.profile(&p)?;emote_burst::test_fresh(&rt,&profile).await?;Ok(Value::Null)},
 "chat.get"=>{let profile=rt.db.profile(&p)?;Ok(json!({"config":chat_overlay::config(&rt,&p),"overlay":livepix::overlay_info(&rt,&profile.id)?}))},
 "chat.test"=>{let profile=rt.db.profile(&p)?;chat_overlay::test(&rt,&profile,&args)},
 "spotify.get"=>{rt.db.profile(&p)?;Ok(json!({"config":spotify::merged(&rt,&p),"connected":spotify::connected(&p),"now":rt.spotify.now(&p),"overlay":livepix::overlay_info(&rt,&p)?}))},
 "spotify.connect"=>{rt.db.profile(&p)?;let cfg=spotify::merged(&rt,&p);let out=spotify::connect(&rt.http,&p,&cfg).await?;rt.spotify.bump(&p);Ok(out)},
 "spotify.disconnect"=>{rt.db.profile(&p)?;spotify::disconnect(&p)?;rt.spotify.bump(&p);rt.spotify.set(&p,None);rt.emit("spotify.now",json!({"profileId":p,"playing":false}));Ok(Value::Null)},
 "spotify.poll"=>{let profile=rt.db.profile(&p)?;let cfg=spotify::merged(&rt,&p);Ok(json!({"track":spotify::poll_once(&rt,&profile,&cfg).await?}))},
 "spotify.test"=>{rt.db.profile(&p)?;if spotify::merged(&rt,&p)["enabled"]!=Value::Bool(true){return Err("O Spotify está desligado: ligue o interruptor da música no overlay antes de testar".into())}rt.emit("spotify.now",spotify::test_payload(&p));Ok(json!({"fired":true}))},
 "audio.get"=>{rt.db.profile(&p)?;let (port,token)=audio::overlay_addr(&rt,&p)?;let s=audio::state(&rt,&p);Ok(json!({"url":format!("http://127.0.0.1:{port}/audio#token={token}&profile={p}"),"clients":s["clients"],"claimed":s["claimed"]}))},
 "audio.test"=>{rt.db.profile(&p)?;let out=audio::test(&rt,&p);Ok(json!({"id":out["id"],"sink":out["sink"]}))},
 "audio.preview"=>{rt.db.profile(&p)?;let out=audio::preview(&rt,&p,args["asset"].as_str().ok_or("Som ausente")?,args["volume"].as_f64().unwrap_or(1.0))?;Ok(json!({"id":out["id"],"sink":out["sink"]}))},
 "audio.stop"=>{rt.db.profile(&p)?;Ok(json!({"stopped":audio::stop(&rt,&p)}))},
 "audio.ack"=>{rt.db.profile(&p)?;audio::ack(&rt,&p,args["id"].as_str().unwrap_or(""),args["phase"].as_str().unwrap_or(""),args["detail"].as_str()).map(|_|Value::Null)},
 "tts.voices"=>{rt.db.profile(&p)?;Ok(tts::voices(&rt).await?)},
 "tts.test"=>{rt.db.profile(&p)?;let out=tts::speak(&rt,&p,"Olá, comunidade! Este é o BotLive na transmissão.").await?;Ok(json!({"id":out["id"],"sink":out["sink"]}))},
 "settings"=>{let key=args["key"].as_str().ok_or("Configuração inválida")?;if !["theme","accent","apiPort","updateEndpoint","updatePublicKey","autoUpdate"].contains(&key){return Err("Configuração não permitida".into())}if key=="accent"&&!args["value"].as_str().is_some_and(|v|v.is_empty()||(v.len()==7&&v.starts_with('#')&&v[1..].chars().all(|c|c.is_ascii_hexdigit()))){return Err("Cor inválida".into())}rt.db.set(key,&args["value"])?;Ok(Value::Null)},
 "settings.get"=>Ok(json!({"theme":rt.db.get("theme"),"accent":rt.db.get("accent"),"apiPort":rt.db.get("apiPort"),"apiToken":rt.api_token,"updateEndpoint":update::endpoint(&rt),"updatePublicKey":update::public_key(&rt),"autoUpdate":rt.db.get("autoUpdate")})),
 "backup.get"=>Ok(backup::config(&rt.db)),
 "backup.save"=>backup::save_config(&rt.db,&args["config"]),
 "backup.list"=>backup::list(&rt.db),
 "backup.now"=>{let folder=args["folder"].as_str().map(str::to_owned).unwrap_or_else(||backup::config(&rt.db)["folder"].as_str().unwrap_or("").to_owned());backup::create(&rt,&folder)},
 "backup.restore"=>{let result=backup::restore(&rt,args["path"].as_str().ok_or("Escolha um arquivo de backup")?);if result.is_ok(){rt.log("","backup","Importação concluída. Reconecte os perfis para usar os dados recebidos.","success")}result},
 "voiceFx.devices"=>{rt.db.profile(&p)?;Ok(crate::voice_fx::devices())},
 "voiceFx.get"=>Ok(rt.voice_fx.get()),
 "voiceFx.level"=>Ok(rt.voice_fx.level()),
 "voiceFx.selection"=>rt.voice_fx.load_selection(&rt,&p),
 "voiceFx.saveSelection"=>rt.voice_fx.save_selection(&rt,&p,args["input"].as_str().unwrap_or(""),args["output"].as_str().unwrap_or("")),
 "voiceFx.start"=>rt.voice_fx.start(&rt,&p,args["input"].as_str().unwrap_or(""),args["output"].as_str().unwrap_or("")),
 "voiceFx.stop"=>rt.voice_fx.stop(&rt,&p),
 "voiceFx.effect"=>rt.voice_fx.set_effect(&rt,&p,args["name"].as_str().unwrap_or("")),
 "voiceFx.gate"=>rt.voice_fx.set_gate(&rt,&p,args["db"].as_f64().unwrap_or(-45.0) as f32),
 "voiceFx.mode"=>{rt.db.profile(&p)?;rt.voice_fx.set_mode(&rt,&p,args["mode"].as_str().unwrap_or("internal"))},
 "voice.listenStop"=>{rt.listen.reset(&p);Ok(Value::Null)},
 "voiceResgate.get"=>{let profile=rt.db.profile(&p)?;Ok(crate::voice_redemptions::view(&rt,&profile))},
 "voiceResgate.save"=>{let profile=rt.db.profile(&p)?;crate::voice_redemptions::save(&rt,&profile,&args["config"])},
 "voiceResgate.cancel"=>{let profile=rt.db.profile(&p)?;crate::voice_redemptions::cancel_all(&rt,&profile).await},
 "voiceResgate.recover"=>{let profile=rt.db.profile(&p)?;crate::voice_redemptions::recover(&rt,&profile).await},
 "voiceResgate.testApply"=>{let profile=rt.db.profile(&p)?;crate::voice_redemptions::apply_manual(&rt,&profile,args["voiceId"].as_str().unwrap_or(""),"Teste da tela",args["seconds"].as_u64().unwrap_or(60),"").await.map(|msg|json!({"message":msg}))},
 _=>Err("Operação desconhecida".into())
 }
}
pub fn run(){
 tauri::Builder::default().plugin(tauri_plugin_dialog::init()).plugin(tauri_plugin_updater::Builder::new().build())
 .setup(|app|{
 let base=app.path().app_data_dir()?;
 let rt=tauri::async_runtime::block_on(async{Runtime::new(base)})?;
 for name in command_counter::migrate_counters(&rt.db){rt.log("","counter",&format!("Contagem ligada em \"{name}\": a resposta usa {{commandCount}}"),"info");}
 *rt.app.lock().unwrap()=Some(app.handle().clone());app.manage(rt.clone());
 let discord_rt=rt.clone();tauri::async_runtime::spawn(async move{
  for profile in discord_rt.db.profiles().unwrap_or_default(){
   if discord::config(&discord_rt,&profile.id)["enabled"]==true{
    if let Err(err)=discord_admin::restore(&discord_rt,&profile.id){discord_rt.log(&profile.id,"discord",&err,"error")}
   }
  }
 });
 let timer_rt=rt.clone();tauri::async_runtime::spawn(timers::run(timer_rt));
 let dashboard_rt=rt.clone();tauri::async_runtime::spawn(dashboard::monitor(dashboard_rt));
 let scheduler_rt=rt.clone();tauri::async_runtime::spawn(scheduler::run(scheduler_rt));
 let spotify_rt=rt.clone();tauri::async_runtime::spawn(spotify::run(spotify_rt));
 let update_rt=rt.clone();tauri::async_runtime::spawn(async move{if update_rt.db.get("autoUpdate")==true {if let Ok(v)=update::check(&update_rt,false).await{if v["available"]==true{update_rt.log("","update","Há uma atualização disponível nas configurações.","info");}}}});
 tauri::async_runtime::spawn(async move{if let Err(e)=local_api::serve(rt.clone()).await{rt.log("","api",&e,"error");}});
 Ok(())
 }).invoke_handler(tauri::generate_handler![rpc]).run(tauri::generate_context!()).expect("Não foi possível iniciar o BotLive");
}

#[cfg(test)] mod integration_tests;
