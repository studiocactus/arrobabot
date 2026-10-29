use crate::{engine::Runtime, livepix, model::Event};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use futures_util::{SinkExt,StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async_with_config, tungstenite::{Message, protocol::WebSocketConfig}};

/// HTML do overlay gerado por scripts/build-overlay.cjs a partir de src/overlayApp.ts.
const OVERLAY_HTML: &str = include_str!("../overlay.html");

/// HTML da fonte de áudio "Áudio do BotLive" (GET /audio), gerado pelo mesmo
/// script a partir de src/audioSourceApp.ts. Página só de áudio: nenhum visual
/// e nenhum código de usuário roda aqui.
const AUDIO_HTML: &str = include_str!("../audio.html");

pub async fn serve(rt: Arc<Runtime>) -> Result<(), String> {
    let port = rt.db.get("apiPort").as_u64().unwrap_or(9876).clamp(1024, 65535) as u16;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.map_err(|_| format!("Porta local {port} ocupada. Altere a porta nas configurações e reinicie."))?;
    *rt.api_port.lock().unwrap() = port;
    let limit = Arc::new(tokio::sync::Semaphore::new(16));
    loop {
        let (socket, _) = listener.accept().await.map_err(|_| "Falha na API local")?;
        let Ok(permit) = limit.clone().try_acquire_owned() else { continue };
        let rt = rt.clone();
        tokio::spawn(async move {
            let _permit = permit;
            // Mesma porta atende a fonte do OBS em HTTP (GET /overlay) e o
            // WebSocket da aplicação; a inspeção só olha, nunca consome os bytes.
            if !is_websocket(&socket).await {
                let _ = http(rt, socket).await;
                return;
            }
            let config = WebSocketConfig::default().max_message_size(Some(64 * 1024)).max_frame_size(Some(64 * 1024));
            let Ok(Ok(mut ws)) = tokio::time::timeout(Duration::from_secs(5), accept_async_with_config(socket, Some(config))).await else { return };
            let Ok(Some(Ok(Message::Text(first)))) = tokio::time::timeout(Duration::from_secs(5), ws.next()).await else { return };
            let Ok(auth) = serde_json::from_str::<Value>(&first) else { return };
            let token = auth["token"].as_str().unwrap_or("");
            if let Some(pid)=crate::heart_rate::profile_for(&rt,token,true){
                if auth["mode"]=="heart-publisher"{heart_source(&rt,ws,&pid,token,true).await;return}
                let _=ws.close(None).await;return;
            }
            if let Some(pid)=crate::heart_rate::profile_for(&rt,token,false){
                if auth["mode"]=="heart-overlay"{heart_source(&rt,ws,&pid,token,false).await;return}
                let _=ws.close(None).await;return;
            }
            // Papel do overlay: só enxerga o design e a meta do próprio perfil e
            // nunca envia eventos — a chave dele não abre a API da aplicação.
            let overlay_profile = if token == rt.api_token.as_str() {
                None
            } else {
                match livepix::overlay_profile(&rt, token) {
                    Some(p) => Some(p),
                    None => { let _ = ws.close(None).await; return; }
                }
            };
            if let Some(pid) = &overlay_profile {
                // Fonte de áudio do OBS: recebe só eventos de som do próprio
                // perfil e responde com confirmações de reprodução.
                if auth["mode"].as_str() == Some("audio") {
                    audio_source(&rt, ws, pid).await;
                    return;
                }
                let design = json!({"type": "design", "payload": {"profileId": pid, "design": livepix::design(&rt, pid)}});
                if ws.send(Message::Text(design.to_string().into())).await.is_err() { return; }
                if ws.send(Message::Text(livepix::goal_message(&rt, pid).to_string().into())).await.is_err() { return; }
                if ws.send(Message::Text(crate::alerts::message(&rt, pid).to_string().into())).await.is_err() { return; }
                if ws.send(Message::Text(crate::chat_overlay::message(&rt, pid).to_string().into())).await.is_err() { return; }
                if ws.send(Message::Text(crate::spotify::message(&rt, pid).to_string().into())).await.is_err() { return; }
                if ws.send(Message::Text(crate::spotify::now_message(&rt, pid).to_string().into())).await.is_err() { return; }
            } else {
                if ws.send(Message::Text(json!({"type": "ready", "version": 1}).to_string().into())).await.is_err() { return; }
                for p in rt.db.profiles().unwrap_or_default() {
                    let state = crate::modules::load(&rt, &p.id);
                    if !state.songs.is_empty() {
                        let _ = ws.send(Message::Text(json!({"type": "songs", "payload": {"profileId": p.id, "queue": state.songs}}).to_string().into())).await;
                    }
                }
            }
            let mut events = rt.broadcast.subscribe();
            loop {
                tokio::select! {
                item = ws.next() => {
                    let Some(Ok(msg)) = item else { break };
                    match msg {
                    Message::Text(t) => {
                        if overlay_profile.is_some() { continue; } // canal somente leitura
                        let result = async {
                            let request: Value = serde_json::from_str(&t).map_err(|_| "JSON inválido".to_string())?;
                            if request["type"] != "event" { return Err("Use type: event".to_string()); }
                            let mut e: Event = serde_json::from_value(request["event"].clone()).map_err(|_| "Evento inválido".to_string())?;
                            if e.kind=="heartRate"{return Err("Batimentos exigem a ponte dedicada do sensor".into())}
                            // External events never assert broadcaster/moderator privileges.
                            e.role = "everyone".into();
                            e.id = uuid::Uuid::new_v4().to_string();
                            rt.submit(e).await
                        }.await;
                        let response = match result { Ok(()) => json!({"type": "ack"}), Err(e) => json!({"type": "error", "message": e}) };
                        if ws.send(Message::Text(response.to_string().into())).await.is_err() { break }
                    },
                    Message::Ping(v) => { if ws.send(Message::Pong(v)).await.is_err() { break } },
                    Message::Close(_) => break, _ => {}
                    }
                },
                event = events.recv() => match event {
                    Ok(v) => {
                        if let Some(pid) = &overlay_profile {
                            let kind = v["type"].as_str().unwrap_or("");
                            let own = v["payload"]["profileId"].as_str() == Some(pid.as_str());
                            if !own || !matches!(kind, "goal" | "design" | "alerts" | "alert" | "emote.burst" | "chat" | "chat.config" | "chat.event" | "chat.delete" | "chat.deleteUser" | "chat.clear" | "spotify.now" | "spotify.config") { continue; }
                        }
                        if ws.send(Message::Text(v.to_string().into())).await.is_err() { break }
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break
                }
                }
            }
        });
    }
}

/// Conexão da fonte "Áudio do BotLive": registra a fonte no perfil (a primeira
/// reproduz; as demais ficam em espera) e só aceita confirmações de reprodução
/// daqui — a fonte nunca envia comandos nem eventos administrativos.
async fn audio_source(rt: &Arc<Runtime>, mut ws: tokio_tungstenite::WebSocketStream<TcpStream>, pid: &str) {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(64);
    let (sink_id, _) = crate::audio::register(rt, pid, tx);
    loop {
        tokio::select! {
        item = ws.next() => {
            match item {
            Some(Ok(Message::Text(t))) => {
                // Canal de ida restrito: só a confirmação de reprodução, com
                // forma validada; qualquer outra mensagem é ignorada.
                if let Ok(msg) = serde_json::from_str::<Value>(&t) {
                    if msg["type"] == "audio.ack" {
                        let _ = crate::audio::ack(
                            rt,
                            pid,
                            msg["id"].as_str().unwrap_or(""),
                            msg["phase"].as_str().unwrap_or(""),
                            msg["detail"].as_str(),
                        );
                    }
                }
            },
            Some(Ok(Message::Ping(v))) => { if ws.send(Message::Pong(v)).await.is_err() { break } },
            Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
            _ => {}
            }
        },
        msg = rx.recv() => {
            let Some(msg) = msg else { break };
            if ws.send(Message::Text(msg.to_string().into())).await.is_err() { break }
        },
        }
    }
    crate::audio::unregister(rt, pid, sink_id);
}

/// Só decide olhando o começo da requisição; os bytes seguem intactos para o
/// handshake do WebSocket quando for o caso.
async fn is_websocket(socket: &TcpStream) -> bool {
    let mut head = [0u8; 4096];
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        // peek does not consume bytes: each observation must replace the buffer.
        let n = match tokio::time::timeout_at(deadline, socket.peek(&mut head)).await {
            Ok(Ok(k)) if k > 0 => k,
            _ => return false,
        };
        if head[..n].windows(4).any(|w| w == b"\r\n\r\n") {
            let low = head[..n].to_ascii_lowercase();
            return low.starts_with(b"get ") && low.windows(18).any(|w| w == b"upgrade: websocket");
        }
        if n == head.len() || tokio::time::Instant::now() >= deadline { return false; }
        // Already-buffered bytes remain readable; yield before checking for more.
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

/// Responde às fontes do OBS (GET /overlay e GET /audio), aos sons
/// (GET /asset?token=…&id=…) e ao texto para voz (GET /tts?token=…&id=…),
/// sem depender de arquivo local.
async fn http(rt: Arc<Runtime>, mut socket: TcpStream) -> Result<(), String> {
    let mut buf = vec![0u8; 8192];
    let mut n = 0usize;
    loop {
        if n >= buf.len() { break; }
        let read = tokio::time::timeout(Duration::from_secs(5), socket.read(&mut buf[n..])).await;
        match read {
            Ok(Ok(k)) if k > 0 => {
                n += k;
                if buf[..n].windows(4).any(|w| w == b"\r\n\r\n") { break; }
            }
            _ => return Ok(()),
        }
    }
    let text = String::from_utf8_lossy(&buf[..n]).to_string();
    let mut parts = text.lines().next().unwrap_or("").split(' ');
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    let path = target.split('?').next().unwrap_or("/");
    let (status, body, ctype): (&str, Vec<u8>, &str) = if method == "GET" && path == "/overlay" {
        ("200 OK", OVERLAY_HTML.as_bytes().to_vec(), "text/html; charset=utf-8")
    } else if method == "GET" && path == "/heart-connect" {
        ("200 OK",include_str!("../../scripts/heart-connect.html").as_bytes().to_vec(),"text/html; charset=utf-8")
    } else if method == "GET" && path == "/heart-overlay" {
        ("200 OK",include_str!("../../scripts/heart-overlay.html").as_bytes().to_vec(),"text/html; charset=utf-8")
    } else if method == "GET" && path == "/audio" {
        ("200 OK", AUDIO_HTML.as_bytes().to_vec(), "text/html; charset=utf-8")
    } else if method == "GET" && path == "/tts" {
        match serve_tts(&rt, target) {
            Ok((bytes, mime)) => ("200 OK", bytes, mime),
            Err(_) => ("404 Not Found", b"N\xc3\xa3o encontrado.\r\n".to_vec(), "text/plain; charset=utf-8"),
        }
    } else if method == "GET" && path == "/asset" {
        match serve_asset(&rt, target) {
            Ok((bytes, mime)) => ("200 OK", bytes, mime),
            Err(_) => ("404 Not Found", b"N\xc3\xa3o encontrado.\r\n".to_vec(), "text/plain; charset=utf-8"),
        }
    } else if method == "GET" {
        ("404 Not Found", b"N\xc3\xa3o encontrado.\r\n".to_vec(), "text/plain; charset=utf-8")
    } else {
        ("405 Method Not Allowed", b"M\xc3\xa9todo n\xc3\xa3o suportado.\r\n".to_vec(), "text/plain; charset=utf-8")
    };
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\r\n",
        body.len()
    );
    let _ = socket.write_all(header.as_bytes()).await;
    let _ = socket.write_all(&body).await;
    let _ = socket.shutdown().await;
    Ok(())
}

/// Um som só sai se o token pertencer ao perfil e o id existir na lista de
/// arquivos dele — a rota nunca serve caminho arbitrário.
fn serve_asset(rt: &Arc<Runtime>, target: &str) -> Result<(Vec<u8>, &'static str), String> {
    let token = query(target, "token").ok_or("Chave ausente")?;
    let id = query(target, "id").ok_or("Som ausente")?;
    let pid = livepix::overlay_profile(rt, &token).ok_or("Chave de overlay inválida")?;
    crate::alerts::sound_file(rt, &pid, &id)
}

/// WAV do texto para voz gerado neste perfil: id em formato estrito, servido
/// só da pasta de cache do próprio perfil.
fn serve_tts(rt: &Arc<Runtime>, target: &str) -> Result<(Vec<u8>, &'static str), String> {
    let token = query(target, "token").ok_or("Chave ausente")?;
    let id = query(target, "id").ok_or("Áudio ausente")?;
    let pid = livepix::overlay_profile(rt, &token).ok_or("Chave de overlay inválida")?;
    crate::tts::file(rt, &pid, &id)
}

fn query(target: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    target.split('?').nth(1)?.split('&').find_map(|part| {
        part.strip_prefix(prefix.as_str()).filter(|v| !v.is_empty() && v.len() <= 64).map(|v| v.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publisher_rejeita_comandos_e_so_confirma_bpm_validado(){
        let dir=tempfile::tempdir().unwrap();let rt=Runtime::new(dir.path().into()).unwrap();
        let p:crate::model::Profile=serde_json::from_value(json!({"id":uuid::Uuid::new_v4().to_string(),"name":"Sensor socket","platform":"twitch","channel":"qa","modules":{"heartRate":true}})).unwrap();rt.db.save_profile(&p).unwrap();*rt.api_port.lock().unwrap()=12345;
        let links=crate::heart_rate::operation(&rt,&p.id,"heartRate.prepare",&json!({})).unwrap();let key=links["bridgeUrl"].as_str().unwrap().split("#token=").nth(1).unwrap().to_owned();
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let addr=listener.local_addr().unwrap();let rt2=rt.clone();let pid=p.id.clone();
        let task=tokio::spawn(async move{let (socket,_)=listener.accept().await.unwrap();let ws=tokio_tungstenite::accept_async(socket).await.unwrap();heart_source(&rt2,ws,&pid,&key,true).await;});
        let (mut client,_)=tokio_tungstenite::connect_async(format!("ws://{addr}")).await.unwrap();
        async fn next(client:&mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>)->Value{let msg=tokio::time::timeout(Duration::from_secs(3),client.next()).await.unwrap().unwrap().unwrap();serde_json::from_str(msg.to_text().unwrap()).unwrap()}
        assert_eq!(next(&mut client).await["type"],"heart.ready");
        for value in [json!({"type":"event","event":{"kind":"heartRate"}}),json!({"type":"heartRate","packet":[1]})]{client.send(Message::Text(value.to_string().into())).await.unwrap();assert_eq!(next(&mut client).await["type"],"error");assert!(crate::heart_rate::state(&rt,&p.id,false)["bpm"].is_null());}
        client.send(Message::Text(json!({"type":"heartRate","packet":[0,83]}).to_string().into())).await.unwrap();let accepted=next(&mut client).await;assert_eq!(accepted["type"],"heart-reading");assert_eq!(accepted["payload"]["bpm"],83);assert!(crate::heart_rate::state(&rt,&p.id,true)["bpm"].is_null());
        client.close(None).await.unwrap();tokio::time::timeout(Duration::from_secs(3),task).await.unwrap().unwrap();assert!(crate::heart_rate::state(&rt,&p.id,false)["bpm"].is_null());
    }

    #[tokio::test]
    async fn websocket_header_can_arrive_in_fragments_without_consuming_bytes() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).await.unwrap();
        let (mut server, _) = listener.accept().await.unwrap();
        let first = b"GET / HTTP/1.1\r\nHost: localhost\r\n";
        let last = b"Upgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        client.write_all(first).await.unwrap();
        let writer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client.write_all(last).await.unwrap();
            client
        });
        assert!(is_websocket(&server).await, "fragmented handshake must remain a WebSocket");
        let _client = writer.await.unwrap();
        let mut received = vec![0; first.len() + last.len()];
        server.read_exact(&mut received).await.unwrap();
        assert_eq!(received, [first.as_slice(), last.as_slice()].concat());
    }
}

