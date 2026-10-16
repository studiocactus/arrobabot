//! Estúdio de Alertas: guarda a configuração por evento e transmite o disparo
//! por WebSocket local. A animação acontece no motor compartilhado
//! (src/alerts.ts) dentro da fonte do OBS e na prévia do Estúdio — o backend
//! só decide o que tocar, com os mesmos textos de teste do widget de
//! referência (payload falso entra pelo mesmo caminho do evento real).
use crate::engine::Runtime;
use crate::model::{Event, Profile};
use serde_json::{json, Value};

/// Tipos de alerta configuráveis. Em `gift` com total > 1 o evento vira `giftbulk`.
pub const KINDS: &[&str] = &["follow", "subscription", "resub", "gift", "giftbulk", "cheer", "raid"];

#[cfg(test)]
fn event_default(kind: &str) -> Value {
    let (tag, name, big, foot) = match kind {
        "follow" => ("SEGUIDOR", "{{user}}", "BEM-VINDO!", "Obrigado por seguir o canal"),
        "subscription" => ("NOVO SUB", "{{user}}", "{{tier}}", "Obrigado pelo apoio!"),
        "resub" => ("RENOVAÇÃO", "{{user}}", "{{months}} MESES", "{{message}}"),
        "gift" => ("SUB DE PRESENTE", "{{user}}", "GANHOU UM SUB", "Obrigado pelo carinho!"),
        "giftbulk" => ("PRESENTES EM LOTE", "{{user}}", "{{amount}} SUBS", "Obrigado pelo carinho!"),
        "cheer" => ("BITS", "{{user}}", "{{amount}} BITS", "{{message}}"),
        _ => ("RAID", "{{user}}", "{{amount}} VIEWERS", "Sejam todos bem-vindos!"),
    };
    json!({"on": true, "tag": tag, "name": name, "big": big, "foot": foot, "sound": "", "color": ""})
}

/// Configuração padrão: desligada até o streamer ativar, textos em pt-BR.
#[cfg(test)]
pub fn defaults() -> Value {
    let mut v = json!({
        "enabled": false, "duration": 8, "speed": 100, "volume": 70,
        "posV": "center", "posH": "center", "marginX": 0, "marginY": 0, "panelWidth": 900,
        "accent": "#c6ff00", "text": "#ffffff", "bg": "#161616", "line": "#3d3d3d",
        "fontSmall": "Press Start 2P", "fontBig": "Montserrat",
        "tagSize": 12, "nameSize": 26, "bigSize": 34, "footSize": 13,
        "brandText": "BOTLIVE",
        "showScanlines": true, "showCorners": true, "showParticles": true, "showGlitch": true,
    });
    let events: Value = KINDS.iter().map(|k| (k.to_string(), event_default(k))).collect();
    v["events"] = events;
    v["code"] = json!({"html": "", "css": "", "js": ""});
    v
}

pub fn config(rt: &Runtime, p: &str) -> Value {
    rt.db.module(p, "alerts")
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
        (4..=32).contains(&n) && s.chars().all(|c| c.is_ascii_alphanumeric() || "#(),. %-".contains(c))
    })
}

