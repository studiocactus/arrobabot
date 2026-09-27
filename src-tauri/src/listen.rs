//! Escuta contínua do microfone: separa a fala do silêncio e entrega cada fala
//! pronta ao servidor local de transcrição. O botão "Gravar comando" e a prévia
//! continuam fora deste caminho; aqui só entra áudio real em tempo real.
use crate::{db::Db,engine::Runtime,model::Event,speech};
use base64::Engine;
use serde_json::{json,Value};
use std::{collections::HashMap,sync::{Arc,Mutex},time::{Duration,Instant}};

/// Amostragem entregue ao servidor: pcm_s16le mono a 16 kHz.
const RATE:u64=16000;
/// Um lote do webview dura 100 ms; o teto deixa margem para lotes maiores.
const MAX_FRAME:usize=64_000;
/// Fala mais curta que isso é chiado de fundo, não frase.
const MIN_SPEECH:u64=350;
/// Fala mais longa que isso é cortada: evita prender memória com o microfone aberto.
const MAX_SPEECH:u64=30000;
const SILENCE_RANGE:(u64,u64)=(200,3000);
const THRESHOLD_RANGE:(f64,f64)=(0.002,0.5);
const DEFAULT_ENDPOINT:&str="http://127.0.0.1:8010/transcribe-pcm16";
const LANGUAGES:&[&str]=&["pt","en","es","fr","de","it","auto"];
type R<T>=Result<T,String>;

