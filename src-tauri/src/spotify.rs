//! Spotify: a música que está tocando agora. O streamer cria o seu aplicativo
//! gratuito no developer.spotify.com (modo desenvolvimento chega e sobra para
//! uso pessoal), autoriza a conta pelo navegador com PKCE e o BotLive lê a faixa
//! atual de 5 em 5 segundos — só leitura: nada é tocado, pausado ou enviado ao
//! Spotify. O estado alimenta o overlay, a prévia do Estúdio e as variáveis
//! `{{nowplaying.*}}` das automações.
use crate::engine::Runtime;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Porta própria da autorização: o Spotify exige a URL de retorno cadastrada
/// exatamente no aplicativo do streamer, e o 43827 já é do YouTube/Kick.
pub const REDIRECT: &str = "http://127.0.0.1:43828/callback";
/// Só leitura do que está tocando (e do nome da conta para mostrar quem conectou).
pub const SCOPES: &str = "user-read-currently-playing user-read-playback-state user-read-private";

/// Fronteira com o cofre do sistema. Fora de teste é o keyring como no resto do
/// app; em teste vira memória — a suíte não pode depender do credenciário do
/// Windows nem deixar token para trás na máquina de quem roda.
#[cfg(not(test))]
mod vault {
    pub fn get(p: &str, k: &str) -> Result<String, String> {
        crate::secrets::get(p, k)
    }
    pub fn set(p: &str, k: &str, v: &str) -> Result<(), String> {
        crate::secrets::set(p, k, v)
    }
}
#[cfg(test)]
mod vault {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    fn memory() -> &'static Mutex<HashMap<String, String>> {
        static M: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
        M.get_or_init(|| Mutex::new(HashMap::new()))
    }
    pub fn get(p: &str, k: &str) -> Result<String, String> {
        memory()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&format!("{p}:{k}"))
            .cloned()
            .ok_or_else(|| "Credencial não configurada".into())
    }
    pub fn set(p: &str, k: &str, v: &str) -> Result<(), String> {
        let mut m = memory().lock().unwrap_or_else(|e| e.into_inner());
        let key = format!("{p}:{k}");
        if v.is_empty() {
            m.remove(&key);
        } else {
            m.insert(key, v.into());
        }
        Ok(())
    }
}

/// A faixa atual, reduzida ao que overlay, prévia e variáveis usam.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub art: String,
    pub url: String,
    pub duration_ms: u64,
    pub progress_ms: u64,
    pub timestamp: u64,
    pub playing: bool,
}

/// Música atual por perfil, em memória — nada disso vira estatística ou banco.
#[derive(Default)]
pub struct States(Mutex<HashMap<String, Entry>>);

#[derive(Default)]
struct Entry {
    track: Option<Track>,
    error: Option<String>,
    backoff_until: i64,
    /// Sessão corrente do perfil. Consulta e renovação capturam a geração antes
    /// de tocar a rede e conferem antes de gravar ou publicar: desconectar ou
    /// trocar de conta no meio descarta a resposta que chegou tarde.
    gen: u64,
    /// Uma consulta por perfil em voo — o laço de 5 s não empilha chamada que o
    /// HTTP pode levar 30 s a responder.
    busy: bool,
}

/// A consulta em curso de um perfil. Devolve a exclusão mesmo se a tarefa cair
/// no meio, sem deixar o perfil travado para sempre.
pub struct Lease<'a> {
    s: &'a States,
    p: &'a str,
}
impl Drop for Lease<'_> {
    fn drop(&mut self) {
        if let Some(e) = self.s.lock().get_mut(self.p) {
            e.busy = false;
        }
    }
}

