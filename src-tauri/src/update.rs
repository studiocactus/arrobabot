use crate::engine::Runtime;
use tauri_plugin_updater::UpdaterExt;
use serde_json::{json,Value};
pub fn defaults()->Value{serde_json::from_str::<Value>(include_str!("../tauri.conf.json")).expect("Configuração Tauri inválida")["plugins"]["updater"].clone()}
pub fn endpoint(rt:&Runtime)->String{rt.db.get("updateEndpoint").as_str().filter(|s|!s.trim().is_empty()).map(str::to_owned).unwrap_or_else(||defaults()["endpoints"][0].as_str().unwrap_or("").to_owned())}
pub fn public_key(rt:&Runtime)->String{rt.db.get("updatePublicKey").as_str().filter(|s|!s.trim().is_empty()).map(str::to_owned).unwrap_or_else(||defaults()["pubkey"].as_str().unwrap_or("").to_owned())}
pub async fn check(rt:&Runtime,install:bool)->Result<Value,String>{
 let endpoint=endpoint(rt);
 let key=public_key(rt);
 if endpoint.is_empty()||key.is_empty(){return Err("Configure o manifesto de atualização e a chave pública do distribuidor.".into())}
 let url=url::Url::parse(&endpoint).map_err(|_|"Endereço de atualização inválido")?;
 if url.scheme()!="https"{return Err("Atualizações exigem HTTPS".into())}
 let app=rt.app.lock().unwrap().clone().ok_or("Aplicativo não iniciado")?;
 let updater=app.updater_builder().pubkey(key).endpoints(vec![url]).map_err(|_|"Endereço de atualização inválido")?.build().map_err(|_|"Configuração do atualizador inválida")?;
 if let Some(update)=updater.check().await.map_err(|_|"Não foi possível verificar atualizações")?{
 let version=update.version.clone();
 if install{update.download_and_install(|_,_|{},||{}).await.map_err(|_|"Falha ao baixar, verificar assinatura ou instalar atualização")?;}
 Ok(json!({"available":true,"version":version,"notes":update.body}))
 }else{Ok(json!({"available":false}))}
}
