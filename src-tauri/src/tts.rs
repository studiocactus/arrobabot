//! Texto para voz gerado **nesta máquina** (SAPI do Windows) e entregue pela
//! mesma saída dos demais sons: a fonte "Áudio do BotLive" do OBS.
//!
//! Nenhum texto sai do computador nem passa por serviço pago: a fala é
//! sintetizada pelo Windows e vira um WAV no cache do perfil
//! (`media/<perfil>/tts/`), servido por `/tts` com a chave de overlay do
//! próprio perfil. O aplicativo e a fonte do OBS tocam o mesmo arquivo — o TTS
//! não é mais um caminho separado preso à saída padrão do Windows.
use crate::engine::Runtime;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(windows)]
use tokio::process::Command;

/// Lista de vozes em cache (a lista não muda de minuto a minuto).
static VOICES: Mutex<Option<(Value, Instant)>> = Mutex::new(None);
const VOICES_TTL: Duration = Duration::from_secs(600);

/// Script fixo: voz, texto e saída entram por variáveis de ambiente apontando
/// para arquivos — o texto do usuário nunca é interpolado no comando
/// (nada de injeção de shell).
#[cfg(windows)]
const SYNTH: &str = r#"
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
$s.Rate = [int]$env:BOTLIVE_TTS_RATE
if ($env:BOTLIVE_TTS_VOICE -ne '') { $s.SelectVoice($env:BOTLIVE_TTS_VOICE) }
$text = [IO.File]::ReadAllText($env:BOTLIVE_TTS_TEXT, [Text.Encoding]::UTF8)
$s.SetOutputToWaveFile($env:BOTLIVE_TTS_WAV)
$s.Speak($text)
$s.Dispose()
"#;

#[cfg(windows)]
const LIST_VOICES: &str = r#"
Add-Type -AssemblyName System.Speech
$s = New-Object System.Speech.Synthesis.SpeechSynthesizer
$list = @()
foreach ($v in $s.GetInstalledVoices()) {
  if ($v.Enabled) {
    $i = $v.VoiceInfo
    $list += @{ name = $i.Name; culture = $i.Culture.Name; gender = $i.Gender.ToString() }
  }
}
$s.Dispose()
ConvertTo-Json -InputObject @($list) -Compress
"#;

/// Codifica em Base64 (UTF-16LE) para -EncodedCommand: o script chega inteiro
/// ao PowerShell sem passar por nenhuma camada de citação.
#[cfg(windows)]
fn encoded(script: &str) -> String {
    let utf16: Vec<u8> = script.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(utf16)
}

#[cfg(windows)]
async fn ps(script: &str, envs: &[(&str, String)], timeout: Duration) -> Result<std::process::Output, String> {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", &encoded(script)])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    // Sem janela: o sintetizador roda em silêncio atrás do aplicativo.
    cmd.creation_flags(0x0800_0000);
    let out = tokio::time::timeout(timeout, cmd.output())
        .await
        .map_err(|_| "A geração do áudio do texto demorou demais e foi interrompida. Reduza o limite de caracteres.".to_string())?
        .map_err(|_| "Não foi possível executar o sintetizador de voz do Windows".to_string())?;
    Ok(out)
}

/// Vozes instaladas neste Windows: [{name, culture, gender}]. Cache local.
pub async fn voices(rt: &Runtime) -> Result<Value, String> {
    let _ = rt;
    #[cfg(not(windows))]
    {
        return Err("Texto para voz disponível apenas no Windows".into());
    }
    #[cfg(windows)]
    {
        if let Some((cached, at)) = VOICES.lock().unwrap().clone() {
            if at.elapsed() < VOICES_TTL {
                return Ok(cached);
            }
        }
        let out = ps(LIST_VOICES, &[], Duration::from_secs(30)).await?;
        if !out.status.success() {
            return Err("Não foi possível listar as vozes do Windows. Confira se o sintetizador de voz está disponível.".into());
        }
        let raw = String::from_utf8_lossy(&out.stdout);
        let parsed: Value = serde_json::from_str(raw.trim()).map_err(|_| "As vozes do Windows voltaram em formato inesperado")?;
        let list = if parsed.is_array() { parsed } else { json!([parsed]) };
        let list = sanitize_voices(&list);
        *VOICES.lock().unwrap() = Some((list.clone(), Instant::now()));
        Ok(list)
    }
}

