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
 let preview=variables::Context::new(&rt.db,&other,&event(&other,"!oi",true),Some(&f),None).unwrap();assert_eq!(preview.render("{{commandCount}}").unwrap(),"0");
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
 let sounds:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="audio.play").collect();
 assert_eq!(sounds.len(),1);
 assert!(sounds[0]["payload"]["src"].as_str().unwrap().contains(asset["id"].as_str().unwrap()),"som do fluxo pela saída única");
 assert_eq!(sounds[0]["payload"]["volume"].as_f64().unwrap(),0.6);
 // Sem autorização a ação falha com aviso no histórico, sem interromper o processo.
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.status=="error"&&l.message.starts_with("ifood · ")));
 // A simulação registra o plano: sem som e com o tipo de envio escolhido.
 engine::process(rt.clone(),event(&p,"!oi",true)).await;
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="audio.play"));
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
 assert!(variables::Context::new(&rt.db,&p,&event(&p,"oi",false),None,None).unwrap().render("{{local.aiResponse}}").is_err());
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
fn event(p:&Profile,text:&str,simulated:bool)->Event{Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),kind:"chat".into(),user:"Ana".into(),user_id:"ana".into(),role:"everyone".into(),message:text.into(),data:Value::Null,simulated,rehearsal:false}}
fn flow(p:&Profile)->Flow{Flow{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),name:"Teste".into(),trigger:Trigger{kind:"command".into(),pattern:"!oi".into(),cooldown:5,user_cooldown:5,..Default::default()},actions:vec!["Um $user","Dois","Três"].into_iter().map(|s|Action{kind:"chat".into(),text:s.into(),target:"".into(),value:0,condition:"".into(),..Default::default()}).collect(),..Default::default()}}
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
async fn porta_local_atende_o_overlay_em_http_e_chave_do_overlay_e_somente_leitura(){
 use futures_util::{SinkExt,StreamExt};use tokio_tungstenite::{connect_async,tungstenite::Message};
 use tokio::io::{AsyncReadExt,AsyncWriteExt};
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let probe=std::net::TcpListener::bind("127.0.0.1:0").unwrap();let port=probe.local_addr().unwrap().port();drop(probe);rt.db.set("apiPort",&json!(port)).unwrap();
 let p=profile("P");rt.db.save_profile(&p).unwrap();
 let info=livepix::overlay_info(&rt,&p.id).unwrap();let token=info["token"].as_str().unwrap().to_owned();
 livepix::operation(&rt,&p.id,"livepix.design_save",&json!({"design":{"layout":"ring","width":55.0,"fillStart":"#ff0000"}})).await.unwrap();
 let server=tokio::spawn(local_api::serve(rt.clone()));
 for _ in 0..50{if *rt.api_port.lock().unwrap()!=0{break}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}

 // GET /overlay devolve o HTML gerado do código compartilhado
 let mut s=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
 s.write_all(b"GET /overlay HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").await.unwrap();
 let mut page=String::new();s.read_to_string(&mut page).await.unwrap();
 assert!(page.starts_with("HTTP/1.1 200"),"{page}");
 assert!(page.contains("bl-ov-root"),"o HTML servido é o overlay gerado");
 // caminho qualquer continua sem servir conteúdo
 let mut s=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
 s.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").await.unwrap();
 let mut other=String::new();s.read_to_string(&mut other).await.unwrap();
 assert!(other.starts_with("HTTP/1.1 404"),"{other}");

 // GET /asset devolve som registrado do perfil com a chave do overlay
 let wav=dir.path().join("jingle.wav");std::fs::write(&wav,b"RIFF0000WAVEfmt ").unwrap();
 let sid=crate::chat_extras::operation(&rt,&p.id,"chatExtras.import",&json!({"path":wav,"kind":"sound"})).unwrap()["id"].as_str().unwrap().to_owned();
 let mut s=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
 s.write_all(format!("GET /asset?token={token}&id={sid} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").as_bytes()).await.unwrap();
 let mut snd=String::new();s.read_to_string(&mut snd).await.unwrap();
 assert!(snd.starts_with("HTTP/1.1 200"),"{snd}");
 assert!(snd.contains("audio/wav"),"o som sai com o mime certo: {snd}");
 // chave errada não devolve som algum
 let mut s=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
 s.write_all(format!("GET /asset?token=outra&id={sid} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").as_bytes()).await.unwrap();
 let mut wrong=String::new();s.read_to_string(&mut wrong).await.unwrap();
 assert!(wrong.starts_with("HTTP/1.1 404"),"{wrong}");

 // WebSocket com a chave do overlay recebe design + meta e não submete eventos
 let (mut ws,_)=connect_async(format!("ws://127.0.0.1:{port}")).await.unwrap();
 ws.send(Message::Text(json!({"token":token}).to_string().into())).await.unwrap();
 let first=ws.next().await.unwrap().unwrap().into_text().unwrap();
 assert!(first.contains("design"),"primeira mensagem deve ser o design: {first}");
 assert!(first.contains("\"layout\": \"ring\"")||first.contains("\"layout\":\"ring\""),"{first}");
 let second=ws.next().await.unwrap().unwrap().into_text().unwrap();
 assert!(second.contains("goal"),"depois do design vem o estado da meta: {second}");
 assert!(second.contains("hidden"),"integração ainda desligada: a meta chega oculta: {second}");
 let third=ws.next().await.unwrap().unwrap().into_text().unwrap();
 assert!(third.contains("alerts"),"depois da meta vêm os alertas: {third}");
 let fourth=ws.next().await.unwrap().unwrap().into_text().unwrap();
 assert!(fourth.contains("chat.config"),"{fourth}");
 let fifth=ws.next().await.unwrap().unwrap().into_text().unwrap();
 assert!(fifth.contains("spotify.config"),"a config da música chega no aperto de mão: {fifth}");
 let sixth=ws.next().await.unwrap().unwrap().into_text().unwrap();
 assert!(sixth.contains("spotify.now"),"a faixa atual chega junto, antes do primeiro poll: {sixth}");
 // A camada de música só recebe faixa do próprio perfil.
 let other=profile("Outro");rt.db.save_profile(&other).unwrap();
 rt.emit("spotify.now",json!({"profileId":other.id,"playing":true}));
 let alien=tokio::time::timeout(std::time::Duration::from_millis(400),ws.next()).await;
 if let Ok(Some(Ok(msg)))=alien{
  let text=msg.to_text().unwrap_or_default();
  assert!(!text.contains("spotify.now"),"música de outro perfil não passa: {text}");
 }
 rt.emit("spotify.now",crate::spotify::test_payload(&p.id));
 let mine=tokio::time::timeout(std::time::Duration::from_millis(2000),ws.next()).await.unwrap().unwrap().unwrap().into_text().unwrap();
 assert!(mine.contains("spotify.now")&&mine.contains("rehearsal"),"a música deste perfil passa com a marca do ensaio: {mine}");
 ws.send(Message::Text(json!({"type":"event","event":event(&p,"invasao",true)}).to_string().into())).await.unwrap();
 let answer=tokio::time::timeout(std::time::Duration::from_millis(400),ws.next()).await;
 if let Ok(Some(Ok(msg)))=answer{
  let text=msg.to_text().unwrap_or_default();
  assert!(!text.contains("ack"),"chave do overlay não pode submeter eventos: {text}");
 }
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
 let got=dispatch(rt.clone(),"settings.get",json!({})).await.unwrap();
 assert_eq!(got["accent"],"#4466aa");assert_eq!(got["theme"],"light");
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
 let c=variables::Context::new(&rt.db,&p,&e,Some(&flow(&p)),None).unwrap();
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
 let fresh=variables::Context::new(&rt.db,&p,&event(&p,"!oi",true),None,None).unwrap();assert!(fresh.render("{{local.name}}").is_err());
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
 let ctx=variables::Context::new(&rt.db,&p,&ev,Some(&f),None).unwrap();
 assert!(ctx.render("{{randomViewer}}").is_err());
 assert_eq!(ctx.render("{{randomViewer|default:alguém}}").unwrap(),"alguém");
 // Quem fala no chat entra no cadastro e passa a ser sorteável.
 engine::process(rt.clone(),event(&p,"Olá a todos",false)).await;
 let ctx=variables::Context::new(&rt.db,&p,&ev,Some(&f),None).unwrap();
 assert_eq!(ctx.render("{{randomViewer}}").unwrap(),"Ana");
 // A prévia entrega o disparo real do timer, e não os dados de teste do chat.
 let preview=dispatch(rt.clone(),"variables.preview",json!({"profileId":p.id,"flow":f,"event":ev})).await.unwrap();
 assert_eq!(preview["steps"][0]["text"],"BotLive||timer");
 assert_eq!(preview["variables"]["user"],"BotLive");
 assert_eq!(preview["variables"]["userId"],"");
}
#[tokio::test]
async fn random_viewer_draws_fresh_on_every_occurrence(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Sorteio");rt.db.save_profile(&p).unwrap();
 rt.db.set_module(&p.id,"community",&json!({"names":{"a":"Ana","b":"Bia"}})).unwrap();
 let ctx=variables::Context::new(&rt.db,&p,&event(&p,"!x",true),None,None).unwrap();
 // Um único contexto, 60 mensagens com duas ocorrências cada: o nome não pode ficar congelado.
 let mut seen=std::collections::HashSet::new();
 for _ in 0..60{let out=ctx.render("{{randomViewer}} contra {{randomViewer}}").unwrap();for part in out.split(" contra "){assert!(["Ana","Bia"].contains(&part),"sorteio fora do cadastro: {part}");seen.insert(part.to_owned());}}
 assert_eq!(seen.len(),2,"um único Context sorteou 120 vezes sem variar o nome");
}
#[tokio::test]
async fn condition_false_stops_and_skip_jumps_one(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Condição");rt.db.save_profile(&p).unwrap();
 let cond=|var:&str,op:&str,value:&str,on_false:&str|Action{kind:"condition".into(),text:String::new(),target:String::new(),value:0,condition:String::new(),cond_var:var.into(),cond_op:op.into(),cond_value:value.into(),cond_false:on_false.into(),..Default::default()};
 let say=|t:&str|Action{kind:"overlay".into(),text:t.into(),target:String::new(),value:0,condition:String::new(),..Default::default()};
 let mut f=flow(&p);f.name="Para".into();f.trigger.pattern="!para".into();f.actions=vec![cond("reward.cost","greater_or_equal","5000","stop"),say("alto")];rt.db.save_flow(&f).unwrap();
 let mut g=flow(&p);g.name="Pula".into();g.trigger.pattern="!pula".into();g.actions=vec![cond("reward.cost","greater_or_equal","5000","skip"),say("pulado"),say("fim")];rt.db.save_flow(&g).unwrap();
 let mut rx=rt.broadcast.subscribe();
 engine::process(rt.clone(),event(&p,"!para",false)).await;
 engine::process(rt.clone(),event(&p,"!pula",false)).await;
 let texts:Vec<String>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="overlay").map(|v|v["payload"]["text"].as_str().unwrap_or("").into()).collect();
 assert_eq!(texts,vec!["fim"],"stop barrou o alto; skip pulou só o pulado");
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.message.contains("STOPPED_BY_CONDITION")),"parada aparece com identidade");
 assert!(logs.iter().any(|l|l.message.contains("→ falso")),"falso mostra resolvido e esperado");
 assert!(rt.runs.list(&p.id).iter().any(|r|r.status=="STOPPED_BY_CONDITION"));
}
#[tokio::test]
async fn duplicate_events_submit_once_and_run_once(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Duplicado");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="Eco".into();f.trigger.pattern="!eco".into();
 f.actions=vec![Action{kind:"overlay".into(),text:"eco".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();
 let e=event(&p,"!eco",false);
 assert!(rt.submit(e.clone()).await.is_ok());
 assert!(rt.submit(e.clone()).await.is_ok(),"duplicata entra como ok, mas a fila ignora");
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(5);
 loop{
  let got:Vec<String>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="overlay").map(|v|v["payload"]["text"].as_str().unwrap_or("").into()).collect();
  if !got.is_empty(){assert_eq!(got,vec!["eco"],"um evento, uma execução");break}
  assert!(tokio::time::Instant::now()<deadline,"o worker não entregou");
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
 }
 tokio::time::sleep(std::time::Duration::from_millis(500)).await;
 assert!(std::iter::from_fn(||rx.try_recv().ok()).all(|v|v["type"]!="overlay"),"nada duplicado depois");
}
#[tokio::test]
async fn disabled_steps_skip_and_wait_runs_without_blocking(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Sequência");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="Flash".into();f.trigger.pattern="!flash".into();
 let mut off=Action{kind:"overlay".into(),text:"nunca".into(),target:String::new(),value:0,condition:String::new(),..Default::default()};
 off.enabled=false;
 let wait=Action{kind:"wait".into(),text:String::new(),target:String::new(),value:5,condition:String::new(),..Default::default()};
 let go=Action{kind:"overlay".into(),text:"fim".into(),target:String::new(),value:0,condition:String::new(),..Default::default()};
 f.actions=vec![off,wait,go];rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();
 engine::process(rt.clone(),event(&p,"!flash",false)).await;
 let texts:Vec<String>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="overlay").map(|v|v["payload"]["text"].as_str().unwrap_or("").into()).collect();
 assert_eq!(texts,vec!["fim"],"só a etapa ligada entrega, depois da espera");
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.message.contains("pulada")),"etapa desligada registra SKIPPED");
 assert!(logs.iter().any(|l|l.message.contains("COMPLETED")),"execução concluída com identidade");
 assert!(rt.runs.list(&p.id).iter().any(|r|r.flow_id==f.id&&r.status=="COMPLETED"));
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
/// Servidor local de transcrição: devolve o primeiro cabeçalho recebido e o tamanho do áudio.
async fn mock_transcribe(text:&str)->(String,tokio::task::JoinHandle<Value>){
 use tokio::io::{AsyncReadExt,AsyncWriteExt};
 let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();let text=text.to_owned();
 let task=tokio::spawn(async move{
  let(mut socket,_)=listener.accept().await.unwrap();let mut bytes=vec![];
  let request=loop{
   let mut part=[0u8;4096];let n=socket.read(&mut part).await.unwrap();assert!(n>0);bytes.extend_from_slice(&part[..n]);assert!(bytes.len()<2000000);
   if let Some(end)=bytes.windows(4).position(|w|w==b"\r\n\r\n"){
    let headers=String::from_utf8_lossy(&bytes[..end]).to_lowercase();
    let len:usize=headers.lines().find_map(|l|l.strip_prefix("content-length:")).unwrap().trim().parse().unwrap();
    if bytes.len()>=end+4+len{break json!({"request":String::from_utf8_lossy(&bytes[..end]).lines().next().unwrap_or("").to_owned(),"body":len})}
   }
  };
  let body=json!({"text":text}).to_string();
  socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
  request
 });
 (format!("http://{address}/transcribe-pcm16"),task)
}
/// 16 kHz mono em PCM16: 16 amostras por milissegundo.
fn speech_pcm(ms:u64,amplitude:f64)->Vec<u8>{
 let samples=(16*ms) as usize;
 (0..samples).flat_map(|i|{let wave=(i as f64*0.4).sin()*amplitude;((wave*32767.0) as i16).to_le_bytes()}).collect()
}
fn speech_b64(bytes:&[u8])->String{use base64::Engine;base64::engine::general_purpose::STANDARD.encode(bytes)}
async fn send_audio(rt:&Arc<Runtime>,profile:&str,ms:u64,amplitude:f64)->Result<Value,String>{
 dispatch(rt.clone(),"voice.frame",json!({"profileId":profile,"pcm":speech_b64(&speech_pcm(ms,amplitude))})).await
}
/// Espera um evento do overlay chegar dentro do prazo.
async fn heard<F:Fn(&Value)->bool>(rx:&mut tokio::sync::broadcast::Receiver<Value>,ms:u64,ok:F)->bool{
 let deadline=std::time::Instant::now()+std::time::Duration::from_millis(ms);
 loop{
  while let Ok(value)=rx.try_recv(){if ok(&value){return true}}
  if std::time::Instant::now()>=deadline{return false}
  tokio::time::sleep(std::time::Duration::from_millis(25)).await;
 }
}
/// Fala de 400 ms seguida dos 200 ms de silêncio que a encerram (silenceMs do painel).
async fn speaking(rt:&Arc<Runtime>,profile:&str){
 for _ in 0..4{send_audio(rt,profile,100,0.5).await.unwrap();}
 tokio::time::sleep(std::time::Duration::from_millis(220)).await;
 send_audio(rt,profile,100,0.0).await.unwrap();
}
#[tokio::test]
async fn continuous_listening_transcribes_calls_the_bot_and_feeds_context(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut p=profile("Escuta");p.modules=json!({"voice":true});rt.db.save_profile(&p).unwrap();
 let(endpoint,answered)=mock_transcribe("arroba, vamos de ranked agora").await;
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"voice","value":json!({"listen":true,"listenEndpoint":endpoint,"language":"pt","activation":"arroba","captions":true,"silenceMs":200,"threshold":0.02})})).await.unwrap();
 let mut f=flow(&p);f.name="Chamada por voz".into();f.trigger.kind="voice".into();f.trigger.pattern="arroba".into();f.trigger.cooldown=0;f.trigger.user_cooldown=0;
 f.actions=vec![Action{kind:"overlay".into(),text:"{{lastSpeech}}".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();
 speaking(&rt,&p.id).await;
 let request=answered.await.unwrap();
 assert!(request["request"].as_str().unwrap().starts_with("POST /transcribe-pcm16"),"o áudio vai para o caminho do RealtimeSTT: {}",request["request"]);
 assert!(request["request"].as_str().unwrap().contains("language=pt"),"o idioma escolhido vai junto: {}",request["request"]);
 assert_eq!(request["body"].as_u64(),Some(16000),"500 ms de PCM16 mono é 16000 bytes, sem cabeçalho WAV");
 assert!(heard(&mut rx,5000,|v|v["type"]=="captions").await,"a fala vira legenda para o overlay");
 assert_eq!(speech::recent(&rt.db,&p.id),vec!["arroba, vamos de ranked agora"]);
 assert!(heard(&mut rx,5000,|v|v["type"]=="overlay"&&v["payload"]["text"]=="arroba, vamos de ranked agora").await,"chamada pelo nome dispara o fluxo com {{lastSpeech}} pronto");
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.kind=="voice"&&l.message.contains("streamer: arroba")),"a fala aparece no Histórico");
 let status=dispatch(rt.clone(),"voice.status",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(status["heard"].as_u64(),Some(1),"o painel conta uma fala reconhecida");
 assert!(status["listening"]==true,"o painel enxerga a escuta em andamento");
 assert!(speech::summary(&rt.db,&p.id).contains("Últimas falas do streamer"));
}
#[tokio::test]
async fn without_the_activation_words_a_speech_is_only_recorded(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut p=profile("Resenha");p.modules=json!({"voice":true});rt.db.save_profile(&p).unwrap();
 let(endpoint,answered)=mock_transcribe("vou pegar água").await;
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"voice","value":json!({"listen":true,"listenEndpoint":endpoint,"activation":"arroba","silenceMs":200})})).await.unwrap();
 let mut f=flow(&p);f.name="Chamada por voz".into();f.trigger.kind="voice".into();f.trigger.pattern="arroba".into();f.trigger.cooldown=0;f.trigger.user_cooldown=0;
 f.actions=vec![Action{kind:"overlay".into(),text:"não pode rodar".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();
 speaking(&rt,&p.id).await;
 answered.await.unwrap();
 assert!(heard(&mut rx,5000,|v|v["type"]=="captions").await,"sem ser chamada a fala ainda vira legenda e contexto");
 assert_eq!(speech::recent(&rt.db,&p.id),vec!["vou pegar água"]);
 tokio::time::sleep(std::time::Duration::from_millis(400)).await;
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="overlay"),"sem a palavra de ativação nenhum fluxo roda");
 assert!(rt.db.logs(&p.id).unwrap().iter().any(|l|l.kind=="voice"&&l.message=="Transcrição: vou pegar água"),"o que foi dito fica no Histórico mesmo sem disparar");
 let status=dispatch(rt.clone(),"voice.status",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(status["heard"].as_u64(),Some(1));
}
#[tokio::test]
async fn the_ear_stays_local_and_respects_module_and_switch(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut off=profile("Sem módulo");off.modules=json!({});rt.db.save_profile(&off).unwrap();
 let err=dispatch(rt.clone(),"voice.frame",json!({"profileId":off.id,"pcm":speech_b64(&speech_pcm(100,0.5))})).await.unwrap_err();
 assert!(err.contains("Ative o Controle por voz"),"módulo desligado não escuta: {err}");
 let mut p=profile("Voz");p.modules=json!({"voice":true});rt.db.save_profile(&p).unwrap();
 let err=dispatch(rt.clone(),"voice.frame",json!({"profileId":p.id,"pcm":speech_b64(&speech_pcm(100,0.5))})).await.unwrap_err();
 assert!(err.contains("desligada"),"interruptor desligado não escuta: {err}");
 let remoto=dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"voice","value":json!({"listen":true,"listenEndpoint":"http://192.168.0.10:8010/transcribe-pcm16"})})).await.unwrap_err();
 assert!(remoto.contains("local"),"o painel recusa servidor fora da máquina: {remoto}");
 rt.db.set_module(&p.id,"voice",&json!({"listen":true,"listenEndpoint":"http://192.168.0.10:8010/transcribe-pcm16"})).unwrap();
 let err=dispatch(rt.clone(),"voice.frame",json!({"profileId":p.id,"pcm":speech_b64(&speech_pcm(100,0.5))})).await.unwrap_err();
 assert!(err.contains("localmente"),"registro gravado fora do painel também não manda áudio: {err}");
 rt.db.set_module(&p.id,"voice",&json!({"listen":true})).unwrap();
 let err=dispatch(rt.clone(),"voice.frame",json!({"profileId":p.id,"pcm":"não é base64!"})).await.unwrap_err();
 assert!(err.contains("inválido"),"áudio corrompido não passa: {err}");
 let err=dispatch(rt.clone(),"voice.frame",json!({"profileId":p.id,"pcm":speech_b64(&vec![0u8;70000])})).await.unwrap_err();
 assert!(err.contains("grande"),"lote grande é recusado: {err}");
 let status=dispatch(rt.clone(),"voice.frame",json!({"profileId":p.id,"pcm":speech_b64(&speech_pcm(100,0.0))})).await.unwrap();
 assert_eq!(status["frames"].as_u64(),Some(1),"o lote aceito é contado");
 assert!(status["enabled"]==true&&status["listening"]==true);
 assert_eq!(status["heard"].as_u64(),Some(0),"silêncio não vira fala");
 assert!(status["error"].as_str().unwrap_or("").is_empty());
 *rt.actor.lock().unwrap()="intruso".into();
 assert!(dispatch(rt,"voice.status",json!({"profileId":p.id})).await.is_err(),"quem não edita o perfil não enxerga a escuta");
}
/// Espera a execução chegar a um estado final devolvendo o registro do motor.
async fn finished_run(rt:&Arc<Runtime>,id:&str)->crate::runs::Run{
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(20);
 loop{
  if let Some(r)=rt.runs.get(id){
   if ["COMPLETED","FAILED","CANCELLED","STOPPED_BY_CONDITION"].contains(&r.status.as_str()){return r}
  }
  assert!(tokio::time::Instant::now()<deadline,"a execução {id} não chegou a um estado final");
  tokio::time::sleep(std::time::Duration::from_millis(20)).await;
 }
}
/// O id que a tela recebe é o que o motor executa: um registro só, sem gêmeo parado.
#[tokio::test]
async fn flow_test_returns_the_execution_it_actually_runs(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Teste de fluxo");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="Cumprimento".into();f.trigger.pattern="!oi".into();
 f.actions=vec![Action{kind:"overlay".into(),text:"oi".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let out=dispatch(rt.clone(),"flow.test",json!({"profileId":p.id,"flow":f.clone(),"testVars":{}})).await.unwrap();
 let id=out["executionId"].as_str().expect("flow.test devolve a execução").to_owned();
 let run=finished_run(&rt,&id).await;
 assert_eq!(run.id,id,"o id devolvido é o que o motor usou");
 assert_eq!(run.flow_id,f.id);
 assert!(run.test,"registro marcado como teste");
 assert_eq!(run.status,"COMPLETED");
 assert_eq!(run.steps[0].status,"SKIPPED","etapa de efeito externo fica pulada, não concluída");
 assert!(run.error.is_none());
 assert_eq!(rt.runs.list(&p.id).iter().filter(|r|r.flow_id==f.id).count(),1,"um flow.test, uma execução: nada de gêmeo parado em PENDING");
 let api=dispatch(rt.clone(),"flow.runs",json!({"profileId":p.id})).await.unwrap();
 let entry=api.as_array().unwrap().iter().find(|r|r["id"]==id).cloned().expect("a tela encontra a execução pelo id");
 assert_eq!(entry["error"],Value::Null);
 assert_eq!(entry["steps"][0]["status"].as_str(),Some("SKIPPED"));
}
/// A prévia só prova lógica: espera, condição e variável local rodam; nada de fora acontece.
#[tokio::test]
async fn flow_test_keeps_local_logic_and_never_touches_the_outside(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Teste sem efeitos");rt.db.save_profile(&p).unwrap();
 let act=|kind:&str,target:&str,text:&str|Action{kind:kind.into(),text:text.into(),target:target.into(),value:0,condition:String::new(),..Default::default()};
 let mut obs=act("obs","","mute");obs.obs_op="mute".into();obs.obs_target="Mic".into();
 let mut punish=act("punish","sender","motivo");punish.punish="timeout".into();punish.value=60;
 let mut wait=act("wait","","");wait.value=5;
 let mut delay=act("delay","","");delay.value=1;
 let mut cond=act("condition","","");cond.cond_var="reward.cost".into();cond.cond_op="greater_or_equal".into();cond.cond_value="0".into();cond.cond_false="stop".into();
 let mut f=flow(&p);f.name="Misto".into();f.trigger.pattern="!misto".into();
 f.actions=vec![
  act("chat","","Oi {{viewer.name}}"),act("overlay","","aparece na tela"),act("script","","1+1"),
  obs,punish,act("webhook","https://example.com/hook","{}"),act("memory","local.memoria","escreve"),
  act("points","","10"),act("variable.set","global.fora","nao pode"),act("variable.set","local.dentro","pode"),
  wait,delay,cond];
 rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();
 let out=dispatch(rt.clone(),"flow.test",json!({"profileId":p.id,"flow":f.clone(),"testVars":{}})).await.unwrap();
 let id=out["executionId"].as_str().unwrap().to_owned();
 let run=finished_run(&rt,&id).await;
 assert_eq!(run.status,"COMPLETED");
 let status=|i:usize|run.steps[i].status.clone();
 for i in 0..9{assert_eq!(status(i),"SKIPPED","etapa {i} não pode agir fora: {}",f.actions[i].kind)}
 assert_eq!(status(9),"SUCCESS","variável local roda de verdade");
 assert_eq!(status(10),"SUCCESS","espera roda de verdade");
 assert_eq!(status(11),"SUCCESS","atraso local roda de verdade");
 assert_eq!(status(12),"TRUE","condição avaliada de verdade");
 assert!(std::iter::from_fn(||rx.try_recv().ok()).all(|v|v["type"]!="overlay"),"nada foi publicado na tela");
 let after=variables::Context::new(&rt.db,&p,&event(&p,"!x",true),None,None).unwrap();
 assert_ne!(after.render("{{global.fora}}").ok().as_deref(),Some("nao pode"),"o teste não grava variável global");
}
/// A falha fica na etapa, com o motivo, e interrompe o resto da sequência.
#[tokio::test]
async fn flow_test_marks_the_failed_step_and_keeps_the_reason(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Teste com falha");rt.db.save_profile(&p).unwrap();
 let mut f=flow(&p);f.name="Quebra".into();f.trigger.pattern="!quebra".into();
 f.actions=vec![
  Action{kind:"variable.set".into(),text:"{{".into(),target:"local.quebra".into(),value:0,condition:String::new(),..Default::default()},
  Action{kind:"overlay".into(),text:"não roda".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let out=dispatch(rt.clone(),"flow.test",json!({"profileId":p.id,"flow":f.clone(),"testVars":{}})).await.unwrap();
 let id=out["executionId"].as_str().unwrap().to_owned();
 let run=finished_run(&rt,&id).await;
 assert_eq!(run.status,"FAILED");
 assert_eq!(run.steps[0].status,"FAILED");
 assert_eq!(run.error.as_deref(),Some("Variável sem fechamento }}"));
 assert_eq!(run.steps[1].status,"PENDING","a falha interrompe; o que vem depois não roda");
 let api=dispatch(rt.clone(),"flow.runs",json!({"profileId":p.id})).await.unwrap();
 let entry=api.as_array().unwrap().iter().find(|r|r["id"]==id).cloned().unwrap();
 assert_eq!(entry["error"].as_str(),Some("Variável sem fechamento }}"),"o motivo chega na API que a tela consulta");
 assert_eq!(entry["steps"][0]["status"].as_str(),Some("FAILED"));
}
/// Cancelar mira a execução devolvida e só termina quando o motor confirma.
#[tokio::test]
async fn flow_test_cancel_stops_the_execution_it_returned(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();let p=profile("Teste cancelado");rt.db.save_profile(&p).unwrap();
 let wait=Action{kind:"wait".into(),text:String::new(),target:String::new(),value:60000,condition:String::new(),..Default::default()};
 let mut f=flow(&p);f.name="Lento".into();f.trigger.pattern="!lento".into();
 f.actions=vec![wait,Action{kind:"overlay".into(),text:"fim".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let out=dispatch(rt.clone(),"flow.test",json!({"profileId":p.id,"flow":f.clone(),"testVars":{}})).await.unwrap();
 let id=out["executionId"].as_str().unwrap().to_owned();
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(20);
 loop{
  if rt.runs.get(&id).is_some_and(|r|r.steps.iter().any(|s|s.status=="RUNNING"||s.status=="WAITING")){break}
  assert!(tokio::time::Instant::now()<deadline,"a execução nem começou a rodar");
  tokio::time::sleep(std::time::Duration::from_millis(20)).await;
 }
 let mut rx=rt.broadcast.subscribe();
 dispatch(rt.clone(),"flow.cancel",json!({"profileId":p.id,"executionId":id})).await.expect("cancelar a execução em andamento");
 let run=finished_run(&rt,&id).await;
 assert_eq!(run.status,"CANCELLED","o estado final vem do motor, não do clique");
 assert!(run.steps.iter().any(|s|s.status=="CANCELLED"));
 let again=dispatch(rt.clone(),"flow.cancel",json!({"profileId":p.id,"executionId":id})).await;
 assert_eq!(again.unwrap_err(),"Execução não encontrada ou já terminada","já terminada não cancela de novo");
 tokio::time::sleep(std::time::Duration::from_millis(200)).await;
 assert!(std::iter::from_fn(||rx.try_recv().ok()).all(|v|v["type"]!="overlay"),"a etapa seguinte não roda depois do cancelamento");
}
#[tokio::test]
async fn the_ai_sees_what_the_streamer_just_said_on_the_mic(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut p=profile("Resenha ao vivo");p.modules=json!({});p.ai.model="test".into();
 let(endpoint,request)=mock_ai("Bora!").await;p.ai.endpoint=endpoint;rt.db.save_profile(&p).unwrap();
 speech::push(&rt.db,&p.id,"vamos de ranked sem stress").unwrap();
 let mut f=flow(&p);f.actions=vec![Action{kind:"ai.generate".into(),text:"Responda".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];rt.db.save_flow(&f).unwrap();
 engine::process(rt.clone(),event(&p,"!oi",false)).await;
 let request=request.await.unwrap();
 let system=request["messages"][0]["content"].as_str().unwrap();
 assert!(system.contains("[Últimas falas do streamer na live: vamos de ranked sem stress]"),"a IA lê o microfone: {system}");
 let(preview_endpoint,preview_request)=mock_ai("Beleza, estamos ao vivo").await;p.ai.endpoint=preview_endpoint;rt.db.save_profile(&p).unwrap();
 dispatch(rt.clone(),"ai.preview",json!({"profileId":p.id,"message":"e aí"})).await.unwrap();
 let preview_request=preview_request.await.unwrap();
 assert!(preview_request["messages"][0]["content"].as_str().unwrap().contains("vamos de ranked sem stress"),"a prévia do painel da IA também lê o microfone");
}

#[tokio::test]
async fn alertas_seguem_o_engine_e_o_painel_valida_a_configuracao(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Alertas");rt.db.save_profile(&p).unwrap();
 let mut rx=rt.broadcast.subscribe();

 // o salvamento do Estúdio valida faixa e tipos antes de gravar
 let mut cfg=crate::alerts::defaults();cfg["enabled"]=json!(true);
 let bad=dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"alerts","value":{"duration":2}})).await;
 assert!(bad.is_err(),"duração fora da faixa de 3 a 20 s deve barrar o salvamento");

 // sem config salva, o follow passa e nada toca
 let follow={let mut e=event(&p,"",false);e.kind="follow".into();e.user="Ana".into();e};
 engine::process(rt.clone(),follow.clone()).await;
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="alert"),"sem ligar os alertas nada toca");

 // ligando pelo painel, o evento real vira alerta com as variáveis dos textos
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"alerts","value":cfg})).await.unwrap();
 engine::process(rt.clone(),follow.clone()).await;
 let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="alert").collect();
 assert_eq!(hits.len(),1);
 assert_eq!(hits[0]["payload"]["kind"],"follow");
 assert_eq!(hits[0]["payload"]["vars"]["name"],"Ana");
 assert_eq!(hits[0]["payload"]["profileId"],p.id);

 // evento simulado também dispara — é o caminho de teste da live
 let mut sim=follow.clone();sim.simulated=true;
 engine::process(rt.clone(),sim).await;
 assert_eq!(std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="alert").count(),1);

 // cada salvamento retransmite a config nova ao overlay na hora
 let mut off=cfg.clone();off["enabled"]=json!(false);
 rt.db.set_module(&p.id,"alerts",&off).unwrap();
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"alerts","value":cfg})).await.unwrap();
 assert!(std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="alerts"),"o overlay recebe a config atualizada");
}