impl States {
    /// Uma travada que sobrevive a pânico alheio: a coleta da live não pode
    /// morrer porque outro pedaço do app falhou segurando o mapa.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
    /// A música atual do perfil, clonada — None quando nada toca.
    pub fn now(&self, p: &str) -> Option<Track> {
        self.lock().get(p).and_then(|e| e.track.clone())
    }
    /// Grava a música atual e diz se o overlay notaria a mudança (faixa nova ou
    /// pausa/retomada); a posição do tempo sozinha não vira nova transmissão.
    pub fn set(&self, p: &str, t: Option<Track>) -> bool {
        let mut map = self.lock();
        let e = map.entry(p.to_owned()).or_default();
        let before = e.track.as_ref().map(|o| (o.id.clone(), o.playing));
        let after = t.as_ref().map(|o| (o.id.clone(), o.playing));
        e.track = t;
        before != after
    }
    /// Uma leitura limpa apaga o último erro registrado.
    pub fn ok(&self, p: &str) {
        if let Some(e) = self.lock().get_mut(p) {
            e.error = None;
        }
    }
    /// Registra o erro e diz se é novo (para o histórico não encher de repetição).
    pub fn mark_error(&self, p: &str, msg: &str) -> bool {
        let mut map = self.lock();
        let e = map.entry(p.to_owned()).or_default();
        let fresh = e.error.as_deref() != Some(msg);
        e.error = Some(msg.into());
        fresh
    }
    /// O Spotify pediu calma (429): não consulte este perfil antes da hora.
    pub fn throttle(&self, p: &str, seconds: i64) {
        let mut map = self.lock();
        let e = map.entry(p.to_owned()).or_default();
        e.backoff_until = chrono::Utc::now().timestamp() + seconds;
    }
    /// True enquanto a espera imposta por `throttle` não venceu.
    pub fn throttled(&self, p: &str) -> bool {
        self.lock().get(p).is_some_and(|e| e.backoff_until > chrono::Utc::now().timestamp())
    }
    /// Número da sessão corrente do perfil — sobe em desconexão e troca de conta.
    pub fn generation(&self, p: &str) -> u64 {
        self.lock().get(p).map(|e| e.gen).unwrap_or(0)
    }
    /// Encerra a sessão: o que ainda estiver em trânsito compara a geração e
    /// descarta o resultado em vez de repovoar a faixa ou o cofre.
    pub fn bump(&self, p: &str) {
        let mut map = self.lock();
        let e = map.entry(p.to_owned()).or_default();
        e.gen = e.gen.wrapping_add(1);
    }
    /// Tenta começar uma leitura deste perfil. None quando já há uma em voo —
    /// o chamador nem sai para a rede, evitando sobreposição e fim fora de ordem.
    pub fn begin<'a>(&'a self, p: &'a str) -> Option<Lease<'a>> {
        let mut map = self.lock();
        let e = map.entry(p.to_owned()).or_default();
        if e.busy {
            return None;
        }
        e.busy = true;
        Some(Lease { s: self, p })
    }
}

/// Configuração do módulo como está no banco (por perfil).
pub fn config(rt: &Runtime, p: &str) -> Value {
    rt.db.module(p, "spotify")
}

/// Padrões do overlay — mesma forma que o Estúdio e a fonte do OBS desenham.
fn design_defaults() -> Value {
    // Montado em partes: o objeto único estourava o limite de recursão do macro json!.
    let mut d = Map::new();
    // Pele e capa
    for (k, v) in [
        ("layout", json!("card")),
        ("cover", json!("square")),
        ("coverGlow", json!(false)),
        ("coverBlur", json!(false)),
    ] {
        d.insert(k.into(), v);
    }
    // Alavancas de conteúdo (o que aparece e o que some junto com o espaço)
    for (k, v) in [
        ("showArt", json!(true)),
        ("showTitle", json!(true)),
        ("showArtist", json!(true)),
        ("showAlbum", json!(true)),
        ("showProgress", json!(true)),
        ("showTimes", json!(true)),
        ("showDuration", json!(true)),
        ("showIcon", json!(true)),
        ("showBg", json!(true)),
        ("showBorder", json!(false)),
        ("showShadow", json!(false)),
    ] {
        d.insert(k.into(), v);
    }
    // Controles novos do Estúdio: borda, sombra, transparência, alinhamento,
    // espaçamento, quebra do nome, peso do título e ordem das linhas
    for (k, v) in [
        ("borderColor", json!("#2a3140")),
        ("shadowSize", json!(16)),
        ("opacity", json!(100)),
        ("align", json!("auto")),
        ("gap", json!(12)),
        ("titleWrap", json!(false)),
        ("titleWeight", json!("600")),
        ("order", json!("title,artist,album")),
    ] {
        d.insert(k.into(), v);
    }
    // Cores
    for (k, v) in [
        ("bg", json!("#0f1319")),
        ("titleColor", json!("#ffffff")),
        ("artistColor", json!("#b9c0cc")),
        ("albumColor", json!("#8a92a3")),
        ("barBg", json!("#2a3140")),
        ("barColor", json!("#1ed760")),
        ("barHeight", json!(6)),
        ("barRadius", json!(3)),
    ] {
        d.insert(k.into(), v);
    }
    // Tipografia e medidas
    for (k, v) in [
        ("font", json!("Segoe UI")),
        ("titleSize", json!(16)),
        ("artistSize", json!(13)),
        ("artSize", json!(88)),
        ("artRadius", json!(10)),
        ("width", json!(460)),
        ("padding", json!(14)),
        ("radius", json!(14)),
    ] {
        d.insert(k.into(), v);
    }
    // Comportamento parado, posição e movimento
    for (k, v) in [
        ("empty", json!("hide")),
        ("emptyText", json!("Nada tocando")),
        ("emptyArtist", json!("")),
        ("hideOnPause", json!(true)),
        ("pauseDelay", json!(0)),
        ("position", json!("bottom-left")),
        ("transition", json!("fade")),
        ("exitTransition", json!("fade")),
        ("transitionMs", json!(400)),
    ] {
        d.insert(k.into(), v);
    }
    Value::Object(d)
}

/// Configuração padrão do módulo: desligado, sem Client ID, overlay neutro.
pub fn defaults() -> Value {
    json!({"enabled": false, "clientId": "", "design": design_defaults()})
}

