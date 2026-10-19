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

/// Nomes e imagens juntos: o nome vai no prompt da IA, o id monta a imagem da CDN.
#[derive(Clone)]
pub struct Emote { pub id: String, pub name: String }

/// Nomes guardados e a hora da última busca (unix em segundos).
pub fn cached(db: &crate::db::Db, p: &str) -> (Vec<String>, i64) {
    let v: Value = db.module(p, "ai_emotes");
    let names = v["names"].as_array().map(|a| a.iter().filter_map(|n| n.as_str().map(str::to_owned)).filter(|n| !n.trim().is_empty()).collect()).unwrap_or_default();
    (names, v["at"].as_i64().unwrap_or(0))
}
/// Os nomes prontos para o prompt, sem chamada de rede.
pub fn list(db: &crate::db::Db, p: &str) -> String { cached(db, p).0.join(", ") }
/// Os ids prontos para a imagem da CDN (explosão de emotes), sem chamada de rede.
pub fn ids(db: &crate::db::Db, p: &str) -> Vec<String> {
    let v: Value = db.module(p, "ai_emotes");
    v["list"].as_array().map(|a| a.iter().filter_map(|e| e["id"].as_str()).filter(|id| !id.is_empty()).map(str::to_owned).collect()).unwrap_or_default()
}
fn store(db: &crate::db::Db, p: &str, list: &[Emote], ext: &std::collections::HashMap<String, String>, at: i64) {
    let names: Vec<&str> = list.iter().map(|e| e.name.as_str()).collect();
    let list: Vec<Value> = list.iter().map(|e| json!({"id": e.id, "name": e.name})).collect();
    let extv: Value = ext.iter().map(|(k, u)| (k.clone(), json!(u))).collect();
    let _ = db.set_module(p, "ai_emotes", &json!({"at": at, "names": names, "list": list, "ext": extv}));
}
/// Busca os emotes do canal e os globais e atualiza a lista guardada. Roda na conexão
/// do canal: falha nenhuma interrompe a live, as respostas apenas seguem sem emote.
pub async fn refresh(rt: &crate::engine::Runtime, p: &Profile) {
    if p.platform != "twitch" { return; }
    let (_, at) = cached(&rt.db, &p.id);
    // Cache antigo (só nomes, sem lista) busca de novo: a explosão precisa dos ids.
    let has_ids = !ids(&rt.db, &p.id).is_empty();
    if chrono::Utc::now().timestamp() - at < TTL && has_ids { return; }
    match tokio::time::timeout(std::time::Duration::from_secs(10), fetch(rt, p)).await {
        Ok(Ok((fresh, ext))) => {
            let total = fresh.len();
            let extras = ext.len();
            store(&rt.db, &p.id, &fresh, &ext, chrono::Utc::now().timestamp());
            notice(rt, &p.id, "emotes", &format!("Emotes da Twitch carregados: {total}, mais {extras} de terceiros (BTTV/7TV/FFZ)."));
        }
        Ok(Err(err)) => retry(rt, p, &err),
        Err(_) => retry(rt, p, "a Twitch não respondeu a tempo"),
    }
}
/// Marca para tentar de novo daqui a 5 minutos e avisa uma vez por sessão.
/// A lista antiga fica no lugar: uma falha não apaga o que já funcionava.
fn retry(rt: &crate::engine::Runtime, p: &Profile, err: &str) {
    let mut v: Value = rt.db.module(&p.id, "ai_emotes");
    v["at"] = json!(chrono::Utc::now().timestamp() - TTL + RETRY);
    let _ = rt.db.set_module(&p.id, "ai_emotes", &v);
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
async fn fetch(rt: &crate::engine::Runtime, p: &Profile) -> Result<(Vec<Emote>, std::collections::HashMap<String, String>), String> {
    let token = crate::oauth::token(&rt.http, p, "bot").await?;
    let channel = helix(rt, p, &token, Some(&p.channel_id)).await?;
    let global = helix(rt, p, &token, None).await?;
    Ok((pick(channel, global), fetch_ext(rt, p).await))
}

/// Emotes de terceiros (BTTV, 7TV e FFZ): a Twitch os manda como palavra
/// comum, e o overlay desenha a imagem pelo nome guardado aqui. Melhor
/// esforço — qualquer falha de rede simplesmente não contribui com emotes.
pub fn ext(db: &crate::db::Db, p: &str) -> std::collections::HashMap<String, String> {
    let v: Value = db.module(p, "ai_emotes");
    v["ext"]
        .as_object()
        .map(|o| o.iter().filter_map(|(k, u)| Some((k.clone(), u.as_str()?.to_owned()))).collect())
        .unwrap_or_default()
}

async fn fetch_ext(rt: &crate::engine::Runtime, p: &Profile) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut out: HashMap<String, String> = HashMap::new();
    if p.platform != "twitch" || p.channel_id.is_empty() {
        return out;
    }
    let id = p.channel_id.as_str();
    // BTTV: emotes do canal (inclui os compartilhados) e os globais.
    if let Ok(v) = get_json(rt, &format!("https://api.betterttv.net/3/cached/users/twitch/{id}")).await {
        for key in ["channelEmotes", "sharedEmotes"] {
            for e in v[key].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
                if let (Some(eid), Some(code)) = (e["id"].as_str(), e["code"].as_str()) {
                    out.entry(code.to_owned()).or_insert_with(|| format!("https://cdn.betterttv.net/emote/{eid}/3x"));
                }
            }
        }
    }
    if let Ok(v) = get_json(rt, "https://api.betterttv.net/3/cached/emotes/global").await {
        for e in v.as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
            if let (Some(eid), Some(code)) = (e["id"].as_str(), e["code"].as_str()) {
                out.entry(code.to_owned()).or_insert_with(|| format!("https://cdn.betterttv.net/emote/{eid}/3x"));
            }
        }
    }
    // 7TV: conjunto do canal e o global.
    for url in [format!("https://7tv.io/v3/users/twitch/{id}"), "https://7tv.io/v3/emote-sets/global".to_owned()] {
        if let Ok(v) = get_json(rt, &url).await {
            let set = if v["emote_set"].is_object() { &v["emote_set"] } else { &v };
            for e in set["emotes"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
                if let (Some(eid), Some(name)) = (e["id"].as_str(), e["name"].as_str()) {
                    out.entry(name.to_owned()).or_insert_with(|| format!("https://cdn.7tv.app/emote/{eid}/3x.webp"));
                }
            }
        }
    }
    // FFZ: a sala e o conjunto global; a imagem maior vem em urls.4.
    for url in [format!("https://api.frankerfacez.com/v1/room/id/{id}"), "https://api.frankerfacez.com/v1/set/global".to_owned()] {
        if let Ok(v) = get_json(rt, &url).await {
            let sets: Vec<Value> = match v["sets"].as_object() {
                Some(map) => map.values().cloned().collect(),
                None => vec![v.clone()],
            };
            for set in sets {
                for e in set["emoticons"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
                    let name = e["name"].as_str().unwrap_or("");
                    let url = e["urls"]["4"].as_str().or(e["urls"]["2"].as_str()).or(e["urls"]["1"].as_str()).unwrap_or("");
                    if !name.is_empty() && !url.is_empty() {
                        let url = if url.starts_with("//") { format!("https:{url}") } else { url.to_owned() };
                        out.entry(name.to_owned()).or_insert(url);
                    }
                }
            }
        }
    }
    while out.len() > 800 {
        if let Some(k) = out.keys().next().cloned() {
            out.remove(&k);
        } else {
            break;
        }
    }
    out
}