/// Só o que a tela precisa, e só vozes com nome curto.
fn sanitize_voices(list: &Value) -> Value {
    let arr = list.as_array().cloned().unwrap_or_default();
    let clean: Vec<Value> = arr
        .into_iter()
        .filter_map(|v| {
            let name = v["name"].as_str()?.trim().to_owned();
            if name.is_empty() || name.len() > 200 || name.contains('\0') {
                return None;
            }
            Some(json!({
                "name": name,
                "culture": v["culture"].as_str().unwrap_or(""),
                "gender": v["gender"].as_str().unwrap_or(""),
            }))
        })
        .collect();
    json!(clean)
}

/// Escolhe a voz do SAPI: nome exato, depois compatível com o voiceURI antigo
/// do navegador, depois a primeira em português, senão o padrão do Windows.
pub fn pick(voices: &Value, saved: &str) -> String {
    let list = voices.as_array().cloned().unwrap_or_default();
    let saved = saved.trim();
    if !saved.is_empty() {
        if list.iter().any(|v| v["name"].as_str() == Some(saved)) {
            return saved.to_owned();
        }
        // Migração: o valor antigo era voiceURI do navegador
        // ("Microsoft Maria - Portuguese (Brazil)").
        let head = saved.split(" - ").next().unwrap_or(saved).trim();
        if let Some(hit) = list.iter().find(|v| {
            let name = v["name"].as_str().unwrap_or("");
            !name.is_empty() && (saved.starts_with(name) || head.eq_ignore_ascii_case(name))
        }) {
            return hit["name"].as_str().unwrap_or("").to_owned();
        }
    }
    list.iter()
        .find(|v| v["culture"].as_str().unwrap_or("").starts_with("pt"))
        .map(|v| v["name"].as_str().unwrap_or("").to_owned())
        .unwrap_or_default()
}

/// Velocidade da tela (0,5–2) na faixa do SAPI (-10 a 10).
pub fn sapi_rate(rate: f64) -> i32 {
    let rate = if rate.is_finite() { rate.clamp(0.5, 2.0) } else { 1.0 };
    (((rate - 1.0) * 5.0).round() as i32).clamp(-10, 10)
}

pub fn valid_config(v: &Value) -> bool {
    let voice_ok = v["voice"].is_null() || v["voice"].as_str().is_some_and(|s| s.len() <= 200 && !s.contains('\0'));
    let rate_ok = v["rate"].is_null() || v["rate"].as_f64().is_some_and(|r| r.is_finite() && (0.5..=2.0).contains(&r));
    let limit_ok = v["limit"].is_null()
        || v["limit"].as_u64().is_some_and(|l| (1..=500).contains(&l))
        || v["limit"].as_f64().is_some_and(|l| l.fract() == 0.0 && (1.0..=500.0).contains(&l));
    voice_ok && rate_ok && limit_ok
}

fn dir_of(rt: &Runtime, p: &str) -> PathBuf {
    rt.base.join("media").join(p).join("tts")
}

/// Id do cache: 32 dígitos hexadecimais + .wav (nunca caminho).
pub fn id_for(voice: &str, rate: i32, text: &str) -> String {
    let mut h = Sha256::new();
    h.update(voice.as_bytes());
    h.update(b"\0");
    h.update(rate.to_le_bytes());
    h.update(b"\0");
    h.update(text.as_bytes());
    let hex = format!("{:x}", h.finalize());
    format!("{}.wav", &hex[..32])
}

pub fn valid_id(id: &str) -> bool {
    id.len() == 36
        && id.ends_with(".wav")
        && id[..32].chars().all(|c| c.is_ascii_hexdigit())
}

/// Endereço do WAV gerado — servido só com a chave do perfil.
pub fn url(rt: &Runtime, p: &str, id: &str) -> Result<String, String> {
    if !valid_id(id) {
        return Err("Áudio do texto para voz inválido".into());
    }
    let info = crate::livepix::overlay_info(rt, p)?;
    Ok(format!("http://127.0.0.1:{}/tts?token={}&id={}", info["port"], info["token"], id))
}