/// Configuração pronta para uso: o que está no banco com padrões para o resto.
pub fn merged(rt: &Runtime, p: &str) -> Value {
    let saved = config(rt, p);
    let mut v = defaults();
    let mut d = design_defaults();
    if let Some(obj) = saved.as_object() {
        for (k, val) in obj {
            if k == "design" {
                if let Some(design) = val.as_object() {
                    for (dk, dv) in design {
                        d[dk.clone()] = dv.clone();
                    }
                }
            } else {
                v[k.clone()] = val.clone();
            }
        }
    }
    v["design"] = d;
    v
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

fn font_ok(v: &Value) -> bool {
    const FONTS: &[&str] = &[
        "Segoe UI", "Arial", "Georgia", "Verdana", "Trebuchet MS", "Courier New",
        "Inter", "Montserrat", "Poppins", "Rubik", "Orbitron", "Bebas Neue",
        "Press Start 2P", "Playfair Display", "Rajdhani", "Quicksand",
    ];
    v.as_str().is_some_and(|s| FONTS.contains(&s))
}

/// O Estúdio envia o objeto inteiro; valor fora de faixa barra o salvamento
/// antes de qualquer coisa chegar ao overlay.
pub fn valid_config(v: &Value) -> bool {
    if !v["enabled"].is_boolean() {
        return false;
    }
    if !text_ok(&v["clientId"], 128)
        || !v["clientId"].as_str().unwrap_or("").chars().all(|c| c.is_ascii_alphanumeric() || "-_. ".contains(c))
    {
        return false;
    }
    let d = &v["design"];
    const BOOLS: &[&str] = &[
        "showArt", "showTitle", "showArtist", "showAlbum", "showProgress", "showTimes",
        "showDuration", "showIcon", "showBg", "showBorder", "showShadow", "titleWrap",
        "coverGlow", "coverBlur", "hideOnPause",
    ];
    if !BOOLS.iter().all(|k| d[k].is_boolean()) {
        return false;
    }
    if !matches!(d["layout"].as_str(), Some("card") | Some("compact") | Some("stacked") | Some("minimal")) {
        return false;
    }
    if !matches!(d["cover"].as_str(), Some("square") | Some("circle")) {
        return false;
    }
    if !matches!(d["empty"].as_str(), Some("hide") | Some("message")) {
        return false;
    }
    if !matches!(d["align"].as_str(), Some("auto") | Some("left") | Some("center")) {
        return false;
    }
    if !matches!(d["titleWeight"].as_str(), Some("400") | Some("500") | Some("600") | Some("700")) {
        return false;
    }
    // A ordem das linhas é uma permutação exata das três chaves; o que vier
    // fora disso o Estúdio já normalizou para o padrão.
    let order_ok = d["order"].as_str().is_some_and(|s| {
        let mut parts: Vec<&str> = s.split(',').collect();
        parts.sort_unstable();
        parts == ["album", "artist", "title"]
    });
    if !order_ok {
        return false;
    }
    if !matches!(d["transition"].as_str(), Some("fade") | Some("slide") | Some("none"))
        || !matches!(d["exitTransition"].as_str(), Some("fade") | Some("slide") | Some("none"))
    {
        return false;
    }
    if !matches!(
        d["position"].as_str(),
        Some("bottom-left") | Some("bottom-right") | Some("top-left") | Some("top-right") | Some("center")
    ) {
        return false;
    }
    if !font_ok(&d["font"]) || !text_ok(&d["emptyText"], 64) || !text_ok(&d["emptyArtist"], 64) {
        return false;
    }
    for k in ["bg", "titleColor", "artistColor", "albumColor", "barBg", "barColor", "borderColor"] {
        if !color_ok(&d[k]) {
            return false;
        }
    }
    num_in(&d["opacity"], 20.0, 100.0)
        && num_in(&d["gap"], 2.0, 24.0)
        && num_in(&d["shadowSize"], 0.0, 48.0)
        && num_in(&d["width"], 240.0, 960.0)
        && num_in(&d["padding"], 0.0, 48.0)
        && num_in(&d["radius"], 0.0, 40.0)
        && num_in(&d["barHeight"], 2.0, 24.0)
        && num_in(&d["barRadius"], 0.0, 12.0)
        && num_in(&d["titleSize"], 10.0, 48.0)
        && num_in(&d["artistSize"], 9.0, 32.0)
        && num_in(&d["artSize"], 48.0, 220.0)
        && num_in(&d["artRadius"], 0.0, 60.0)
        && num_in(&d["transitionMs"], 0.0, 2000.0)
        && num_in(&d["pauseDelay"], 0.0, 30.0)
}

fn api_base() -> String {
    std::env::var("BOTLIVE_SPOTIFY_API").unwrap_or_else(|_| "https://api.spotify.com".into())
}

fn accounts_base() -> String {
    std::env::var("BOTLIVE_SPOTIFY_ACCOUNTS").unwrap_or_else(|_| "https://accounts.spotify.com".into())
}

fn store(p: &str, v: &Value) -> Result<(), String> {
    vault::set(p, "spotify_token", v["access_token"].as_str().ok_or("Token ausente no Spotify")?)?;
    // O Spotify pode rotacionar o refresh a cada renovação — o novo também entra no cofre.
    if let Some(s) = v["refresh_token"].as_str() {
        vault::set(p, "spotify_refresh", s)?;
    }
    vault::set(p, "spotify_expires", &(chrono::Utc::now().timestamp() + v["expires_in"].as_i64().unwrap_or(3600)).to_string())?;
    Ok(())
}

/// True quando o perfil tem autorização do Spotify guardada.
pub fn connected(p: &str) -> bool {
    vault::get(p, "spotify_refresh").is_ok()
}

/// Apaga a autorização do perfil — a música some do overlay na hora.
pub fn disconnect(p: &str) -> Result<(), String> {
    for k in ["spotify_token", "spotify_refresh", "spotify_expires"] {
        vault::set(p, k, "")?;
    }
    Ok(())
}

/// Só em teste: credenciais já vencidas no cofre, para exercitar a renovação
/// contra um servidor lento sem esperar o relógio passar.
#[cfg(test)]
pub fn seed_expired(p: &str) -> Result<(), String> {
    vault::set(p, "spotify_token", "token-antigo")?;
    vault::set(p, "spotify_refresh", "refresh-antigo")?;
    vault::set(p, "spotify_expires", "1")
}

/// Autoriza a conta no navegador (PKCE, sem segredo) e guarda os tokens no cofre.
pub async fn connect(client: &reqwest::Client, p: &str, cfg: &Value) -> Result<Value, String> {
    connect_with(client, p, cfg, |url| open::that(url).map_err(|e| e.to_string())).await
}

/// O mesmo com o abridor de navegador injetado — os testes não abrem aba nenhuma.
pub async fn connect_with<F: Fn(&str) -> Result<(), String>>(client: &reqwest::Client, p: &str, cfg: &Value, opener: F) -> Result<Value, String> {
    let id = cfg["clientId"].as_str().unwrap_or("").trim().to_owned();
    if id.is_empty() {
        return Err("Cadastre um aplicativo em developer.spotify.com/dashboard (Web API) e informe o Client ID".into());
    }
    let listener = TcpListener::bind("127.0.0.1:43828").await.map_err(|_| "A porta 43828 de autorização está ocupada".to_string())?;
    let state = uuid::Uuid::new_v4().to_string();
    let verifier = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut url = url::Url::parse(&format!("{}/authorize", accounts_base())).map_err(|_| "Endereço do Spotify inválido".to_string())?;
    url.query_pairs_mut().extend_pairs([
        ("client_id", id.as_str()),
        ("redirect_uri", REDIRECT),
        ("response_type", "code"),
        ("scope", SCOPES),
        ("state", state.as_str()),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
    ]);
    opener(url.as_str())?;
    let code = tokio::time::timeout(std::time::Duration::from_secs(180), async {
        loop {
            let (mut socket, _) = listener.accept().await.map_err(|_| "Falha na autorização")?;
            let mut buf = [0u8; 8192];
            let n = tokio::time::timeout(std::time::Duration::from_secs(5), socket.read(&mut buf)).await.map_err(|_| "Tempo esgotado")?.map_err(|_| "Callback inválido")?;
            let request = String::from_utf8_lossy(&buf[..n]);
            let path = request.split_whitespace().nth(1).unwrap_or("/");
            let url = url::Url::parse(&format!("http://127.0.0.1{path}")).map_err(|_| "Callback inválido")?;
            let args: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
            let valid = url.path() == "/callback" && args.get("state") == Some(&state);
            let body = if valid { "Autorizacao recebida. Volte ao BotLive." } else { "Requisicao invalida." };
            let response = format!("HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", if valid { "200 OK" } else { "400 Bad Request" }, body.len());
            let _ = socket.write_all(response.as_bytes()).await;
            if valid {
                return args.get("code").cloned().ok_or("Autorização não concedida".to_string());
            }
        }
    })
    .await
    .map_err(|_| "O tempo de autorização terminou. Tente novamente.".to_string())??;
    let res = client
        .post(format!("{}/api/token", accounts_base()))
        .form(&[("grant_type", "authorization_code"), ("code", code.as_str()), ("redirect_uri", REDIRECT), ("client_id", id.as_str()), ("code_verifier", verifier.as_str())])
        .send()
        .await
        .map_err(|_| "Falha ao trocar o código pelo Spotify".to_string())?;
    let ok = res.status().is_success();
    let v: Value = res.json().await.map_err(|_| "Resposta do Spotify inválida".to_string())?;
    if !ok {
        return Err("O Spotify recusou a autorização. Confira o Client ID e a URL de retorno no aplicativo.".into());
    }
    let token = v["access_token"].as_str().unwrap_or_default().to_owned();
    store(p, &v)?;
    let me: Value = client
        .get(format!("{}/v1/me", api_base()))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|_| "Falha ao ler a conta do Spotify".to_string())?
        .json()
        .await
        .unwrap_or(Value::Null);
    Ok(json!({"authorized": true, "user": me["display_name"].as_str().unwrap_or_default(), "product": me["product"].as_str().unwrap_or_default()}))
}

