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
    const BOOLS: &[&str] = &["enabled", "showBot", "hideBots", "hideCommands", "stripes", "upper", "firstOn", "prefixOn", "eventsOn", "evShowMsg"];
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
    // Origem da cor do nome: Twitch (color real do EventSub), personalizada
    // (cor estável por espectador) ou fixa (mesma cor para todos).
    if !matches!(v["nameMode"].as_str(), Some("twitch") | Some("custom") | Some("fixed")) {
        return false;
    }
    if !text_ok(&v["prefix"], 16) || !color_ok(&v["prefixColor"]) {
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
    if !["broadcaster", "moderator", "vip", "subscriber"].iter().all(|k| color_ok(&v["hl"][k]))
        || !color_ok(&v["firstColor"])
    {
        return false;
    }
    // Eventos inline: um tipo por caixa, com interruptor, cor, som e texto.
    crate::alerts::KINDS.iter().all(|k| {
        let ev = &v["ev"][k];
        ev["on"].is_boolean() && color_ok(&ev["color"]) && text_ok(&ev["sound"], 64) && text_ok(&ev["text"], 160)
    }) && {
        // Modo código: HTML, CSS e JS do próprio streamer (vazios por padrão).
        ["html", "css", "js"].iter().all(|k| text_ok(&v["code"][k], 20_000))
    }
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
        "first": first, "bot": false, "color": twitch_color(&e.data["color"]),
    }));
}

/// Cor que o espectador escolheu na Twitch (`#rrggbb`); vazia quando nunca
/// escolheu — o motor resolve por hash ou pela cor fixa do perfil.
fn twitch_color(v: &Value) -> String {
    let s = v.as_str().unwrap_or("");
    match s.strip_prefix('#') {
        Some(body) if body.len() == 6 && body.chars().all(|c| c.is_ascii_hexdigit()) => s.to_owned(),
        _ => String::new(),
    }
}

/// Evento de canal (follow, sub, presente, bits, raid) vira caixa inline na
/// coluna do chat — cor e texto por evento, independente da camada de alertas.
/// O som sai pela linha de áudio do aplicativo, nunca pela fonte do OBS.
pub fn event(rt: &Runtime, p: &Profile, e: &Event) {
    if e.simulated {
        return;
    }
    let cfg = config(rt, &p.id);
    if cfg["enabled"] != true || cfg["eventsOn"] != true {
        return;
    }
    let Some((kind, mut vars)) = crate::alerts::vars(e) else {
        return;
    };
    let ev = &cfg["ev"][kind.as_str()];
    if ev["on"] != true {
        return;
    }
    // No estilo do widget de referência: presente mostra quem ganhou ({name})
    // e quem deu ({sender}); nos demais dois são o próprio participante.
    if kind == "gift" {
        let sender = e.user.clone();
        let name = e.data["target_user_name"].as_str().filter(|s| !s.is_empty()).unwrap_or(&e.user).to_owned();
        vars["name"] = json!(name);
        vars["sender"] = json!(sender);
    } else {
        vars["sender"] = vars["name"].clone();
    }
    let text = fill_box(&cfg, ev["text"].as_str().unwrap_or(""), &vars);
    rt.emit("chat.event", json!({"profileId": p.id, "kind": kind, "color": ev["color"], "text": text}));
    line_event_sound(rt, p, ev["sound"].as_str().unwrap_or(""));
}

/// Texto final da caixa: template preenchido e, quando ligado em `evShowMsg`,
/// a mensagem do espectador como segunda linha (renovação e bits com texto) —
/// o mesmo desenho que a prévia do Estúdio monta em `eventBoxHtml`.
fn fill_box(cfg: &Value, tpl: &str, vars: &Value) -> String {
    let mut out = fill_event(tpl, vars);
    if cfg["evShowMsg"] == true {
        let extra = match &vars["message"] {
            Value::String(s) if !s.is_empty() => esc_html(s),
            _ => String::new(),
        };
        if !extra.is_empty() {
            out.push_str(&format!("<span class=\"bl-evmsg\">{extra}</span>"));
        }
    }
    out
}

/// Som do evento na linha do aplicativo (asset de som, volume cheio).
fn line_event_sound(rt: &Runtime, p: &Profile, asset: &str) {
    if asset.is_empty() || crate::chat_extras::asset(rt, &p.id, asset, "sound").is_err() {
        return;
    }
    crate::chat_extras::play_line(rt, p, asset, None, 1.0);
}

/// Escapa valor de espectador para o texto sair como texto, nunca como HTML.
fn esc_html(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '&' => "&amp;".into(),
            '"' => "&quot;".into(),
            '\'' => "&#39;".into(),
            _ => c.to_string(),
        })
        .collect()
}

