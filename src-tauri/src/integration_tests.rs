use super::*;
use serde_json::json;
#[tokio::test]
async fn command_counts_follow_permissions_cooldowns_and_simulations(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("Contadores");p.editors=vec!["mod".into()];rt.db.save_profile(&p).unwrap();let other=profile("Outro");rt.db.save_profile(&other).unwrap();
 let mut f=flow(&p);f.counter=true;f.trigger.permission="moderator".into();f.actions=vec![Action{kind:"overlay".into(),text:"{{commandCount}}".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];rt.db.save_flow(&f).unwrap();
 engine::process(rt.clone(),event(&p,"!oi",false)).await;assert_eq!(command_counter::get(&rt.db,&p.id,&f.id).unwrap(),0);
 let mut e=event(&p,"!oi",false);e.role="moderator".into();let mut rx=rt.broadcast.subscribe();engine::process(rt.clone(),e.clone()).await;engine::process(rt.clone(),e.clone()).await;
 assert_eq!(command_counter::get(&rt.db,&p.id,&f.id).unwrap(),1);assert!(std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="overlay"&&v["payload"]["text"]=="1"));
 e.simulated=true;engine::process(rt.clone(),e).await;assert_eq!(command_counter::get(&rt.db,&p.id,&f.id).unwrap(),1);assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message.contains("Executaria overlay: 2")));
 assert_eq!(command_counter::get(&rt.db,&other.id,&f.id).unwrap(),0);
 let preview=variables::Context::new(&rt.db,&other,&event(&other,"!oi",true),Some(&f)).unwrap();assert_eq!(preview.render("{{commandCount}}").unwrap(),"0");
 *rt.actor.lock().unwrap()="mod".into();assert!(dispatch(rt.clone(),"command.counter.set",json!({"profileId":other.id,"id":f.id,"value":12})).await.is_err());
 assert_eq!(dispatch(rt.clone(),"command.counter.set",json!({"profileId":p.id,"id":f.id,"value":12})).await.unwrap(),12);
}
#[tokio::test]
async fn timer_target_offline_preview_and_external_rejection(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Timers");rt.db.save_profile(&p).unwrap();let mut a=flow(&p);a.trigger.kind="timer".into();a.trigger.cooldown=0;a.trigger.user_cooldown=0;a.actions=vec![Action{kind:"overlay".into(),text:"Timer A".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];let mut b=a.clone();b.id=uuid::Uuid::new_v4().to_string();b.actions[0].text="Timer B".into();rt.db.save_flow(&a).unwrap();rt.db.save_flow(&b).unwrap();
 let mut e=event(&p,"",false);e.kind="timer".into();e.data=json!({"timerId":a.id,"timerRevision":timers::revision(&a)});e.role="broadcaster".into();assert!(rt.submit(e.clone()).await.is_err());
 let mut rx=rt.broadcast.subscribe();engine::process(rt.clone(),e.clone()).await;assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="overlay"));
 rt.status(&p.id,"online");rt.timer_pending.lock().unwrap().insert(a.id.clone());engine::process(rt.clone(),e.clone()).await;assert!(rt.timer_pending.lock().unwrap().is_empty());let values:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="overlay").collect();assert_eq!(values.len(),1);assert_eq!(values[0]["payload"]["text"],"Timer A");
 a.name="Alterado".into();rt.db.save_flow(&a).unwrap();engine::process(rt.clone(),e.clone()).await;assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="overlay"));
 rt.status(&p.id,"offline");e.simulated=true;engine::process(rt.clone(),e).await;assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message.contains("Executaria overlay: Timer A")));
 a.timer_seconds=0;assert!(rt.db.save_flow(&a).is_err());a.timer_seconds=30;a.counter=true;assert!(rt.db.save_flow(&a).is_err());
}
#[tokio::test]
async fn flow_audio_and_twitch_delivery_type(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let p=profile("Áudio e envio");rt.db.save_profile(&p).unwrap();
 let file=dir.path().join("jingle.wav");std::fs::write(&file,b"RIFF0000WAVEfmt ").unwrap();
 let asset=dispatch(rt.clone(),"chatExtras.import",json!({"profileId":p.id,"kind":"sound","path":file})).await.unwrap();
 let mut f=flow(&p);f.name="ifood".into();f.audio=asset["id"].as_str().unwrap().into();f.audio_volume=0.6;f.send_type="announce".into();f.send_color="purple".into();
 let mut sem_arquivo=f.clone();sem_arquivo.audio=uuid::Uuid::new_v4().to_string();
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":sem_arquivo})).await.is_err());
 let mut cor=f.clone();cor.send_color="vermelho".into();
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":cor})).await.is_err());
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":f.clone()})).await.is_ok());
 let mut rx=rt.broadcast.subscribe();
 engine::process(rt.clone(),event(&p,"!oi",false)).await;
 let sounds:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="viewer-sound").collect();
 assert_eq!(sounds.len(),1);assert_eq!(sounds[0]["payload"]["asset"].as_str().unwrap(),asset["id"].as_str().unwrap());assert_eq!(sounds[0]["payload"]["volume"].as_f64().unwrap(),0.6);
 // Sem autorização a ação falha com aviso no histórico, sem interromper o processo.
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.status=="error"&&l.message.starts_with("ifood · ")));
 // A simulação registra o plano: sem som e com o tipo de envio escolhido.
 engine::process(rt.clone(),event(&p,"!oi",true)).await;
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="viewer-sound"));
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.message.contains("Tocaria o áudio do fluxo")));
 assert!(logs.iter().any(|l|l.message.starts_with("[Simulação] [Anúncio]")));
}
#[tokio::test]
async fn txt_engine_simulation_and_file_configuration_permissions(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let mut p=profile("TXT");p.editors=vec!["mod".into()];rt.db.save_profile(&p).unwrap();
 let file=dir.path().join("frases.txt");std::fs::write(&file,"Olá {{user}}: {{message}}").unwrap();
 let a=dispatch(rt.clone(),"chatExtras.import",json!({"profileId":p.id,"kind":"txt","path":file})).await.unwrap();
 let config=json!({"repliesEnabled":true,"soundsEnabled":false,"people":[],"replies":[{"id":uuid::Uuid::new_v4(),"enabled":true,"keyword":"oi","matching":"word","selection":"sequence","asset":a["id"],"cooldown":30,"userCooldown":60}]});
 dispatch(rt.clone(),"chatExtras.save",json!({"profileId":p.id,"config":config})).await.unwrap();
 engine::process(rt.clone(),event(&p,"oi {{global.secret}}",true)).await;
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message=="[Simulação] Olá Ana: oi {{global.secret}}"));
 *rt.actor.lock().unwrap()="mod".into();assert!(dispatch(rt.clone(),"chatExtras.save",json!({"profileId":p.id,"config":config})).await.is_err());assert!(dispatch(rt.clone(),"chatExtras.import",json!({"profileId":p.id,"path":file,"kind":"txt"})).await.is_err());
 assert!(dispatch(rt.clone(),"chatExtras.preview",json!({"profileId":p.id,"asset":a["id"]})).await.is_ok());
 *rt.actor.lock().unwrap()="intruso".into();assert!(dispatch(rt,"chatExtras.get",json!({"profileId":p.id})).await.is_err());
}
async fn mock_ai(answer:&str)->(String,tokio::task::JoinHandle<Value>){
 mock_ai_status(answer,200).await
}
async fn mock_ai_status(answer:&str,status:u16)->(String,tokio::task::JoinHandle<Value>){
 use tokio::io::{AsyncReadExt,AsyncWriteExt};
 let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();let answer=answer.to_owned();
 let task=tokio::spawn(async move{
  let(mut socket,_)=listener.accept().await.unwrap();let mut bytes=vec![];
  let request=loop{
   let mut part=[0u8;4096];let n=socket.read(&mut part).await.unwrap();assert!(n>0);bytes.extend_from_slice(&part[..n]);assert!(bytes.len()<100000);
   if let Some(end)=bytes.windows(4).position(|w|w==b"\r\n\r\n"){
    let headers=String::from_utf8_lossy(&bytes[..end]).to_lowercase();
    let len:usize=headers.lines().find_map(|l|l.strip_prefix("content-length:")).unwrap().trim().parse().unwrap();
    if bytes.len()>=end+4+len{break serde_json::from_slice::<Value>(&bytes[end+4..end+4+len]).unwrap()}
   }
  };
  let body=json!({"message":{"content":answer}}).to_string();
  socket.write_all(format!("HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();request
 });
 (format!("http://{address}"),task)
}
#[tokio::test]
async fn api_errors_never_publish_fallback_or_execute_following_actions(){
 for kind in ["ai","ai.generate"] {for status in [401,429,500] {
  let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("Falhas");
  p.ai.model="test".into();p.ai.remember=true;p.ai.fallback="Não consegui responder agora. Tente novamente em instantes.".into();
  let(endpoint,request)=mock_ai_status("Detalhe privado do provedor",status).await;p.ai.endpoint=endpoint;rt.db.save_profile(&p).unwrap();
  let mut f=flow(&p);f.actions=vec![Action{kind:kind.into(),text:"Responda".into(),..Default::default()},Action{kind:"overlay".into(),text:"Não pode executar".into(),..Default::default()}];rt.db.save_flow(&f).unwrap();
  let mut rx=rt.broadcast.subscribe();engine::process(rt.clone(),event(&p,"!oi falha de teste",false)).await;request.await.unwrap();
  while let Ok(e)=rx.try_recv(){assert_ne!(e["type"],"overlay");}
  let logs=rt.db.logs(&p.id).unwrap();assert!(logs.iter().any(|l|l.status=="error"&&l.message.contains(&format!("HTTP {status}"))));
  assert!(!logs.iter().any(|l|(l.kind=="chat"&&l.status=="success")||l.message.contains("Detalhe privado")||l.message.contains(&p.ai.fallback)));
  assert_eq!(logs.iter().filter(|l|l.status=="error").count(),1,"somente a falha do provedor, sem tentativa posterior de envio");
  assert_eq!(vault::list(&rt.base,&p.id).unwrap().len(),1);
 }}
}
#[tokio::test]
async fn contextual_ai_uses_chat_and_stores_single_pass_response_without_sending(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("Resenha");p.modules=json!({});p.ai.model="test".into();
 let (endpoint,request)=mock_ai("Hoje sim! {{message}}").await;p.ai.endpoint=endpoint;rt.db.save_profile(&p).unwrap();
 let other=profile("Outro");rt.db.save_profile(&other).unwrap();
 let mut echo=event(&p,"Mensagem do próprio bot",false);echo.user_id=p.bot_id.clone();engine::process(rt.clone(),echo).await;
 engine::process(rt.clone(),event(&other,"Não deve entrar no contexto",false)).await;
 engine::process(rt.clone(),event(&p,"Ontem estava errando os tiros",false)).await;
 let mut f=flow(&p);f.actions=vec![Action{kind:"ai.generate".into(),text:"Faça uma resenha leve".into(),target:"local.resenha".into(),value:0,condition:"".into(),..Default::default()},Action{kind:"overlay".into(),text:"{{local.resenha}} / {{local.aiSuccess}}".into(),target:"".into(),value:0,condition:"".into(),..Default::default()}];rt.db.save_flow(&f).unwrap();
 let mut receiver=rt.broadcast.subscribe();engine::process(rt.clone(),event(&p,"!oi Thenees hoje está amassando",false)).await;
 let request=request.await.unwrap();let prompt:Value=serde_json::from_str(request["messages"][1]["content"].as_str().unwrap()).unwrap();
 assert_eq!(prompt["conversation"]["currentMessage"]["message"],"!oi Thenees hoje está amassando");
 assert_eq!(prompt["conversation"]["recentChat"].as_array().unwrap().len(),1);
 assert_eq!(prompt["conversation"]["recentChat"][0]["message"],"Ontem estava errando os tiros");
 assert!(request["messages"][0]["content"].as_str().unwrap().contains("Não invente"));
 let mut found=false;while let Ok(e)=receiver.try_recv(){if e["type"]=="overlay"{assert_eq!(e["payload"]["text"],"Hoje sim! {{message}} / true");found=true}}assert!(found);
 assert!(!rt.db.logs(&p.id).unwrap().iter().any(|l|l.kind=="chat"&&l.status=="success"));
 assert!(variables::Context::new(&rt.db,&p,&event(&p,"oi",false),None).unwrap().render("{{local.aiResponse}}").is_err());
}
#[tokio::test]
async fn ai_simulation_preview_and_failures_stop_the_flow(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("Simulação");p.ai.fallback="Volto já".into();rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.actions=vec![Action{kind:"ai.generate".into(),text:"Resenha".into(),target:"local.aiResponse".into(),value:0,condition:"".into(),..Default::default()},Action{kind:"chat".into(),text:"{{local.aiResponse}} / {{local.aiSuccess}}".into(),target:"".into(),value:0,condition:"".into(),..Default::default()}];rt.db.save_flow(&f).unwrap();
 engine::process(rt.clone(),event(&p,"!oi",true)).await;
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message=="[Simulação] [Prévia: resposta contextual da IA] / false"));
 let preview=dispatch(rt.clone(),"variables.preview",json!({"profileId":p.id,"flow":f,"event":event(&p,"!oi",true)})).await.unwrap();
 assert_eq!(preview["steps"][1]["text"],"[Prévia: resposta contextual da IA] / false");
 f.actions[1].kind="overlay".into();rt.db.save_flow(&f).unwrap();let mut rx=rt.broadcast.subscribe();engine::process(rt.clone(),event(&p,"!oi",false)).await;
 while let Ok(e)=rx.try_recv(){assert_ne!(e["type"],"overlay","falha da IA deve interromper as próximas ações");}
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.status=="error"));
 assert!(!rt.db.logs(&p.id).unwrap().iter().any(|l|l.message.contains("Volto já")));
 f.actions[0].target="global.resposta".into();assert!(model::validate_flow(&f).is_err());
 f.actions[0].target="local.aiSuccess".into();assert!(model::validate_flow(&f).is_err());
}
#[tokio::test]
async fn contextual_preview_calls_provider_without_chat_or_memory_writes(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("Prévia");p.ai.model="test".into();p.ai.remember=true;
 let(endpoint,request)=mock_ai("Hoje até a mira resolveu trabalhar!").await;p.ai.endpoint=endpoint;rt.db.save_profile(&p).unwrap();
 let before=vault::list(&rt.base,&p.id).unwrap().len();
 let result=dispatch(rt.clone(),"ai.preview",json!({"profileId":p.id,"message":"Está amassando!","recent":"Bia: ele acertou tudo","instruction":"Brinque com {{message}}"})).await.unwrap();
 assert_eq!(result,"Hoje até a mira resolveu trabalhar!");
 let request=request.await.unwrap();let prompt:Value=serde_json::from_str(request["messages"][1]["content"].as_str().unwrap()).unwrap();
 assert_eq!(prompt["automationRequest"],"Brinque com Está amassando!");
 assert_eq!(prompt["conversation"]["recentChat"][0]["message"],"Bia: ele acertou tudo");
 assert_eq!(vault::list(&rt.base,&p.id).unwrap().len(),before);assert!(rt.db.logs(&p.id).unwrap().is_empty());
 assert!(rt.conversation.lock().unwrap().receive(&event(&p,"nova",false)).is_empty());
 *rt.actor.lock().unwrap()="intruso".into();assert!(dispatch(rt,"ai.preview",json!({"profileId":p.id,"message":"oi"})).await.is_err());
}
#[tokio::test]
async fn ai_answer_uses_live_state_base_and_per_action_controls(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut p=profile("Ao vivo");p.ai.model="test".into();p.ai.personality="Fale como uma pessoa real.".into();
 let (endpoint,request)=mock_ai("Boa, valeu!").await;p.ai.endpoint=endpoint;rt.db.save_profile(&p).unwrap();
 let src=tempfile::tempdir().unwrap();let source=src.path().join("base");
 std::fs::create_dir_all(source.join("tom-e-comportamento")).unwrap();
 std::fs::write(source.join("tom-e-comportamento").join("anti.md"),"# Anti\nnunca diga olá mundo\n").unwrap();
 dispatch(rt.clone(),"knowledge.import",json!({"profileId":p.id,"path":source.to_str().unwrap()})).await.unwrap();
 for _ in 0..3 {engine::process(rt.clone(),event(&p,"bora demais",false)).await;}
 rt.live.set_category(&p.id,"Counter-Strike 2");
 let mut sub=event(&p,"",false);sub.kind="subscription".into();sub.user="Bia".into();sub.data=json!({"cumulative_months":3});
 engine::process(rt.clone(),sub).await;
 let mut f=flow(&p);
 f.actions=vec![Action{kind:"ai.generate".into(),text:"Faça uma saudação".into(),ai_anchor:"message".into(),ai_length:"short".into(),ai_style:"gírias do chat".into(),ai_no_repeat:Some(true),..Default::default()},
 Action{kind:"overlay".into(),text:"{{local.aiResponse}} / {{local.aiSuccess}}".into(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let mut receiver=rt.broadcast.subscribe();
 engine::process(rt.clone(),event(&p,"!oi",false)).await;
 let request=request.await.unwrap();
 let system=request["messages"][0]["content"].as_str().unwrap();
 assert!(system.contains("[Contexto agora:"),"o que a live está fazendo agora entra no pedido");
 assert!(system.contains("jogando Counter-Strike 2"));
 assert!(system.contains("sub de Bia"));
 assert!(system.contains("calor alto"));
 assert!(system.contains("<conhecimento>"),"a base entra porque a ação não a desliga");
 assert!(system.contains("nunca diga olá mundo"));
 assert!(system.contains("Foque em currentMessage"),"a ancoragem da ação vale");
 assert!(system.contains("Tom deste bloco: gírias do chat"),"o estilo da ação vale");
 assert!(system.contains("no máximo 120 caracteres"),"o tamanho da ação vale");
 let prompt:Value=serde_json::from_str(request["messages"][1]["content"].as_str().unwrap()).unwrap();
 assert_eq!(prompt["automationRequest"],"Faça uma saudação");
 let mut found=false;while let Ok(e)=receiver.try_recv(){if e["type"]=="overlay"{assert_eq!(e["payload"]["text"],"Boa, valeu! / true");found=true}}assert!(found);
 // Ação que desliga a base: o bloco some e a ancoragem volta ao padrão do perfil.
 let mut p2=profile("Sem base");p2.ai.model="test".into();rt.db.save_profile(&p2).unwrap();
 rt.live.clear(&p2.id);
 assert!(crate::knowledge::context(&rt.base,&p2,None,"").is_empty());
 let mut a=Action{kind:"ai".into(),text:"oi".into(),..Default::default()};a.ai_knowledge="off".into();
 assert!(!a.knowledge(&p2.ai));
}
fn profile(name:&str)->Profile{Profile{id:uuid::Uuid::new_v4().to_string(),name:name.into(),platform:"twitch".into(),channel:name.into(),channel_id:"123".into(),bot_id:"456".into(),client_id:"client".into(),blocklist:vec!["proibido".into()],topics:vec![],editors:vec![],ai:AiConfig::default(),modules:json!({"points":true,"raffles":true,"predictions":true,"queue":true})}}
fn event(p:&Profile,text:&str,simulated:bool)->Event{Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),kind:"chat".into(),user:"Ana".into(),user_id:"ana".into(),role:"everyone".into(),message:text.into(),data:Value::Null,simulated}}
fn flow(p:&Profile)->Flow{Flow{counter:false,timer_seconds:300,audio:String::new(),audio_volume:1.0,send_type:"chat".into(),send_color:"primary".into(),reply_to:false,id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),name:"Teste".into(),enabled:true,trigger:Trigger{kind:"command".into(),pattern:"!oi".into(),permission:"everyone".into(),cooldown:5,user_cooldown:5},actions:vec!["Um $user","Dois","Três"].into_iter().map(|s|Action{kind:"chat".into(),text:s.into(),target:"".into(),value:0,condition:"".into(),..Default::default()}).collect(),layout:Value::Null}}
#[tokio::test]
async fn ordered_execution_cooldown_filter_and_isolation(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Canal A");let b=profile("Canal B");rt.db.save_profile(&p).unwrap();rt.db.save_profile(&b).unwrap();
 let f=flow(&p);rt.db.save_flow(&f).unwrap();
 engine::process(rt.clone(),event(&p,"!oi",true)).await;
 let logs=rt.db.logs(&p.id).unwrap();let messages:Vec<_>=logs.iter().rev().filter(|l|l.kind=="chat"&&l.status=="success").map(|l|l.message.as_str()).collect();
 assert_eq!(messages,vec!["[Simulação] Um Ana","[Simulação] Dois","[Simulação] Três"]);
 engine::process(rt.clone(),event(&p,"!oi",true)).await;
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.kind=="cooldown"));
 engine::process(rt.clone(),event(&b,"!oi",true)).await;
 assert!(!rt.db.logs(&b.id).unwrap().iter().any(|l|l.kind=="action"));
 assert!(rt.send(&p,&event(&p,"",true),"PROIBIDO").await.is_err());
}
#[tokio::test]
async fn preset_roundtrip_conflicts_and_no_credentials(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let a=profile("A");let b=profile("B");rt.db.save_profile(&a).unwrap();rt.db.save_profile(&b).unwrap();rt.db.save_flow(&flow(&a)).unwrap();
 let theme=presets::export(&rt,&a.id,"theme","Padrão",vec![]).unwrap();assert_eq!(theme["data"]["theme"],"dark");presets::apply(&rt,&b.id,&theme,"cancel").unwrap();
 let preset=presets::export(&rt,&a.id,"profile","Completo",vec![]).unwrap();
 assert_eq!(preset["data"]["profile"]["clientId"],"");assert_eq!(preset["data"]["profile"]["channelId"],"");
 presets::apply(&rt,&b.id,&preset,"cancel").unwrap();
 let imported=rt.db.flows(&b.id).unwrap();assert_eq!(imported.len(),1);assert!(!imported[0].enabled);assert_eq!(imported[0].profile_id,b.id);
 assert!(presets::apply(&rt,&b.id,&preset,"cancel").is_err());
 presets::apply(&rt,&b.id,&preset,"skip").unwrap();assert_eq!(rt.db.flows(&b.id).unwrap().len(),1);
 assert_eq!(rt.db.profile(&b.id).unwrap().channel_id,"123");
}
#[tokio::test]
async fn refunds_are_once_and_predictions_conserve_points(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("P");rt.db.save_profile(&p).unwrap();
 modules::change_points(&rt,&p.id,"ana",100).unwrap();
 modules::admin(&rt,&p.id,"raffle_open",json!({"keyword":"!sorteio","price":10})).unwrap();
 modules::chat(&rt,&p,&event(&p,"!ticket 2",false)).unwrap();
 let before=modules::load(&rt,&p.id).balances["ana"];
 modules::admin(&rt,&p.id,"raffle_cancel",Value::Null).unwrap();
 assert_eq!(modules::load(&rt,&p.id).balances["ana"],before+20);
 assert!(modules::admin(&rt,&p.id,"raffle_cancel",Value::Null).is_err());
 modules::change_points(&rt,&p.id,"bia",100).unwrap();
 modules::admin(&rt,&p.id,"prediction_open",json!({"title":"Teste","options":["Sim","Não"]})).unwrap();
 modules::chat(&rt,&p,&event(&p,"!bet 1 25",false)).unwrap();
 let mut b=event(&p,"!bet 2 50",false);b.user_id="bia".into();b.user="Bia".into();modules::chat(&rt,&p,&b).unwrap();
 let state=modules::load(&rt,&p.id);let total=state.balances.values().sum::<i64>()+75;
 modules::admin(&rt,&p.id,"prediction_settle",json!({"result":0})).unwrap();
 let result=modules::load(&rt,&p.id);assert_eq!(result.balances.values().sum::<i64>(),total);
 assert!(modules::admin(&rt,&p.id,"prediction_settle",json!({"result":0})).is_err());
 assert!(modules::admin(&rt,&p.id,"prediction_cancel",Value::Null).is_err());
}
#[tokio::test]
async fn rejected_module_operation_is_atomic(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("P");rt.db.save_profile(&p).unwrap();
 modules::change_points(&rt,&p.id,"ana",10).unwrap();
 modules::admin(&rt,&p.id,"shop_add",json!({"name":"Caro","cost":100,"stock":1})).unwrap();
 let before=rt.db.module(&p.id,"community");
 assert!(modules::chat(&rt,&p,&event(&p,"!resgatar 1",false)).is_err());
 assert_eq!(before,rt.db.module(&p.id,"community"));
}
#[tokio::test]
async fn database_survives_reopen_and_delete_preserves_other_profile(){
 let dir=tempfile::tempdir().unwrap();let a=profile("A");let b=profile("B");
 {let db=db::Db::open(&dir.path().join("db.sqlite")).unwrap();db.save_profile(&a).unwrap();db.save_profile(&b).unwrap();db.save_flow(&flow(&a)).unwrap();}
 let db=db::Db::open(&dir.path().join("db.sqlite")).unwrap();assert_eq!(db.flows(&a.id).unwrap().len(),1);db.delete_profile(&a.id).unwrap();assert!(db.profile(&b.id).is_ok());assert!(db.flows(&a.id).unwrap().is_empty());
}
#[tokio::test]
async fn websocket_rejects_unauthorized_and_accepts_events(){
 use futures_util::{SinkExt,StreamExt};use tokio_tungstenite::{connect_async,tungstenite::Message};
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 // Select a free local port for this test rather than assuming the default is free.
 let probe=std::net::TcpListener::bind("127.0.0.1:0").unwrap();let port=probe.local_addr().unwrap().port();drop(probe);rt.db.set("apiPort",&json!(port)).unwrap();
 let p=profile("P");rt.db.save_profile(&p).unwrap();
 let server=tokio::spawn(local_api::serve(rt.clone()));
 for _ in 0..50{if *rt.api_port.lock().unwrap()!=0{break}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
 let (mut ws,_)=connect_async(format!("ws://127.0.0.1:{port}")).await.unwrap();
 ws.send(Message::Text(json!({"token":"invalid"}).to_string().into())).await.unwrap();
 assert!(matches!(ws.next().await,Some(Ok(Message::Close(_)))|None));
 let (mut ws,_)=connect_async(format!("ws://127.0.0.1:{port}")).await.unwrap();
 ws.send(Message::Text(json!({"token":rt.api_token}).to_string().into())).await.unwrap();
 let first=ws.next().await.unwrap().unwrap().into_text().unwrap();assert!(first.contains("ready"));
 ws.send(Message::Text(json!({"type":"event","event":event(&p,"teste",true)}).to_string().into())).await.unwrap();
 let next=ws.next().await.unwrap().unwrap().into_text().unwrap();assert!(next.contains("ack"));
 server.abort();
}

#[tokio::test]
async fn editor_permissions_are_enforced_by_backend(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut a=profile("A");a.editors=vec!["mod".into()];let b=profile("B");
 rt.db.save_profile(&a).unwrap();rt.db.save_profile(&b).unwrap();*rt.actor.lock().unwrap()="mod".into();
 assert!(access::guard(&rt,"flow.save",&json!({"flow":flow(&a)})).is_ok());
 assert!(access::guard(&rt,"flow.save",&json!({"flow":flow(&b)})).is_err());
 assert!(access::guard(&rt,"settings.get",&json!({})).is_err());
 let mut escalation=a.clone();escalation.editors.push("intruso".into());
 assert!(access::guard(&rt,"profile.save",&json!({"profile":escalation})).is_err());
 *rt.actor.lock().unwrap()="locked".into();
 assert_eq!(access::guard(&rt,"snapshot",&json!({})).unwrap_err(),"SESSION_LOCKED");
}
#[tokio::test]
async fn ollama_adapter_blocks_generated_content(){
 use tokio::io::{AsyncReadExt,AsyncWriteExt};
 let server=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let port=server.local_addr().unwrap().port();
 let task=tokio::spawn(async move{let(mut socket,_)=server.accept().await.unwrap();let mut data=[0u8;8192];let _=socket.read(&mut data).await.unwrap();let body=r#"{"message":{"content":"Texto PROIBIDO"}}"#;let response=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);socket.write_all(response.as_bytes()).await.unwrap();});
 let dir=tempfile::tempdir().unwrap();let mut p=profile("IA");p.ai.model="test".into();p.ai.endpoint=format!("http://127.0.0.1:{port}");
 let result=ai::generate(&reqwest::Client::new(),dir.path(),&p,"Ana","Olá").await;
 assert!(result.unwrap_err().contains("bloqueada"));task.await.unwrap();
}

#[tokio::test]
async fn module_configuration_and_theme_roundtrip(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let a=profile("A");let b=profile("B");
 rt.db.save_profile(&a).unwrap();rt.db.save_profile(&b).unwrap();
 rt.db.set_module(&a.id,"points",&json!({"currency":"folhas","amount":7,"interval":60,"subscriberMultiplier":3})).unwrap();
 rt.db.set("theme",&json!("light")).unwrap();rt.db.set("accent",&json!("#4466aa")).unwrap();
 let preset=presets::export(&rt,&a.id,"profile","Completo",vec![]).unwrap();
 presets::apply(&rt,&b.id,&preset,"cancel").unwrap();
 assert_eq!(rt.db.module(&b.id,"points")["amount"],7);assert_eq!(rt.db.get("accent"),"#4466aa");
 let mut e=event(&b,"!pontos",false);e.role="subscriber".into();
 let reply=modules::chat(&rt,&b,&e).unwrap().unwrap();
 assert!(reply.contains("21 folhas"));assert_eq!(modules::load(&rt,&b.id).balances["ana"],21);
 modules::chat(&rt,&b,&e).unwrap();assert_eq!(modules::load(&rt,&b.id).balances["ana"],21);
}
#[tokio::test]
async fn song_limits_reject_atomically_and_emit_overlay(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("P");p.modules["songs"]=json!(true);rt.db.save_profile(&p).unwrap();
 rt.db.set_module(&p.id,"songs",&json!({"perUser":1,"limit":2})).unwrap();let mut events=rt.broadcast.subscribe();
 modules::chat(&rt,&p,&event(&p,"!musica https://youtu.be/test",false)).unwrap();
 let mut found=false;while let Ok(e)=events.try_recv(){if e["type"]=="songs"{found=true;assert_eq!(e["payload"]["queue"].as_array().unwrap().len(),1)}}
 assert!(found);let before=rt.db.module(&p.id,"community");
 assert!(modules::chat(&rt,&p,&event(&p,"!musica https://youtu.be/second",false)).is_err());
 assert_eq!(rt.db.module(&p.id,"community"),before);
}

#[test]
fn packaged_updater_configuration_is_deserializable(){
 let config:Value=serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
 let updater:tauri_plugin_updater::Config=serde_json::from_value(config["plugins"]["updater"].clone()).unwrap();
 assert_eq!(updater.endpoints.len(),1);
 assert!(config["plugins"]["updater"]["pubkey"].as_str().unwrap().len()>40);
 assert!(!updater.allow_downgrades);
}

#[tokio::test]
async fn updater_defaults_and_explicit_overrides(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 assert_eq!(update::endpoint(&rt),"https://github.com/studiocactus/arrobabot/releases/latest/download/latest.json");
 assert!(!update::public_key(&rt).is_empty());
 rt.db.set("updateEndpoint",&json!("")).unwrap();assert!(update::endpoint(&rt).contains("github.com"));
 rt.db.set("updateEndpoint",&json!("https://example.com/updates.json")).unwrap();assert_eq!(update::endpoint(&rt),"https://example.com/updates.json");
}

#[tokio::test]
async fn variable_templates_are_single_pass_bounded_and_typed(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("P");rt.db.save_profile(&p).unwrap();
 let mut e=event(&p,"!oi  café  20",true);e.user="$channel {{global.secret}} %message%".into();e.data=json!({"reward":{"title":"Hidratar"},"items":[{"amount":4}],"user":"Intruso"});
 let c=variables::Context::new(&rt.db,&p,&e,Some(&flow(&p))).unwrap();
 assert_eq!(c.render("$user").unwrap(),"$user");
 assert_eq!(variables::migrate("$userId $unknown $messageSuffix"),"{{userId}} $unknown $messageSuffix");
 assert_eq!(variables::migrate(r"\$user $5 R$100 arg"),r"\$user $5 R$100 arg");
 assert_eq!(c.render(&variables::migrate("$userId $unknown $messageSuffix")).unwrap(),"ana $unknown $messageSuffix");
 assert_eq!(c.render("{{arg0|upper}} / %arg1% / {{rawInput}} / {{argCount}}").unwrap(),"CAFÉ / 20 / café  20 / 2");
 assert_eq!(c.render("{{data.reward.title}} {{data.items.0.amount|add:2}} {{arg1|number:2}}").unwrap(),"Hidratar 6 20.00");
 assert_eq!(c.render("{{missing|default:amigo|upper}} {{arg0|length}}").unwrap(),"AMIGO 4");
 assert_eq!(c.render(r"\{{user}} \$user \%userName%").unwrap(),"{{user}} $user %userName%");
 assert!(c.render("{{missing}}").is_err());assert!(c.render("{{user|unsupported}}").is_err());assert!(c.render("{{user").is_err());assert!(c.render("{{arg1|number:7}}").is_err());
 assert_eq!(c.render("{{random:3,3}}").unwrap(),"3","o sorteio passa pelo mesmo caminho das demais variáveis");
 assert!(c.render("R$ {{random:3.0,3.0}}").unwrap().starts_with("R$ 3,0"),"centavos em vírgula no meio da frase");
 assert!(c.render(&"x".repeat(65537)).is_err());
}
#[test]
fn legacy_dollar_markers_migrate_when_saved_and_read(){
 let dir=tempfile::tempdir().unwrap();let path=dir.path().join("vars.sqlite");let p=profile("P");
 let mut f=flow(&p);
 f.actions[0].text="Bem-vindo, $user! Chegou em $channel.".into();
 f.actions.push(Action{kind:"memory".into(),text:"{{user}} entrou".into(),target:"usuarios/$user.md".into(),value:0,condition:"$user".into(),..Default::default()});
 f.actions.push(Action{kind:"script".into(),text:"\"$user\"".into(),target:String::new(),value:0,condition:String::new(),..Default::default()});
 {let db=db::Db::open(&path).unwrap();db.save_profile(&p).unwrap();db.save_flow(&f).unwrap();
  let got=db.flows(&p.id).unwrap();
  assert_eq!(got[0].actions[0].text,"Bem-vindo, {{user}}! Chegou em {{channel}}.");
  assert_eq!(got[0].actions[3].target,"usuarios/{{user}}.md");
  assert_eq!(got[0].actions[3].condition,"$user");
  assert_eq!(got[0].actions[4].text,"\"$user\"");
  assert!(db.logs(&p.id).unwrap().iter().any(|l|l.kind=="variables"));}
 assert_eq!(variables::migrate("$user.arg0"),"{{user}}.arg0");
 assert_eq!(variables::migrate("$arg12"),"{{arg12}}");
 assert_eq!(variables::migrate("$argumento $arg"),"$argumento $arg");
}
#[test]
fn variable_persistence_session_user_and_profile_isolation(){
 let dir=tempfile::tempdir().unwrap();let path=dir.path().join("vars.sqlite");let a=profile("A");let b=profile("B");
 {let db=db::Db::open(&path).unwrap();db.save_profile(&a).unwrap();db.save_profile(&b).unwrap();
 for key in ["global.count","user.count","session.count","sessionUser.count"] {variables::mutate(&db,&a.id,"twitch","ana",key,"increment",json!(2)).unwrap();}
 assert!(variables::mutate(&db,&a.id,"twitch","","user.count","set",json!(2)).is_err());
 assert_eq!(variables::list(&db,&a.id,"twitch:ana").unwrap().as_array().unwrap().len(),4);
 assert_eq!(variables::list(&db,&a.id,"twitch:bia").unwrap().as_array().unwrap().len(),2);
 assert_eq!(variables::list(&db,&a.id,"youtube:ana").unwrap().as_array().unwrap().len(),2);
 assert!(variables::list(&db,&b.id,"twitch:ana").unwrap().as_array().unwrap().is_empty());}
 let db=db::Db::open(&path).unwrap();assert_eq!(variables::list(&db,&a.id,"twitch:ana").unwrap().as_array().unwrap().len(),2);
 db.delete_profile(&a.id).unwrap();let count:i64=db.0.lock().unwrap().query_row("SELECT COUNT(*) FROM variables",[],|r|r.get(0)).unwrap();assert_eq!(count,0);
}
#[test]
fn variable_increments_are_atomic_and_invalid_changes_do_not_write(){
 let db=std::sync::Arc::new(db::Db::open(std::path::Path::new(":memory:")).unwrap());let p=profile("P");db.save_profile(&p).unwrap();
 let threads:Vec<_>=(0..8).map(|_|{let db=db.clone();let p=p.clone();std::thread::spawn(move||{for _ in 0..50{variables::mutate(&db,&p.id,"twitch","","global.total","increment",json!(1)).unwrap();}})}).collect();for t in threads{t.join().unwrap();}
 assert_eq!(variables::list(&db,&p.id,"").unwrap()[0]["value"],400);
 assert!(variables::mutate(&db,&p.id,"twitch","","global.total","increment",json!("oops")).is_err());
 assert_eq!(variables::list(&db,&p.id,"").unwrap()[0]["value"],400);
 variables::mutate(&db,&p.id,"twitch","","global.total","set",json!(i64::MAX)).unwrap();
 assert!(variables::mutate(&db,&p.id,"twitch","","global.total","increment",json!(1)).is_err());
 assert!(variables::mutate(&db,&p.id,"twitch","","global.bad.name","set",json!(1)).is_err());
}
#[tokio::test]
async fn variable_sequence_simulation_preview_and_permissions(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let mut p=profile("P");p.editors=vec!["mod".into()];rt.db.save_profile(&p).unwrap();let other=profile("other");rt.db.save_profile(&other).unwrap();
 variables::mutate(&rt.db,&p.id,"twitch","","global.count","set",json!(10)).unwrap();
 let mut f=flow(&p);f.actions=vec![Action{kind:"variable.increment".into(),text:"2".into(),target:"global.count".into(),value:0,condition:"".into(),..Default::default()},Action{kind:"variable.set".into(),text:"{{user|upper}}".into(),target:"local.name".into(),value:0,condition:"".into(),..Default::default()},Action{kind:"chat".into(),text:"{{local.name}}: {{global.count}}".into(),target:"".into(),value:0,condition:"".into(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();engine::process(rt.clone(),event(&p,"!oi",true)).await;
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message=="[Simulação] ANA: 12"));assert_eq!(variables::list(&rt.db,&p.id,"").unwrap()[0]["value"],10);
 let preview=dispatch(rt.clone(),"variables.preview",json!({"profileId":p.id,"flow":f,"event":event(&p,"!oi",false)})).await.unwrap();assert_eq!(preview["steps"][2]["text"],"ANA: 12");assert_eq!(variables::list(&rt.db,&p.id,"").unwrap()[0]["value"],10);
 let fresh=variables::Context::new(&rt.db,&p,&event(&p,"!oi",true),None).unwrap();assert!(fresh.render("{{local.name}}").is_err());
 // Real execution with a local overlay exercises persistence without external chat.
 f.actions[2].kind="overlay".into();rt.db.save_flow(&f).unwrap();let mut events=rt.broadcast.subscribe();
 engine::process(rt.clone(),event(&p,"!oi",false)).await;
 assert_eq!(variables::list(&rt.db,&p.id,"").unwrap()[0]["value"],12);
 let mut found=false;while let Ok(v)=events.try_recv(){if v["type"]=="overlay"{assert_eq!(v["payload"]["text"],"ANA: 12");found=true}}assert!(found);
 *rt.actor.lock().unwrap()="mod".into();assert!(dispatch(rt.clone(),"variables.list",json!({"profileId":other.id})).await.is_err());assert!(dispatch(rt.clone(),"variables.change",json!({"profileId":p.id,"target":"global.count","value":8})).await.is_ok());
 *rt.actor.lock().unwrap()="locked".into();assert!(dispatch(rt.clone(),"variables.preview",json!({"profileId":p.id})).await.is_err());
}
#[tokio::test]
async fn timer_preview_delivers_the_real_event_and_sorts_a_chatter(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Prévia do timer");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="Minecraft".into();f.trigger.kind="timer".into();f.trigger.pattern.clear();f.trigger.cooldown=0;f.trigger.user_cooldown=0;f.actions=vec![Action{kind:"chat".into(),text:"{{user}}|{{userId}}|{{eventType}}".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 let ev=event(&p,"!minecraft Ana",true);
 // Sem ninguém no cadastro o sorteio não existe: a ação interrompe e orienta o uso do padrão.
 let ctx=variables::Context::new(&rt.db,&p,&ev,Some(&f)).unwrap();
 assert!(ctx.render("{{randomViewer}}").is_err());
 assert_eq!(ctx.render("{{randomViewer|default:alguém}}").unwrap(),"alguém");
 // Quem fala no chat entra no cadastro e passa a ser sorteável.
 engine::process(rt.clone(),event(&p,"Olá a todos",false)).await;
 let ctx=variables::Context::new(&rt.db,&p,&ev,Some(&f)).unwrap();
 assert_eq!(ctx.render("{{randomViewer}}").unwrap(),"Ana");
 // A prévia entrega o disparo real do timer, e não os dados de teste do chat.
 let preview=dispatch(rt.clone(),"variables.preview",json!({"profileId":p.id,"flow":f,"event":ev})).await.unwrap();
 assert_eq!(preview["steps"][0]["text"],"BotLive||timer");
 assert_eq!(preview["variables"]["user"],"BotLive");
 assert_eq!(preview["variables"]["userId"],"");
}
#[tokio::test]
async fn only_one_automation_publishes_when_several_match(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Uma resposta");rt.db.save_profile(&p).unwrap();
 let mut comando=flow(&p);comando.name="Comando".into();comando.actions=vec![Action{kind:"overlay".into(),text:"efeito paralelo".into(),target:String::new(),value:0,condition:String::new(),..Default::default()},Action{kind:"chat".into(),text:"resposta do comando".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 let mut contem=flow(&p);contem.name="Contém".into();contem.id=uuid::Uuid::new_v4().to_string();contem.trigger.kind="contains".into();contem.trigger.pattern="oi".into();contem.trigger.cooldown=0;contem.trigger.user_cooldown=0;
 contem.actions=vec![Action{kind:"chat".into(),text:"resposta do contém".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&contem).unwrap();rt.db.save_flow(&comando).unwrap();
 let mut rx=rt.broadcast.subscribe();
 engine::process(rt.clone(),event(&p,"!oi tudo bem",false)).await;
 let events:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
 assert!(events.iter().any(|v|v["type"]=="overlay"&&v["payload"]["text"]=="efeito paralelo"),"o efeito paralelo roda nos dois lados");
 let msgs:Vec<String>=rt.db.logs(&p.id).unwrap().into_iter().map(|l|l.message).collect();
 assert!(msgs.iter().any(|m|m=="Iniciando Comando"),"o vencedor roda");
 assert!(msgs.iter().any(|m|m=="Iniciando Contém"),"o outro fluxo também roda");
 assert!(msgs.iter().any(|m|m.contains("Contém · Outra automação já respondeu esta mensagem")),"o Histórico registra o que ficou de fora");
 assert!(msgs.iter().any(|m|m.starts_with("Comando · ")),"é o comando que tenta publicar");
 assert!(msgs.iter().all(|m|!m.starts_with("Contém ·")||m.starts_with("Contém · Outra automação")),"a resposta do perdedor nunca é publicada");
}
#[tokio::test]
async fn punish_action_is_previewed_validated_and_refused_without_a_twitch_account(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Punição");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="Silenciar".into();f.trigger.pattern="!silenciar".into();
 f.actions=vec![Action{kind:"punish".into(),text:"Regras da live".into(),target:"sender".into(),value:60,condition:String::new(),punish:"timeout".into(),..Default::default()}];
 dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":f.clone()})).await.unwrap();
 // A prévia mostra o plano e não pune ninguém.
 engine::process(rt.clone(),event(&p,"!silenciar @alguem",true)).await;
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message=="Executaria punish: Regras da live"&&l.status=="success"));
 // Quem não tem conta da Twitch no evento é recusado antes de qualquer chamada externa.
 engine::process(rt.clone(),event(&p,"!silenciar",false)).await;
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.status=="error"&&l.message.contains("não traz uma conta da Twitch")));
 // Perfil fora da Twitch também é recusado.
 let mut yt=profile("Outra plataforma");yt.platform="youtube".into();rt.db.save_profile(&yt).unwrap();
 let mut g=f.clone();g.profile_id=yt.id.clone();g.id=uuid::Uuid::new_v4().to_string();rt.db.save_flow(&g).unwrap();
 let mut ev=event(&yt,"!silenciar",false);ev.user_id="12345678".into();
 engine::process(rt.clone(),ev).await;
 assert!(rt.db.logs(&yt.id).unwrap().iter().any(|l|l.status=="error"&&l.message.contains("A punição usa a Twitch")));
 // Entrada inválida não passa.
 let mut modo=f.clone();modo.actions[0].punish="mute".into();
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":modo})).await.is_err());
 let mut alvo=f.clone();alvo.actions[0].target="qualquer".into();
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":alvo})).await.is_err());
 let mut duracao=f.clone();duracao.actions[0].value=0;
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":duracao})).await.is_err());
 // Alvo pelo primeiro argumento e aviso sem duração passam.
 let mut primeiro=f.clone();primeiro.actions[0].target="first".into();primeiro.actions[0].punish="warn".into();primeiro.actions[0].value=0;
 assert!(dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":primeiro})).await.is_ok());
}
#[tokio::test]
async fn backup_is_owner_only_and_command_count_turns_on_when_it_is_used(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let p=profile("Backup");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="ifood".into();f.trigger.pattern="!ifood".into();f.counter=false;
 f.actions=vec![Action{kind:"chat".into(),text:"{{commandCount}}º pedido".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 dispatch(rt.clone(),"flow.save",json!({"profileId":p.id,"flow":f.clone()})).await.unwrap();
 assert!(rt.db.flows(&p.id).unwrap().remove(0).counter,"quem usa o contador é ligado na gravação");
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.message.contains("Contagem ligada em \"ifood\"")));
 assert!(dispatch(rt.clone(),"backup.save",json!({"config":json!({"days":0})})).await.is_err(),"retenção fora do intervalo não passa");
 let bk=dir.path().join("backups");std::fs::create_dir_all(&bk).unwrap();
 let folder=bk.to_string_lossy().into_owned();
 dispatch(rt.clone(),"backup.save",json!({"config":json!({"folder":folder,"auto":true,"days":7,"weeks":4,"months":12})})).await.unwrap();
 let feito=dispatch(rt.clone(),"backup.now",json!({})).await.unwrap();
 assert!(std::path::Path::new(feito["path"].as_str().unwrap()).is_file());
 assert_eq!(dispatch(rt.clone(),"backup.list",json!({})).await.unwrap().as_array().map(|v|v.len()),Some(1));
 assert!(dispatch(rt.clone(),"backup.save",json!({"config":json!({"folder":""})})).await.is_err(),"automático exige pasta");
 *rt.actor.lock().unwrap()="mod".into();
 assert!(dispatch(rt.clone(),"backup.now",json!({})).await.is_err(),"só o proprietário faz backup");
 assert!(dispatch(rt.clone(),"backup.get",json!({})).await.is_err());
 assert!(dispatch(rt.clone(),"backup.restore",json!({"path":"qualquer"})).await.is_err());
 *rt.actor.lock().unwrap()="owner".into();
 let estranho=dir.path().join("estranho.botlivebak");std::fs::write(&estranho,"{\"format\":\"de outra coisa\"}").unwrap();
 assert!(dispatch(rt.clone(),"backup.restore",json!({"path":estranho.to_string_lossy().to_string()})).await.is_err());
}