#[tokio::test]
async fn chat_no_overlay_segue_o_motor_e_a_delecao_sincroniza(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Chat");rt.db.save_profile(&p).unwrap();
 let mut rx=rt.broadcast.subscribe();

 // sem ligar o chat, a mensagem do espectador não vai ao overlay
 let msg={let mut e=event(&p,"boa noite, canal!",false);e.role="moderator".into();e.data=json!({"message_id":"msg-1","badges":[{"set_id":"moderator"}],"message":{"text":"boa noite, canal!","fragments":[{"type":"text","text":"boa noite, canal!"}]}});e};
 engine::process(rt.clone(),msg.clone()).await;
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="chat"),"chat desligado não transmite nada");

 // ligando pelo painel (mesmo caminho do Estúdio), a mensagem leva papel, badges e emotes
 let mut cfg=crate::chat_overlay::defaults();cfg["enabled"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"chatOverlay","value":cfg})).await.unwrap();
 let msg2={let mut m=msg.clone();m.message="valeu Kappa".into();m.data=json!({"message_id":"msg-1","badges":[{"set_id":"moderator"},{"set_id":"subscriber"}],"message":{"text":"valeu Kappa","fragments":[{"type":"text","text":"valeu "},{"type":"emote","text":"Kappa","emote":{"id":"25"}}]}});m};
 engine::process(rt.clone(),msg2.clone()).await;
 let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat").collect();
 assert_eq!(hits.len(),1);
 assert_eq!(hits[0]["payload"]["id"],"msg-1","o overlay remove depois pelo id da mensagem, não da notificação");
 assert_eq!(hits[0]["payload"]["role"],"moderator");
 assert_eq!(hits[0]["payload"]["badges"][1],"subscriber");
 assert_eq!(hits[0]["payload"]["frags"][1]["id"],"25","emote chega com id para a CDN");
 assert_eq!(hits[0]["payload"]["first"],true,"a primeira mensagem do espectador é marcada");
 assert_eq!(hits[0]["payload"]["bot"],false);

 // a mesma pessoa de novo não é primeira mensagem
 engine::process(rt.clone(),msg2.clone()).await;
 let again:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat").collect();
 assert_eq!(again[0]["payload"]["first"],false);

 // teste simulado (fluxo de teste) não vaza no chat do overlay
 let mut sim=msg2.clone();sim.simulated=true;
 engine::process(rt.clone(),sim).await;
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="chat"),"evento simulado não entra no chat do overlay");

 // resposta do bot também entra, com o nome configurado
 crate::chat_overlay::bot(&rt,&p,"Valeu, Ana!");
 let bots:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat").collect();
 assert_eq!(bots[0]["payload"]["bot"],true);
 assert_eq!(bots[0]["payload"]["user"],"BotLive");

 // apagar a mensagem na Twitch remove da fonte do OBS
 let del=platforms::normalize_twitch(&p,&json!({"metadata":{"subscription_type":"channel.chat.message_delete","message_id":"n1"},"payload":{"event":{"broadcaster_user_id":"123","target_user_id":"ana","target_user_name":"Ana","message_id":"msg-1"}}})).unwrap();
 assert_eq!(del.kind,"chat.delete");assert_eq!(del.message,"msg-1");assert_eq!(del.user,"Ana");
 engine::process(rt.clone(),del).await;
 assert!(std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="chat.delete"&&v["payload"]["id"]=="msg-1"));

 // limpar o chat zera a coluna
 let clear=platforms::normalize_twitch(&p,&json!({"metadata":{"subscription_type":"channel.chat.clear","message_id":"n2"},"payload":{"event":{"broadcaster_user_id":"123"}}})).unwrap();
 assert_eq!(clear.kind,"chat.clear");
 engine::process(rt.clone(),clear).await;
 assert!(std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="chat.clear"));
}