/// O painel do Estúdio envia o objeto inteiro; aqui só validamos o que o
/// motor de animação vai consumir. Valores fora de faixa barram o salvamento.
pub fn valid_config(v: &Value) -> bool {
    let bools = ["enabled", "showScanlines", "showCorners", "showParticles", "showGlitch"];
    if !bools.iter().all(|k| v[k].is_boolean()) {
        return false;
    }
    if !num_in(&v["duration"], 3.0, 20.0) || !num_in(&v["speed"], 50.0, 200.0) || !num_in(&v["volume"], 0.0, 100.0) {
        return false;
    }
    if !matches!(v["posV"].as_str(), Some("top") | Some("center") | Some("bottom")) {
        return false;
    }
    if !matches!(v["posH"].as_str(), Some("left") | Some("center") | Some("right")) {
        return false;
    }
    if !num_in(&v["marginX"], 0.0, 400.0) || !num_in(&v["marginY"], 0.0, 400.0) || !num_in(&v["panelWidth"], 320.0, 1760.0) {
        return false;
    }
    if !["accent", "text", "bg", "line"].iter().all(|k| color_ok(&v[k])) {
        return false;
    }
    const FONTS: &[&str] = &[
        "Segoe UI", "Arial", "Georgia", "Verdana", "Trebuchet MS", "Courier New",
        "Inter", "Montserrat", "Poppins", "Rubik", "Orbitron", "Bebas Neue",
        "Press Start 2P", "Playfair Display", "Rajdhani", "Quicksand",
    ];
    if !FONTS.contains(&v["fontSmall"].as_str().unwrap_or("")) || !FONTS.contains(&v["fontBig"].as_str().unwrap_or("")) {
        return false;
    }
    if !["tagSize", "nameSize", "bigSize", "footSize"].iter().all(|k| num_in(&v[k], 8.0, 72.0)) {
        return false;
    }
    if !text_ok(&v["brandText"], 32) {
        return false;
    }
    // Modo código: HTML, CSS e JS do próprio streamer (vazios por padrão).
    if !["html", "css", "js"].iter().all(|k| text_ok(&v["code"][k], 20_000)) {
        return false;
    }
    KINDS.iter().all(|kind| {
        let e = &v["events"][kind];
        e["on"].is_boolean()
            && text_ok(&e["tag"], 160)
            && text_ok(&e["name"], 160)
            && text_ok(&e["big"], 160)
            && text_ok(&e["foot"], 160)
            && text_ok(&e["sound"], 64)
            // Cor da tarja: vazio = cor global do painel.
            && (e["color"].as_str().is_some_and(|s| s.is_empty()) || color_ok(&e["color"]))
    })
}

/// Mensagem inicial para a fonte do OBS: a config do próprio perfil.
pub fn message(rt: &Runtime, p: &str) -> Value {
    json!({"type": "alerts", "payload": {"profileId": p, "config": config(rt, p)}})
}

/// Rótulo do tier conforme a Twitch envia ("1000"/"2000"/"3000").
fn tier_label(t: Option<&str>) -> &'static str {
    match t {
        Some("2000") | Some("tier2") | Some("2") => "TIER 2",
        Some("3000") | Some("tier3") | Some("3") => "TIER 3",
        _ => "TIER 1",
    }
}

/// Evento de plataforma → tipo de alerta + variáveis dos textos `{{…}}`.
/// `gift` com total > 1 vira `giftbulk` (mesmo critério do widget de referência).
pub fn vars(e: &Event) -> Option<(String, Value)> {
    let kind = match e.kind.as_str() {
        "follow" => "follow",
        "subscription" => "subscription",
        "resub" => "resub",
        "gift" => {
            if e.data["total"].as_i64().unwrap_or(1) > 1 {
                "giftbulk"
            } else {
                "gift"
            }
        }
        "cheer" => "cheer",
        "raid" => "raid",
        _ => return None,
    };
    let amount = e.data["total"]
        .as_i64()
        .or(e.data["bits"].as_i64())
        .or(e.data["viewers"].as_i64())
        .unwrap_or(1)
        .max(1);
    let months = e.data["cumulative_months"].as_i64().unwrap_or(1).max(1);
    let tier = tier_label(e.data["tier"].as_str());
    Some((
        kind.into(),
        json!({"name": e.user, "amount": amount, "months": months, "tier": tier, "message": e.message}),
    ))
}

/// Transmite o alerta para as fontes do OBS (e para a prévia que escutar).
/// O som sai sempre na linha de áudio do aplicativo — o aparelho escolhido na
/// aba Áudio, que o OBS captura como fonte única do bot.
/// `rehearsal` marca o alerta de teste: o overlay exibe o selo ENSAIO.
fn dispatch(rt: &Runtime, p: &Profile, kind: &str, vars: Value, rehearsal: bool) {
    let mut payload = json!({"profileId": p.id, "kind": kind, "vars": vars, "platform": p.platform});
    if rehearsal { payload["rehearsal"] = json!(true); }
    let _ = rt.emit("alert", payload);
    line_sound(rt, p, kind);
}

