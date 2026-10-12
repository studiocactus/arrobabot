//! Um só lugar para pedidos à API Helix da Twitch. URL base, Client-Id e
//! Authorization saem daqui; cada módulo escolhe método, caminho, consulta e
//! o que fazer da resposta — sem repetir o cabeçalho obrigatório em 30 pontos.
use crate::model::Profile;
use reqwest::{Client, Method, RequestBuilder};

/// Pedido para `https://api.twitch.tv/helix/{path}` já com Client-Id e Bearer.
pub fn call(http:&Client,p:&Profile,token:&str,method:Method,path:&str,query:&[(&str,&str)])->RequestBuilder{
 http.request(method,format!("https://api.twitch.tv/helix/{path}"))
  .header("Client-Id",&p.client_id)
  .bearer_auth(token)
  .query(query)
}
pub fn get(http:&Client,p:&Profile,token:&str,path:&str,query:&[(&str,&str)])->RequestBuilder{call(http,p,token,Method::GET,path,query)}
pub fn post(http:&Client,p:&Profile,token:&str,path:&str,query:&[(&str,&str)])->RequestBuilder{call(http,p,token,Method::POST,path,query)}
pub fn patch(http:&Client,p:&Profile,token:&str,path:&str,query:&[(&str,&str)])->RequestBuilder{call(http,p,token,Method::PATCH,path,query)}
pub fn delete(http:&Client,p:&Profile,token:&str,path:&str,query:&[(&str,&str)])->RequestBuilder{call(http,p,token,Method::DELETE,path,query)}
#[cfg(test)] mod tests {
 use super::*;
 fn profile()->Profile{serde_json::from_value(serde_json::json!({"id":"p","name":"P","platform":"twitch","channel":"canal"})).unwrap()}
 #[test] fn o_pedido_padrao_le_url_client_id_e_token(){
  let p=profile();
  let req=get(&Client::new(),&p,"TOKEN123","users",&[("login","ana")]).build().unwrap();
  assert_eq!(req.url().as_str(),"https://api.twitch.tv/helix/users?login=ana");
  assert_eq!(req.headers().get("client-id").unwrap().to_str().unwrap(),p.client_id.as_str());
  assert_eq!(req.headers().get("authorization").unwrap().to_str().unwrap(),"Bearer TOKEN123");
 }
 #[test] fn cada_verbo_monte_seu_metodo_no_mesmo_caminho(){
  let p=profile();
  assert_eq!(post(&Client::new(),&p,"t","chat/shoutouts",&[]).build().unwrap().method(),Method::POST);
  assert_eq!(patch(&Client::new(),&p,"t","channels",&[]).build().unwrap().method(),Method::PATCH);
  assert_eq!(delete(&Client::new(),&p,"t","channels/vips",&[]).build().unwrap().method(),Method::DELETE);
 }
}