/// Token de leitura atual, renovando quando expira. A geração da sessão é
/// capturada antes da chamada: uma renovação que termina depois de a conta
/// desconectar não reescreve as credenciais apagadas.
pub async fn token(rt: &Runtime, p: &str, cfg: &Value) -> Result<String, String> {
    let gen = rt.spotify.generation(p);
    let current = vault::get(p, "spotify_token")?;
    let expires = vault::get(p, "spotify_expires").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    if chrono::Utc::now().timestamp() + 60 < expires {
        return Ok(current);
    }
    let refresh = vault::get(p, "spotify_refresh")?;
    let res = rt
        .http
        .post(format!("{}/api/token", accounts_base()))
        .form(&[("grant_type", "refresh_token"), ("refresh_token", refresh.as_str()), ("client_id", cfg["clientId"].as_str().unwrap_or(""))])
        .send()
        .await
        .map_err(|_| "Não foi possível renovar a autorização do Spotify".to_string())?;
    if !res.status().is_success() {
        return Err("A autorização do Spotify expirou. Autorize a conta novamente.".into());
    }
    let v: Value = res.json().await.map_err(|_| "Resposta do Spotify inválida".to_string())?;
    if rt.spotify.generation(p) != gen {
        return Err("A conta foi desconectada durante a renovação do Spotify".into());
    }
    store(p, &v)?;
    Ok(v["access_token"].as_str().unwrap_or_default().into())
}