/// Preenche `{name} {amount} {months} {tier} {sender} {message}` escapando os
/// valores: o texto é do streamer, o nome de espectador nunca vira HTML.
fn fill_event(text: &str, vars: &Value) -> String {
    let mut out = text.to_owned();
    for key in ["name", "amount", "months", "tier", "sender", "message"] {
        let value = match &vars[key] {
            Value::String(s) => esc_html(s),
            Value::Number(n) => n.to_string(),
            _ => String::new(),
        };
        out = out.replace(&format!("{{{key}}}"), &value);
    }
    out
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
    let mut v = json!({
        "enabled": false, "botName": "BotLive", "showBot": true, "hideBots": false, "bots": "",
        "hideCommands": false, "blocked": "",
        "direction": "up", "max": 100, "hideAfter": 0, "enter": "fade",
        "stripes": false, "align": "left", "width": 360,
        "upper": false, "nameFixed": "", "nameMode": "twitch",
        "prefixOn": true, "prefix": "> ", "prefixColor": "#c6ff00",
        "font": "Segoe UI", "fontSize": 16, "nameSize": 16, "color": "#ffffff",
        "badges": {"broadcaster": true, "moderator": true, "vip": true, "subscriber": true,
                   "founder": true, "prime": true, "bits": true, "gift": true, "other": true},
        "hlMode": "bar",
        "hl": {"broadcaster": "#e91916", "moderator": "#00ad03", "vip": "#e005b9", "subscriber": "#1f9bff"},
        "firstOn": true, "firstColor": "#ffb26b",
        "eventsOn": false, "evShowMsg": true,
        "code": {"html": "", "css": "", "js": ""}
    });
    // Os 7 tipos saem em blocos separados: um único json! com tudo estoura o
    // limite de recursão do macro.
    v["ev"] = json!({
        "follow": {"on": true, "color": "#c6ff00", "sound": "", "text": "NOVO FOLLOW / {name}"},
        "subscription": {"on": true, "color": "#b388ff", "sound": "", "text": "NOVO SUB / {name} / {tier}"},
        "resub": {"on": true, "color": "#8c9eff", "sound": "", "text": "RESUB / {name} / {months} MESES / {tier}"},
        "gift": {"on": true, "color": "#ff7ad9", "sound": "", "text": "SUB PRESENTE / {sender} > {name}"},
        "giftbulk": {"on": true, "color": "#ff4db8", "sound": "", "text": "{name} PRESENTEOU {amount} SUBS"},
        "cheer": {"on": true, "color": "#ffd24d", "sound": "", "text": "{name} / {amount} BITS"},
        "raid": {"on": true, "color": "#ff8a3d", "sound": "", "text": "RAID / {name} / {amount} VIEWERS"}
    });
    v
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

    #[test]
    fn origem_de_cor_prefixo_eventos_e_codigo_sao_validados() {
        let mut v = defaults();
        v["nameMode"] = json!("neon");
        assert!(!valid_config(&v), "origem de cor fora de twitch/custom/fixed barra");
        let mut v = defaults();
        v["prefix"] = json!(">>".repeat(10));
        assert!(!valid_config(&v), "prefixo longo demais barra");
        let mut v = defaults();
        v["ev"]["raid"]["color"] = json!("");
        assert!(!valid_config(&v), "cor de evento vazia barra");
        let mut v = defaults();
        v["ev"]["follow"]["text"] = json!("x".repeat(161));
        assert!(!valid_config(&v), "texto de evento passa de 160 caracteres");
        let mut v = defaults();
        v["ev"].as_object_mut().unwrap().remove("cheer");
        assert!(!valid_config(&v), "os 7 tipos de evento precisam estar presentes");
        let mut v = defaults();
        v["code"]["js"] = json!("x".repeat(20_001));
        assert!(!valid_config(&v), "código acima de 20 mil caracteres barra");
        let mut v = defaults();
        v["eventsOn"] = json!(true);
        v["code"]["css"] = json!(".bl-chat-ev{border-color:#ff0}");
        assert!(valid_config(&v), "eventos ligados e CSS próprio salvam normalmente");
    }

    #[test]
    fn texto_de_evento_preenche_variaveis_e_escapa() {
        let vars = json!({"name": "<b>ana</b>", "amount": 5, "months": 12,
                          "tier": "TIER 2", "sender": "leo", "message": "oi"});
        let out = fill_event("SUB PRESENTE / {sender} > {name} e {amount}", &vars);
        assert_eq!(out, "SUB PRESENTE / leo > &lt;b&gt;ana&lt;/b&gt; e 5");
        assert_eq!(fill_event("RESUB / {name} / {months} MESES", &vars), "RESUB / &lt;b&gt;ana&lt;/b&gt; / 12 MESES");
        assert_eq!(fill_event("sem placeholder", &vars), "sem placeholder");
    }

    #[test]
    fn caixa_mostra_a_mensagem_do_espectador_quando_ligada() {
        let vars = json!({"name": "ana", "message": "<b>boa live</b>"});
        let mut cfg = defaults();
        cfg["evShowMsg"] = json!(true);
        assert_eq!(
            fill_box(&cfg, "RESUB / {name}", &vars),
            "RESUB / ana<span class=\"bl-evmsg\">&lt;b&gt;boa live&lt;/b&gt;</span>"
        );
        cfg["evShowMsg"] = json!(false);
        assert_eq!(fill_box(&cfg, "RESUB / {name}", &vars), "RESUB / ana", "desligado não desenha a linha");
        cfg["evShowMsg"] = json!(true);
        assert_eq!(
            fill_box(&cfg, "FOLLOW / {name}", &json!({"name": "ana", "message": ""})),
            "FOLLOW / ana",
            "sem mensagem não sobra linha vazia"
        );
    }

    #[test]
    fn cor_da_twitch_so_passa_como_hex() {
        assert_eq!(twitch_color(&json!("#9146ff")), "#9146ff");
        assert_eq!(twitch_color(&json!("")), "", "nunca escolheu cor vem vazia");
        assert_eq!(twitch_color(&json!("red")), "", "fora de hex é ignorado");
        assert_eq!(twitch_color(&json!("#9146ff<style>")), "", "tentativa de HTML é descartada");
        assert_eq!(twitch_color(&Value::Null), "");
    }
}