#[tokio::test]
async fn evento_simulado_fica_visivel_no_historico_mas_nao_entra_nas_estatisticas(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Estatísticas");rt.db.save_profile(&p).unwrap();

 // ensaio de follow e de chat: registra no histórico com status `ensaio`…
 let mut sim_follow=event(&p,"",false);sim_follow.kind="follow".into();sim_follow.simulated=true;
 engine::process(rt.clone(),sim_follow).await;
 engine::process(rt.clone(),event(&p,"mensagem de teste",true)).await;
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.kind=="follow"&&l.status=="ensaio"),"o ensaio fica visível no histórico");
 assert!(logs.iter().any(|l|l.kind=="chat"&&l.status=="ensaio"));
 // …mas nada conta no painel.
 let s=stats::read(&rt,&p.id).unwrap();
 assert_eq!(s["followers"],0,"seguidor de teste não conta");
 assert_eq!(s["messages"],0,"mensagem de teste não conta");

 // o evento real continua contando normalmente.
 let mut real_follow=event(&p,"",false);real_follow.kind="follow".into();
 engine::process(rt.clone(),real_follow).await;
 engine::process(rt.clone(),event(&p,"ola a todos",false)).await;
 let s=stats::read(&rt,&p.id).unwrap();
 assert_eq!(s["followers"],1);
 assert_eq!(s["messages"],1);
}

