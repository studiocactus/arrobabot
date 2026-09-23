use crate::{engine::Runtime,model::Event};
use serde_json::{json,Value};
use base64::Engine;
pub async fn transcribe(rt:&Runtime,p:&str,audio:&str)->Result<Value,String>{
 let profile=rt.db.profile(p)?;
 if profile.modules["voice"]!=true{return Err("Ative o controle por voz neste perfil".into())}
 if audio.len()>4_000_000{return Err("Gravação muito longa".into())}
 let config=rt.db.module(p,"voice");
 let endpoint=config["endpoint"].as_str().unwrap_or("http://127.0.0.1:8080/inference");
 let url=url::Url::parse(endpoint).map_err(|_|"Endereço de reconhecimento inválido")?;
 if !matches!(url.host_str(),Some("127.0.0.1"|"localhost"|"[::1]"))||url.scheme()!="http"{return Err("O reconhecimento de voz deve rodar localmente via HTTP".into())}
 let bytes=base64::engine::general_purpose::STANDARD.decode(audio).map_err(|_|"Áudio inválido")?;
 if bytes.len()<44||&bytes[..4]!=b"RIFF"||&bytes[8..12]!=b"WAVE"{return Err("Use áudio WAV PCM".into())}
 let part=reqwest::multipart::Part::bytes(bytes).file_name("comando.wav").mime_str("audio/wav").map_err(|_|"Áudio inválido")?;
 let form=reqwest::multipart::Form::new().part("file",part).text("response_format","json").text("language","pt");
 let res=rt.http.post(url).multipart(form).send().await.map_err(|_|"Inicie o whisper-server local e confira a porta nas configurações de voz")?;
 if !res.status().is_success(){return Err(format!("Reconhecimento local retornou HTTP {}",res.status().as_u16()))}
 let v:Value=res.json().await.map_err(|_|"Resposta de reconhecimento inválida")?;
 let text=v["text"].as_str().ok_or("Nenhuma fala reconhecida")?.trim();
 if text.is_empty(){return Err("Nenhuma fala reconhecida".into())}
 rt.submit(Event{id:uuid::Uuid::new_v4().to_string(),profile_id:p.into(),kind:"voice".into(),user:"streamer".into(),user_id:"local-streamer".into(),role:"broadcaster".into(),message:text.into(),data:Value::Null,simulated:false}).await?;
 Ok(json!({"text":text}))
}
