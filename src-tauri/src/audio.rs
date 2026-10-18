//! Áudio do BotLive: **uma saída só** para todos os sons do bot — a fonte de
//! navegador "Áudio do BotLive" (http://127.0.0.1:PORT/audio) que você adiciona
//! no OBS. Espectador, fluxo (comandos, timers, automações e resgates), alertas,
//! meta e texto para voz passam todos por este único `play`.
//!
//! Enquanto a fonte do OBS está conectada e reivindicou a reprodução, o evento
//! vai só para ela (o aplicativo não toca nada ao mesmo tempo). Sem fonte
//! conectada, o aplicativo toca o mesmo evento na saída padrão do Windows —
//! nunca os dois ao mesmo tempo. Os volumes individuais continuam em cada
//! tela; aqui só se decide **para onde** o som vai.
use crate::engine::Runtime;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::Sender;

// ---- Configuração antiga (aparelho de saída) ----
// Continua legível para não perder dados de quem já usava a tela Áudio; o
// aparelho não é mais usado por nenhum som do bot.

pub fn config(rt: &Runtime, p: &str) -> Value {
    let v = rt.db.module(p, "audio");
    if v.is_null() {
        // Quem já tinha um destino de som guardado em Respostas e sons não
        // perde a escolha guardada: a primeira leitura carrega o aparelho antigo.
        let legacy: crate::chat_extras::Config = rt.db.module_cfg(p, "chatExtras");
        return json!({"device": legacy.audio_device_id});
    }
    json!({"device": v["device"].as_str().unwrap_or("")})
}

pub fn device(rt: &Runtime, p: &str) -> String {
    config(rt, p)["device"].as_str().unwrap_or("").to_owned()
}

pub fn valid_config(v: &Value) -> bool {
    v["device"].as_str().is_some_and(|s| s.len() <= 512)
}

#[cfg(test)]
pub fn defaults() -> Value {
    json!({"device": ""})
}

// ---- Registro de fontes de áudio conectadas (uma só reproduz) ----

struct Entry {
    id: u64,
    tx: Sender<Value>,
}

/// Uma entrada por conexão da fonte "Áudio do BotLive". A **primeira** da
/// lista é a dona da reprodução; as demais ficam em espera (standby) e não
/// recebem eventos — assim duas fontes abertas nunca tocam o mesmo som em
/// duplicidade. Ao sair a dona, a seguinte assume.
#[derive(Default)]
pub struct Sinks {
    clients: Mutex<HashMap<String, Vec<Entry>>>,
    next: Mutex<u64>,
}

fn emit_state(rt: &Runtime, p: &str) {
    let s = rt.audio.state(p);
    let _ = rt.emit(
        "audio.state",
        json!({"profileId": p, "clients": s["clients"], "claimed": s["claimed"]}),
    );
}

impl Sinks {
    /// Registra a conexão e devolve (id, se é a dona). A doação/standby é
    /// comunicada por mensagem `audio.claim` / `audio.standby` na própria
    /// conexão, feita pelo chamador.
    pub fn register(&self, p: &str, tx: Sender<Value>) -> (u64, bool) {
        let mut map = self.clients.lock().unwrap();
        let list = map.entry(p.to_owned()).or_default();
        let claimed = list.is_empty();
        let mut id = self.next.lock().unwrap();
        *id += 1;
        let id = *id;
        list.push(Entry { id, tx });
        (id, claimed)
    }

    pub fn unregister(&self, p: &str, id: u64) {
        let mut promoted = false;
        {
            let mut map = self.clients.lock().unwrap();
            if let Some(list) = map.get_mut(p) {
                let was_first = list.first().map(|e| e.id) == Some(id);
                list.retain(|e| e.id != id);
                if list.is_empty() {
                    map.remove(p);
                } else if was_first {
                    // A dona saiu: a próxima assume a reprodução.
                    promoted = true;
                }
            } else {
                return;
            }
        }
        if promoted {
            if let Some(tx) = self.claimed(p) {
                let _ = tx.try_send(json!({"type": "audio.claim"}));
            }
        }
    }