/// Som do alerta na linha de áudio do aplicativo — a mesma saída que o OBS
/// captura como fonte única do bot. A fonte de navegador ficou só visual.
fn line_sound(rt: &Runtime, p: &Profile, kind: &str) {
    let cfg = config(rt, &p.id);
    let Some(sound) = cfg["events"][kind]["sound"].as_str().filter(|s| !s.is_empty()) else { return };
    let Ok(info) = crate::livepix::overlay_info(rt, &p.id) else { return };
    let _ = rt.emit(
        "alert-sound",
        json!({
            "profileId": p.id,
            "src": format!("http://127.0.0.1:{}/asset?token={}&id={}", info["port"], info["token"], sound),
            // O volume salvo é 0–100, mas o player da linha aceita 0–1
            // (Math.min(1, volume) engolia qualquer valor acima de 1).
            "volume": (cfg["volume"].as_f64().unwrap_or(70.0) / 100.0).clamp(0.0, 1.0),
            "deviceId": crate::audio::device(rt, &p.id),
        }),
    );
}

/// Evento real (ou simulado pelo próprio fluxo) que virou alerta no overlay.
/// Respeita a chave mestre e o interruptor de cada evento; sem config salva,
/// nada toca — quem nunca abriu o Estúdio não muda o overlay existente.
pub fn fire(rt: &Runtime, p: &Profile, e: &Event) {
    let cfg = config(rt, &p.id);
    if cfg["enabled"] != Value::Bool(true) {
        return;
    }
    let Some((kind, vars)) = vars(e) else { return };
    if cfg["events"][&kind]["on"] != Value::Bool(true) {
        return;
    }
    // Simulação de evento e evento de ensaio chegam marcados ao overlay.
    dispatch(rt, p, &kind, vars, e.rehearsal || e.simulated);
}

/// Evento de teste do Estúdio montado como um evento de verdade — simulado e
/// de ensaio — para entrar pelo funil da live (`submit → engine::process`).
/// `giftbulk` sai como `gift` com total > 1: o motor deriva o tipo, como a
/// Twitch entrega. Os dados espelham os exemplos de sempre, calculados depois
/// pelo mesmo `vars()` que o overlay recebe.
fn rehearsal_event(p: &Profile, kind: &str) -> Option<Event> {
    let (event_kind, user, message, data) = match kind {
        "follow" => ("follow", "NovoSeguidor", "", json!({"total": 1})),
        "subscription" => ("subscription", "UsuarioSub", "", json!({"total": 1, "cumulative_months": 1, "tier": "1000"})),
        "resub" => ("resub", "UsuarioFiel", "Melhor stream!", json!({"total": 1, "cumulative_months": 8, "tier": "1000"})),
        "gift" => ("gift", "Presenteado123", "", json!({"total": 1})),
        "giftbulk" => ("gift", "Generoso123", "", json!({"total": 10})),
        "cheer" => ("cheer", "AnaBits", "GG, que jogo!", json!({"bits": 500})),
        "raid" => ("raid", "RaidMan", "", json!({"viewers": 37})),
        _ => return None,
    };
    Some(Event {
        id: uuid::Uuid::new_v4().to_string(),
        profile_id: p.id.clone(),
        kind: event_kind.into(),
        user: user.into(),
        user_id: format!("ensaio-{kind}"),
        role: "everyone".into(),
        message: message.into(),
        data,
        simulated: true,
        rehearsal: true,
    })
}

/// Nome do evento como a pílula do Estúdio mostra (em minúsculas no aviso).
fn label(kind: &str) -> &'static str {
    match kind {
        "follow" => "seguidor",
        "subscription" => "novo sub",
        "resub" => "renovação",
        "gift" => "sub de presente",
        "giftbulk" => "presentes em lote",
        "cheer" => "bits",
        _ => "raid",
    }
}

/// Botão de teste do Estúdio: o evento de teste entra pelo funil da live
/// (`submit → engine::process → alerts::fire`) — a chave mestre e o
/// interruptor do evento valem igual à live, inclusive para a caixa de chat,
/// a explosão de emotes e os rótulos. Com algum deles desligado nada dispara
/// e o motivo volta para o Estúdio avisar; ligado, devolve as mesmas
/// variáveis que o overlay vai receber, para a prévia animar em paralelo.
pub async fn test(rt: &Runtime, p: &Profile, kind: &str) -> Result<Value, String> {
    if !KINDS.contains(&kind) {
        return Err("Evento de alerta inválido".into());
    }
    let cfg = config(rt, &p.id);
    if cfg["enabled"] != Value::Bool(true) {
        return Ok(json!({"fired": false, "reason": "Ligue os alertas no overlay: em live o alerta não tocaria."}));
    }
    if cfg["events"][kind]["on"] != Value::Bool(true) {
        return Ok(json!({"fired": false, "reason": format!("Ligue o alerta de {} no overlay: em live ele não tocaria.", label(kind))}));
    }
    let e = rehearsal_event(p, kind).ok_or("Evento de alerta inválido")?;
    let Some((_, vars)) = vars(&e) else { return Err("Evento de alerta inválido".into()) };
    rt.submit(e).await?;
    Ok(json!({"fired": true, "kind": kind, "vars": vars}))
}