#[tokio::test]
async fn ensaio_nunca_vaza_para_o_discord(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Discord");rt.db.save_profile(&p).unwrap();
 rt.db.set_module(&p.id,"discordBot",&json!({
   "enabled":true,
   "notify":{"enabled":true,"channelId":"111","kinds":{"follow":{"enabled":true}}},
   "mirror":{"enabled":true,"toDiscord":true,"channelId":"222"}
 })).unwrap();

 // mensagem simulada com espelho ligado: suprimida e registrada, sem post agendado
 engine::process(rt.clone(),event(&p,"mensagem de teste",true)).await;
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.kind=="discord"&&l.status=="ensaio"&&l.message.contains("[Ensaio] Espelho do chat")));

 // follow simulado com aviso ligado: idem
 let mut sim_follow=event(&p,"",false);sim_follow.kind="follow".into();sim_follow.simulated=true;
 engine::process(rt.clone(),sim_follow).await;
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.kind=="discord"&&l.status=="ensaio"&&l.message.contains("[Ensaio] Aviso ao Discord")));
 assert!(!logs.iter().any(|l|l.kind=="discord"&&l.status=="error"),"nenhum post real tentado durante o ensaio");
}

#[tokio::test]
async fn ensaio_mexe_no_rotulo_e_simulacao_simples_nao(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Rótulos");rt.db.save_profile(&p).unwrap();
 rt.db.set_module(&p.id,"labels",&json!({"enabled":true,"folder":dir.path().join("rotulos").to_string_lossy(),"labels":{}})).unwrap();
 let mut sim=event(&p,"",false);sim.kind="cheer".into();sim.simulated=true;sim.user="Falso".into();sim.data=json!({"bits":500});

 // simulação simples da API (sem ensaio): o rótulo não muda
 engine::process(rt.clone(),sim.clone()).await;
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(1);
 loop{
  let d=rt.db.module(&p.id,"labels_data");
  assert_ne!(d["recent_cheer"].as_str(),Some("Falso"),"simulação simples da API não mexe no rótulo");
  if tokio::time::Instant::now()>=deadline{break}
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
 }

 // ensaio: o rótulo mostra quem "chegou", sem tocar os contadores da sessão
 sim.user="AnaBits".into();sim.rehearsal=true;
 engine::process(rt.clone(),sim.clone()).await;
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(5);
 loop{
  let d=rt.db.module(&p.id,"labels_data");
  if d["recent_cheer"].as_str()==Some("AnaBits"){break}
  assert!(tokio::time::Instant::now()<deadline,"o rótulo do ensaio não chegou");
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
 }
 let d=rt.db.module(&p.id,"labels_data");
 assert_eq!(d["recent_cheer_bits"],json!(500));
 assert_eq!(d["session_bits"].as_i64().unwrap_or(0),0,"bits de ensaio não entram na sessão");
}

#[tokio::test]
async fn execucao_de_fluxo_em_teste_nao_conta_comando(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Comandos");rt.db.save_profile(&p).unwrap();
 let f=flow(&p);rt.db.save_flow(&f).unwrap();

 // mesmo caminho do botão de teste do Estúdio (test=true)
 let done=engine::execute_flow(&rt,&p,&event(&p,"!oi",false),&f,true,&[],None,true,None).await;
 assert!(done);
 let s=stats::read(&rt,&p.id).unwrap();
 assert_eq!(s["commands"],json!([]),"execução de teste não vira estatística de comandos");

 // a execução real conta normalmente
 engine::process(rt.clone(),event(&p,"!oi",false)).await;
 let s=stats::read(&rt,&p.id).unwrap();
 let commands=s["commands"].as_array().unwrap();
 assert_eq!(commands.len(),1);
 assert_eq!(commands[0]["count"],1);
}