/// Serve o WAV gerado: id em formato estrito, dentro da pasta tts do perfil,
/// até 5 MiB e com assinatura de WAV. Caminho arbitrário é impossível.
pub fn file(rt: &Runtime, p: &str, id: &str) -> Result<(Vec<u8>, &'static str), String> {
    if !valid_id(id) {
        return Err("Áudio do texto para voz inválido".into());
    }
    let dir = dir_of(rt, p).canonicalize().map_err(|_| "Áudio do texto para voz não encontrado")?;
    let path = dir.join(id).canonicalize().map_err(|_| "Áudio do texto para voz não encontrado")?;
    if !path.starts_with(&dir) || path == dir || path.file_name().map(|n| n.to_string_lossy().into_owned()) != Some(id.to_owned()) {
        return Err("Caminho de áudio inválido".into());
    }
    let meta = std::fs::metadata(&path).map_err(|_| "Áudio do texto para voz não encontrado")?;
    if meta.len() < 44 || meta.len() > 5 * 1024 * 1024 {
        return Err("Áudio do texto para voz fora do tamanho permitido".into());
    }
    let mut bytes = std::fs::read(&path).map_err(|_| "Não foi possível ler o áudio do texto para voz")?;
    bytes.truncate(5 * 1024 * 1024 + 1);
    if !(bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE")) {
        return Err("O áudio do texto para voz não é um WAV válido".into());
    }
    Ok((bytes, "audio/wav"))
}

/// Mantém o cache curto: os `keep` arquivos mais novos ficam.
fn cleanup(dir: &Path, keep: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            if !meta.is_file() {
                return None;
            }
            let fresh = meta.modified().ok()?;
            Some((fresh, e.path()))
        })
        .collect();
    if files.len() <= keep {
        return;
    }
    files.sort_by_key(|(at, _)| *at);
    let extra = files.len().saturating_sub(keep);
    for (_, path) in files.into_iter().take(extra) {
        let _ = std::fs::remove_file(path);
    }
}

async fn generate(voice: &str, rate: i32, text: &str, wav: &Path) -> Result<(), String> {
    let dir = wav.parent().ok_or("Caminho inválido")?;
    std::fs::create_dir_all(dir).map_err(|_| "Não foi possível criar a pasta de áudio do perfil")?;
    let txt = wav.with_extension("txt");
    std::fs::write(&txt, text).map_err(|_| "Não foi possível gravar o texto para leitura")?;
    let result = ps(
        SYNTH,
        &[
            ("BOTLIVE_TTS_TEXT", txt.to_string_lossy().into_owned()),
            ("BOTLIVE_TTS_WAV", wav.to_string_lossy().into_owned()),
            ("BOTLIVE_TTS_VOICE", voice.to_owned()),
            ("BOTLIVE_TTS_RATE", rate.to_string()),
        ],
        Duration::from_secs(60),
    )
    .await;
    let _ = std::fs::remove_file(&txt);
    let out = result?;
    if !out.status.success() {
        let _ = std::fs::remove_file(wav);
        let err = String::from_utf8_lossy(&out.stderr).to_lowercase();
        return Err(if err.contains("voice") || err.contains("selectvoice") {
            "A voz escolhida não existe mais neste Windows. Abra Comunidade → Texto para voz e escolha outra voz.".into()
        } else {
            "Não foi possível gerar o áudio do texto. Confira a voz e o texto configurados.".into()
        });
    }
    let bytes = std::fs::read(wav).map_err(|_| "O Windows não gerou o áudio do texto")?;
    if bytes.len() < 44 || bytes.len() > 5 * 1024 * 1024 || !(bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE")) {
        let _ = std::fs::remove_file(wav);
        return Err("O Windows gerou um arquivo que não é WAV válido.".into());
    }
    Ok(())
}

