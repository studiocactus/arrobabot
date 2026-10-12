//! Chat no overlay: reduz cada mensagem do chat a papel, badges e emotes e
//! transmite por WebSocket local para a fonte do OBS. Os filtros, o layout da
//! coluna e as animações vivem no motor compartilhado (src/chatOverlay.ts)
//! dentro da fonte do OBS e na prévia do Estúdio — aqui só decidimos o que
//! passa e sincronizamos as deleções da moderação.
use crate::engine::Runtime;
use crate::model::{Event, Profile};
use serde_json::{json, Value};

pub fn config(rt: &Runtime, p: &str) -> Value {
    rt.db.module(p, "chatOverlay")
}

fn text_ok(v: &Value, max: usize) -> bool {
    v.as_str().is_some_and(|s| s.chars().count() <= max)
}

fn num_in(v: &Value, min: f64, max: f64) -> bool {
    v.as_f64().is_some_and(|n| n >= min && n <= max)
}

fn color_ok(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        let n = s.chars().count();
        (4..=32).contains(&n) && s.chars().all(|c| c.is_ascii_alphanumeric() || "#(),. %".contains(c))
    })
}

/// O Estúdio envia o objeto inteiro; valores fora de faixa barram o salvamento
/// antes de qualquer coisa chegar ao overlay.
pub fn valid_config(v: &Value) -> bool {
    const BOOLS: &[&str] = &["enabled", "showBot", "hideBots", "hideCommands", "stripes", "upper", "firstOn"];
    if !BOOLS.iter().all(|k| v[k].is_boolean()) {
        return false;
    }
    if !text_ok(&v["botName"], 64) || !text_ok(&v["bots"], 200) || !text_ok(&v["blocked"], 200) {
        return false;
    }
    if !matches!(v["direction"].as_str(), Some("up") | Some("down")) {
        return false;
    }
    if !matches!(v["enter"].as_str(), Some("fade") | Some("slide") | Some("pixel") | Some("type")) {
        return false;
    }
    if !matches!(v["align"].as_str(), Some("left") | Some("center") | Some("right")) {
        return false;
    }
    if !num_in(&v["max"], 10.0, 300.0) || !num_in(&v["hideAfter"], 0.0, 120.0) || !num_in(&v["width"], 240.0, 960.0) {
        return false;
    }
    if !color_ok(&v["color"]) {
        return false;
    }
    // Cor fixa vazia é válida: o nome fica com a cor do próprio espectador.
    if v["nameFixed"].as_str().is_some_and(|s| !s.is_empty()) && !color_ok(&v["nameFixed"]) {
        return false;
    }
    const FONTS: &[&str] = &[
        "Segoe UI", "Arial", "Georgia", "Verdana", "Trebuchet MS", "Courier New",
        "Inter", "Montserrat", "Poppins", "Rubik", "Orbitron", "Bebas Neue",
        "Press Start 2P", "Playfair Display", "Rajdhani", "Quicksand",
    ];
    if !FONTS.contains(&v["font"].as_str().unwrap_or("")) {
        return false;
    }
    if !num_in(&v["fontSize"], 10.0, 28.0) || !num_in(&v["nameSize"], 10.0, 36.0) {
        return false;
    }
    if !matches!(v["hlMode"].as_str(), Some("none") | Some("bar") | Some("bg") | Some("both")) {
        return false;
    }
    const BADGES: &[&str] = &["broadcaster", "moderator", "vip", "subscriber", "founder", "prime", "bits", "gift", "other"];
    if !BADGES.iter().all(|k| v["badges"][k].is_boolean()) {
        return false;
    }
    ["broadcaster", "moderator", "vip", "subscriber"].iter().all(|k| color_ok(&v["hl"][k]))
        && color_ok(&v["firstColor"])
}

/// Mensagem inicial para a fonte do OBS: a configuração do próprio perfil.
pub fn message(rt: &Runtime, pid: &str) -> Value {
    json!({"type": "chat.config", "payload": {"profileId": pid, "config": config(rt, pid)}})
}

