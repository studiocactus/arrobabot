use crate::{engine::Runtime,model::Event};
use serde_json::{json,Value};
use base64::Engine;
/// "Gravar comando": transforma a fala em evento de voz. O caminho padrão é o
/// mesmo servidor local da escuta contínua (RealtimeSTT) — um só servidor para
/// o uso normal. O whisper-server continua aceito para quem já o usa: se o
/// campo antigo estiver preenchido, ele responde por compatibilidade.
pub async fn transcribe(rt:&Runtime,p:&str,audio:&str)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 if profile.modules["voice"]!=true{return Err("Ative o controle por voz neste perfil".into())}
 if audio.len()>4_000_000{return Err("Gravação muito longa".into())}
 // A transcrição leva segundos: a sessão de escuta pode parar ou o perfil trocar
 // no meio. A geração de agora é a referência — se mudar, o resultado tardio não
 // pode disparar o comando.
 let gen=rt.listen.generation(p);
 let config=rt.db.module(p,"voice");
 let legacy=config["endpoint"].as_str().unwrap_or("").trim().to_owned();
 let bytes=base64::engine::general_purpose::STANDARD.decode(audio).map_err(|_|"Áudio inválido")?;
 let text=if legacy.is_empty(){
  // Padrão: RealtimeSTT da escuta contínua, o mesmo que já está configurado.
  let cfg=crate::listen::Config::read(&rt.db,p);
  let pcm=crate::listen::wav16(&bytes)?;
  if pcm.len()<3200{return Err("Fala muito curta: fale o comando inteiro.".into())}
  let text=crate::listen::transcribe(&rt.http,&cfg,&pcm).await?;
  if text.trim().is_empty(){return Err("Nenhuma fala reconhecida".into())}
  text
 }else{
  whisper(rt,p,&legacy,&bytes).await?
 };
 let text=text.trim().to_owned();
 // A escuta parou ou o perfil trocou durante a transcrição: o resultado chegou
 // tarde e não pode mais virar comando.
 if rt.listen.generation(p)!=gen{
  return Err("A captura foi interrompida: o resultado veio tarde e não foi disparado".into());
 }
 rt.submit(Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.into(),kind:"voice".into(),user:"streamer".into(),user_id:"local-streamer".into(),role:"broadcaster".into(),message:text.clone(),data:json!({"via":"button"}),simulated:false,rehearsal:false}).await?;
 Ok(json!({"text":text}))
}
/// Caminho antigo do whisper-server: mantido para quem já tem o aplicativo.
async fn whisper(rt:&Runtime,_p:&str,endpoint:&str,bytes:&[u8])->Result<String,String>{
 let url=url::Url::parse(endpoint).map_err(|_|"Endereço de reconhecimento inválido")?;
 if !matches!(url.host_str(),Some("127.0.0.1"|"localhost"|"[::1]"))||url.scheme()!="http"{return Err("O reconhecimento de voz deve rodar localmente via HTTP".into())}
 if bytes.len()<44||&bytes[..4]!=b"RIFF"||&bytes[8..12]!=b"WAVE"{return Err("Use áudio WAV PCM".into())}
 let part=reqwest::multipart::Part::bytes(bytes.to_vec()).file_name("comando.wav").mime_str("audio/wav").map_err(|_|"Áudio inválido")?;
 let form=reqwest::multipart::Form::new().part("file",part).text("response_format","json").text("language","pt");
 let res=rt.http.post(url).multipart(form).send().await.map_err(|_|"Inicie o whisper-server local e confira a porta nas configurações de voz")?;
 if !res.status().is_success(){return Err(format!("Reconhecimento local retornou HTTP {}",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Resposta de reconhecimento inválida")?;
 let text=v["text"].as_str().ok_or("Nenhuma fala reconhecida")?.trim().to_owned();
 if text.is_empty(){return Err("Nenhuma fala reconhecida".into())}
 Ok(text)
}
