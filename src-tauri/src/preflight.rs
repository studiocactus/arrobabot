//! Pré-voo: varredura estática das configurações que, sem isto, só quebrariam ao
//! vivo. Sem rede e sem alterar nada — o relatório aponta o problema, em português
//! claro, e a tela onde consertar. O endereço do overlay não entra: ele é gerado
//! pelo aplicativo na hora e não falta.
use crate::{alerts, chat_extras, chat_overlay, engine::Runtime, model::Profile, obs, secrets};
use serde_json::{json, Value};

/// Um item do relatório: o problema e a tela de conserto (id de navegação).
fn item(message: String, page: &'static str) -> Value {
    json!({"message": message, "page": page})
}

/// True quando o arquivo por trás de um registro (áudio, frases) não existe mais.
/// O registro fica no banco, mas o arquivo foi apagado fora do app: só quebraria
/// ao vivo, quando o motor fosse tocar ou ler.
fn file_gone(rt: &Runtime, p: &str, id: &str, kind: &str) -> bool {
    if id.trim().is_empty() {
        return false;
    }
    match chat_extras::asset(rt, p, id, kind) {
        Ok(asset) => chat_extras::audio_path(rt, p, &asset).is_err(),
        Err(_) => true,
    }
}

/// Relatório do pré-voo do perfil selecionado, na ordem em que a live precisaria
/// de cada peça: conexões, overlays, sons e por fim as automações.
pub fn check(rt: &Runtime, p: &Profile) -> Value {
    let mut items: Vec<Value> = Vec::new();

    // Autorização da Twitch: sem ela o bot não lê nem responde o chat.
    let authorized = p.platform != "twitch" || secrets::get(&p.id, "bot_refresh").is_ok();
    if !authorized {
        items.push(item(
            "O bot não está autorizado na Twitch: comandos, chat e alertas não funcionariam.".into(),
            "profiles",
        ));
    }

    // Alertas no overlay: o interruptor mestre e o de cada evento decidem o que aparece.
    let alerts_cfg = alerts::config(rt, &p.id);
    if alerts_cfg["enabled"] != Value::Bool(true) {
        items.push(item(
            "Os alertas no overlay estão desligados: seguidor, sub, bits e raid não apareceriam.".into(),
            "alertas",
        ));
    } else {
        for kind in alerts::KINDS {
            if alerts_cfg["events"][kind]["on"] != Value::Bool(true) {
                items.push(item(
                    format!("O alerta de {} está desligado no overlay.", alerts::label(kind)),
                    "alertas",
                ));
            }
            if file_gone(rt, &p.id, alerts_cfg["events"][kind]["sound"].as_str().unwrap_or(""), "sound") {
                items.push(item(
                    format!("O som do alerta de {} foi apagado de Sons de Live.", alerts::label(kind)),
                    "sounds",
                ));
            }
        }
    }

    // Chat no OBS: as mensagens do público só aparecem com o interruptor ligado.
    if chat_overlay::config(rt, &p.id)["enabled"] != Value::Bool(true) {
        items.push(item(
            "O chat no OBS está desligado: as mensagens do público não apareceriam.".into(),
            "chatobs",
        ));
    }

    // Sons referenciados por pessoas e respostas: o registro fica, o arquivo não.
    let extras = chat_extras::config(rt, &p.id);
    for person in extras.people.iter().filter(|x| x.enabled) {
        if file_gone(rt, &p.id, &person.asset, "sound") {
            items.push(item(
                format!("O som de {} foi apagado de Sons de Live.", person.nickname),
                "sounds",
            ));
        }
    }
    for reply in extras.replies.iter().filter(|x| x.enabled) {
        if file_gone(rt, &p.id, &reply.asset, "txt") {
            items.push(item(
                format!("A resposta automática com “{}” perdeu o arquivo de frases.", reply.keyword),
                "sounds",
            ));
        }
        if file_gone(rt, &p.id, &reply.sound, "sound") {
            items.push(item(
                format!("O som da resposta com “{}” foi apagado de Sons de Live.", reply.keyword),
                "sounds",
            ));
        }
    }

    // Automações: cada etapa tem o seu requisito — arquivo, módulo ou conexão.
    let obs_on = obs::config(rt, &p.id).enabled;
    let module = |key: &str| p.modules[key] == Value::Bool(true);
    let mut gaps: [Option<String>; 5] = Default::default();
    for f in rt.db.flows(&p.id).unwrap_or_default().iter().filter(|f| f.enabled) {
        if file_gone(rt, &p.id, &f.audio, "sound") {
            items.push(item(
                format!("O áudio de “{}” foi apagado de Sons de Live.", f.name),
                "sounds",
            ));
        }
        for a in f.actions.iter().filter(|a| a.enabled) {
            let slot = match a.kind.as_str() {
                // 0: tts sem módulo · 1: pontos sem módulo · 2: discord sem módulo
                "tts" if !module("tts") => Some(0),
                "points" if !module("points") => Some(1),
                "discord" if !module("discord") => Some(2),
                "obs" if !obs_on => Some(3),
                "ai" | "ai.generate" if p.ai.model.trim().is_empty() => Some(4),
                "webhook" if !a.target.starts_with("http") => {
                    items.push(item(
                        format!("A automação “{}” chama um endereço de webhook inválido.", f.name),
                        "flows",
                    ));
                    None
                }
                _ => None,
            };
            if let Some(i) = slot {
                gaps[i].get_or_insert_with(|| f.name.clone());
            }
        }
    }
    let mut gap = |message: String, page: &'static str, slot: &Option<String>| {
        if let Some(name) = slot {
            items.push(item(message.replace("{}", name), page));
        }
    };
    gap("“{}” lê mensagens em voz alta, mas o módulo Texto para voz está desligado.".into(), "community", &gaps[0]);
    gap("“{}” dá pontos, mas o módulo Pontos & loja está desligado.".into(), "community", &gaps[1]);
    gap("“{}” envia ao Discord, mas o módulo Discord está desligado.".into(), "community", &gaps[2]);
    gap("“{}” mexe em cenas do OBS, mas o OBS Studio está desligado.".into(), "obs", &gaps[3]);
    gap("“{}” usa IA, mas nenhum modelo está escolhido.".into(), "ai", &gaps[4]);

    json!({"items": items})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_carrega_o_caminho_do_conserto() {
        let v = item("Alerta desligado".into(), "alertas");
        assert_eq!(v["message"], "Alerta desligado");
        assert_eq!(v["page"], "alertas");
    }
}
