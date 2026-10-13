//! Central de Áudio: um lugar só para decidir para onde vai cada som do bot.
//! Duas saídas possíveis por origem — a **linha de áudio** (a saída que o
//! aplicativo toca, que você aponta para um cabo virtual como o VB-Cable e
//! captura no OBS) e o **navegador** (a fonte do overlay, que o OBS toca
//! dentro do painel de fontes). A Voz da live continua sendo a linha do
//! microfone (cpal) e aqui só expomos a seleção para ficar tudo junto.
use crate::engine::Runtime;
use serde_json::{json, Value};

fn line_key(origin: &str) -> &'static str {
    match origin {
        "flow" => "flowLine",
        _ => "peopleLine",
    }
}
fn browser_key(origin: &str) -> &'static str {
    match origin {
        "flow" => "flowBrowser",
        _ => "peopleBrowser",
    }
}

pub fn config(rt: &Runtime, p: &str) -> Value {
    let v = rt.db.module(p, "audio");
    if v.is_null() {
        // Quem já tinha um destino de som guardado em Respostas e sons não
        // perde a escolha: a primeira leitura carrega o aparelho antigo.
        let legacy: crate::chat_extras::Config = rt.db.module_cfg(p, "chatExtras");
        return json!({
            "peopleLine": true, "peopleBrowser": false,
            "flowLine": true, "flowBrowser": false,
            "alertLine": false, "alertBrowser": true,
            "device": legacy.audio_device_id,
        });
    }
    v
}

/// Dispositivo de saída (setSinkId do navegador) escolhido na Central.
pub fn device(rt: &Runtime, p: &str) -> String {
    config(rt, p)["device"].as_str().unwrap_or("").to_owned()
}

/// Rotas de uma origem de som: linha (aplicativo) e navegador (overlay).
pub fn routes(rt: &Runtime, p: &str, origin: &str) -> Value {
    let c = config(rt, p);
    json!({
        "line": c[line_key(origin)].as_bool().unwrap_or(false),
        "browser": c[browser_key(origin)].as_bool().unwrap_or(false),
        "device": c["device"].as_str().unwrap_or(""),
    })
}

/// Os alertas têm interruptores próprios: o painel continua no navegador (a
/// fonte lê `alertBrowser` no snapshot `audio.config`) e o som pode ir também
/// (ou só) para a linha do aplicativo.
pub fn alerts_line(rt: &Runtime, p: &str) -> bool {
    config(rt, p)["alertLine"].as_bool().unwrap_or(false)
}

pub fn valid_config(v: &Value) -> bool {
    ["peopleLine", "peopleBrowser", "flowLine", "flowBrowser", "alertLine", "alertBrowser"]
        .iter()
        .all(|k| v[k].is_boolean())
        && v["device"].as_str().is_some_and(|s| s.len() <= 512)
}

/// Mensagem inicial para a fonte do OBS: quais origens passam pelo navegador.
pub fn message(rt: &Runtime, pid: &str) -> Value {
    json!({"type": "audio.config", "payload": {"profileId": pid, "config": config(rt, pid)}})
}

/// Configuração padrão: sons de espectador e de fluxo na linha (como
/// sempre), sons dos alertas no navegador (como sempre). Espelhada em
/// src/audio.ts.
#[cfg(test)]
pub fn defaults() -> Value {
    json!({
        "peopleLine": true, "peopleBrowser": false,
        "flowLine": true, "flowBrowser": false,
        "alertLine": false, "alertBrowser": true,
        "device": "",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> (tempfile::TempDir, std::sync::Arc<Runtime>) {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        (dir, rt)
    }

    #[test]
    fn padrao_valido_e_campos_estranhos_barram() {
        assert!(valid_config(&defaults()));
        let mut v = defaults();
        v["flowLine"] = json!("sim");
        assert!(!valid_config(&v), "interruptor precisa ser booleano");
        let mut v = defaults();
        v["device"] = json!("x".repeat(600));
        assert!(!valid_config(&v), "dispositivo gigante barra");
        let mut v = defaults();
        v.as_object_mut().unwrap().remove("alertBrowser");
        assert!(!valid_config(&v), "chave ausente barra");
    }

    #[tokio::test]
    async fn config_vazia_hereda_o_dispositivo_e_as_rotas_tem_origem() {
        let (_dir, rt) = rt();
        let p: crate::model::Profile =
            serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal"})).unwrap();
        rt.db.save_profile(&p).unwrap();
        // Sem nada salvo, o destino é o padrão.
        assert_eq!(crate::audio::config(&rt, &p.id)["device"], "");
        rt.db.set_module(&p.id, "chatExtras", &json!({"audioDeviceId":"cable-1"})).unwrap();
        // Migração: quem já escolheu saída em Respostas e sons mantém.
        assert_eq!(crate::audio::config(&rt, &p.id)["device"], "cable-1");
        // Padrão: sons na linha, alertas no navegador.
        assert_eq!(crate::audio::routes(&rt, &p.id, "people")["line"], true);
        assert_eq!(crate::audio::routes(&rt, &p.id, "people")["browser"], false);
        assert_eq!(crate::audio::routes(&rt, &p.id, "flow")["line"], true);
        assert!(!crate::audio::alerts_line(&rt, &p.id));
        assert_eq!(crate::audio::config(&rt, &p.id)["alertBrowser"], true);
        // Salvou na Central, a escolha nova vale sozinha.
        let mut v = defaults();
        v["device"] = json!("hdmi-2");
        v["flowBrowser"] = json!(true);
        rt.db.set_module(&p.id, "audio", &v).unwrap();
        assert_eq!(crate::audio::device(&rt, &p.id), "hdmi-2");
        assert_eq!(crate::audio::routes(&rt, &p.id, "flow")["browser"], true);
        assert_eq!(crate::audio::routes(&rt, &p.id, "people")["browser"], false);
    }

    #[test]
    fn valida_direto_no_caminho_do_comando() {
        // module.config salva o valor cru: a validação é a única barreira.
        let mut v = defaults();
        v["peopleLine"] = json!("on");
        assert!(!valid_config(&v));
        assert!(valid_config(&defaults()));
    }
}