/// Sons do perfil (kind "sound") para o seletor do Estúdio.
pub fn sounds(rt: &Runtime, p: &str) -> Value {
    crate::chat_extras::assets(rt, p)
        .into_iter()
        .filter(|a| a.kind == "sound")
        .map(|a| json!({"id": a.id, "name": a.name}))
        .collect::<Vec<_>>()
        .into()
}

/// Carrega o bytes de um som para a fonte do OBS (GET /asset?token&id).
pub fn sound_file(rt: &Runtime, p: &str, id: &str) -> Result<(Vec<u8>, &'static str), String> {
    let a = crate::chat_extras::asset(rt, p, id, "sound")?;
    let path = crate::chat_extras::audio_path(rt, p, &a)?;
    let meta = std::fs::metadata(&path).map_err(|_| "Som não encontrado".to_string())?;
    if !meta.is_file() || meta.len() > 5 * 1024 * 1024 {
        return Err("Som inválido ou maior que 5 MiB".into());
    }
    let bytes = std::fs::read(&path).map_err(|_| "Falha ao ler o som".to_string())?;
    let mime = match path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp3" => "audio/mpeg",
        _ => return Err("Formato de áudio inválido".into()),
    };
    Ok((bytes, mime))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> Profile {
        serde_json::from_value(json!({"id": uuid::Uuid::new_v4(), "name": "Teste", "platform": "twitch", "channel": "canal"})).unwrap()
    }
    fn e(p: &Profile, kind: &str, data: Value) -> Event {
        serde_json::from_value(json!({"id": "1", "profileId": p.id, "kind": kind, "user": "Ana", "userId": "1", "message": "bora", "data": data})).unwrap()
    }
    fn rt() -> (tempfile::TempDir, std::sync::Arc<Runtime>) {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().join("app")).unwrap();
        (dir, rt)
    }
    fn save(rt: &Runtime, p: &Profile, patch: Value) {
        let mut c = defaults();
        merge(&mut c, &patch);
        rt.db.set_module(&p.id, "alerts", &c).unwrap();
    }
    fn merge(a: &mut Value, b: &Value) {
        if let (Some(ao), Some(bo)) = (a.as_object_mut(), b.as_object()) {
            for (k, v) in bo {
                ao.entry(k.clone()).or_insert_with(|| v.clone());
                if v.is_object() {
                    merge(&mut ao[k], v);
                } else {
                    ao[k] = v.clone();
                }
            }
        }
    }

    #[test]
    fn defaults_cover_every_kind_and_validate() {
        let v = defaults();
        assert_eq!(KINDS.len(), 7);
        assert!(KINDS.iter().all(|k| v["events"][k]["on"] == Value::Bool(true)));
        assert!(valid_config(&v), "configuração padrão deve ser válida");
        assert_eq!(v["enabled"], Value::Bool(false), "desligada até o streamer ativar");
    }

    #[test]
    fn validation_rejects_out_of_range_and_unknown_fonts() {
        let mut bad = defaults();
        bad["duration"] = json!(2);
        assert!(!valid_config(&bad));
        let mut bad = defaults();
        bad["speed"] = json!(300);
        assert!(!valid_config(&bad));
        let mut bad = defaults();
        bad["posV"] = json!("side");
        assert!(!valid_config(&bad));
        let mut bad = defaults();
        bad["fontBig"] = json!("Comic Sans");
        assert!(!valid_config(&bad));
        let mut bad = defaults();
        bad["events"]["cheer"]["big"] = json!("x".repeat(161));
        assert!(!valid_config(&bad));
        let mut bad = defaults();
        bad["events"]["raid"] = json!({});
        assert!(!valid_config(&bad));
    }

    #[test]
    fn validation_accepts_empty_or_hex_event_tag_color() {
        let mut v = defaults();
        v["events"]["follow"]["color"] = json!("");
        assert!(valid_config(&v), "vazio = cor global do painel");
        v["events"]["follow"]["color"] = json!("#c6ff00");
        assert!(valid_config(&v), "cor por evento válida");
        v["events"]["follow"]["color"] = json!("verde!");
        assert!(!valid_config(&v), "caractere fora do formato é barrado");
        let mut v = defaults();
        v["events"]["follow"].as_object_mut().unwrap().remove("color");
        assert!(!valid_config(&v), "campo presente e sempre string");
    }

    #[test]
    fn event_mapping_extracts_amounts_and_splits_bulk_gifts() {
        let p = p();
        assert_eq!(vars(&e(&p, "follow", json!({}))).unwrap().0, "follow");
        let (k, v) = vars(&e(&p, "cheer", json!({"bits": 500}))).unwrap();
        assert_eq!((k.as_str(), v["amount"].as_i64().unwrap()), ("cheer", 500));
        let (k, v) = vars(&e(&p, "raid", json!({"viewers": 37}))).unwrap();
        assert_eq!((k.as_str(), v["amount"].as_i64().unwrap()), ("raid", 37));
        let (k, v) = vars(&e(&p, "resub", json!({"cumulative_months": 8, "tier": "2000"}))).unwrap();
        assert_eq!((k.as_str(), v["months"].as_i64().unwrap(), v["tier"].as_str().unwrap()), ("resub", 8, "TIER 2"));
        let (k, _) = vars(&e(&p, "gift", json!({"total": 1}))).unwrap();
        assert_eq!(k, "gift");
        let (k, v) = vars(&e(&p, "gift", json!({"total": 10}))).unwrap();
        assert_eq!((k.as_str(), v["amount"].as_i64().unwrap()), ("giftbulk", 10));
        assert!(vars(&e(&p, "chat", json!({}))).is_none());
    }

    #[tokio::test]
    async fn fire_respects_master_switch_and_event_toggle() {
        let (_dir, rt) = rt();
        let p = p();
        rt.db.save_profile(&p).unwrap();
        let mut rx = rt.broadcast.subscribe();
        let follow = e(&p, "follow", json!({}));
        fire(&rt, &p, &follow);
        assert!(!std::iter::from_fn(|| rx.try_recv().ok()).any(|v| v["type"] == "alert"), "sem config salva nada toca");
        save(&rt, &p, json!({"enabled": true}));
        fire(&rt, &p, &follow);
        let alert: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).filter(|v| v["type"] == "alert").collect();
        assert_eq!(alert.len(), 1);
        assert_eq!(alert[0]["payload"]["kind"], "follow");
        assert_eq!(alert[0]["payload"]["vars"]["name"], "Ana");
        assert_eq!(alert[0]["payload"]["platform"], "twitch");
        save(&rt, &p, json!({"events": {"follow": {"on": false}}}));
        fire(&rt, &p, &follow);
        assert!(!std::iter::from_fn(|| rx.try_recv().ok()).any(|v| v["type"] == "alert"), "evento desligado não toca");
    }

    /// Espera o worker entregar um alerta pelo broadcast (o teste entra na fila
    /// do engine como qualquer evento real).
    async fn wait_alert(rx: &mut tokio::sync::broadcast::Receiver<Value>) -> Value {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let got: Vec<Value> = std::iter::from_fn(|| rx.try_recv().ok())
                .filter(|v| v["type"] == "alert")
                .collect();
            if let Some(v) = got.into_iter().next() { return v }
            assert!(tokio::time::Instant::now() < deadline, "o worker não entregou o alerta do ensaio");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn test_button_follows_the_same_switches_as_live() {
        let (_dir, rt) = rt();
        let p = p();
        rt.db.save_profile(&p).unwrap();
        let mut rx = rt.broadcast.subscribe();

        // Padrão (tudo desligado): nada dispara e o motivo aponta a chave mestre.
        let out = test(&rt, &p, "cheer").await.unwrap();
        assert_eq!(out["fired"], false);
        assert!(out["reason"].as_str().unwrap().contains("Ligue os alertas"));
        assert!(!std::iter::from_fn(|| rx.try_recv().ok()).any(|v| v["type"] == "alert"), "com a chave desligada o funil não chega ao alerta");

        // Mestre ligada: o teste vira evento de verdade e o alerta sai marcado de ensaio.
        save(&rt, &p, json!({"enabled": true}));
        let out = test(&rt, &p, "cheer").await.unwrap();
        assert_eq!(out["fired"], true);
        let alert = wait_alert(&mut rx).await;
        assert_eq!(alert["payload"]["kind"], "cheer");
        assert_eq!(alert["payload"]["rehearsal"], true, "o overlay recebe a marca de ensaio");
        assert_eq!(alert["payload"]["vars"]["name"], "AnaBits");
        assert_eq!(out["vars"], alert["payload"]["vars"], "a prévia anima com as mesmas variáveis do OBS");

        // Evento específico desligado: volta o motivo do evento e nada toca.
        save(&rt, &p, json!({"enabled": true, "events": {"cheer": {"on": false}}}));
        let out = test(&rt, &p, "cheer").await.unwrap();
        assert_eq!(out["fired"], false);
        assert!(out["reason"].as_str().unwrap().contains("Ligue o alerta de bits"));

        // Kind inválido continua barrado.
        assert!(test(&rt, &p, "nope").await.is_err());
    }

    #[tokio::test]
    async fn message_and_sounds_shapes() {
        let (_dir, rt) = rt();
        let p = p();
        rt.db.save_profile(&p).unwrap();
        let m = message(&rt, &p.id);
        assert_eq!(m["type"], "alerts");
        assert_eq!(m["payload"]["profileId"], p.id);
        assert!(m["payload"]["config"].is_null(), "sem config salva o overlay recebe objeto vazio e o motor aplica os padrões");
        save(&rt, &p, json!({}));
        assert_eq!(message(&rt, &p.id)["payload"]["config"]["duration"], 8);
        assert_eq!(sounds(&rt, &p.id), json!([]));
    }

    #[tokio::test]
    async fn sound_file_serves_registered_wav() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().join("app")).unwrap();
        let p = p();
        rt.db.save_profile(&p).unwrap();
        let file = dir.path().join("jingle.wav");
        std::fs::write(&file, b"RIFF0000WAVEfmt ").unwrap();
        let id = crate::chat_extras::operation(
            &rt,
            &p.id,
            "chatExtras.import",
            &json!({"path": file, "kind": "sound"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
        let (bytes, mime) = sound_file(&rt, &p.id, &id).unwrap();
        assert_eq!(mime, "audio/wav");
        assert!(bytes.starts_with(b"RIFF"));
        assert!(sound_file(&rt, &p.id, "outro").is_err());
    }

    #[tokio::test]
    async fn som_do_alerta_sai_sempre_na_linha_do_aplicativo() {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().join("app")).unwrap();
        let p = p();
        rt.db.save_profile(&p).unwrap();
        let file = dir.path().join("jingle.wav");
        std::fs::write(&file, b"RIFF0000WAVEfmt ").unwrap();
        let id = crate::chat_extras::operation(&rt, &p.id, "chatExtras.import", &json!({"path": file, "kind": "sound"}))
            .unwrap()["id"].as_str().unwrap().to_string();
        save(&rt, &p, json!({"enabled": true, "volume": 70, "events": {"follow": {"on": true, "sound": id}}}));
        rt.db.set_module(&p.id, "audio", &json!({"device": "cable-3"})).unwrap();
        let mut rx = rt.broadcast.subscribe();
        fire(&rt, &p, &e(&p, "follow", json!({})));
        let line: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).filter(|v| v["type"] == "alert-sound").collect();
        assert_eq!(line.len(), 1, "o alerta toca na linha");
        let src = line[0]["payload"]["src"].as_str().unwrap();
        assert!(src.contains("/asset?token="), "som servido pela API local: {src}");
        assert!(src.contains(&id));
        assert_eq!(line[0]["payload"]["deviceId"], "cable-3", "aparelho vem da aba Áudio");
        assert_eq!(
            line[0]["payload"]["volume"], 0.7,
            "volume salvo 0–100 chega ao player como 0–1 (senão Math.min(1,…) engolia tudo acima de 1 e tocava 100%)"
        );
        // Sem som escolhido no alerta não há linha — o painel continua visual.
        save(&rt, &p, json!({"events": {"follow": {"sound": ""}}}));
        let mut rx = rt.broadcast.subscribe();
        fire(&rt, &p, &e(&p, "follow", json!({})));
        assert!(!std::iter::from_fn(|| rx.try_recv().ok()).any(|v| v["type"] == "alert-sound"), "sem som não há linha");
    }
}
