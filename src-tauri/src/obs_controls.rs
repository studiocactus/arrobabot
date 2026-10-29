//! Controles tipados; usam a mesma conexão, fila e bloqueios das ações anteriores.
use super::*;

#[derive(Default)]
pub struct Options { pub scene:String, pub filter:String, pub content:String }

pub fn validate(op:&str,o:&Options)->Result<(),String>{
 if o.scene.chars().count()>200||o.filter.chars().count()>200{return Err("Cena e filtro aceitam até 200 caracteres".into())}
 if op.starts_with("filter_")&&o.filter.trim().is_empty(){return Err("Escolha o filtro da fonte".into())}
 if matches!(op,"set_text"|"set_image"|"set_browser_url")&&o.content.chars().count()>16000{return Err("Conteúdo do OBS: até 16000 caracteres".into())}
 if op=="set_browser_url"{
  let u=url::Url::parse(o.content.trim()).map_err(|_|"Informe um endereço completo, começando por http:// ou https://")?;
  if !matches!(u.scheme(),"http"|"https")||u.host_str().is_none()||!u.username().is_empty()||u.password().is_some(){return Err("A fonte de navegador exige HTTP/HTTPS, sem usuário ou senha na URL".into())}
 }
 if op=="set_image"{
  let path=std::path::Path::new(o.content.trim());
  if !path.is_absolute()||!path.is_file(){return Err("Escolha um arquivo de imagem existente neste computador".into())}
  let extension=path.extension().and_then(|v|v.to_str()).unwrap_or("").to_ascii_lowercase();
  if !["png","jpg","jpeg","gif","bmp","webp"].contains(&extension.as_str()){return Err("Escolha uma imagem PNG, JPG, GIF, BMP ou WebP".into())}
 }
 Ok(())
}

pub(super) async fn apply_configured(conn:&mut Conn,op:&str,target:&str,num:f64,o:&Options)->Result<String,String>{
 if op.starts_with("filter_"){
  let current=call(conn,"GetSourceFilter",json!({"sourceName":target,"filterName":o.filter})).await?;
  let enabled=match op{"filter_enable"=>true,"filter_disable"=>false,"filter_toggle"=>!current["filterEnabled"].as_bool().ok_or("OBS não informou o estado do filtro")?,_=>return Err("Operação de filtro inválida".into())};
  call(conn,"SetSourceFilterEnabled",json!({"sourceName":target,"filterName":o.filter,"filterEnabled":enabled})).await?;
  return Ok(format!("Filtro {} de {target}: {}",o.filter,if enabled{"ativado"}else{"desativado"}))
 }
 if matches!(op,"set_text"|"set_image"|"set_browser_url"){
  let current=call(conn,"GetInputSettings",json!({"inputName":target})).await?;
  let kind=current["inputKind"].as_str().ok_or("OBS não informou o tipo da fonte")?;
  let settings=match op{
   "set_text" if kind.starts_with("text_gdiplus")=>json!({"text":o.content,"read_from_file":false}),
   "set_text" if kind.starts_with("text_ft2_source")=>json!({"text":o.content,"from_file":false}),
   "set_image" if kind=="image_source"=>json!({"file":o.content.trim()}),
   "set_browser_url" if kind=="browser_source"=>json!({"url":o.content.trim(),"is_local_file":false}),
   _=>return Err("A fonte escolhida não é compatível com esta operação. Escolha uma fonte do tipo indicado no painel.".into())
  };
  call(conn,"SetInputSettings",json!({"inputName":target,"inputSettings":settings,"overlay":true})).await?;
  return Ok(format!("Conteúdo de {target} atualizado com confirmação do OBS"))
 }
 apply(conn,op,target,num).await
}

pub async fn targets(rt:&Arc<Runtime>,p:&Profile,scope:&str)->Result<Value,String>{
 let mut conn=connect(rt,p).await?;
 let scenes=call(&mut conn,"GetSceneList",json!({})).await?;
 let groups=call(&mut conn,"GetGroupList",json!({})).await?;
 let inputs=call(&mut conn,"GetInputList",json!({})).await?;
 let current=scenes["currentProgramSceneName"].as_str().unwrap_or("");let selected=if scope.is_empty(){current}else{scope};
 let is_group=groups["groups"].as_array().is_some_and(|a|a.iter().any(|v|v.as_str()==Some(selected)));
 let items=if selected.is_empty(){json!({"sceneItems":[]})}else{call(&mut conn,if is_group{"GetGroupSceneItemList"}else{"GetSceneItemList"},json!({"sceneName":selected})).await?};
 let result=json!({"current":current,"scenes":scenes["scenes"].as_array().unwrap_or(&Vec::new()).iter().filter_map(|v|v["sceneName"].as_str()).collect::<Vec<_>>(),"groups":groups["groups"],"inputs":inputs["inputs"],"sources":items["sceneItems"]});
 close(conn).await;Ok(result)
}
pub async fn filters(rt:&Arc<Runtime>,p:&Profile,target:&str)->Result<Value,String>{
 if target.trim().is_empty(){return Err("Escolha a fonte antes do filtro".into())}
 let mut conn=connect(rt,p).await?;let result=call(&mut conn,"GetSourceFilterList",json!({"sourceName":target})).await?;close(conn).await;
 Ok(json!({"filters":result["filters"].as_array().unwrap_or(&Vec::new()).iter().map(|v|json!({"name":v["filterName"],"enabled":v["filterEnabled"]})).collect::<Vec<_>>()}))
}
