use crate::{model::Profile,secrets};
use serde_json::{json,Value};
use base64::Engine;
use sha2::{Sha256,Digest};
use tokio::{net::TcpListener,io::{AsyncReadExt,AsyncWriteExt}};
pub async fn device_start(client:&reqwest::Client,p:&Profile,account:&str)->Result<Value,String> {
 if p.client_id.is_empty(){return Err("Informe o Client ID do aplicativo Twitch nas opções avançadas do perfil".into())}
 let scopes=if account=="channel" {"moderator:read:followers channel:read:subscriptions bits:read channel:read:redemptions moderator:manage:banned_users moderator:manage:warnings"}else{"user:read:chat user:write:chat"};
 let res=client.post("https://id.twitch.tv/oauth2/device").form(&[("client_id",p.client_id.as_str()),("scopes",scopes)]).send().await.map_err(|_|"Falha ao iniciar autorização Twitch")?;
 if !res.status().is_success(){return Err("Twitch recusou o Client ID. Cadastre o aplicativo como cliente público.".into())}
 res.json().await.map_err(|_|"Resposta OAuth inválida".into())
}
pub async fn device_finish(client:&reqwest::Client,p:&Profile,account:&str,code:&str)->Result<Value,String> {
 let res=client.post("https://id.twitch.tv/oauth2/token").form(&[("client_id",p.client_id.as_str()),("device_code",code),("grant_type","urn:ietf:params:oauth:grant-type:device_code")]).send().await.map_err(|_|"Falha na autorização")?;
 let ok=res.status().is_success();let v:Value=res.json().await.map_err(|_|"Resposta OAuth inválida")?;
 if !ok {return Err(v["message"].as_str().unwrap_or("Autorize no navegador e tente novamente").to_owned())}
 store(p,account,&v)?;
 let token=v["access_token"].as_str().ok_or("Token ausente")?;
 let identity:Value=client.get("https://id.twitch.tv/oauth2/validate").header("Authorization",format!("OAuth {token}")).send().await.map_err(|_|"Falha ao validar conta")?.json().await.map_err(|_|"Conta inválida")?;
 Ok(identity)
}
fn store(p:&Profile,account:&str,v:&Value)->Result<(),String> {
 if !["bot","channel"].contains(&account){return Err("Conta inválida".into())}
 secrets::set(&p.id,&format!("{account}_token"),v["access_token"].as_str().ok_or("Token ausente")?)?;
 if let Some(s)=v["refresh_token"].as_str(){secrets::set(&p.id,&format!("{account}_refresh"),s)?;}
 secrets::set(&p.id,&format!("{account}_expires"),&(chrono::Utc::now().timestamp()+v["expires_in"].as_i64().unwrap_or(3600)).to_string())?;
 Ok(())
}
pub async fn token(client:&reqwest::Client,p:&Profile,account:&str)->Result<String,String> {
 let current=secrets::get(&p.id,&format!("{account}_token"))?;

 if p.platform!="twitch" {
 let expires=secrets::get(&p.id,&format!("{account}_expires")).ok().and_then(|s|s.parse::<i64>().ok()).unwrap_or(0);
 if chrono::Utc::now().timestamp()+60<expires {return Ok(current)}
 let refresh=secrets::get(&p.id,&format!("{account}_refresh"))?;
 let endpoint=if p.platform=="youtube"{"https://oauth2.googleapis.com/token"}else{"https://id.kick.com/oauth/token"};
 let mut form=vec![("grant_type","refresh_token".to_string()),("refresh_token",refresh),("client_id",p.client_id.clone())];
 if let Ok(secret)=secrets::get(&p.id,"client_secret"){form.push(("client_secret",secret));}
 let res=client.post(endpoint).form(&form).send().await.map_err(|_|"Não foi possível renovar a autorização")?;
 if !res.status().is_success(){return Err("A autorização expirou. Conecte a conta novamente.".into())}
 let v:Value=res.json().await.map_err(|_|"Resposta OAuth inválida")?;store(p,account,&v)?;
 return Ok(v["access_token"].as_str().ok_or("Token ausente")?.into())
 }

 let res=client.get("https://id.twitch.tv/oauth2/validate").header("Authorization",format!("OAuth {current}")).send().await.map_err(|_|"Não foi possível validar a autorização")?;
 if res.status().is_success(){return Ok(current)}
 let refresh=secrets::get(&p.id,&format!("{account}_refresh"))?;
 let res=client.post("https://id.twitch.tv/oauth2/token").form(&[("grant_type","refresh_token"),("refresh_token",refresh.as_str()),("client_id",p.client_id.as_str())]).send().await.map_err(|_|"Não foi possível renovar a autorização")?;
 if !res.status().is_success(){return Err("A autorização expirou. Conecte a conta novamente.".into())}
 let v:Value=res.json().await.map_err(|_|"Resposta OAuth inválida")?;store(p,account,&v)?;
 Ok(v["access_token"].as_str().unwrap_or_default().into())
}
pub async fn browser_auth(client:&reqwest::Client,p:&Profile,account:&str)->Result<Value,String>{
 if p.client_id.is_empty(){return Err("Informe o Client ID nas opções avançadas".into())}
 let listener=TcpListener::bind("127.0.0.1:43827").await.map_err(|_|"A porta 43827 de autorização está ocupada")?;
 let redirect="http://127.0.0.1:43827/callback";
 let state=uuid::Uuid::new_v4().to_string();
 let verifier=format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple());
 let challenge=base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
 let (auth,token_url,scope)=match p.platform.as_str(){
 "youtube"=>("https://accounts.google.com/o/oauth2/v2/auth","https://oauth2.googleapis.com/token","https://www.googleapis.com/auth/youtube.force-ssl"),
 "kick"=>("https://id.kick.com/oauth/authorize","https://id.kick.com/oauth/token","user:read channel:read chat:write events:subscribe"),
 _=>return Err("Use a autorização por código da Twitch".into())
 };
 let mut url=url::Url::parse(auth).unwrap();
 url.query_pairs_mut().extend_pairs([("client_id",p.client_id.as_str()),("redirect_uri",redirect),("response_type","code"),("scope",scope),("state",state.as_str()),("code_challenge",challenge.as_str()),("code_challenge_method","S256"),("access_type","offline"),("prompt","consent")]);
 open::that(url.as_str()).map_err(|_|"Não foi possível abrir o navegador")?;
 let code=tokio::time::timeout(std::time::Duration::from_secs(180),async {
 loop {
 let (mut socket,_)=listener.accept().await.map_err(|_|"Falha na autorização")?;
 let mut buf=[0u8;8192];let n=tokio::time::timeout(std::time::Duration::from_secs(5),socket.read(&mut buf)).await.map_err(|_|"Tempo esgotado")?.map_err(|_|"Callback inválido")?;
 let request=String::from_utf8_lossy(&buf[..n]);let path=request.split_whitespace().nth(1).unwrap_or("/");
 let url=url::Url::parse(&format!("http://127.0.0.1{path}")).map_err(|_|"Callback inválido")?;
 let args:std::collections::HashMap<_,_>=url.query_pairs().into_owned().collect();
 let valid=url.path()=="/callback"&&args.get("state")==Some(&state);
 let body=if valid {"Autorizacao recebida. Volte ao BotLive."}else{"Requisicao invalida."};
 let response=format!("HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",if valid{"200 OK"}else{"400 Bad Request"},body.len());
 let _=socket.write_all(response.as_bytes()).await;
 if valid {return args.get("code").cloned().ok_or("Autorização não concedida");}
 }
 }).await.map_err(|_|"O tempo de autorização terminou. Tente novamente.")??;
 let mut form=vec![("grant_type","authorization_code".to_string()),("code",code),("client_id",p.client_id.clone()),("redirect_uri",redirect.to_string()),("code_verifier",verifier)];
 if let Ok(secret)=secrets::get(&p.id,"client_secret"){form.push(("client_secret",secret));}
 let res=client.post(token_url).form(&form).send().await.map_err(|_|"Falha ao trocar código por autorização")?;
 if !res.status().is_success(){return Err("A plataforma recusou a autorização. Confira Client ID, segredo e URL de retorno.".into())}
 let v:Value=res.json().await.map_err(|_|"Resposta OAuth inválida")?;store(p,account,&v)?;
 Ok(json!({"authorized":true}))
}
