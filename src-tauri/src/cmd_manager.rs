//! Gerenciador de comandos criados via chat da Twitch.
//! Permite a moderadores e streamers criar, editar e remover automacoes simples de texto.
//! Os comandos criados aqui viram fluxos normais e aparecem na pagina Comandos.
use crate::{db::Db, model::{Action, Flow, Trigger}};
use serde_json::Value;

/// Operacoes possiveis via !cmd.
#[derive(Debug, Clone, PartialEq)]
pub enum CmdOp {
 Add { name: String, response: String },
 Del { name: String },
 Edit { name: String, response: String },
 List,
}

/// Parsa a string do comando !cmd e retorna a operacao desejada.
///
/// Exemplos:
/// - `!cmd add !oi Ola` -> Ok(CmdOp::Add { name: "!oi", response: "Ola" })
/// - `!cmd del !oi` -> Ok(CmdOp::Del { name: "!oi" })
/// - `!cmd edit !oi Ola` -> Ok(CmdOp::Edit { name: "!oi", response: "Ola" })
/// - `!cmd list` -> Ok(CmdOp::List)
pub fn parse(input: &str) -> Result<CmdOp, String> {
 let parts: Vec<&str> = input.splitn(3, ' ').collect();
 if parts.len() < 2 || parts[0] != "!cmd" {
  return Err("Use: !cmd add !nome resposta, !cmd del !nome, !cmd edit !nome resposta ou !cmd list".to_string());
 }
 match parts[1] {
  "add" => {
   if parts.len() < 3 {
    return Err("Informe o nome do comando e a resposta. Ex: !cmd add !oi Ola!".to_string());
   }
   let mut name_parts = parts[2].splitn(2, ' ');
   let name = name_parts.next().unwrap_or("").trim();
   valid_name(name)?;
   let response = name_parts.next().unwrap_or("").trim().to_string();
   if response.is_empty() {
    return Err("Informe a resposta do comando".to_string());
   }
   Ok(CmdOp::Add { name: name.to_string(), response })
  }
  "del" => {
   if parts.len() < 3 {
    return Err("Informe o nome do comando para deletar. Ex: !cmd del !oi".to_string());
   }
   let name = parts[2].split_whitespace().next().unwrap_or("").trim();
   valid_name(name)?;
   Ok(CmdOp::Del { name: name.to_string() })
  }
  "edit" => {
   if parts.len() < 3 {
    return Err("Informe o nome do comando e a nova resposta. Ex: !cmd edit !oi Ola!".to_string());
   }
   let mut name_parts = parts[2].splitn(2, ' ');
   let name = name_parts.next().unwrap_or("").trim();
   valid_name(name)?;
   let response = name_parts.next().unwrap_or("").trim().to_string();
   if response.is_empty() {
    return Err("Informe a nova resposta do comando".to_string());
   }
   Ok(CmdOp::Edit { name: name.to_string(), response })
  }
  "list" => Ok(CmdOp::List),
  _ => Err("Acao invalida. Use: add, del, edit ou list".to_string()),
 }
}

fn valid_name(name: &str) -> Result<(), String> {
 if !name.starts_with('!') || name.len() < 2 {
  return Err("O nome do comando deve comecar com !, como !oi".to_string());
 }
 if name.contains(char::is_whitespace) {
  return Err("O nome do comando nao pode ter espaco".to_string());
 }
 Ok(())
}

fn same_command(pattern: &str, name: &str) -> bool {
 pattern.split(',').map(str::trim).any(|alt| alt.eq_ignore_ascii_case(name))
}

fn chat_flow(profile_id: &str, id: String, name: &str, response: String) -> Flow {
 Flow {
  counter: false,
  timer_seconds: 300,
  audio: String::new(),
  audio_volume: 1.0,
  send_type: "chat".into(),
  send_color: "primary".into(),
  reply_to: false,
  id,
  profile_id: profile_id.to_owned(),
  name: name.to_owned(),
  enabled: true,
  trigger: Trigger { kind: "command".into(), pattern: name.to_owned(), permission: "everyone".into(), cooldown: 0, user_cooldown: 0 },
  actions: vec![Action { kind: "chat".into(), text: response, ..Default::default() }],
  layout: Value::Null,
 }
}