async fn get_json(rt: &crate::engine::Runtime, url: &str) -> Result<Value, String> {
    let res = tokio::time::timeout(std::time::Duration::from_secs(8), rt.http.get(url).send())
        .await
        .map_err(|_| "sem resposta".to_string())?
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status().as_u16()));
    }
    res.json().await.map_err(|e| e.to_string())
}
async fn helix(rt: &crate::engine::Runtime, p: &Profile, token: &str, broadcaster: Option<&str>) -> Result<Vec<Emote>, String> {
    let mut req = crate::helix::get(&rt.http, p, token, "chat/emotes", &[]);
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
    Ok(data.iter().filter_map(|e| {
        let id = e["id"].as_str().unwrap_or("");
        let name = e["name"].as_str().unwrap_or("");
        if id.is_empty() || name.is_empty() { None } else { Some(Emote { id: id.to_owned(), name: name.to_owned() }) }
    }).collect())
}
/// Junta os emotes do canal com os globais, sem repetir e sem passar do limite que
/// cabe no prompt. Os do canal vêm primeiro: são os que identificam este chat.
pub fn pick(channel: Vec<Emote>, global: Vec<Emote>) -> Vec<Emote> {
    let mut out: Vec<Emote> = Vec::new();
    for e in channel.into_iter().chain(global) {
        if out.len() >= LIMIT { break; }
        let n = e.name.trim();
        if n.is_empty() || out.iter().any(|x| x.name == n) { continue; }
        out.push(Emote { id: e.id, name: n.to_owned() });
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    fn e(id: &str, name: &str) -> Emote { Emote { id: id.into(), name: name.into() } }
    #[test]
    fn keeps_channel_emotes_first_and_never_repeats() {
        let got = pick(vec![e("1", "arrobaGG"), e("2", "COEE")], vec![e("88", "COEE"), e("3", "Kappa")]);
        let names: Vec<&str> = got.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names, vec!["arrobaGG", "COEE", "Kappa"]);
        assert_eq!(got[1].id, "2", "emote repetido mantém o id do canal");
        assert!(pick(vec![e("1", " ")], vec![]).is_empty(), "nome vazio não vira emote");
    }
    #[test]
    fn stops_at_the_prompt_limit() {
        let channel: Vec<Emote> = (0..100).map(|i| e(&format!("id{i}"), &format!("c{i}"))).collect();
        let got = pick(channel, vec![e("9", "Kappa")]);
        assert_eq!(got.len(), LIMIT, "o prompt não recebe lista infinita");
        assert_eq!(got.last().unwrap().name, "c59", "os emotes do canal vêm antes dos globais");
        assert!(pick(vec![], vec![e("9", "Kappa")]).iter().any(|x| x.name == "Kappa"));
    }
    #[test]
    fn cache_roundtrip_reads_back_what_was_stored() {
        let db = crate::db::Db::open(std::path::Path::new(":memory:")).unwrap();
        let p = crate::model::Profile{id:uuid::Uuid::new_v4().to_string(),name:"P".into(),platform:"twitch".into(),channel:"canal".into(),channel_id:"123".into(),bot_id:"456".into(),client_id:"client".into(),blocklist:vec![],topics:vec![],editors:vec![],ai:crate::model::AiConfig::default(),modules:serde_json::json!({})};
        db.save_profile(&p).unwrap();
        assert_eq!(list(&db, &p.id), "", "sem busca, sem emote no prompt");
        assert!(ids(&db, &p.id).is_empty(), "sem busca, sem id para a explosão");
        assert_eq!(cached(&db, &p.id).1, 0);
        let mut extras = std::collections::HashMap::new();
        extras.insert("KappaExt".to_owned(), "https://cdn.7tv.app/emote/1/3x.webp".to_owned());
        store(&db, &p.id, &[e("25", "Kappa"), e("88", "LUL")], &extras, 1_700_000_000);
        assert_eq!(list(&db, &p.id), "Kappa, LUL");
        assert_eq!(ids(&db, &p.id), vec!["25", "88"], "o id acompanha o nome para a CDN");
        assert_eq!(ext(&db, &p.id).get("KappaExt").map(String::as_str), Some("https://cdn.7tv.app/emote/1/3x.webp"), "emote de terceiros sobrevive ao cache");
        assert_eq!(cached(&db, &p.id).1, 1_700_000_000);
        assert!(ids(&db, "outro").is_empty(), "cada perfil guarda a sua lista");
    }
}