    /// Transmissora da conexão que está com a reprodução (primeira da lista).
    pub fn claimed(&self, p: &str) -> Option<Sender<Value>> {
        let map = self.clients.lock().unwrap();
        map.get(p).and_then(|l| l.first()).map(|e| e.tx.clone())
    }

    pub fn state(&self, p: &str) -> Value {
        let map = self.clients.lock().unwrap();
        let list = map.get(p).map(|l| l.len()).unwrap_or(0);
        json!({"clients": list, "claimed": list > 0})
    }

    pub fn clients(&self, p: &str) -> usize {
        let map = self.clients.lock().unwrap();
        map.get(p).map(|l| l.len()).unwrap_or(0)
    }

    /// Transmissora de uma conexão específica (para avisar o papel na hora do
    /// registro).
    pub fn tx_of(&self, p: &str, id: u64) -> Option<Sender<Value>> {
        let map = self.clients.lock().unwrap();
        map.get(p).and_then(|l| l.iter().find(|e| e.id == id)).map(|e| e.tx.clone())
    }

    /// Desliga todas as conexões do perfil (usado em testes).
    #[cfg(test)]
    pub fn ids(&self, p: &str) -> Vec<u64> {
        let map = self.clients.lock().unwrap();
        map.get(p).map(|l| l.iter().map(|e| e.id).collect()).unwrap_or_default()
    }
}

pub fn register(rt: &Runtime, p: &str, tx: Sender<Value>) -> (u64, bool) {
    let (id, claimed) = rt.audio.register(p, tx);
    // Avisamos o papel na própria conexão: a dona recebe claim (e passa a
    // receber os sons); as demais recebem standby e ficam paradas.
    let hello = if claimed {
        json!({"type": "audio.claim", "payload": {"profileId": p}})
    } else {
        json!({"type": "audio.standby", "payload": {"profileId": p, "reason": "duplicate"}})
    };
    if let Some(tx) = rt.audio.tx_of(p, id) {
        let _ = tx.try_send(hello);
    }
    emit_state(rt, p);
    (id, claimed)
}

pub fn unregister(rt: &Runtime, p: &str, id: u64) {
    rt.audio.unregister(p, id);
    emit_state(rt, p);
}

pub fn state(rt: &Runtime, p: &str) -> Value {
    rt.audio.state(p)
}

// ---- Reprodução ----

/// Um som pronto para sair: endereço + volume. O destino é decidido aqui,
/// uma vez só: fonte do OBS conectada? vai para lá. Senão, o aplicativo toca.
pub struct Req<'a> {
    pub kind: &'a str, // sound | alert | tts | test | preview
    pub src: String,
    pub volume: f64,
    pub nickname: Option<&'a str>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Envia o som pela saída única. Devolve o payload emitido (com o `id` que a
/// fonte confirma por `audio.ack`).
pub fn play(rt: &Runtime, p: &str, req: Req) -> Value {
    let volume = if req.volume.is_finite() { req.volume.clamp(0.0, 1.0) } else { 1.0 };
    let payload = json!({
        "profileId": p,
        "id": uuid::Uuid::new_v4().simple().to_string(),
        "kind": req.kind,
        "src": req.src,
        "volume": volume,
        "nickname": req.nickname,
        "issuedAt": now_ms(),
    });
    // Fonte do OBS reivindicada: só ela recebe. O aplicativo ouve o mesmo
    // evento com sink="obs" apenas para diagnóstico — nunca toca junto.
    let sink = if let Some(tx) = rt.audio.claimed(p) {
        if tx.try_send(json!({"type": "audio.play", "payload": payload.clone()})).is_err() {
            let _ = rt.log(
                p,
                "audio",
                "A fonte de áudio do OBS não está respondendo; o som foi descartado. Reabra a fonte no OBS.",
                "error",
            );
        }
        "obs"
    } else {
        "local"
    };
    let mut out = payload;
    out["sink"] = json!(sink);
    let _ = rt.emit("audio.play", out.clone());
    out
}

