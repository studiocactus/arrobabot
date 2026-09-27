//! Emotes da Twitch para as respostas da IA.
//! A lista é buscada quando o canal conecta e fica guardada no perfil: a IA recebe os
//! nomes junto da regra e decide onde cabe um, sem custo de rede em cada resposta.
//! As APIs usadas (chat/emotes do canal e globais) pedem só um token de usuário, sem
//! escopo novo, então nada muda na autorização das contas.
use crate::model::Profile;
use serde_json::{json,Value};

/// Vale 12 horas: a lista muda quando alguém ganha emote novo, não a cada live.
pub const TTL: i64 = 12 * 3600;
/// Quando a busca falha, a próxima tentativa acontece em 5 minutos, não na resposta seguinte.
const RETRY: i64 = 300;
/// Limite do que entra no prompt. A Twitch tem centenas de emotes globais e a regra
/// precisa continuar curta para não empurrar a conversa para fora do contexto.
const LIMIT: usize = 60;

/// Nomes guardados e a hora da última busca (unix em segundos).
pub fn cached(db: &crate::db::Db, p: &str) -> (Vec<String>, i64) {
    let v: Value = db.module(p, "ai_emotes");
    let names = v["names"].as_array().map(|a| a.iter().filter_map(|n| n.as_str().map(str::to_owned)).filter(|n| !n.trim().is_empty()).collect()).unwrap_or_default();
    (names, v["at"].as_i64().unwrap_or(0))
}
/// Os nomes prontos para o prompt, sem chamada de rede.
pub fn list(db: &crate::db::Db, p: &str) -> String { cached(db, p).0.join(", ") }
fn store(db: &crate::db::Db, p: &str, names: &[String], at: i64) {
    let _ = db.set_module(p, "ai_emotes", &json!({"at": at, "names": names}));
}
/// Busca os emotes do canal e os globais e atualiza a lista guardada. Roda na conexão
/// do canal: falha nenhuma interrompe a live, as respostas apenas seguem sem emote.
pub async fn refresh(rt: &crate::engine::Runtime, p: &Profile) {
    if p.platform != "twitch" { return; }
    let (names, at) = cached(&rt.db, &p.id);
    if chrono::Utc::now().timestamp() - at < TTL { return; }
    match tokio::time::timeout(std::time::Duration::from_secs(10), fetch(rt, p)).await {
        Ok(Ok(fresh)) => {
            let total = fresh.len();
            store(&rt.db, &p.id, &fresh, chrono::Utc::now().timestamp());
            notice(rt, &p.id, "emotes", &format!("Emotes da Twitch carregados para a IA: {total}."));
        }
        Ok(Err(err)) => retry(rt, p, &names, &err),
        Err(_) => retry(rt, p, &names, "a Twitch não respondeu a tempo"),
    }
}
/// Marca para tentar de novo daqui a 5 minutos e avisa uma vez por sessão.
fn retry(rt: &crate::engine::Runtime, p: &Profile, names: &[String], err: &str) {
    store(&rt.db, &p.id, names, chrono::Utc::now().timestamp() - TTL + RETRY);
    notice(rt, &p.id, "emotes.error", &format!("Sem emotes da Twitch agora: {err}. As respostas seguem sem emote."));
}
/// Aviso uma vez por sessão: a conexão reconecta várias vezes e o histórico não enche.
fn notice(rt: &crate::engine::Runtime, p: &str, kind: &str, msg: &str) {
    static SEEN: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    let mut seen = SEEN.lock().unwrap();
    let key = format!("{p}:{kind}");
    if seen.iter().any(|k| *k == key) { return; }
    if seen.len() > 100 { seen.clear(); }
    seen.push(key);
    rt.log(p, "ai", msg, "info");
}
async fn fetch(rt: &crate::engine::Runtime, p: &Profile) -> Result<Vec<String>, String> {
    let token = crate::oauth::token(&rt.http, p, "bot").await?;
    let channel = helix(rt, p, &token, Some(&p.channel_id)).await?;
    let global = helix(rt, p, &token, None).await?;
    Ok(pick(channel, global))
}
async fn helix(rt: &crate::engine::Runtime, p: &Profile, token: &str, broadcaster: Option<&str>) -> Result<Vec<String>, String> {
    let mut req = rt.http.get("https://api.twitch.tv/helix/chat/emotes")
        .header("Client-Id", p.client_id.as_str()).bearer_auth(token);
    if let Some(id) = broadcaster { req = req.query(&[("broadcaster_id", id)]); }
    let res = req.send().await.map_err(|_| "sem resposta da Twitch".to_owned())?;
    if !res.status().is_success() {
        return Err(match res.status().as_u16() {
            401 => "a autorização do canal expirou".into(),
            403 => "a Twitch negou o acesso".into(),
            s => format!("HTTP {s}"),
        });
    }
    let v: Value = res.json().await.map_err(|_| "resposta inválida da Twitch".to_owned())?;
    let data = v["data"].as_array().ok_or("a Twitch não devolveu a lista de emotes")?;
    Ok(data.iter().filter_map(|e| e["name"].as_str().map(str::to_owned)).collect())
}
/// Junta os emotes do canal com os globais, sem repetir e sem passar do limite que
/// cabe no prompt. Os do canal vêm primeiro: são os que identificam este chat.
pub fn pick(channel: Vec<String>, global: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in channel.into_iter().chain(global) {
        if out.len() >= LIMIT { break; }
        let n = name.trim();
        if n.is_empty() || out.iter().any(|x| x == n) { continue; }
        out.push(n.to_owned());
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_channel_emotes_first_and_never_repeats() {
        let got = pick(vec!["arrobaGG".into(), "COEE".into()], vec!["COEE".into(), "Kappa".into()]);
        assert_eq!(got, vec!["arrobaGG", "COEE", "Kappa"]);
        assert!(pick(vec![" ".into()], vec![]).is_empty(), "nome vazio não vira emote");
    }
    #[test]
    fn stops_at_the_prompt_limit() {
        let channel: Vec<String> = (0..100).map(|i| format!("c{i}")).collect();
        let got = pick(channel, vec!["Kappa".into()]);
        assert_eq!(got.len(), LIMIT, "o prompt não recebe lista infinita");
        assert_eq!(got.last().unwrap(), "c59", "os emotes do canal vêm antes dos globais");
        assert!(pick(vec![], vec!["Kappa".into()]).contains(&"Kappa".to_owned()));
    }
    #[test]
    fn cache_roundtrip_reads_back_what_was_stored() {
        let db = crate::db::Db::open(std::path::Path::new(":memory:")).unwrap();
        let p = crate::model::Profile{id:uuid::Uuid::new_v4().to_string(),name:"P".into(),platform:"twitch".into(),channel:"canal".into(),channel_id:"123".into(),bot_id:"456".into(),client_id:"client".into(),blocklist:vec![],topics:vec![],editors:vec![],ai:crate::model::AiConfig::default(),modules:serde_json::json!({})};
        db.save_profile(&p).unwrap();
        assert_eq!(list(&db, &p.id), "", "sem busca, sem emote no prompt");
        assert_eq!(cached(&db, &p.id).1, 0);
        store(&db, &p.id, &["Kappa".to_owned(), "LUL".to_owned()], 1_700_000_000);
        assert_eq!(list(&db, &p.id), "Kappa, LUL");
        assert_eq!(cached(&db, &p.id).1, 1_700_000_000);
        assert_eq!(list(&db, "outro"), "", "cada perfil guarda a sua lista");
    }
}