#[tokio::test] async fn match_do_chat_sorteia_o_par_e_guarda_os_intervalos(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut p=profile("Match");p.modules=json!({"affinity":true});rt.db.save_profile(&p).unwrap();
 // sem configuração salva valem os padrões: 2 minutos globais e 5 por pessoa.
 let mut b=event(&p,"oi tudo bem",false);b.user="Bruno".into();b.user_id="bruno".into();b.data=json!({"chatter_user_login":"bruno"});
 crate::twitch_ops::note_chatter(&rt,&b);modules::chat(&rt,&p,&b.clone()).unwrap();
 let a=event(&p,"você é meu amor!",false);
 let reply=modules::chat(&rt,&p,&a).unwrap().expect("palavra no chat com gente ativa deve casar");
 assert!(reply.contains("Ana")&&reply.contains("Bruno"),"{reply}");
 assert!(reply.contains('%'),"{reply}");
 // a mesma pessoa de novo cai no intervalo por pessoa
 assert!(modules::chat(&rt,&p,&event(&p,"amor é bom",false)).unwrap().is_none());
 // outra pessoa também não responde: o intervalo global cobre o chat inteiro
 let mut c=event(&p,"que casal",false);c.user="Cauã".into();c.user_id="caua".into();c.data=json!({"chatter_user_login":"caua"});
 crate::twitch_ops::note_chatter(&rt,&c);
 assert!(modules::chat(&rt,&p,&c).unwrap().is_none());
}

#[tokio::test] async fn match_do_chat_exige_duas_pessoas_e_comando_com_dois_nomes(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let mut p=profile("Match");p.modules=json!({"affinity":true});rt.db.save_profile(&p).unwrap();
 rt.db.set_module(&p.id,"affinity",&json!({"cooldown":0,"userCooldown":0})).unwrap();
 // sozinho no chat não há match a anunciar
 assert!(modules::chat(&rt,&p,&event(&p,"você é meu amor!",false)).unwrap().is_none());
 // o login da live entra no perfil como "Match" e não é sorteado por padrão
 let mut live=event(&p,"oi",false);live.user="Match".into();live.user_id="live".into();live.data=json!({"chatter_user_login":"match"});
 crate::twitch_ops::note_chatter(&rt,&live);
 assert!(modules::chat(&rt,&p,&event(&p,"que casal",false)).unwrap().is_none());
 // a menção vale quando a pessoa está no chat
 let mut b=event(&p,"oi",false);b.user="Bruno".into();b.user_id="bruno".into();b.data=json!({"chatter_user_login":"bruno"});
 crate::twitch_ops::note_chatter(&rt,&b);modules::chat(&rt,&p,&b.clone()).unwrap();
 let reply=modules::chat(&rt,&p,&event(&p,"amor @Bruno",false)).unwrap().unwrap();
 assert!(reply.contains("Ana x Bruno"),"{reply}");
 // !match com dois nomes usa a mesma conta, sem depender de quem está no chat
 let reply=modules::chat(&rt,&p,&event(&p,"!match carlos diana",false)).unwrap().unwrap();
 assert!(reply.contains("carlos x diana"),"{reply}");
 // uso errado vira registro no log, como nos outros comandos
 assert!(modules::chat(&rt,&p,&event(&p,"!match carlos",false)).is_err());
 assert!(modules::chat(&rt,&p,&event(&p,"!match carlos carlos",false)).is_err());
}

#[tokio::test] async fn explosao_de_emotes_conta_uso_e_dispara_no_evento(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Explosão");rt.db.save_profile(&p).unwrap();
 // sem lista de emotes, o teste avisa em vez de prometer efeito
 assert!(crate::emote_burst::test(&rt,&p).is_err());
 // a lista do canal chega pelo cache da IA, com os ids prontos para a CDN
 rt.db.set_module(&p.id,"ai_emotes",&json!({"at":chrono::Utc::now().timestamp(),"names":["Kappa","LUL"],"list":[{"id":"25","name":"Kappa"},{"id":"88","name":"LUL"}]})).unwrap();
 rt.db.set_module(&p.id,"emoteBurst",&json!({"enabled":true,"count":24,"size":48,"source":"top","topN":10,"alsoChat":true,"events":{"sub":true,"resub":true,"gift":true,"meta":true}})).unwrap();
 // a mensagem usa os mesmos fragmentos da Twitch que o chat do OBS
 let mut e=event(&p,"Kappa LUL Kappa",false);
 e.data=json!({"message":{"fragments":[{"type":"emote","text":"Kappa","emote":{"emote_id":"25"}},{"type":"emote","text":"LUL","emote":{"emote_id":"88"}},{"type":"text","text":" "},{"type":"emote","text":"Kappa","emote":{"emote_id":"25"}}]}});
 crate::emote_burst::count(&rt,&p.id,&e);
 assert_eq!(crate::emote_burst::totals(&rt,&p.id).get("25"),Some(&2),"Kappa apareceu duas vezes");
 assert_eq!(crate::emote_burst::totals(&rt,&p.id).get("88"),Some(&1));
 // o sub dispara a explosão com os mais usados na frente
 let mut sub=event(&p,"",false);sub.kind="subscription".into();
 let mut rx=rt.broadcast.subscribe();
 crate::emote_burst::from_event(&rt,&p,&sub);
 let msg=std::iter::from_fn(||rx.try_recv().ok()).find(|v|v["type"]=="emote.burst").expect("sub deve explodir");
 assert_eq!(msg["payload"]["via"],"sub");
 assert_eq!(msg["payload"]["size"],48);
 assert_eq!(msg["payload"]["alsoChat"],true);
 let ids:Vec<&str>=msg["payload"]["ids"].as_array().unwrap().iter().filter_map(|v|v.as_str()).collect();
 assert_eq!(ids.len(),24,"a quantidade pedida chega inteira");
 assert_eq!(ids[0],"25","o mais usado abre a explosão");
 assert_eq!(ids[1],"88");
 assert!(msg["payload"]["rehearsal"].is_null(),"evento real não leva a marca de ensaio");
 // o botão de teste da tela explode marcado como ensaio (selo ENSAIO no OBS)
 let mut rx=rt.broadcast.subscribe();
 crate::emote_burst::test(&rt,&p).expect("com emotes carregados o teste explode");
 let t=std::iter::from_fn(||rx.try_recv().ok()).find(|v|v["type"]=="emote.burst").expect("teste deve explodir");
 assert_eq!(t["payload"]["rehearsal"],true,"explosão de teste carrega a marca de ensaio");
 // evento fora dos quatro escolhidos não dispara
 let mut raid=event(&p,"",false);raid.kind="raid".into();
 let mut rx=rt.broadcast.subscribe();
 crate::emote_burst::from_event(&rt,&p,&raid);
 assert!(std::iter::from_fn(||rx.try_recv().ok()).all(|v|v["type"]!="emote.burst"),"raid não é evento de explosão");
 // o tique do agendador guarda os totais para a próxima sessão
 crate::emote_burst::flush(&rt);
 assert_eq!(rt.db.module(&p.id,"emoteBurstStats")["25"],json!(2));
}

#[tokio::test]
async fn payloads_de_ensaio_carregam_a_marcacao_e_o_real_nao(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Selo");rt.db.save_profile(&p).unwrap();
 let mut rx=rt.broadcast.subscribe();

 // chat e alertas ligados pelo mesmo caminho do Estúdio
 let mut ccfg=crate::chat_overlay::defaults();ccfg["enabled"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"chatOverlay","value":ccfg})).await.unwrap();
 let mut acfg=crate::alerts::defaults();acfg["enabled"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"alerts","value":acfg})).await.unwrap();

 // mensagem real: payload sem a marca
 engine::process(rt.clone(),event(&p,"ola a todos",false)).await;
 let chat:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat").collect();
 assert_eq!(chat.len(),1);
 assert!(chat[0]["payload"]["rehearsal"].is_null(),"mensagem real não carrega a marca");

 // a mesma passada como ensaio: marcada
 let mut ensaio=event(&p,"ola de ensaio",false);ensaio.rehearsal=true;
 engine::process(rt.clone(),ensaio).await;
 let chat:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat").collect();
 assert_eq!(chat[0]["payload"]["rehearsal"],true,"mensagem de ensaio carrega a marca");

 // botão de teste do Estúdio: o evento entra na fila do motor e o alerta
 // chega marcado (a entrega é assíncrona, como em qualquer evento real)
 dispatch(rt.clone(),"alerts.test",json!({"profileId":p.id,"kind":"follow"})).await.unwrap();
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(5);
 let hit;
 loop{
  let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="alert").collect();
  if let Some(v)=hits.into_iter().next(){hit=v;break}
  assert!(tokio::time::Instant::now()<deadline,"o alerta do teste não chegou");
  tokio::time::sleep(std::time::Duration::from_millis(50)).await;
 }
 assert_eq!(hit["payload"]["rehearsal"],true,"teste de alerta carrega a marca");
 assert_eq!(hit["payload"]["vars"]["name"],"NovoSeguidor");

 // evento real de follow: sem marca
 let mut follow=event(&p,"",false);follow.kind="follow".into();
 engine::process(rt.clone(),follow).await;
 let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="alert").collect();
 assert_eq!(hits.len(),1);
 assert!(hits[0]["payload"]["rehearsal"].is_null(),"alerta real não carrega a marca");
}