/// A faixa atual, ou None quando o Spotify não tem música (204, anúncio ou
/// podcast — a v1 acompanha apenas faixas de música).
pub async fn fetch(client: &reqwest::Client, token: &str) -> Result<Option<Track>, String> {
    let res = client
        .get(format!("{}/v1/me/player/currently-playing", api_base()))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Falha ao consultar o Spotify".to_string())?;
    match res.status().as_u16() {
        204 => Ok(None),
        200 => Ok(parse(&res.json().await.map_err(|_| "Resposta do Spotify inválida".to_string())?)),
        401 => Err("A autorização do Spotify foi recusada. Autorize a conta novamente.".into()),
        429 => Err("spotify:rate".into()),
        code => Err(format!("O Spotify respondeu com erro ({code})")),
    }
}

/// Reduz a resposta da API ao que importa; anúncio e podcast viram "nada tocando".
pub fn parse(v: &Value) -> Option<Track> {
    if v["currently_playing_type"].as_str().unwrap_or("track") != "track" {
        return None;
    }
    let item = &v["item"];
    if !item["id"].is_string() {
        return None;
    }
    Some(Track {
        id: item["id"].as_str().unwrap_or_default().into(),
        name: item["name"].as_str().unwrap_or_default().into(),
        artist: item["artists"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x["name"].as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default(),
        album: item["album"]["name"].as_str().unwrap_or_default().into(),
        art: item["album"]["images"].as_array().and_then(|l| l.first()).and_then(|i| i["url"].as_str()).unwrap_or_default().into(),
        url: item["external_urls"]["spotify"].as_str().unwrap_or_default().into(),
        duration_ms: item["duration_ms"].as_u64().unwrap_or(0),
        progress_ms: v["progress_ms"].as_u64().unwrap_or(0),
        timestamp: v["timestamp"].as_u64().unwrap_or(chrono::Utc::now().timestamp_millis().max(0) as u64),
        playing: v["is_playing"] == Value::Bool(true),
    })
}

/// Posição e duração no relógio m:ss que o português lê naturalmente.
pub fn fmt_time(ms: u64) -> String {
    format!("{}:{:02}", ms / 60000, (ms % 60000) / 1000)
}

/// O corpo do evento spotify.now — mesma forma no broadcast, no snapshot da
/// fonte do OBS e no teste do Estúdio.
fn now_json(p: &str, t: Option<&Track>) -> Value {
    match t {
        Some(t) => json!({"profileId": p, "playing": t.playing, "id": t.id, "name": t.name, "artist": t.artist, "album": t.album, "art": t.art, "url": t.url, "durationMs": t.duration_ms, "progressMs": t.progress_ms, "timestamp": t.timestamp}),
        None => json!({"profileId": p, "playing": false}),
    }
}

fn payload(rt: &Runtime, p: &str, t: Option<&Track>) {
    rt.emit("spotify.now", now_json(p, t));
}

/// Mensagem inicial da fonte do OBS: a configuração do perfil (forma de chat.config).
pub fn message(rt: &Runtime, pid: &str) -> Value {
    json!({"type": "spotify.config", "payload": {"profileId": pid, "config": merged(rt, pid)}})
}

/// Segunda mensagem do aperto de mão: a faixa atual, para o overlay mostrar a
/// música na hora em que a fonte do OBS recarrega — os polls só transmitem mudanças.
pub fn now_message(rt: &Runtime, pid: &str) -> Value {
    json!({"type": "spotify.now", "payload": now_json(pid, rt.spotify.now(pid).as_ref())})
}

/// Amostra do Estúdio: entra pelo mesmo caminho do evento real, com a marca
/// `rehearsal` que o overlay traduz no selo ENSAIO (mesmo costume dos alertas).
pub fn test_payload(p: &str) -> Value {
    let mut v = now_json(p, None);
    let now = chrono::Utc::now().timestamp_millis().max(0) as u64;
    v["playing"] = Value::Bool(true);
    v["id"] = json!("sample");
    v["name"] = json!("Faixa de teste");
    v["artist"] = json!("Spotify — ensaio do overlay");
    v["album"] = json!("Estúdio do BotLive");
    v["art"] = json!("");
    v["url"] = json!("");
    v["durationMs"] = json!(210000);
    v["progressMs"] = json!(45000);
    v["timestamp"] = json!(now);
    v["rehearsal"] = Value::Bool(true);
    v
}

/// Uma leitura da música atual: renova o token se preciso, atualiza o estado do
/// perfil e transmite ao overlay/à prévia quando o assunto mudou ou a posição
/// andou. Devolve a música (ou None) para o painel mostrar na hora.
/// Só uma leitura por perfil em voo: se já houver uma, devolve o estado atual
/// sem sair para a rede — respostas fora de ordem não atropelam a mais nova.
pub async fn poll_once(rt: &Runtime, p: &crate::model::Profile, cfg: &Value) -> Result<Option<Track>, String> {
    let _lease = match rt.spotify.begin(&p.id) {
        Some(l) => l,
        None => return Ok(rt.spotify.now(&p.id)),
    };
    // A sessão de agora é a referência: desconectar ou trocar de conta no meio
    // da consulta invalida o resultado que ainda vai chegar.
    let gen = rt.spotify.generation(&p.id);
    let token = match token(rt, &p.id, cfg).await {
        Ok(t) => t,
        Err(e) => {
            if rt.spotify.generation(&p.id) == gen {
                forget(rt, &p.id);
            }
            return Err(e);
        }
    };
    let track = match fetch(&rt.http, &token).await {
        Ok(t) => t,
        Err(e) => {
            if e == "spotify:rate" {
                // 429 vira espera: o laço não insiste e o Histórico não enche.
                rt.spotify.throttle(&p.id, 30);
                return Err("O Spotify pediu uma pausa nas consultas. Aguarde um instante.".into());
            }
            if rt.spotify.generation(&p.id) == gen {
                forget(rt, &p.id);
            }
            return Err(e);
        }
    };
    // Resposta atrasada (desconexão ou troca de conta durante a consulta): o
    // resultado não pode mais repovoar a faixa nem acender o overlay.
    if rt.spotify.generation(&p.id) != gen {
        return Ok(rt.spotify.now(&p.id));
    }
    let fresh = rt.spotify.set(&p.id, track.clone());
    rt.spotify.ok(&p.id);
    // Tocando e pausado retransmitem sempre (a posição anda a cada leitura); a
    // parada só retransmite na virada, para o overlay sumir do OBS na hora.
    if track.is_some() || fresh {
        payload(rt, &p.id, track.as_ref());
    }
    Ok(track)
}

fn forget(rt: &Runtime, p: &str) {
    if rt.spotify.set(p, None) {
        payload(rt, p, None);
    }
}

/// Laço de coleta: de 5 em 5 segundos, só para perfis com o módulo ligado e
/// conta autorizada. A janela rolante do Spotify comporta uma leitura por 5 s
/// com folga; erro novo vai para o Histórico uma única vez. Cada perfil corre
/// em tarefa própria: um pânico isolado não derruba a coleta dos outros nem o
/// laço — na live isso é a diferença entre um aviso e um overlay morto.
pub async fn run(rt: Arc<Runtime>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let profiles = match rt.db.profiles() {
            Ok(v) => v,
            Err(_) => continue,
        };
        for p in profiles {
            let rt = rt.clone();
            tokio::spawn(async move {
                let cfg = merged(&rt, &p.id);
                if cfg["enabled"] != Value::Bool(true) {
                    forget(&rt, &p.id);
                    return;
                }
                if rt.spotify.throttled(&p.id) || !connected(&p.id) {
                    return;
                }
                if let Err(e) = poll_once(&rt, &p, &cfg).await {
                    if rt.spotify.mark_error(&p.id, &e) {
                        rt.log(&p.id, "spotify", &e, "error");
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_padrao_passa_na_validacao() {
        assert!(valid_config(&defaults()));
    }

    #[test]
    fn validacao_rejeita_o_que_quebraria_o_overlay() {
        let mut v = defaults();
        v["design"]["layout"] = json!("gigante");
        assert!(!valid_config(&v), "layout fora do conhecido");
        let mut v = defaults();
        v["design"]["barHeight"] = json!(999);
        assert!(!valid_config(&v), "barra fora de faixa");
        let mut v = defaults();
        v["enabled"] = json!("sim");
        assert!(!valid_config(&v), "interruptor não é booleano");
        let mut v = defaults();
        v["clientId"] = json!("id com espaço inválido!");
        assert!(!valid_config(&v), "Client ID com símbolo estranho");
    }

    #[test]
    fn validacao_conhece_as_novas_alavancas_do_design() {
        let mut v = defaults();
        v["design"]["cover"] = json!("vinil");
        assert!(!valid_config(&v), "formato de capa fora do conhecido");
        let mut v = defaults();
        v["design"]["position"] = json!("meio-baixo");
        assert!(!valid_config(&v), "posição fora do conhecido");
        let mut v = defaults();
        v["design"]["exitTransition"] = json!("girar");
        assert!(!valid_config(&v), "saída fora do conhecido");
        let mut v = defaults();
        v["design"]["pauseDelay"] = json!(45);
        assert!(!valid_config(&v), "espera da pausa passa de 30 s");
        let mut v = defaults();
        v["design"]["hideOnPause"] = json!("sim");
        assert!(!valid_config(&v), "esconder na pausa não é booleano");
        let mut v = defaults();
        v["design"]["coverGlow"] = json!("sim");
        assert!(!valid_config(&v), "brilho da capa não é booleano");
        let mut v = defaults();
        v["design"]["emptyArtist"] = json!("x".repeat(65));
        assert!(!valid_config(&v), "linha do parado longa demais");
        let mut v = defaults();
        v["design"]["layout"] = json!("minimal");
        v["design"]["position"] = json!("center");
        v["design"]["pauseDelay"] = json!(30);
        assert!(valid_config(&v), "minimal, centro e 30 s de espera são válidos");
    }

    #[test]
    fn validacao_cobre_os_controles_novos_do_estudio() {
        let mut v = defaults();
        v["design"]["opacity"] = json!(5);
        assert!(!valid_config(&v), "transparência abaixo do mínimo de 20%");
        let mut v = defaults();
        v["design"]["opacity"] = json!("opaca");
        assert!(!valid_config(&v), "transparência não é número");
        let mut v = defaults();
        v["design"]["gap"] = json!(200);
        assert!(!valid_config(&v), "espaçamento passa dos 24 px");
        let mut v = defaults();
        v["design"]["shadowSize"] = json!(120);
        assert!(!valid_config(&v), "sombra fora dos 48 px");
        let mut v = defaults();
        v["design"]["align"] = json!("justificado");
        assert!(!valid_config(&v), "alinhamento fora do conhecido");
        let mut v = defaults();
        v["design"]["titleWeight"] = json!("900");
        assert!(!valid_config(&v), "peso do título fora do conhecido");
        let mut v = defaults();
        v["design"]["order"] = json!("artist,title");
        assert!(!valid_config(&v), "ordem sem as três linhas");
        let mut v = defaults();
        v["design"]["order"] = json!("album,album,title");
        assert!(!valid_config(&v), "ordem com linha repetida não é permutação");
        let mut v = defaults();
        v["design"]["showIcon"] = json!("sim");
        assert!(!valid_config(&v), "ícone não é booleano");
        let mut v = defaults();
        v["design"]["titleWrap"] = json!("sim");
        assert!(!valid_config(&v), "quebra do nome não é booleano");
        let mut v = defaults();
        v["design"]["borderColor"] = json!("rgb(1,2,3)");
        assert!(valid_config(&v), "cor da borda segue a mesma regra das outras cores");
        let mut v = defaults();
        v["design"]["align"] = json!("center");
        v["design"]["titleWeight"] = json!("700");
        v["design"]["order"] = json!("artist,album,title");
        v["design"]["opacity"] = json!(20);
        v["design"]["gap"] = json!(2);
        v["design"]["shadowSize"] = json!(48);
        assert!(valid_config(&v), "limites dos controles novos são válidos");
    }

    #[test]
    fn chaves_do_design_sao_as_que_o_estudio_desenha() {
        let mut keys: Vec<_> = design_defaults().as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(keys, vec![
            "albumColor", "align", "artRadius", "artSize", "artistColor", "artistSize", "barBg",
            "barColor", "barHeight", "barRadius", "bg", "borderColor", "cover", "coverBlur",
            "coverGlow", "empty", "emptyArtist", "emptyText", "exitTransition", "font", "gap",
            "hideOnPause", "layout", "opacity", "order", "padding", "pauseDelay", "position",
            "radius", "shadowSize", "showAlbum", "showArt", "showArtist", "showBg", "showBorder",
            "showDuration", "showIcon", "showProgress", "showShadow", "showTimes", "showTitle",
            "titleColor", "titleSize", "titleWeight", "titleWrap", "transition", "transitionMs",
            "width",
        ]);
    }

    #[test]
    fn parse_extrai_a_faixa_completa() {
        let v: Value = serde_json::from_str(r#"{
            "is_playing": true, "progress_ms": 83250, "timestamp": 1760000000000,
            "currently_playing_type": "track",
            "item": {"id": "t1", "name": "Tocando Agora", "duration_ms": 201000,
                "artists": [{"name": "Cactus"}, {"name": "convidada"}],
                "album": {"name": "Ao Vivo", "images": [{"url": "https://i.scdn.co/capa.jpg", "width": 640}]},
                "external_urls": {"spotify": "https://open.spotify.com/track/t1"}}}
        "#).unwrap();
        let t = parse(&v).unwrap();
        assert_eq!(t.name, "Tocando Agora");
        assert_eq!(t.artist, "Cactus, convidada");
        assert_eq!(t.album, "Ao Vivo");
        assert_eq!(t.art, "https://i.scdn.co/capa.jpg");
        assert_eq!(t.url, "https://open.spotify.com/track/t1");
        assert_eq!(t.duration_ms, 201000);
        assert_eq!(t.progress_ms, 83250);
        assert!(t.playing);
        assert_eq!(fmt_time(t.progress_ms), "1:23");
        assert_eq!(fmt_time(t.duration_ms), "3:21");
    }

    #[test]
    fn parse_ignora_podcast_e_anuncio() {
        let ad: Value = serde_json::from_str(r#"{"is_playing":true,"currently_playing_type":"ad","item":null}"#).unwrap();
        assert!(parse(&ad).is_none(), "anúncio não vira música");
        let ep: Value = serde_json::from_str(r#"{"is_playing":true,"currently_playing_type":"episode","item":{"id":"e1","name":"Episódio"}}"#).unwrap();
        assert!(parse(&ep).is_none(), "podcast fica de fora na v1");
        let nothing: Value = serde_json::from_str(r#"{"currently_playing_type":"track","item":null}"#).unwrap();
        assert!(parse(&nothing).is_none());
    }

    #[test]
    fn estado_só_acende_na_virada_que_o_overlay_nota() {
        let s = States::default();
        let t = |id: &str, playing: bool| Some(Track { id: id.into(), playing, ..Track::default() });
        assert!(s.set("p", t("a", true)), "primeira faixa é mudança");
        assert!(!s.set("p", t("a", true)), "a mesma faixa não reacende");
        assert!(s.set("p", t("a", false)), "pausa é mudança");
        assert!(s.set("p", t("b", true)), "troca de faixa é mudança");
        assert!(s.set("p", None), "parar é mudança");
        assert!(!s.set("p", None), "parada repetida não reacende");
        assert!(s.now("p").is_none());
        let s2 = States::default();
        assert!(s2.mark_error("p", "x"));
        assert!(!s2.mark_error("p", "x"), "erro repetido não enche o Histórico");
        s2.ok("p");
        assert!(s2.mark_error("p", "x"), "leitura limpa permite avisar de novo");
    }

    #[test]
    fn espera_do_429_poupa_a_consulta_repetida() {
        let s = States::default();
        assert!(!s.throttled("p"), "sem espera no começo");
        s.throttle("p", 30);
        assert!(s.throttled("p"), "espera ativa bloqueia a consulta");
        s.throttle("p", 0);
        assert!(!s.throttled("p"), "espera vencida libera a consulta");
    }

    #[test]
    fn so_uma_leitura_por_perfil_e_a_trava_solta_sozinha() {
        let s = States::default();
        let g = s.generation("p");
        {
            let _voo = s.begin("p").expect("primeira leitura entra");
            assert!(s.begin("p").is_none(), "segunda leitura espera a primeira sair");
            // Perfil alheio não disputa a mesma trava.
            assert!(s.begin("outro").is_some(), "cada perfil tem a sua leitura");
        }
        assert!(s.begin("p").is_some(), "leitura terminada devolve a trava");
        // Trava caída (pânico ou descarte da tarefa) não deixa o perfil mudo.
        s.bump("p");
        assert!(s.generation("p") > g, "encerrar a sessão muda o número dela");
        let g2 = s.generation("p");
        s.bump("p");
        assert!(s.generation("p") > g2, "trocar de conta sobe de novo");
        assert!(s.begin("p").is_some(), "nada disso deixa o perfil travado");
    }

    #[tokio::test]
    async fn snapshot_do_obs_traz_config_e_faixa_atual() {
        let dir = tempfile::tempdir().unwrap();
        let rt = crate::engine::Runtime::new(dir.path().join("app")).unwrap();
        let p = "perfil-1";
        let cfg = message(&rt, p);
        assert_eq!(cfg["type"], "spotify.config");
        assert_eq!(cfg["payload"]["profileId"], p);
        assert_eq!(cfg["payload"]["config"]["enabled"], false, "sem registro vem o padrão");
        assert_eq!(cfg["payload"]["config"]["design"]["position"], "bottom-left");
        // Com música no estado, o aperto de mão já entrega a faixa ao OBS recarregado.
        let t = Track {
            id: "t1".into(), name: "Tocando Agora".into(), artist: "Cactus".into(),
            playing: true, duration_ms: 200000, progress_ms: 10000, timestamp: 1234, ..Track::default()
        };
        assert!(rt.spotify.set(p, Some(t)));
        let now = now_message(&rt, p);
        assert_eq!(now["type"], "spotify.now");
        assert_eq!(now["payload"]["profileId"], p);
        assert_eq!(now["payload"]["name"], "Tocando Agora");
        assert_eq!(now["payload"]["durationMs"], 200000);
        assert_eq!(now["payload"]["playing"], true);
    }

    #[test]
    fn teste_do_estudio_marca_rehearsal_para_o_selo() {
        let v = test_payload("p1");
        assert_eq!(v["profileId"], "p1");
        assert_eq!(v["rehearsal"], true, "a marca liga o selo ENSAIO no overlay");
        assert_eq!(v["playing"], true, "o teste desenha tocando, não parado");
        assert_eq!(v["name"], "Faixa de teste");
        assert_eq!(v["durationMs"], 210000);
        assert!(v["timestamp"].as_u64().is_some_and(|n| n > 0));
    }
}
