pub fn get(profile:&str,key:&str)->Result<String,String> {
 keyring::Entry::new("studio.cactus.botlive",&format!("{profile}:{key}")).map_err(|_|"Não foi possível abrir o cofre do sistema")?.get_password().map_err(|_|"Credencial não configurada. Conecte sua conta ou salve a chave nas configurações.".into())
}
pub fn set(profile:&str,key:&str,value:&str)->Result<(),String> {
 let entry=keyring::Entry::new("studio.cactus.botlive",&format!("{profile}:{key}")).map_err(|_|"Não foi possível abrir o cofre do sistema")?;
 if value.is_empty() {match entry.delete_credential(){Ok(())|Err(keyring::Error::NoEntry)=>Ok(()),Err(_)=>Err("Não foi possível apagar a credencial".into())}}
 else {entry.set_password(value).map_err(|_|"Não foi possível salvar no cofre do sistema".into())}
}
pub fn clear(profile:&str) {for k in ["bot_token","bot_refresh","channel_token","channel_refresh","ai_key","ai_origin","client_secret","discord_webhook","discord_token","bot_expires","channel_expires","obsidian_key"] {let _=set(profile,k,"");}}