/// As capacidades de sensor não têm acesso à API geral ou a outros overlays.
async fn heart_source(rt:&Arc<Runtime>,mut ws:tokio_tungstenite::WebSocketStream<TcpStream>,pid:&str,key:&str,publisher:bool){
 if publisher {if let Err(err)=crate::heart_rate::connected(rt,pid,key,true){let _=ws.send(Message::Text(json!({"type":"error","message":err}).to_string().into())).await;return}}
 let initial=if publisher{json!({"type":"heart.ready"})}else{json!({"type":"heart-rate","payload":crate::heart_rate::state(rt,pid,true)})};
 if ws.send(Message::Text(initial.to_string().into())).await.is_err(){if publisher{let _=crate::heart_rate::connected(rt,pid,key,false);}return}
 let mut updates=rt.broadcast.subscribe();let mut tick=tokio::time::interval(Duration::from_secs(2));
 loop{tokio::select!{
  _=tick.tick()=>{if crate::heart_rate::profile_for(rt,key,publisher).as_deref()!=Some(pid){break}if !publisher&&ws.send(Message::Text(json!({"type":"heart-rate","payload":crate::heart_rate::state(rt,pid,true)}).to_string().into())).await.is_err(){break}},
  item=ws.next()=>{match item{
   Some(Ok(Message::Text(raw))) if publisher=>{
    if crate::heart_rate::profile_for(rt,key,true).as_deref()!=Some(pid){break}
    let response=match serde_json::from_str::<Value>(&raw){Ok(v) if v["type"]=="heartRate"=>crate::heart_rate::receive(rt,pid,key,&v["packet"]).await, _=>Err("Esta conexão só aceita medições Bluetooth".into())};
    let reply=match response{Err(err)=>json!({"type":"error","message":err}),Ok(())=>json!({"type":"heart-reading","payload":crate::heart_rate::state(rt,pid,false)})};
    if ws.send(Message::Text(reply.to_string().into())).await.is_err(){break}
   },Some(Ok(Message::Ping(v)))=>{if ws.send(Message::Pong(v)).await.is_err(){break}},Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,_=>{}
  }},
  value=updates.recv()=>{if let Ok(v)=value{if !publisher&&v["type"]=="heart-rate"&&v["payload"]["profileId"]==pid{if ws.send(Message::Text(v.to_string().into())).await.is_err(){break}}}}
 }}
 if publisher{let _=crate::heart_rate::connected(rt,pid,key,false);rt.emit("heart-rate",crate::heart_rate::state(rt,pid,true));}
 let _=ws.close(None).await;
}