/// Endereço http de um arquivo de som do perfil (servido por /asset com a
/// chave de overlay — nunca caminho arbitrário).
pub fn asset_src(rt: &Runtime, p: &str, asset: &str) -> Result<String, String> {
    let info = crate::livepix::overlay_info(rt, p)?;
    Ok(format!(
        "http://127.0.0.1:{}/asset?token={}&id={}",
        info["port"], info["token"], asset
    ))
}

/// Som da biblioteca pela saída única (valida que o arquivo é do perfil).
pub fn preview(rt: &Runtime, p: &str, asset: &str, volume: f64) -> Result<Value, String> {
    crate::chat_extras::asset(rt, p, asset, "sound")?;
    let src = asset_src(rt, p, asset)?;
    Ok(play(rt, p, Req { kind: "preview", src, volume, nickname: None }))
}

/// Para todos os sons imediatamente: limpa a fila da fonte do OBS (quando
/// houver) e a fila local do aplicativo. Devolve se a fonte do OBS recebeu.
pub fn stop(rt: &Runtime, p: &str) -> bool {
    let claimed = rt.audio.claimed(p).is_some();
    if let Some(tx) = rt.audio.claimed(p) {
        let _ = tx.try_send(json!({"type": "audio.stop"}));
    }
    let _ = rt.emit("audio.stopLocal", json!({"profileId": p}));
    claimed
}

/// Confirmação de reprodução vinda da fonte do OBS (via WebSocket) ou do
/// próprio aplicativo (reprodução local): "som enviado" ≠ "tocando".
pub fn ack(rt: &Runtime, p: &str, id: &str, phase: &str, detail: Option<&str>) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("Identificador de som inválido".into());
    }
    if !matches!(phase, "started" | "done" | "error" | "stopped") {
        return Err("Estado de reprodução inválido".into());
    }
    let detail = detail.map(|d| d.chars().take(200).collect::<String>());
    let _ = rt.emit(
        "audio.feedback",
        json!({"profileId": p, "id": id, "phase": phase, "detail": detail}),
    );
    Ok(())
}