#[tokio::test]
async fn ensaio_da_meta_sai_pelo_caminho_real_sem_persistir(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Meta ensaio");rt.db.save_profile(&p).unwrap();
 let mut rx=rt.broadcast.subscribe();

 // integração desligada: nada dispara e a resposta diz o que falta (igual ao
 // vivo, onde a meta não apareceria)
 let out=dispatch(rt.clone(),"livepix.simulate",json!({"profileId":p.id,"mode":"donate"})).await.unwrap();
 assert_eq!(out["fired"],false);
 assert!(out["reason"].as_str().unwrap().contains("Ative a integração"));
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="goal"),"sem integração não há payload");

 // ligar pelo mesmo caminho do Estúdio, com uma meta
 dispatch(rt.clone(),"livepix.enable",json!({"profileId":p.id,"goals":[{"title":"Meta do mês","target":50000,"font_family":"Arial"}]})).await.unwrap();

 // doação de ensaio: publica o goal com o selo, sem tocar no contador guardado
 let out=dispatch(rt.clone(),"livepix.simulate",json!({"profileId":p.id,"mode":"donate"})).await.unwrap();
 assert_eq!(out["fired"],true);
 let msgs:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
 let hit=msgs.iter().find(|v|v["type"]=="goal"&&v["payload"]["rehearsal"]==true).expect("doação de ensaio publica o goal");
 assert_eq!(hit["payload"]["current"],8500,"sobe ~17% do alvo a partir do zero");
 assert_eq!(hit["payload"]["target"],50000);
 assert_eq!(hit["payload"]["percent"],17);
 assert_eq!(hit["payload"]["celebrate"],false);
 assert_eq!(hit["payload"]["hidden"],false);
 assert_eq!(crate::livepix::data(&rt,&p.id)["current"],0,"o contador real não muda no ensaio");

 // celebração: 100% e, com a explosão ligada na Meta, o estouro carrega o selo
 rt.db.set_module(&p.id,"ai_emotes",&json!({"at":0,"names":["Kappa"],"list":[{"id":"25","name":"Kappa"}]})).unwrap();
 rt.db.set_module(&p.id,"emoteBurst",&json!({"enabled":true,"count":10,"size":40,"source":"top","topN":5,"alsoChat":true,"events":{"meta":true}})).unwrap();
 let out=dispatch(rt.clone(),"livepix.simulate",json!({"profileId":p.id,"mode":"celebrate"})).await.unwrap();
 assert_eq!(out["fired"],true);
 assert_eq!(out["celebrate"],true);
 let msgs:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
 let hit=msgs.iter().find(|v|v["type"]=="goal"&&v["payload"]["rehearsal"]==true).expect("celebração publica o goal");
 assert_eq!(hit["payload"]["current"],50000);
 assert_eq!(hit["payload"]["percent"],100);
 assert_eq!(hit["payload"]["celebrate"],true);
 let burst=msgs.iter().find(|v|v["type"]=="emote.burst").expect("celebração dispara a explosão da Meta");
 assert_eq!(burst["payload"]["rehearsal"],true);
 assert_eq!(burst["payload"]["via"],"meta");
 assert_eq!(crate::livepix::data(&rt,&p.id)["current"],0,"nenhuma simulação grava o contador");
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.status=="ensaio"&&l.message.contains("Ensaio: celebração da meta")),"o ensaio fica visível no histórico");
}

