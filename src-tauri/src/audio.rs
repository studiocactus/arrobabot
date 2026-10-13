//! Áudio do BotLive: um lugar só para decidir por onde saem os sons do bot.
//! Todos os sons — espectador, fluxo (comandos, timers, automações e resgates),
//! alertas e voz — saem pela **linha de áudio** do aplicativo: o aparelho de
//! saída escolhido aqui (ex.: CABLE Input do VB-Cable) que você captura no OBS
//! como uma única fonte de áudio, com um fader para o bot inteiro. A fonte de
//! navegador do overlay ficou só visual, sem som.
use crate::engine::Runtime;
use serde_json::{json, Value};

pub fn config(rt: &Runtime, p: &str) -> Value {
    let v = rt.db.module(p, "audio");
    if v.is_null() {
        // Quem já tinha um destino de som guardado em Respostas e sons não
        // perde a escolha: a primeira leitura carrega o aparelho antigo.
        let legacy: crate::chat_extras::Config = rt.db.module_cfg(p, "chatExtras");
        return json!({"device": legacy.audio_device_id});
    }
    // Configurações antigas (com interruptores por origem) valem só no aparelho.
    json!({"device": v["device"].as_str().unwrap_or("")})
}

pub fn device(rt: &Runtime, p: &str) -> String {
    config(rt, p)["device"].as_str().unwrap_or("").to_owned()
}

pub fn valid_config(v: &Value) -> bool {
    v["device"].as_str().is_some_and(|s| s.len() <= 512)
}

/// Configuração padrão: aparelho vazio (saída padrão do Windows). Espelhada em
/// src/audio.ts.
#[cfg(test)]
pub fn defaults() -> Value {
    json!({"device": ""})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> (tempfile::TempDir, std::sync::Arc<Runtime>) {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        (dir, rt)
    }

    #[tokio::test]
    async fn config_vazia_hereda_o_aparelho_antigo() {
        let (_dir, rt) = rt();
        let p: crate::model::Profile =
            serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal"})).unwrap();
        rt.db.save_profile(&p).unwrap();
        // Sem nada salvo, o destino é a saída padrão do Windows.
        assert_eq!(crate::audio::config(&rt, &p.id)["device"], "");
        rt.db.set_module(&p.id, "chatExtras", &json!({"audioDeviceId":"cable-1"})).unwrap();
        // Migração: quem já escolheu saída em Respostas e sons mantém.
        assert_eq!(crate::audio::device(&rt, &p.id), "cable-1");
        // Salvou na Central, a escolha nova vale sozinha.
        rt.db.set_module(&p.id, "audio", &json!({"device":"hdmi-2"})).unwrap();
        assert_eq!(crate::audio::device(&rt, &p.id), "hdmi-2");
    }

    #[tokio::test]
    async fn config_antiga_com_interruptores_vale_somente_no_aparelho() {
        let (_dir, rt) = rt();
        let p: crate::model::Profile =
            serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal"})).unwrap();
        rt.db.save_profile(&p).unwrap();
        // Formato da 0.6 (seis interruptores): o aparelho continua valendo e o
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
}