/// Configuração do módulo de voz usada pela escuta contínua.
pub struct Config{pub listen:bool,pub endpoint:String,pub language:String,pub activation:String,pub captions:bool,pub silence:u64,pub threshold:f64}
impl Config{
 pub fn read(db:&Db,profile:&str)->Self{
  let v=db.module(profile,"voice");
  let text=|key:&str|v[key].as_str().filter(|s|!s.is_empty()).map(str::to_owned);
  Self{
   listen:v["listen"]==true,
   endpoint:text("listenEndpoint").unwrap_or_else(||DEFAULT_ENDPOINT.to_owned()),
   language:text("language").unwrap_or_else(||"pt".to_owned()),
   activation:text("activation").unwrap_or_default(),
   captions:v["captions"]!=false,
   silence:v["silenceMs"].as_u64().unwrap_or(600).clamp(SILENCE_RANGE.0,SILENCE_RANGE.1),
   threshold:v["threshold"].as_f64().unwrap_or(0.02).clamp(THRESHOLD_RANGE.0,THRESHOLD_RANGE.1),
  }
 }
}
/// Só aceita servidor na própria máquina: o áudio da live não sai do computador.
fn local(endpoint:&str)->Result<url::Url,String>{
 let url=url::Url::parse(endpoint).map_err(|_|"Endereço de transcrição inválido".to_string())?;
 if url.scheme()!="http"||!matches!(url.host_str(),Some("127.0.0.1"|"localhost"|"[::1]")){return Err("O reconhecimento de voz deve rodar localmente via HTTP".into())}
 Ok(url)
}
/// Salva o painel de voz: recusa endereço remoto e valores fora dos limites.
pub fn valid_config(v:&Value)->bool{
 if !v.is_object(){return false}
 for key in ["listenEndpoint","language","activation"]{if !v[key].is_null()&&!v[key].is_string(){return false}}
 if let Some(endpoint)=v["listenEndpoint"].as_str(){if !endpoint.is_empty()&&local(endpoint).is_err(){return false}}
 if let Some(language)=v["language"].as_str(){if !LANGUAGES.contains(&language){return false}}
 if v["activation"].as_str().is_some_and(|a|a.len()>200){return false}
 if let Some(ms)=v["silenceMs"].as_u64(){if !(SILENCE_RANGE.0..=SILENCE_RANGE.1).contains(&ms){return false}}
 if let Some(t)=v["threshold"].as_f64(){if !t.is_finite()||!(THRESHOLD_RANGE.0..=THRESHOLD_RANGE.1).contains(&t){return false}}
 true
}
/// Energia média do lote, de 0 a 1: é o que separa fala de silêncio.
fn level(bytes:&[u8])->f64{
 let mut sum=0f64;let mut count=0usize;
 for pair in bytes.chunks_exact(2){let sample=i16::from_le_bytes([pair[0],pair[1]]) as f64/32768.0;sum+=sample*sample;count+=1;}
 if count==0{0.0}else{(sum/count as f64).sqrt()}
}
#[derive(Default)]
struct Session{speaking:bool,started:Option<Instant>,last_voice:Option<Instant>,buffer:Vec<u8>,frames:u64,heard:u64,last:String,error:String,updated:Option<Instant>}
fn milliseconds(bytes:usize)->u64{(bytes/2) as u64*1000/RATE}
/// Junta o lote na fala em andamento e devolve a fala quando o silêncio a encerra.
fn cut(s:&mut Session,bytes:&[u8],voice:bool,cfg:&Config,now:Instant)->Option<Vec<u8>>{
 if voice{
  if !s.speaking{s.speaking=true;s.started=Some(now);s.buffer.clear();}
  s.last_voice=Some(now);s.buffer.extend_from_slice(bytes);
 }else if s.speaking{s.buffer.extend_from_slice(bytes);}
 if !s.speaking{return None}
 let spoken=s.started.map(|at|now.saturating_duration_since(at).as_millis() as u64).unwrap_or(0);
 let silent=s.last_voice.map(|at|now.saturating_duration_since(at).as_millis() as u64).unwrap_or(0);
 if spoken>=MAX_SPEECH||silent>=cfg.silence{return close(s)}
 None
}
fn close(s:&mut Session)->Option<Vec<u8>>{
 let pcm=std::mem::take(&mut s.buffer);
 s.speaking=false;s.started=None;s.last_voice=None;
 if milliseconds(pcm.len())<MIN_SPEECH{return None}
 Some(pcm)
}
fn describe(s:&Session,cfg:&Config,now:Instant)->Value{
 json!({"enabled":cfg.listen,"listening":s.updated.is_some_and(|at|now.saturating_duration_since(at)<Duration::from_secs(5)),"frames":s.frames,"heard":s.heard,"last":s.last,"error":s.error,"endpoint":cfg.endpoint,"language":cfg.language,"activation":cfg.activation,"captions":cfg.captions})
}
fn empty(cfg:&Config)->Value{describe(&Session::default(),cfg,Instant::now())}
#[derive(Default)]
pub struct Sessions(Mutex<HashMap<String,Session>>);
impl Sessions{
 pub fn reset(&self,profile:&str){self.0.lock().unwrap().remove(profile);}
 /// Recebe 100 ms de PCM16 e devolve o estado da escuta.
 pub fn frame(&self,rt:&Arc<Runtime>,profile:&str,payload:&str)->R<Value>{
  let p=rt.db.profile(profile)?;
  if p.modules["voice"]!=true{return Err("Ative o Controle por voz neste perfil".into())}
  let cfg=Config::read(&rt.db,profile);
  if !cfg.listen{return Err("A escuta contínua está desligada. Ligue o interruptor no painel de voz.".into())}
  local(&cfg.endpoint)?;
  let bytes=base64::engine::general_purpose::STANDARD.decode(payload).map_err(|_|"Áudio inválido".to_string())?;
  if bytes.len()<2||bytes.len()%2!=0{return Err("Lote de áudio inválido: use PCM16".into())}
  if bytes.len()>MAX_FRAME{return Err("Lote de áudio muito grande".into())}
  let voice=level(&bytes)>=cfg.threshold;let now=Instant::now();let ready;let status;
  {
   let mut map=self.0.lock().unwrap();
   let s=map.entry(profile.to_owned()).or_default();
   s.frames+=1;s.updated=Some(now);
   ready=cut(s,&bytes,voice,&cfg,now);
   status=describe(s,&cfg,now);
  }
  if let Some(pcm)=ready{let rt=rt.clone();let profile=profile.to_owned();tokio::spawn(finish(rt,profile,cfg,pcm));}
  Ok(status)
 }
}
/// Estado mostrado no painel de voz, mesmo antes do primeiro lote.
pub fn status(rt:&Runtime,profile:&str)->Value{
 let cfg=Config::read(&rt.db,profile);let now=Instant::now();
 let map=rt.listen.0.lock().unwrap();
 match map.get(profile){Some(s)=>describe(s,&cfg,now),None=>empty(&cfg)}
}
/// Ponto de entrada das chamadas do webview.
pub fn frame(rt:&Arc<Runtime>,profile:&str,payload:&str)->R<Value>{rt.listen.frame(rt,profile,payload)}
/// Chama o servidor local e transforma o resultado em evento, contexto e legenda.
async fn finish(rt:Arc<Runtime>,profile:String,cfg:Config,pcm:Vec<u8>){
 match transcribe(&rt.http,&cfg,&pcm).await{
  Ok(text)=>{
   let recovered={let mut map=rt.listen.0.lock().unwrap();let known=map.contains_key(&profile);let recovered=map.get(&profile).is_some_and(|s|!s.error.is_empty());
    if let Some(s)=map.get_mut(&profile){if !text.is_empty(){s.last=text.clone();s.heard+=1;}s.error.clear();}
    known&&recovered};
   if recovered{rt.log(&profile,"voice","Escuta contínua reconectada ao servidor de transcrição","success");}
   if text.is_empty(){return}
   if let Err(err)=speech::push(&rt.db,&profile,&text){rt.log(&profile,"voice",&format!("Escuta contínua: {err}"),"error");}
   if cfg.captions{rt.emit("captions",json!({"profileId":profile,"text":text}));}
   // Sem palavra de ativação toda fala dispara os fluxos; com ela, só a chamada.
   if cfg.activation.is_empty()||crate::model::mention_any(&cfg.activation,&text){
    let event=Event{id:uuid::Uuid::new_v4().to_string(),profile_id:profile.clone(),kind:"voice".into(),user:"streamer".into(),user_id:"local-streamer".into(),role:"broadcaster".into(),message:text.clone(),data:Value::Null,simulated:false};
    if let Err(err)=rt.submit(event).await{rt.log(&profile,"voice",&format!("Escuta contínua: {err}"),"error");}
   }else{rt.log(&profile,"voice",&format!("Transcrição: {text}"),"info");}
  }
  Err(err)=>{
   let changed={let mut map=rt.listen.0.lock().unwrap();let changed=map.get(&profile).is_none_or(|s|s.error!=err);
    if let Some(s)=map.get_mut(&profile){s.error=err.clone();}
    changed};
   if changed{rt.log(&profile,"voice",&format!("Escuta contínua: {err}"),"error");}
  }
 }
}
/// Monta a chamada que o RealtimeSTT aceita: PCM16 de 16 kHz no corpo binário.
fn url_for(cfg:&Config)->R<url::Url>{
 let mut url=local(&cfg.endpoint)?;
 // encoding=pcm16 é o parâmetro que o RealtimeSTT aceita; pcm_s16le é o formato
 // do arquivo e o servidor responde HTTP 400 para qualquer outro valor.
 {let mut query=url.query_pairs_mut();query.append_pair("sample_rate","16000");query.append_pair("encoding","pcm16");query.append_pair("language",&cfg.language);}
 Ok(url)
}
async fn transcribe(client:&reqwest::Client,cfg:&Config,pcm:&[u8])->R<String>{
 let url=url_for(cfg)?;
 let res=client.post(url).header(reqwest::header::CONTENT_TYPE,"application/octet-stream").body(pcm.to_vec()).send().await
  .map_err(|_|"Servidor de transcrição indisponível. Inicie o servidor local RealtimeSTT e confira o endereço no painel de voz.".to_string())?;
 if res.status().as_u16()==503{return Err("Servidor de transcrição ainda carregando o modelo. Aguarde ele responder 200 em /health e tente de novo.".into())}
 if !res.status().is_success(){return Err(format!("Transcrição local retornou HTTP {}",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Resposta de transcrição inválida".to_string())?;
 Ok(v["text"].as_str().unwrap_or("").trim().to_owned())
}
#[cfg(test)] mod tests {
 use super::*;
 fn pcm(ms:u64,amplitude:f64)->Vec<u8>{
  let samples=(RATE as u64*ms/1000) as usize;
  (0..samples).flat_map(|i|{let wave=(i as f64*0.4).sin()*amplitude;((wave*32767.0) as i16).to_le_bytes()}).collect()
 }
 #[test] fn o_nivel_separa_fala_de_silencio(){
  assert!(level(&pcm(100,0.5))>0.02,"fala forte passa do limite");
  assert!(level(&vec![0u8;3200])<0.02,"silêncio não passa");
  let cfg=Config{listen:true,endpoint:DEFAULT_ENDPOINT.into(),language:"pt".into(),activation:String::new(),captions:true,silence:600,threshold:0.02};
  let mut s=Session::default();let start=Instant::now();
  assert!(cut(&mut s,&pcm(100,0.5),true,&cfg,start).is_none(),"primeiro lote abre a fala e não fecha");
  assert!(cut(&mut s,&pcm(100,0.5),true,&cfg,start+Duration::from_millis(100)).is_none());
  for step in 2..7{assert!(cut(&mut s,&pcm(100,0.0),false,&cfg,start+Duration::from_millis(step*100)).is_none(),"silêncio de {step}00 ms ainda não fecha");}
  let done=cut(&mut s,&pcm(100,0.0),false,&cfg,start+Duration::from_millis(700));
  let done=done.expect("600 ms de silêncio fecham a fala");
  assert_eq!(milliseconds(done.len()),800,"a fala leva junto o silêncio que a encerrou");
  assert!(s.buffer.is_empty()&&!s.speaking,"a sessão fica limpa para a próxima fala");
 }
 #[test] fn fala_curta_e_fala_muito_longa_sao_tratadas(){
  let cfg=Config{listen:true,endpoint:DEFAULT_ENDPOINT.into(),language:"pt".into(),activation:String::new(),captions:true,silence:600,threshold:0.02};
  let start=Instant::now();let mut s=Session::default();
  cut(&mut s,&pcm(100,0.5),true,&cfg,start);
  assert!(cut(&mut s,&pcm(100,0.0),false,&cfg,start+Duration::from_millis(700)).is_none(),"fala de 100 ms é descartada");
  let mut s=Session::default();
  for step in 0..300{cut(&mut s,&pcm(100,0.5),true,&cfg,start+Duration::from_millis(step*100));}
  assert!(s.speaking&&milliseconds(s.buffer.len())==30000,"teto de 30 s segura a fala");
  let long=cut(&mut s,&pcm(100,0.5),true,&cfg,start+Duration::from_millis(31000));
  assert!(long.is_some(),"o teto encerra a fala mesmo sem silêncio");
  assert!(!s.speaking&&s.buffer.is_empty());
 }
 #[test] fn so_enderecos_locais_e_limites_do_painel_sao_aceitos(){
  assert!(valid_config(&json!({"endpoint":"http://127.0.0.1:8080/inference"})),"config antiga continua válida");
  assert!(valid_config(&json!({"listenEndpoint":"http://localhost:8010/transcribe-pcm16","language":"pt","activation":"arroba","silenceMs":600,"threshold":0.02})));
  assert!(!valid_config(&json!({"listenEndpoint":"http://192.168.0.10:8010/transcribe-pcm16"})),"servidor fora da máquina não é aceito");
  assert!(!valid_config(&json!({"listenEndpoint":"https://127.0.0.1:8010/x"})),"só HTTP local");
  assert!(!valid_config(&json!({"language":"jp"})));
  assert!(!valid_config(&json!({"silenceMs":50})));
  assert!(!valid_config(&json!({"threshold":9.0})));
  assert!(!valid_config(&json!({"activation":"x".repeat(300)})));
  assert!(!valid_config(&json!({"listenEndpoint":7})),"tipo errado é recusado");
  assert!(!valid_config(&json!("sim")));
  assert!(local(DEFAULT_ENDPOINT).is_ok());
  assert!(local("http://example.com/inference").is_err());
  assert!(local("não é url").is_err());
 }
 #[test] fn a_configuracao_do_painel_tem_padroes_seguros(){
  let dir=tempfile::tempdir().unwrap();let db=Db::open(&dir.path().join("t.sqlite")).unwrap();
  // module_state tem chave estrangeira para profiles e o id precisa ser UUID.
  const PID:&str="00000000-0000-4000-8000-000000000002";
  let p:crate::model::Profile=serde_json::from_value(json!({"id":PID,"name":"P","platform":"twitch","channel":"canal"})).unwrap();
  db.save_profile(&p).unwrap();
  let cfg=Config::read(&db,PID);
  assert!(!cfg.listen&&cfg.endpoint==DEFAULT_ENDPOINT&&cfg.language=="pt"&&cfg.captions);
  assert_eq!(cfg.silence,600);assert!((cfg.threshold-0.02).abs()<f64::EPSILON);
  db.set_module(PID,"voice",&json!({"listen":true,"listenEndpoint":"http://127.0.0.1:9000/t","language":"en","activation":"bot","captions":false,"silenceMs":9000,"threshold":0.5})).unwrap();
  let cfg=Config::read(&db,PID);
  assert!(cfg.listen&&cfg.language=="en"&&!cfg.captions);
  assert_eq!(cfg.silence,SILENCE_RANGE.1,"valor além do limite é segurado");
  assert_eq!(cfg.threshold,THRESHOLD_RANGE.1);
 }
 fn cfg_for(language:&str)->Config{Config{listen:true,endpoint:DEFAULT_ENDPOINT.into(),language:language.into(),activation:String::new(),captions:true,silence:600,threshold:0.02}}
 #[test] fn a_chamada_usa_o_contrato_do_realtimestt(){
  let url=url_for(&cfg_for("pt")).unwrap();let query=url.query().unwrap_or_default();
  assert!(query.contains("sample_rate=16000"),"amostragem de 16 kHz: {query}");
  assert!(query.contains("encoding=pcm16"),"pcm_s16le volta com HTTP 400: {query}");
  assert!(query.contains("language=pt"),"o idioma escolhido viaja na chamada: {query}");
  let auto=url_for(&cfg_for("auto")).unwrap();
  assert!(auto.query().unwrap_or_default().contains("language=auto"),"sem idioma o servidor assume inglês: {auto}");
 }
 /// Validação manual, com o servidor RealtimeSTT real desta máquina em 127.0.0.1:8010:
 /// `BOTLIVE_STT_PCM=C:\pasta\fala-16k.wav cargo test --locked --lib -- --ignored fala_real --nocapture`
 #[tokio::test]
 #[ignore="exige o servidor RealtimeSTT real rodando na máquina"]
 async fn fala_real_atravesa_o_nosso_cliente(){
  let Ok(path)=std::env::var("BOTLIVE_STT_PCM")else{return};
  let pcm=wav16(&path).expect("WAV mono de 16 kHz em PCM16");
  let text=transcribe(&reqwest::Client::new(),&cfg_for("pt"),&pcm).await.expect("o servidor real deve transcrever");
  println!("transcrição real: {text}");
  assert!(!text.is_empty(),"a fala falada precisa virar texto");
 }
 /// Lê o WAV gerado fora do repositório: confere canal, taxa e profundidade.
 fn wav16(path:&str)->Result<Vec<u8>,String>{
  let raw=std::fs::read(path).map_err(|e|format!("{path}: {e}"))?;
  if raw.len()<44||&raw[0..4]!=b"RIFF"||&raw[8..12]!=b"WAVE"{return Err(format!("{path} não é um WAV"))}
  let mut at=12;let mut channels=0u16;let mut rate=0u64;let mut bits=0u16;let mut data=None;
  while at+8<=raw.len(){
   let mut size=[0u8;4];size.copy_from_slice(&raw[at+4..at+8]);let size=u32::from_le_bytes(size) as usize;
   let body=at+8;let end=body.saturating_add(size).min(raw.len());
   let id=std::str::from_utf8(&raw[at..at+4]).unwrap_or("");
   if id=="fmt "&&end-body>=16{
    channels=u16::from_le_bytes([raw[body],raw[body+1]]);
    rate=u32::from_le_bytes([raw[body+4],raw[body+5],raw[body+6],raw[body+7]]) as u64;
    bits=u16::from_le_bytes([raw[body+14],raw[body+15]]);
   }else if id=="data"{data=Some(raw[body..end].to_vec())}
   at=body+size+size%2;
  }
  if channels!=1||bits!=16||rate!=RATE{return Err(format!("o arquivo precisa ser mono de 16 kHz PCM16; recebi {channels} canal(is), {bits} bits e {rate} Hz"))}
  data.ok_or_else(||"o WAV não tem amostras".to_string())
 }
}