#[tokio::test]
async fn chat_de_ensaio_sai_pelo_caminho_real_do_chat(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Chat ensaio");rt.db.save_profile(&p).unwrap();
 let mut rx=rt.broadcast.subscribe();
 let sample=||json!({"id":"ensaio-1","user":"Emoteiro","userId":"u-ensaio","role":"moderator","text":"olha o Kappa","badges":["moderator"],"frags":[{"t":"t","v":"olha o "},{"t":"e","v":"Kappa","id":"25"}]});

 // chat desligado: nada dispara e a resposta diz o motivo (em live, nada apareceria)
 let out=dispatch(rt.clone(),"chat.test",json!({"profileId":p.id,"mode":"message","sample":sample()})).await.unwrap();
 assert_eq!(out["fired"],false);
 assert!(out["reason"].as_str().unwrap().contains("Ligue o chat"));
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="chat"));

 // ligado pelo mesmo caminho do Estúdio
 let mut cfg=crate::chat_overlay::defaults();cfg["enabled"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"chatOverlay","value":cfg})).await.unwrap();

 // a amostra vira payload de chat com o selo, papel, badges e emotes
 let out=dispatch(rt.clone(),"chat.test",json!({"profileId":p.id,"mode":"message","sample":sample()})).await.unwrap();
 assert_eq!(out["fired"],true);
 let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat").collect();
 assert_eq!(hits.len(),1);
 let payload=&hits[0]["payload"];
 assert_eq!(payload["rehearsal"],true,"mensagem de ensaio carrega o selo");
 assert_eq!(payload["id"],"ensaio-1","o overlay remove depois pelo id da amostra");
 assert_eq!(payload["user"],"Emoteiro");
 assert_eq!(payload["role"],"moderator");
 assert_eq!(payload["badges"][0],"moderator");
 assert_eq!(payload["frags"][1]["t"],"e","o emote volta a ser fragmento pelo caminho real");
 assert_eq!(payload["frags"][1]["id"],"25");
 assert_eq!(payload["first"],true,"a pessoa de teste nunca passou por aqui");
 assert_eq!(payload["bot"],false);

 // eventos de canal desligados: o teste aponta o interruptor que falta
 let out=dispatch(rt.clone(),"chat.test",json!({"profileId":p.id,"mode":"event","kind":"follow"})).await.unwrap();
 assert_eq!(out["fired"],false);
 assert!(out["reason"].as_str().unwrap().contains("eventos de canal"));

 // ligando os eventos, o follow de teste vira caixa na coluna, com o selo
 let mut cfg=crate::chat_overlay::defaults();cfg["enabled"]=json!(true);cfg["eventsOn"]=json!(true);cfg["ev"]["follow"]["on"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"chatOverlay","value":cfg})).await.unwrap();
 let out=dispatch(rt.clone(),"chat.test",json!({"profileId":p.id,"mode":"event","kind":"follow"})).await.unwrap();
 assert_eq!(out["fired"],true);
 let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat.event").collect();
 assert_eq!(hits.len(),1);
 assert_eq!(hits[0]["payload"]["rehearsal"],true,"evento de ensaio carrega o selo");
 assert_eq!(hits[0]["payload"]["kind"],"follow");
 assert!(hits[0]["payload"]["text"].as_str().unwrap().contains("Sortuda"),"o texto sai preenchido pelo motor");

 // presente em lote: o Event sai como `gift` e o motor deriva `giftbulk`,
 // exatamente como a Twitch entrega (mesma caixa do Estúdio)
 let out=dispatch(rt.clone(),"chat.test",json!({"profileId":p.id,"mode":"event","kind":"giftbulk"})).await.unwrap();
 assert_eq!(out["fired"],true);
 let hits:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="chat.event").collect();
 assert_eq!(hits.len(),1);
 assert_eq!(hits[0]["payload"]["kind"],"giftbulk");
 assert!(hits[0]["payload"]["text"].as_str().unwrap().contains("10 SUBS"),"amount do total, como o Estúdio mostra");
}
/// Comando de ensaio: a mensagem de verdade entra pelo funil da live — matches,
/// permissão, disputa e intervalo do bucket de ensaio. A lógica roda, o externo
/// fica barrado e a resposta do bot aparece como [Simulação], nunca na Twitch.
#[tokio::test]
async fn comando_de_ensaio_percorre_o_funil_sem_queimar_a_live(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Comando ensaio");rt.db.save_profile(&p).unwrap();
 let act=|kind:&str,target:&str,text:&str|Action{kind:kind.into(),text:text.into(),target:target.into(),value:0,condition:String::new(),..Default::default()};
 let mut obs=act("obs","","mute");obs.obs_op="mute".into();obs.obs_target="Mic".into();
 let mut f=flow(&p);f.name="Cumprimento".into();f.counter=true;f.trigger.cooldown=300;f.trigger.user_cooldown=300;
 f.actions=vec![act("variable.set","local.dentro","pode"),act("chat","","Oi {{user}}!"),act("points","","10"),obs,act("variable.set","global.fora","nao pode")];
 rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();

 // mensagem com emote, como um espectador digitaria
 let out=dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"!oi Kappa","role":"everyone"})).await.unwrap();
 assert_eq!(out["fired"],true,"o comando casa e dispara");
 assert_eq!(out["flow"],"Cumprimento","o vencedor é a mesma automação que a live escolheria");
 let name=out["flow"].as_str().unwrap().to_owned();
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(10);
 let run=loop{
  if let Some(r)=rt.runs.list(&p.id).into_iter().find(|r|r.flow_name==name&&["COMPLETED","FAILED"].contains(&r.status.as_str())){break r}
  assert!(tokio::time::Instant::now()<deadline,"a execução do ensaio não terminou");
  tokio::time::sleep(std::time::Duration::from_millis(20)).await;
 };
 assert_eq!(run.status,"COMPLETED");
 assert_eq!(run.steps[0].status,"SUCCESS","variável local roda de verdade");
 assert_eq!(run.steps[1].status,"SUCCESS","a resposta roda e vira [Simulação]");
 assert_eq!(run.steps[2].status,"SKIPPED","pontos não persistem no ensaio");
 assert_eq!(run.steps[3].status,"SKIPPED","OBS fica de fora");
 assert_eq!(run.steps[4].status,"SKIPPED","variável global não grava");
 let logs=rt.db.logs(&p.id).unwrap();
 assert!(logs.iter().any(|l|l.message=="[Simulação] Oi Teste!"),"a resposta aparece no Histórico, sem publicar");
 assert!(logs.iter().any(|l|l.status=="ensaio"&&l.message.contains("Iniciando Cumprimento")),"o ensaio não conta nas estatísticas");
 assert_eq!(command_counter::get(&rt.db,&p.id,&f.id).unwrap(),0,"contador de comando não persiste no ensaio");
 assert!(rt.emote_counts.lock().unwrap().is_empty(),"emote de teste não infla a explosão de emotes");
 assert!(!std::iter::from_fn(||rx.try_recv().ok()).any(|v|v["type"]=="chat"),"o comando de ensaio não aparece no chat do OBS");

 // o intervalo vem do bucket de ensaio — o da live segue intacto
 let probe={let mut e=event(&p,"!oi",true);e.rehearsal=true;e.user_id="ensaio".into();e};
 let (global,user)=engine::cooldown_keys(&p.id,&f,&probe);
 assert!(global.ends_with(":ensaio")&&user.ends_with(":ensaio"),"bucket do ensaio");
 let (live_global,_)=engine::cooldown_keys(&p.id,&f,&event(&p,"!oi",false));
 assert!(!rt.cooldowns.lock().unwrap().contains_key(&live_global),"a live não pode herdar o intervalo do ensaio");
 let out=dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"!oi","role":"everyone"})).await.unwrap();
 assert_eq!(out["fired"],false,"o segundo ensaio cai no intervalo, como na live");
 assert!(out["reason"].as_str().unwrap().contains("intervalo"));

 // a live responde no mesmo comando: queima o bucket dela e conta o contador
 engine::process(rt.clone(),event(&p,"!oi",false)).await;
 let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(10);
 let run=loop{
  let runs=rt.runs.list(&p.id);
  if runs.len()>=2{break runs.into_iter().next().expect("a execução da live")}
  assert!(tokio::time::Instant::now()<deadline,"a execução da live não terminou");
  tokio::time::sleep(std::time::Duration::from_millis(20)).await;
 };
 assert_eq!(run.flow_name,"Cumprimento");
 assert_eq!(command_counter::get(&rt.db,&p.id,&f.id).unwrap(),1,"a live conta o comando de verdade");
 assert!(rt.cooldowns.lock().unwrap().contains_key(&live_global),"a live queima o bucket dela");
}
/// O ensaio de comando avisa o que a live faria: permissão que não confere,
/// automação desligada ou comando sem dono — nada dispara e o motivo volta.
#[tokio::test]
async fn flow_rehearse_avisa_o_que_nao_responderia_na_live(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Rehearse avisos");rt.db.save_profile(&p).unwrap();

 // comando sem nenhuma automação no perfil
 let out=dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"!sumido","role":"everyone"})).await.unwrap();
 assert_eq!(out["fired"],false);
 assert!(out["reason"].as_str().unwrap().contains("Nenhuma automação"));
 assert!(dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"   ","role":"everyone"})).await.is_err(),"comando vazio não entra no funil");

 // gatilho só para moderadores: como espectador não responderia, como na live
 let mut f=flow(&p);f.name="Segredo".into();f.trigger.pattern="!segredo".into();f.trigger.permission="moderator".into();
 f.actions=vec![Action{kind:"chat".into(),text:"abriu".into(),target:String::new(),value:0,condition:String::new(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let out=dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"!segredo","role":"everyone"})).await.unwrap();
 assert_eq!(out["fired"],false);
 assert!(out["reason"].as_str().unwrap().contains("Nenhuma automação"),"sem a permissão, nenhuma automação responderia");
 let out=dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"!segredo","role":"moderator"})).await.unwrap();
 assert_eq!(out["fired"],true,"com a permissão certa o comando dispara");
 assert_eq!(out["flow"],"Segredo");

 // desligada, a automação some do funil — igualzinho à live
 let mut off=f.clone();off.enabled=false;rt.db.save_flow(&off).unwrap();
 let out=dispatch(rt.clone(),"flow.rehearse",json!({"profileId":p.id,"message":"!segredo","role":"moderator"})).await.unwrap();
 assert_eq!(out["fired"],false);
 assert!(out["reason"].as_str().unwrap().contains("Nenhuma automação"));
}
/// Pré-voo: a varredura estática acusa o que só quebraria ao vivo, cada item com a
/// tela de conserto — e recolhe o aviso quando a peça é ligada.
#[tokio::test]
async fn preflight_acusa_o_que_so_quebraria_ao_vivo(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Pré-voo");rt.db.save_profile(&p).unwrap();
 let act=|kind:&str,target:&str|Action{kind:kind.into(),text:"x".into(),target:target.into(),value:0,condition:String::new(),..Default::default()};
 let mut f=flow(&p);f.name="Tudo quebrado".into();f.audio="audio-sumido".into();
 f.actions=vec![act("tts",""),act("webhook","htp:/quebrado")];
 rt.db.save_flow(&f).unwrap();

 let out=dispatch(rt.clone(),"preflight.check",json!({"profileId":p.id})).await.unwrap();
 let items:Vec<(String,String)>=out["items"].as_array().unwrap().iter()
  .map(|v|(v["message"].as_str().unwrap().into(),v["page"].as_str().unwrap().into())).collect();
 let has=|needle:&str|items.iter().any(|(m,_)|m.contains(needle));
 assert!(has("não está autorizado na Twitch"),"sem autorização nada funcionaria");
 assert!(has("alertas no overlay estão desligados"));
 assert!(has("chat no OBS está desligado"));
 assert!(has("O áudio de “Tudo quebrado” foi apagado"),"arquivo apagado só quebraria ao vivo");
 assert!(has("lê mensagens em voz alta"),"ação de voz sem o módulo");
 assert!(has("webhook inválido"));
 assert!(items.iter().all(|(_,page)|["profiles","alertas","chatobs","sounds","community","obs","ai","flows"].contains(&page.as_str())),"cada item aponta uma tela real");

 // ligando as peças, o pré-voo recolhe os avisos — o arquivo continua apagado
 let mut acfg=crate::alerts::defaults();acfg["enabled"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"alerts","value":acfg})).await.unwrap();
 let mut ccfg=crate::chat_overlay::defaults();ccfg["enabled"]=json!(true);
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"chatOverlay","value":ccfg})).await.unwrap();
 let mut p2=rt.db.profile(&p.id).unwrap();p2.modules["tts"]=json!(true);rt.db.save_profile(&p2).unwrap();
 let out=dispatch(rt.clone(),"preflight.check",json!({"profileId":p.id})).await.unwrap();
 let items:Vec<String>=out["items"].as_array().unwrap().iter().map(|v|v["message"].as_str().unwrap().into()).collect();
 let has=|needle:&str|items.iter().any(|m|m.contains(needle));
 assert!(!has("alertas no overlay estão desligados"),"alerta ligado some do relatório");
 assert!(!has("chat no OBS está desligado"),"chat ligado some do relatório");
 assert!(!has("lê mensagens em voz alta"),"módulo ligado some do relatório");
 assert!(has("O áudio de “Tudo quebrado” foi apagado"),"o arquivo continua faltando até ser restaurado");
 assert!(has("webhook inválido"));
}
/// Spotify: autorização pelo navegador, leitura da faixa, pausa, parada, limite
/// de consultas e desconexão — tudo contra um servidor falso, sem rede e sem
/// tocar a conta de ninguém.
#[tokio::test]
async fn spotify_autoriza_lê_a_faixa_e_transmite_para_o_overlay(){
 use tokio::io::{AsyncReadExt, AsyncWriteExt};
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
 let p=profile("Spotify");rt.db.save_profile(&p).unwrap();

 // Servidor falso: troca de token, conta e "tocando agora" com quatro estados.
 let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let addr=listener.local_addr().unwrap();
 std::env::set_var("BOTLIVE_SPOTIFY_API",format!("http://{addr}"));
 std::env::set_var("BOTLIVE_SPOTIFY_ACCOUNTS",format!("http://{addr}"));
 fn faixa(playing:bool,progress:u64)->String{format!(r#"{{"is_playing":{playing},"progress_ms":{progress},"timestamp":1760000000000,"currently_playing_type":"track","item":{{"id":"t1","name":"Tocando Agora","duration_ms":201000,"artists":[{{"name":"Cactus"}}],"album":{{"name":"Ao Vivo","images":[{{"url":"https://i.scdn.co/capa.jpg"}}]}},"external_urls":{{"spotify":"https://open.spotify.com/track/t1"}}}}}}"#)}
 let route=std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0)); // 0 faixa, 1 pausa, 2 parado, 3 limite
 // Lentidão contornável e contadores: é o que permite arrumar uma desconexão
 // no meio da consulta e provar que a segunda leitura nem saiu para a rede.
 let lento_faixa=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
 let lento_token=std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
 let consultas=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
 let renovacoes=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
 let server=route.clone();
 let faixa_lenta=lento_faixa.clone();let token_lento=lento_token.clone();
 let consultas_srv=consultas.clone();let renovacoes_srv=renovacoes.clone();
 tokio::spawn(async move{loop{
  let (mut s,_) = match listener.accept().await{Ok(x)=>x,Err(_)=>continue};
  let mut buf=[0u8;8192];let n=match s.read(&mut buf).await{Ok(n)=>n,Err(_)=>continue};
  let req=String::from_utf8_lossy(&buf[..n]).to_string();
  let path=req.split_whitespace().nth(1).unwrap_or("/").to_owned();
  let (status,body)=match path.as_str(){
   "/api/token" => {
    renovacoes_srv.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
    if token_lento.load(std::sync::atomic::Ordering::SeqCst){
     tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
    ("200 OK",r#"{"access_token":"token-fake","refresh_token":"refresh-fake","expires_in":3600}"#.to_owned())
   },
   "/v1/me" => ("200 OK",r#"{"display_name":"Cactus","product":"premium"}"#.to_owned()),
   "/v1/me/player/currently-playing" => {
    consultas_srv.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
    if faixa_lenta.load(std::sync::atomic::Ordering::SeqCst){
     tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
    match server.load(std::sync::atomic::Ordering::SeqCst){
     0 => ("200 OK",faixa(true,12000)),
     1 => ("200 OK",faixa(false,45000)),
     2 => ("204 No Content",String::new()),
     _ => ("429 Too Many Requests",String::new()),
    }
   },
   _ => ("404 Not Found","{}".to_owned()),
  };
  let resp=format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
  let _=s.write_all(resp.as_bytes()).await;
 }});

 // Configuração como o Estúdio salvaria: ligado, com o Client ID do app do streamer.
 let mut cfg=crate::spotify::defaults();cfg["enabled"]=json!(true);cfg["clientId"]=json!("app-id-123");
 dispatch(rt.clone(),"module.config",json!({"profileId":p.id,"key":"spotify","value":cfg})).await.unwrap();

 // Pré-voo antes da autorização: acusa, apontando a tela certa.
 let out=dispatch(rt.clone(),"preflight.check",json!({"profileId":p.id})).await.unwrap();
 let msgs:Vec<String>=out["items"].as_array().unwrap().iter().map(|v|v["message"].as_str().unwrap_or("").into()).collect();
 assert!(msgs.iter().any(|m|m.contains("não está autorizado")),"sem autorização o pré-voo avisa");

 // Autorização pelo navegador: o abridor pega a URL e o callback volta na porta.
 let seen=std::sync::Arc::new(std::sync::Mutex::new(String::new()));
 let seen2=seen.clone();
 let out=crate::spotify::connect_with(&rt.http,&p.id,&cfg,move |url|{
  *seen2.lock().unwrap()=url.to_owned();
  let cb=url.to_owned();
  tokio::spawn(async move{
   tokio::time::sleep(std::time::Duration::from_millis(150)).await;
   let u=url::Url::parse(&cb).expect("URL de autorização");
   let state=u.query_pairs().find(|(k,_)|k=="state").expect("state na URL").1.to_string();
   let _=reqwest::get(format!("http://127.0.0.1:43828/callback?code=codigox&state={state}")).await;
  });
  Ok(())
 }).await.unwrap();
 assert!(seen.lock().unwrap().contains("code_challenge"),"PKCE na URL de autorização");
 assert_eq!(out["authorized"],true);
 assert_eq!(out["user"],"Cactus");
 assert!(crate::spotify::connected(&p.id),"token guardado no cofre");

 // Leitura da faixa: estado, transmissão e variáveis das automações.
 let mut rx=rt.broadcast.subscribe();
 let out=dispatch(rt.clone(),"spotify.poll",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(out["track"]["name"],"Tocando Agora");
 assert_eq!(rt.spotify.now(&p.id).unwrap().artist,"Cactus");
 let sent:Vec<Value>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="spotify.now").collect();
 assert_eq!(sent.last().unwrap()["payload"]["name"],"Tocando Agora");
 assert_eq!(sent.last().unwrap()["payload"]["playing"],true);
 let ctx=variables::Context::new(&rt.db,&p,&event(&p,"!tocando",true),None,rt.spotify.now(&p.id)).unwrap();
 assert_eq!(ctx.render("{{nowplaying.musica}}").unwrap(),"Tocando Agora");
 assert_eq!(ctx.render("{{nowplaying.tempo}}").unwrap(),"0:12");
 assert_eq!(ctx.render("{{nowplaying.duracao}}").unwrap(),"3:21");

 // Pausa e parada: o overlay recebe a virada na hora.
 route.store(1,std::sync::atomic::Ordering::SeqCst);
 dispatch(rt.clone(),"spotify.poll",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(rt.spotify.now(&p.id).unwrap().playing,false,"pausa chega ao estado");
 route.store(2,std::sync::atomic::Ordering::SeqCst);
 let out=dispatch(rt.clone(),"spotify.poll",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(out["track"],Value::Null,"parado não tem faixa");
 assert!(rt.spotify.now(&p.id).is_none());

 // Limite de consultas: vira espera, não insistência — e o Histórico não enche.
 route.store(3,std::sync::atomic::Ordering::SeqCst);
 let err=dispatch(rt.clone(),"spotify.poll",json!({"profileId":p.id})).await.unwrap_err();
 assert!(err.contains("pausa nas consultas"),"aviso em português");
 assert!(rt.spotify.throttled(&p.id),"429 impõe espera ao laço");
 let msg="O Spotify pediu uma pausa nas consultas. Aguarde um instante.";
 assert!(rt.spotify.mark_error(&p.id,msg),"erro novo pode ir para o Histórico");
 assert!(!rt.spotify.mark_error(&p.id,msg),"o mesmo erro não repete no Histórico");

 // Consulta em voo: a leitura seguinte não empilha chamada na API e a resposta
 // que chega depois da desconexão é descartada — a música some do overlay na
 // hora e não volta a acender sozinha.
 route.store(0,std::sync::atomic::Ordering::SeqCst);
 lento_faixa.store(true,std::sync::atomic::Ordering::SeqCst);
 let mut rx=rt.broadcast.subscribe();
 let consultas_antes=consultas.load(std::sync::atomic::Ordering::SeqCst);
 let rt2=rt.clone();let p2=p.clone();let cfg2=cfg.clone();
 let voando=tokio::spawn(async move{crate::spotify::poll_once(&rt2,&p2,&cfg2).await});
 // Espera a consulta chegar à rede — a resposta está retida nos 600 ms.
 let mut esperou=0;
 while consultas.load(std::sync::atomic::Ordering::SeqCst)==consultas_antes{
  assert!(esperou<300,"a consulta em voo não saiu para a rede");
  tokio::time::sleep(std::time::Duration::from_millis(10)).await;esperou+=1;
 }
 // A leitura seguinte devolve o estado atual sem nova chamada.
 let out=dispatch(rt.clone(),"spotify.poll",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(consultas.load(std::sync::atomic::Ordering::SeqCst),consultas_antes+1,"a segunda leitura não empilha chamada");
 assert_eq!(out["track"],Value::Null,"sem consulta nova, o estado é o que vale");
 // Desconexão no meio da consulta: o resultado atrasado não repovoa a faixa.
 dispatch(rt.clone(),"spotify.disconnect",json!({"profileId":p.id})).await.unwrap();
 let atrasada=voando.await.unwrap().unwrap();
 assert!(atrasada.is_none(),"resposta anterior à desconexão é descartada");
 assert!(rt.spotify.now(&p.id).is_none(),"a faixa atrasada não volta ao estado");
 assert!(!crate::spotify::connected(&p.id),"cofre limpo");
 let tardios:Vec<Value>=std::iter::from_fn(||rx.try_recv().ok()).filter(|v|v["type"]=="spotify.now").collect();
 assert!(tardios.iter().all(|v|v["payload"]["playing"]!=Value::Bool(true)),"nenhum spotify.now atrasado reacende o overlay");
 lento_faixa.store(false,std::sync::atomic::Ordering::SeqCst);

 // Renovação em voo: a troca de token que termina depois da desconexão não
 // reescreve as credenciais apagadas — a conta segue desconectada.
 crate::spotify::seed_expired(&p.id).unwrap();
 assert!(crate::spotify::connected(&p.id),"cofre reabastecido para a renovação");
 lento_token.store(true,std::sync::atomic::Ordering::SeqCst);
 let renovacoes_antes=renovacoes.load(std::sync::atomic::Ordering::SeqCst);
 let consultas_renov=consultas.load(std::sync::atomic::Ordering::SeqCst);
 let rt3=rt.clone();let p3=p.clone();let cfg3=cfg.clone();
 let renova=tokio::spawn(async move{crate::spotify::poll_once(&rt3,&p3,&cfg3).await});
 let mut esperou=0;
 while renovacoes.load(std::sync::atomic::Ordering::SeqCst)==renovacoes_antes{
  assert!(esperou<300,"a renovação não saiu para a rede");
  tokio::time::sleep(std::time::Duration::from_millis(10)).await;esperou+=1;
 }
 dispatch(rt.clone(),"spotify.disconnect",json!({"profileId":p.id})).await.unwrap();
 let erro=renova.await.unwrap().unwrap_err();
 assert!(erro.contains("renovação do Spotify"),"avisa a renovação interrompida: {erro}");
 assert!(!crate::spotify::connected(&p.id),"renovação atrasada não reescreve o cofre");
 assert!(rt.spotify.now(&p.id).is_none());
 assert_eq!(consultas.load(std::sync::atomic::Ordering::SeqCst),consultas_renov,"nem chega a consultar a faixa");
 let out=dispatch(rt.clone(),"spotify.get",json!({"profileId":p.id})).await.unwrap();
 assert_eq!(out["connected"],false,"painel vê a conta desconectada");
 lento_token.store(false,std::sync::atomic::Ordering::SeqCst);
}

/// Regressão do bloqueio da auditoria 0.11.0: as URLs de mídia montavam o
/// token pela representação JSON (com aspas) e nenhuma rota autenticada
/// atendia — `/asset`, `/tts` e o endereço da fonte do OBS. Aqui as URLs
/// geradas são usadas de verdade: servidor real, resposta 200 e conteúdo WAV,
/// não apenas prefixo/sufixo conferido na string.
#[tokio::test]
async fn urls_de_som_e_tts_funcionam_nas_rotas_autenticadas_de_verdade(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let p=profile("Rotas");rt.db.save_profile(&p).unwrap();
 // API local de verdade, numa porta livre desta máquina.
 let port=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
 rt.db.set("apiPort",&json!(port)).unwrap();
 tokio::spawn(local_api::serve(rt.clone()));
 let mut up=false;
 for _ in 0..100{if tokio::net::TcpStream::connect(("127.0.0.1",port)).await.is_ok(){up=true;break}tokio::time::sleep(std::time::Duration::from_millis(50)).await}
 assert!(up,"a API local sobe na porta do teste");
 // WAV canônico de 44 bytes, válido para importação e para as rotas.
 let mut wav=b"RIFF".to_vec();wav.extend_from_slice(&36u32.to_le_bytes());wav.extend_from_slice(b"WAVEfmt ");wav.extend_from_slice(&16u32.to_le_bytes());
 wav.extend_from_slice(&[1u8,0,2,0,0x44,0xac,0,0,0x10,0xb1,2,0,4,0,0x10,0]);wav.extend_from_slice(b"data");wav.extend_from_slice(&0u32.to_le_bytes());
 let file=dir.path().join("jingle.wav");std::fs::write(&file,&wav).unwrap();
 let id=chat_extras::operation(&rt,&p.id,"chatExtras.import",&json!({"path":file,"kind":"sound"})).unwrap()["id"].as_str().unwrap().to_string();
 let url=audio::asset_src(&rt,&p.id,&id).unwrap();
 assert!(!url.contains('"'),"o token nunca vai com aspas: {url}");
 let resp=rt.http.get(&url).send().await.unwrap();
 assert_eq!(resp.status(),200,"/asset atende a URL gerada");
 let body=resp.bytes().await.unwrap();
 assert!(body.starts_with(b"RIFF")&&body.get(8..12)==Some(b"WAVE"),"o corpo é o WAV de verdade, não um prefixo de URL");
 // TTS servido do cache do perfil, com o id no formato estrito.
 let tid=tts::id_for("voz",0,"oi");
 std::fs::create_dir_all(tts::dir_of(&rt,&p.id)).unwrap();
 std::fs::write(tts::dir_of(&rt,&p.id).join(&tid),&wav).unwrap();
 let turl=tts::url(&rt,&p.id,&tid).unwrap();
 assert!(!turl.contains('"'),"o token nunca vai com aspas: {turl}");
 let resp=rt.http.get(&turl).send().await.unwrap();
 assert_eq!(resp.status(),200,"/tts atende a URL gerada");
 assert!(resp.bytes().await.unwrap().starts_with(b"RIFF"),"o corpo é o WAV falado");
 // O endereço da fonte do OBS autentica de verdade (sem aspas no fragmento).
 let (oport,otoken)=audio::overlay_addr(&rt,&p.id).unwrap();
 let frag=format!("http://127.0.0.1:{oport}/audio#token={otoken}&profile={}",p.id);
 assert!(!frag.contains('"'));
 let tok=frag.split("#token=").nth(1).unwrap().split('&').next().unwrap();
 assert_eq!(livepix::overlay_profile(&rt,tok).as_deref(),Some(p.id.as_str()),"o token do endereço abre a fonte no perfil certo");
}

/// O que o bot fala aparece no chat do overlay (o dono quer ver), sem disparar
/// automações — e o eco do EventSub da conta do bot não duplica o que o
/// aplicativo já mostrou no envio.
#[tokio::test]
async fn a_fala_do_bot_aparece_no_chat_sem_duplicar_nem_disparar_automacoes(){
 let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().join("app")).unwrap();let mut p=profile("Bot");p.bot_id="bot-arroba".into();rt.db.save_profile(&p).unwrap();
 rt.db.set_module(&p.id,"chatOverlay",&json!({"enabled":true,"showBot":true})).unwrap();
 // Automação de toda mensagem: não pode ser acionada pela fala do bot.
 let mut f=flow(&p);f.trigger.kind="chat".into();f.trigger.pattern="".into();
 f.actions=vec![Action{kind:"overlay".into(),text:"alguém falou".into(),target:"".into(),value:0,condition:"".into(),..Default::default()}];
 rt.db.save_flow(&f).unwrap();
 let mut rx=rt.broadcast.subscribe();
 // 1) O envio instantâneo do aplicativo já desenha a fala do bot...
 chat_overlay::bot(&rt,&p,"fala automática");
 // 2) ...e o retorno do EventSub da conta do bot não pode duplicar.
 let mut echo=event(&p,"fala automática",false);echo.user_id="bot-arroba".into();echo.data=json!({"bot":true});
 engine::process(rt.clone(),echo).await;
 // 3) Uma fala manual da conta do bot aparece — uma vez.
 let mut manual=event(&p,"oi pessoal",false);manual.user_id="bot-arroba".into();manual.data=json!({"bot":true});
 engine::process(rt.clone(),manual).await;
 let all:Vec<_>=std::iter::from_fn(||rx.try_recv().ok()).collect();
 let chats:Vec<_>=all.iter().filter(|v|v["type"]=="chat").collect();
 assert_eq!(chats.len(),2,"uma do envio e uma da fala manual — sem duplicata");
 assert!(chats.iter().all(|v|v["payload"]["bot"]==true),"as duas saem marcadas como bot");
 assert!(!all.iter().any(|v|v["type"]=="overlay"),"fala do bot não dispara automação");
}