/// Executa a operacao no banco de dados.
/// Retorna a mensagem de sucesso ou erro para enviar ao chat.
pub fn execute(db: &Db, profile_id: &str, op: CmdOp) -> Result<String, String> {
 match op {
  CmdOp::Add { name, response } => {
   let flows = db.flows(profile_id).map_err(|e| e.to_string())?;
   if flows.iter().any(|f| f.trigger.kind == "command" && same_command(&f.trigger.pattern, &name)) {
    return Err(format!("O comando {name} ja existe. Use !cmd edit {name} nova resposta para alterar."));
   }
   let flow = chat_flow(profile_id, uuid::Uuid::new_v4().to_string(), &name, response);
   db.save_flow(&flow).map_err(|e| format!("Erro ao salvar fluxo: {e}"))?;
   Ok(format!("Comando {name} criado com sucesso!"))
  }
  CmdOp::Del { name } => {
   let flows = db.flows(profile_id).map_err(|e| e.to_string())?;
   let target = flows.iter().find(|f| f.trigger.kind == "command" && same_command(&f.trigger.pattern, &name));
   match target {
    Some(f) => {
     db.delete_flow(profile_id, &f.id).map_err(|e| format!("Erro ao deletar fluxo: {e}"))?;
     Ok(format!("Comando {name} removido."))
    }
    None => Err(format!("Comando {name} nao encontrado neste perfil.")),
   }
  }
  CmdOp::Edit { name, response } => {
   let flows = db.flows(profile_id).map_err(|e| e.to_string())?;
   let target = flows.iter().find(|f| f.trigger.kind == "command" && same_command(&f.trigger.pattern, &name));
   match target {
    Some(f) => {
     let mut updated = f.clone();
     updated.trigger.pattern = name.clone();
     updated.actions = vec![Action { kind: "chat".into(), text: response, ..Default::default() }];
     db.save_flow(&updated).map_err(|e| format!("Erro ao atualizar fluxo: {e}"))?;
     Ok(format!("Comando {name} atualizado com sucesso!"))
    }
    None => Err(format!("Comando {name} nao encontrado neste perfil.")),
   }
  }
  CmdOp::List => {
   let flows = db.flows(profile_id).map_err(|e| e.to_string())?;
   let mut names: Vec<String> = flows.iter()
    .filter(|f| f.enabled && f.trigger.kind == "command")
    .flat_map(|f| f.trigger.pattern.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect::<Vec<_>>())
    .collect();
   names.sort();
   names.dedup();
   if names.is_empty() {
    return Ok("Nenhum comando encontrado neste perfil.".to_string());
   }
   const MAX: usize = 10;
   let shown: Vec<String> = names.iter().take(MAX).cloned().collect();
   let mut text = String::from("Comandos: ");
   text.push_str(&shown.join(", "));
   if names.len() > MAX {
    text.push_str(&format!(" ... e mais {} (veja a pagina Comandos)", names.len() - MAX));
   }
   Ok(text)
  }
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn test_parse_add() {
  assert_eq!(parse("!cmd add !oi Ola"), Ok(CmdOp::Add { name: "!oi".to_string(), response: "Ola".to_string() }));
 }
 #[test]
 fn test_parse_del() {
  assert_eq!(parse("!cmd del !oi"), Ok(CmdOp::Del { name: "!oi".to_string() }));
 }
 #[test]
 fn test_parse_edit() {
  assert_eq!(parse("!cmd edit !oi Tudo bem"), Ok(CmdOp::Edit { name: "!oi".to_string(), response: "Tudo bem".to_string() }));
 }
 #[test]
 fn test_parse_list() {
  assert!(matches!(parse("!cmd list"), Ok(CmdOp::List)));
 }
 #[test]
 fn test_parse_invalido() {
  let result = parse("comando aleatorio");
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("Use:"));
 }
 #[test]
 fn test_parse_nome_sem_exclamacao() {
  assert!(parse("!cmd add oi Ola").is_err());
  assert!(parse("!cmd del oi").is_err());
 }
}