/// Fragmentos compactos para o motor: texto corrido ou emote com id da CDN.
/// Cheermotes viram texto (o bits já aparece escrito na mensagem).
pub fn frags(data: &Value, text: &str) -> Value {
    if let Some(list) = data["message"]["fragments"].as_array() {
        let mut out: Vec<Value> = Vec::new();
        for f in list {
            match f["type"].as_str() {
                Some("emote") => {
                    let id = f["emote"]["emote_id"].as_str().or(f["emote"]["id"].as_str()).unwrap_or("");
                    let v = f["text"].as_str().unwrap_or("");
                    if !id.is_empty() && !v.is_empty() {
                        out.push(json!({"t": "e", "v": v, "id": id}));
                    }
                }
                _ => {
                    let v = f["text"].as_str().unwrap_or("");
                    if !v.is_empty() {
                        out.push(json!({"t": "t", "v": v}));
                    }
                }
            }
        }
        if !out.is_empty() {
            return Value::Array(out);
        }
    }
    json!([{"t": "t", "v": text}])
}

/// Primeira mensagem do espectador desde que o BotLive abriu (em memória).
fn first_time(rt: &Runtime, p: &Profile, user: &str) -> bool {
    let mut seen = rt.chat_seen.lock().unwrap();
    let set = seen.entry(p.id.clone()).or_default();
    if set.len() > 30_000 {
        set.clear();
    }
    let fresh = !set.contains(user);
    set.insert(user.to_owned());
    fresh
}

/// Mensagem real do chat vira evento do overlay, com a identificação que o
/// motor usa para badges, emotes e destaque por papel.
pub fn fire(rt: &Runtime, p: &Profile, e: &Event) {
    if e.simulated {
        return;
    }
    // Mensagem que veio do Discord não existe no chat da plataforma.
    if crate::discord::from_discord(e).is_some() {
        return;
    }
    let cfg = config(rt, &p.id);
    if cfg["enabled"] != true {
        return;
    }
    // O overlay remove depois pelo id da mensagem de chat, não da notificação.
    let id = e.data["message_id"].as_str().unwrap_or(&e.id).to_owned();
    let badges: Vec<Value> = e.data["badges"].as_array()
        .map(|bs| bs.iter().filter_map(|b| b["set_id"].as_str().map(Value::from)).collect())
        .unwrap_or_default();
    let first = first_time(rt, p, &e.user_id);
    rt.emit("chat", json!({
        "profileId": p.id, "id": id, "user": e.user, "userId": e.user_id, "role": e.role,
        "text": e.message, "badges": badges, "frags": frags(&e.data, &e.message),
        "first": first, "bot": false,
    }));
}

/// Mensagem que o próprio BotLive acabou de enviar entra no overlay como
/// resposta, para o chat da fonte não ficar metade vazio.
pub fn bot(rt: &Runtime, p: &Profile, text: &str) {
    let cfg = config(rt, &p.id);
    if cfg["enabled"] != true || cfg["showBot"] != true {
        return;
    }
    let name = cfg["botName"].as_str().filter(|s| !s.is_empty()).unwrap_or("BotLive");
    rt.emit("chat", json!({
        "profileId": p.id, "id": uuid::Uuid::new_v4().to_string(), "user": name,
        "userId": "bot", "role": "everyone", "text": text, "badges": [],
        "frags": [{"t": "t", "v": text}], "first": false, "bot": true,
    }));
}

/// Moderação da Twitch (apagar mensagem, limpar chat) remove da fonte do OBS
/// o que já estava renderizado.
pub fn removed(rt: &Runtime, p: &Profile, e: &Event) {
    match e.kind.as_str() {
        "chat.delete" => {
            rt.emit("chat.delete", json!({"profileId": p.id, "id": e.message}));
            rt.log(&p.id, "chat", "Mensagem apagada no chat", "info");
        }
        "chat.deleteUser" => {
            rt.emit("chat.deleteUser", json!({"profileId": p.id, "userId": e.user_id}));
            rt.log(&p.id, "chat", &format!("Mensagens de {} apagadas no chat", e.user), "info");
        }
        _ => {
            rt.emit("chat.clear", json!({"profileId": p.id}));
            rt.log(&p.id, "chat", "Chat limpo por moderação", "info");
        }
    }
}