/// Fala o texto pela **saída única** do BotLive. Gera o WAV (com cache) e
/// entrega pelo mesmo `audio::play` dos demais sons. Devolve o evento emitido.
pub async fn speak(rt: &Runtime, p: &str, text: &str) -> Result<Value, String> {
    let cfg = rt.db.module(p, "tts");
    let limit = cfg["limit"]
        .as_u64()
        .or_else(|| cfg["limit"].as_f64().filter(|l| l.fract() == 0.0).map(|l| l as u64))
        .unwrap_or(300)
        .clamp(1, 500) as usize;
    let rate = cfg["rate"].as_f64().filter(|r| r.is_finite() && (0.5..=2.0).contains(r)).unwrap_or(1.0);
    let full: String = text.chars().take(limit).collect();
    let full = full.trim().to_owned();
    if full.is_empty() || full.contains('\0') {
        return Err("Texto para voz vazio ou inválido".into());
    }
    let saved = cfg["voice"].as_str().unwrap_or("");
    let installed = voices(rt).await?;
    let voice = pick(&installed, saved);
    let id = id_for(&voice, sapi_rate(rate), &full);
    let wav = dir_of(rt, p).join(&id);
    if !wav.is_file() {
        generate(&voice, sapi_rate(rate), &full, &wav).await?;
        cleanup(&dir_of(rt, p), 80);
    }
    let src = url(rt, p, &id)?;
    Ok(crate::audio::play(rt, p, crate::audio::Req { kind: "tts", src, volume: 1.0, nickname: None }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt() -> (tempfile::TempDir, std::sync::Arc<Runtime>) {
        let dir = tempfile::tempdir().unwrap();
        let rt = Runtime::new(dir.path().into()).unwrap();
        (dir, rt)
    }

    fn profile() -> crate::model::Profile {
        serde_json::from_value(json!({"id":uuid::Uuid::new_v4(),"name":"Teste","platform":"twitch","channel":"canal"})).unwrap()
    }

    #[test]
    fn id_do_cache_e_hexadecimal_e_nunca_caminho() {
        let id = id_for("Microsoft Maria", 0, "Olá!");
        assert!(valid_id(&id), "id gerado é válido");
        assert_eq!(id.len(), 36);
        assert!(!id.contains('/') && !id.contains('\\') && !id.contains(".."));
        // Mesmo texto, mesma voz, mesma velocidade → mesmo arquivo (cache).
        assert_eq!(id, id_for("Microsoft Maria", 0, "Olá!"));
        assert_ne!(id, id_for("Microsoft Maria", 1, "Olá!"), "velocidade muda o cache");
        assert_ne!(id, id_for("Microsoft Daniel", 0, "Olá!"), "voz muda o cache");
        assert!(!valid_id("../../etc/passwd.wav"), "travessia barrada");
        assert!(!valid_id("x.wav"), "id curto barrado");
        assert!(!valid_id(""), "vazio barrado");
        assert!(!valid_id(&format!("{}.exe", &id_for("a", 0, "b")[..32])), "só .wav");
    }

    #[test]
    fn escolha_de_voz_cai_para_o_portugues_quando_nao_acha_o_nome_antigo() {
        let list = json!([
            {"name":"Microsoft David","culture":"en-US","gender":"Male"},
            {"name":"Microsoft Maria","culture":"pt-BR","gender":"Female"},
            {"name":"Microsoft Daniel","culture":"pt-BR","gender":"Male"}
        ]);
        assert_eq!(pick(&list, "Microsoft Maria"), "Microsoft Maria", "nome exato passa direto");
        assert_eq!(pick(&list, "Microsoft Maria - Portuguese (Brazil)"), "Microsoft Maria", "voiceURI antigo do navegador migra");
        assert_eq!(pick(&list, "Microsoft Maria - Português (Brasil)"), "Microsoft Maria");
        assert_eq!(pick(&list, ""), "Microsoft Maria", "padrão da tela vira a voz em português");
        assert_eq!(pick(&list, "VozQueNaoExiste"), "Microsoft Maria", "voz apagada cai na portuguesa");
        assert_eq!(pick(&json!([]), "qualquer"), "", "sem vozes, o padrão do Windows fala");
        assert_eq!(pick(&list, "  Microsoft Maria  "), "Microsoft Maria", "espaços em volta não atrapalham");
    }

    #[test]
    fn velocidade_da_tela_vai_para_a_faixa_do_sapi() {
        assert_eq!(sapi_rate(1.0), 0, "velocidade normal não muda nada");
        assert_eq!(sapi_rate(0.5), -3, "mais devagar vira negativo");
        assert_eq!(sapi_rate(2.0), 5, "mais rápido vira positivo");
        assert_eq!(sapi_rate(f64::NAN), 0, "valor quebrado vira padrão");
        assert_eq!(sapi_rate(99.0), 5, "acima do topo vira o teto da faixa");
    }

    #[test]
    fn config_valida_aceita_o_que_a_tela_grava_e_barra_exageros() {
        assert!(valid_config(&json!({"voice":"Microsoft Maria","rate":1,"limit":300})));
        assert!(valid_config(&json!({})), "config vazia é válida (padrões)");
        assert!(valid_config(&json!({"voice":"Voz antiga do navegador","rate":0.5,"limit":1})));
        assert!(!valid_config(&json!({"rate":3.0})), "velocidade acima de 2 barra");
        assert!(!valid_config(&json!({"limit":0})), "limite zero barra");
        assert!(!valid_config(&json!({"limit":501})), "limite acima de 500 barra");
        assert!(!valid_config(&json!({"voice":"x".repeat(300)})), "voz gigante barra");
        assert!(!valid_config(&json!({"voice":12})), "voz precisa ser texto");
    }

    #[tokio::test]
    async fn speak_sem_texto_falha_antes_de_qualquer_sintetizador() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let err = speak(&rt, &p.id, "   \n  ").await.unwrap_err();
        assert!(err.contains("vazio"), "texto em branco é erro claro");
        assert!(speak(&rt, &p.id, "\u{0}abc").await.is_err(), "NUL barrado");
    }

    #[tokio::test]
    async fn arquivo_só_sai_da_pasta_tts_do_perfil() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let good = id_for("voz", 0, "oi");
        let dir = dir_of(&rt, &p.id);
        std::fs::create_dir_all(&dir).unwrap();
        let mut wav = b"RIFF\x24\x00\x00\x00WAVE".to_vec();
        wav.extend_from_slice(&[0u8; 64]);
        std::fs::write(dir.join(&good), &wav).unwrap();
        // Fora da pasta: nem com id válido.
        assert!(file(&rt, &p.id, "00000000000000000000000000000000.wav").is_err(), "id inexistente não serve");
        let (bytes, mime) = file(&rt, &p.id, &good).unwrap();
        assert_eq!(mime, "audio/wav");
        assert_eq!(bytes.len(), wav.len());
        // WAV corrompido.
        std::fs::write(dir.join(id_for("voz", 0, "mal")), b"NOPE0000WAVE000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert!(file(&rt, &p.id, &id_for("voz", 0, "mal")).is_err(), "arquivo que não é WAV não serve");
        assert!(file(&rt, &p.id, "../00000000000000000000000000000000.wav").is_err(), "travessia barrada na rota");
        assert!(file(&rt, &p.id, &format!("{}.txt", &good[..32])).is_err(), "só WAV");
    }

    #[tokio::test]
    async fn url_do_tts_aponta_para_a_rota_local_com_a_chave_do_perfil() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let id = id_for("voz", 0, "oi");
        let u = url(&rt, &p.id, &id).unwrap();
        assert!(u.starts_with("http://127.0.0.1:9876/tts?token="), "rota local: {u}");
        assert!(u.ends_with(&format!("&id={id}")), "carrega o id: {u}");
        assert!(url(&rt, &p.id, "fora.wav").is_err(), "id inválido não gera endereço");
    }

    #[test]
    fn cache_mantem_os_arquivos_mais_novos() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..10 {
            std::fs::write(dir.path().join(format!("{i}.wav")), b"x").unwrap();
        }
        cleanup(dir.path(), 4);
        let left = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(left, 4, "sobram só os mais recentes da lista");
        cleanup(dir.path(), 4);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 4, "segunda passada é estável");
        cleanup(dir.path(), 0);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "keep 0 limpa tudo");
    }

    #[test]
    fn vozes_listadas_sao_apenas_o_que_a_tela_precisa() {
        let clean = sanitize_voices(&json!([
            {"name":"Microsoft Maria","culture":"pt-BR","gender":"Female","age":"Adult"},
            {"name":"","culture":"pt-BR"},
            {"name":123},
            {"name":"x".repeat(300),"culture":"pt-BR"}
        ]));
        assert_eq!(clean.as_array().unwrap().len(), 1, "só voz com nome de verdade");
        assert_eq!(clean[0]["name"], "Microsoft Maria");
        assert!(clean[0].get("age").is_none(), "campo extra não passa");
    }

    /// Som real no Windows: gera um WAV de verdade e confirma a entrega pela
    /// saída única. Roda com `cargo test -- --ignored tts` (uma vez por máquina).
    #[tokio::test]
    #[ignore = "usa o sintetizador real do Windows; roda sob demanda"]
    async fn gera_wav_real_e_entrega_pela_saida_unica() {
        let (_dir, rt) = rt();
        let p = profile();
        rt.db.save_profile(&p).unwrap();
        let out = speak(&rt, &p.id, "Olá, comunidade! Este é o BotLive na transmissão.").await.expect("sintetização real");
        assert_eq!(out["kind"], "tts");
        assert!(out["src"].as_str().unwrap().contains("/tts?token="), "entrega pela rota local");
        let id = out["src"].as_str().unwrap().rsplit("id=").next().unwrap();
        let (bytes, _) = file(&rt, &p.id, id).expect("WAV servível");
        assert!(bytes.starts_with(b"RIFF"), "WAV de verdade");
        assert!(bytes.len() > 10_000, "tem áudio dentro");
    }
}