/// Bipe (WAV 8 kHz mono) gerado aqui: o teste de áudio não depende de arquivo.
pub fn beep_data_uri() -> String {
    let (hz, seconds) = (440.0f64, 0.35);
    let rate = 8000usize;
    let n = (rate as f64 * seconds) as usize;
    let mut bytes = vec![0u8; 44 + n * 2];
    let view = {
        let b: &mut [u8] = &mut bytes;
        b
    };
    view[0..4].copy_from_slice(b"RIFF");
    view[4..8].copy_from_slice(&((36 + n * 2) as u32).to_le_bytes());
    view[8..12].copy_from_slice(b"WAVE");
    view[12..16].copy_from_slice(b"fmt ");
    view[16..20].copy_from_slice(&16u32.to_le_bytes());
    view[20..22].copy_from_slice(&1u16.to_le_bytes());
    view[22..24].copy_from_slice(&1u16.to_le_bytes());
    view[24..28].copy_from_slice(&(rate as u32).to_le_bytes());
    view[28..32].copy_from_slice(&((rate * 2) as u32).to_le_bytes());
    view[32..34].copy_from_slice(&2u16.to_le_bytes());
    view[34..36].copy_from_slice(&16u16.to_le_bytes());
    view[36..40].copy_from_slice(b"data");
    view[40..44].copy_from_slice(&((n * 2) as u32).to_le_bytes());
    for i in 0..n {
        let t = i as f64 / rate as f64;
        let env = (1.0 - i as f64 / n as f64).min(1.0);
        let v = (t * hz * 2.0 * std::f64::consts::PI).sin() * 0.4 * env;
        let s = (v * i16::MAX as f64) as i16;
        bytes[44 + i * 2..44 + i * 2 + 2].copy_from_slice(&s.to_le_bytes());
    }
    use base64::Engine;
    format!(
        "data:audio/wav;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    )
}

/// Bipe do botão "Testar áudio" — mesmo caminho de qualquer som.
pub fn test(rt: &Runtime, p: &str) -> Value {
    play(rt, p, Req { kind: "test", src: beep_data_uri(), volume: 0.8, nickname: None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn rt() -> (tempfile::TempDir, std::sync::Arc<Runtime>) {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        (dir, rt)
    }

    fn profile() -> crate::model::Profile {
        serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal"})).unwrap()
    }

    #[tokio::test]
    async fn config_vazia_hereda_o_aparelho_antigo() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        // Sem nada salvo, o destino antigo era a saída padrão do Windows.
        assert_eq!(crate::audio::config(&rt, &p.id)["device"], "");
        rt.db.set_module(&p.id, "chatExtras", &json!({"audioDeviceId":"cable-1"})).unwrap();
        // Migração: quem já escolheu saída em Respostas e sons mantém o dado.
        assert_eq!(crate::audio::device(&rt, &p.id), "cable-1");
        rt.db.set_module(&p.id, "audio", &json!({"device":"hdmi-2"})).unwrap();
        assert_eq!(crate::audio::device(&rt, &p.id), "hdmi-2");
    }

    #[tokio::test]
    async fn config_antiga_com_interruptores_vale_somente_no_aparelho() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        // Formato da 0.6 (seis interruptores): o aparelho continua legível e o
        // resto é ignorado — não existe mais rota por navegador.
        rt.db.set_module(&p.id, "audio", &json!({"peopleLine":true,"peopleBrowser":false,"flowLine":true,"flowBrowser":true,"alertLine":false,"alertBrowser":true,"device":"cable-9"})).unwrap();
        assert_eq!(crate::audio::config(&rt, &p.id)["device"], "cable-9");
        assert!(crate::audio::config(&rt, &p.id).get("flowBrowser").is_none(), "os interruptores antigos não sobrevivem à leitura");
    }

    #[test]
    fn valid_config_só_aceita_aparelho_curto() {
        assert!(valid_config(&defaults()));
        let mut v = json!({"device":"x".repeat(600)});
        assert!(!valid_config(&v), "aparelho gigante barra");
        v["device"] = json!(12);
        assert!(!valid_config(&v), "aparelho precisa ser texto");
        assert!(!valid_config(&json!({})), "sem aparelho não é configuração válida");
    }

    #[tokio::test]
    async fn sem_fonte_o_evento_vai_para_o_aplicativo_local() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let mut rx = rt.broadcast.subscribe();
        let payload = play(&rt, &p.id, Req { kind: "sound", src: "http://x/y.wav".into(), volume: 2.0, nickname: Some("Ana") });
        let seen: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        let evt = seen.iter().find(|v| v["type"] == "audio.play").expect("evento único de áudio");
        assert_eq!(evt["payload"]["sink"], "local", "sem fonte no OBS toca local");
        assert_eq!(evt["payload"]["volume"], 1.0, "volume acima de 1 é cortado");
        assert_eq!(evt["payload"]["nickname"], "Ana");
        assert_eq!(evt["payload"]["id"], payload["id"]);
        assert!(evt["payload"]["issuedAt"].as_u64().unwrap() > 0, "evento carrega a hora para descartar atrasos");
        assert!(!seen.iter().any(|v| v["type"] == "viewer-sound"), "o evento antigo não existe mais");
    }

    #[tokio::test]
    async fn fonte_conectada_recebe_so_elas_e_aplicativo_nao_toca() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let (tx, mut rx) = mpsc::channel::<Value>(16);
        let (id, claimed) = register(&rt, &p.id, tx);
        assert!(claimed, "a primeira fonte é a dona");
        assert_eq!(rx.try_recv().unwrap()["type"], "audio.claim", "a dona é avisada antes de qualquer som");
        assert_eq!(state(&rt, &p.id)["clients"], 1);
        let mut app = rt.broadcast.subscribe();
        let payload = play(&rt, &p.id, Req { kind: "alert", src: "http://x/a.wav".into(), volume: 0.5, nickname: None });
        let to_sink = rx.try_recv().unwrap();
        assert_eq!(to_sink["type"], "audio.play");
        assert_eq!(to_sink["payload"]["id"], payload["id"]);
        let seen: Vec<_> = std::iter::from_fn(|| app.try_recv().ok()).collect();
        let evt = seen.iter().find(|v| v["type"] == "audio.play").unwrap();
        assert_eq!(evt["payload"]["sink"], "obs", "com fonte conectada o app não toca");
        unregister(&rt, &p.id, id);
        assert_eq!(state(&rt, &p.id)["clients"], 0);
    }

    #[tokio::test]
    async fn segunda_fonte_fica_em_espera_e_assume_quando_a_primeira_sai() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let (tx1, mut rx1) = mpsc::channel::<Value>(16);
        let (tx2, mut rx2) = mpsc::channel::<Value>(16);
        let (id1, claimed) = register(&rt, &p.id, tx1);
        assert!(claimed);
        assert_eq!(rx1.try_recv().unwrap()["type"], "audio.claim", "a dona é avisada");
        let (_id2, claimed2) = register(&rt, &p.id, tx2);
        assert!(!claimed2, "a segunda fonte não vira dona");
        assert_eq!(state(&rt, &p.id)["clients"], 2, "diagnóstico enxerga as duas");
        let standby = rx2.try_recv().unwrap();
        assert_eq!(standby["type"], "audio.standby", "standby é avisada, não toca no escuro");
        play(&rt, &p.id, Req { kind: "sound", src: "http://x/y.wav".into(), volume: 1.0, nickname: None });
        assert!(rx1.try_recv().unwrap()["type"] == "audio.play");
        assert!(rx2.try_recv().is_err(), "a em espera não recebe som (nada toca em duplicidade)");
        unregister(&rt, &p.id, id1);
        let claim = rx2.try_recv().unwrap();
        assert_eq!(claim["type"], "audio.claim", "a espera assume a reprodução");
    }

    #[tokio::test]
    async fn stop_avisa_a_fonte_e_limpa_a_fila_local() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let (tx, mut rx) = mpsc::channel::<Value>(16);
        register(&rt, &p.id, tx);
        assert_eq!(rx.try_recv().unwrap()["type"], "audio.claim", "a dona começa avisada");
        let mut app = rt.broadcast.subscribe();
        assert!(stop(&rt, &p.id), "fonte do OBS recebeu a parada");
        assert_eq!(rx.try_recv().unwrap()["type"], "audio.stop");
        let seen: Vec<_> = std::iter::from_fn(|| app.try_recv().ok()).collect();
        assert!(seen.iter().any(|v| v["type"] == "audio.stopLocal"), "fila local também limpa");
        unregister_all(&rt, &p.id);
        assert!(!stop(&rt, &p.id), "sem fonte não há o que o OBS parar");
    }

    /// Desliga todas as conexões do perfil (utilitário de teste).
    fn unregister_all(rt: &std::sync::Arc<Runtime>, p: &str) {
        for id in rt.audio.ids(p) {
            unregister(rt, p, id);
        }
    }

    #[tokio::test]
    async fn ack_validado_vai_para_o_aplicativo_como_feedback() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let mut rx = rt.broadcast.subscribe();
        ack(&rt, &p.id, "abc-123", "started", None).unwrap();
        let seen: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        let fb = seen.iter().find(|v| v["type"] == "audio.feedback").unwrap();
        assert_eq!(fb["payload"]["phase"], "started");
        assert!(ack(&rt, &p.id, &"x".repeat(80), "started", None).is_err(), "id longo demais");
        assert!(ack(&rt, &p.id, "../../etc", "started", None).is_err(), "id com caminho barrado");
        assert!(ack(&rt, &p.id, "ok", "ouviu", None).is_err(), "fase fora da lista barrada");
        ack(&rt, &p.id, "ok", "error", Some(&"e".repeat(400))).unwrap();
        assert!(ack(&rt, &p.id, "", "error", None).is_err(), "id vazio barrado");
    }

    #[test]
    fn bipe_e_um_wav_valido() {
        let uri = beep_data_uri();
        assert!(uri.starts_with("data:audio/wav;base64,"));
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(&uri["data:audio/wav;base64,".len()..]).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert!(bytes.len() > 1000, "bipe tem áudio de verdade");
    }
}