/// Configuração padrão: desligada até o streamer ativar. Espelhada em
/// src/chatOverlay.ts (DEFAULT_CHAT) — o motor do app normaliza por conta.
#[cfg(test)]
pub fn defaults() -> Value {
    json!({
        "enabled": false, "botName": "BotLive", "showBot": true, "hideBots": false, "bots": "",
        "hideCommands": false, "blocked": "",
        "direction": "up", "max": 100, "hideAfter": 0, "enter": "fade",
        "stripes": false, "align": "left", "width": 360,
        "upper": false, "nameFixed": "",
        "font": "Segoe UI", "fontSize": 16, "nameSize": 16, "color": "#ffffff",
        "badges": {"broadcaster": true, "moderator": true, "vip": true, "subscriber": true,
                   "founder": true, "prime": true, "bits": true, "gift": true, "other": true},
        "hlMode": "bar",
        "hl": {"broadcaster": "#e91916", "moderator": "#00ad03", "vip": "#e005b9", "subscriber": "#1f9bff"},
        "firstOn": true, "firstColor": "#ffb26b"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padrao_valido_e_faixas_estranhas_barram() {
        assert!(valid_config(&defaults()), "a configuração padrão salva normalmente");
        let mut v = defaults();
        v["max"] = json!(500);
        assert!(!valid_config(&v), "mais de 300 mensagens em tela barra");
        let mut v = defaults();
        v["width"] = json!(100);
        assert!(!valid_config(&v), "coluna estreita demais barra");
        let mut v = defaults();
        v["hideAfter"] = json!(900);
        assert!(!valid_config(&v), "some depois de 120 s no máximo");
        let mut v = defaults();
        v["direction"] = json!("sideways");
        assert!(!valid_config(&v), "direção fora de up/down barra");
        let mut v = defaults();
        v["badges"]["vip"] = json!("yes");
        assert!(!valid_config(&v), "badge precisa ser booleano");
        let mut v = defaults();
        v["font"] = json!("Comic Sans");
        assert!(!valid_config(&v), "fonte fora da lista barra");
        let mut v = defaults();
        v["nameFixed"] = json!("");
        assert!(valid_config(&v), "cor fixa vazia = cor por espectador");
        let mut v = defaults();
        v["firstOn"] = json!(false);
        v["hlMode"] = json!("none");
        assert!(valid_config(&v), "desligar destaques é escolha válida");
    }

    #[test]
    fn fragmentos_viram_texto_e_emote_com_id() {
        let data = json!({"message": {"text": "boa Kappa demais", "fragments": [
            {"type": "text", "text": "boa "},
            {"type": "emote", "text": "Kappa", "emote": {"id": "25"}},
            {"type": "cheermote", "text": "cheer100"}]}});
        let f = frags(&data, "boa Kappa demais");
        assert_eq!(f[0], json!({"t": "t", "v": "boa "}));
        assert_eq!(f[1], json!({"t": "e", "v": "Kappa", "id": "25"}));
        assert_eq!(f[2], json!({"t": "t", "v": "cheer100"}), "cheermote vira texto");
        // YouTube e payloads sem fragmentos caem no texto corrido.
        assert_eq!(frags(&Value::Null, "oi tudo bem"), json!([{"t": "t", "v": "oi tudo bem"}]));
    }

    #[test]
    fn padrao_desligado_e_nome_do_bot_razoavel() {
        assert_eq!(defaults()["enabled"], false, "nada muda no overlay existente sem consentimento");
        assert_eq!(defaults()["showBot"], true, "as respostas do bot aparecem por padrão");
        assert_eq!(defaults()["botName"], "BotLive");
    }
}
