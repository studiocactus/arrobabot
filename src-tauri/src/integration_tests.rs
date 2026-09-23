use super::*;
use serde_json::json;
fn profile(name:&str)->Profile{Profile{id:uuid::Uuid::new_v4().to_string(),name:name.into(),platform:"twitch".into(),channel:name.into(),channel_id:"123".into(),bot_id:"456".into(),client_id:"client".into(),blocklist:vec!["proibido".into()],topics:vec![],editors:vec![],ai:AiConfig::default(),modules:json!({"points":true,"raffles":true,"predictions":true,"queue":true})}}
fn event(p:&Profile,text:&str,simulated:bool)->Event{Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),kind:"chat".into(),user:"Ana".into(),user_id:"ana".into(),role:"everyone".into(),message:text.into(),data:Value::Null,simulated}}
fn flow(p:&Profile)->Flow{Flow{id:uuid::Uuid::new_v4().to_string(),profile_id:p.id.clone(),name:"Teste".into(),enabled:true,trigger:Trigger{kind:"command".into(),pattern:"!oi".into(),permission:"everyone".into(),cooldown:5,user_cooldown:5},actions:vec!["Um $user","Dois","Três"].into_iter().map(|s|Action{kind:"chat".into(),text:s.into(),target:"".into(),value:0,condition:"".into()}).collect(),layout:Value::Null}}
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
 assert_eq!(c.render("$user").unwrap(),e.user);assert_eq!(c.render("$userId $unknown $messageSuffix").unwrap(),"ana $unknown $messageSuffix");
 assert_eq!(c.render("{{arg0|upper}} / %arg1% / {{rawInput}} / {{argCount}}").unwrap(),"CAFÉ / 20 / café  20 / 2");
 assert_eq!(c.render("{{data.reward.title}} {{data.items.0.amount|add:2}} {{arg1|number:2}}").unwrap(),"Hidratar 6 20.00");
 assert_eq!(c.render("{{missing|default:amigo|upper}} {{arg0|length}}").unwrap(),"AMIGO 4");
 assert_eq!(c.render(r"\{{user}} \$user \%userName%").unwrap(),"{{user}} $user %userName%");
 assert!(c.render("{{missing}}").is_err());assert!(c.render("{{user|unsupported}}").is_err());assert!(c.render("{{user").is_err());assert!(c.render("{{arg1|number:7}}").is_err());
 assert!(c.render(&"x".repeat(65537)).is_err());
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
 let mut f=flow(&p);f.actions=vec![Action{kind:"variable.increment".into(),text:"2".into(),target:"global.count".into(),value:0,condition:"".into()},Action{kind:"variable.set".into(),text:"{{user|upper}}".into(),target:"local.name".into(),value:0,condition:"".into()},Action{kind:"chat".into(),text:"{{local.name}}: {{global.count}}".into(),target:"".into(),value:0,condition:"".into()}];
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
