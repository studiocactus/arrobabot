use crate::{engine::Runtime, livepix, model::Event};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use futures_util::{SinkExt,StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async_with_config, tungstenite::{Message, protocol::WebSocketConfig}};

/// HTML do overlay gerado por scripts/build-overlay.cjs a partir de src/overlayApp.ts.
const OVERLAY_HTML: &str = include_str!("../overlay.html");

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
                let _ = http(socket).await;
                return;
            }
            let config = WebSocketConfig::default().max_message_size(Some(64 * 1024)).max_frame_size(Some(64 * 1024));
            let Ok(Ok(mut ws)) = tokio::time::timeout(Duration::from_secs(5), accept_async_with_config(socket, Some(config))).await else { return };
            let Ok(Some(Ok(Message::Text(first)))) = tokio::time::timeout(Duration::from_secs(5), ws.next()).await else { return };
            let Ok(auth) = serde_json::from_str::<Value>(&first) else { return };
            let token = auth["token"].as_str().unwrap_or("");
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
                let design = json!({"type": "design", "payload": {"profileId": pid, "design": livepix::design(&rt, pid)}});
                if ws.send(Message::Text(design.to_string().into())).await.is_err() { return; }
                if ws.send(Message::Text(livepix::goal_message(&rt, pid).to_string().into())).await.is_err() { return; }
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
                            if !own || (kind != "goal" && kind != "design") { continue; }
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

/// Só decide olhando o começo da requisição; os bytes seguem intactos para o
/// handshake do WebSocket quando for o caso.
async fn is_websocket(socket: &TcpStream) -> bool {
    let mut head = [0u8; 4096];
    let mut n = 0usize;
    loop {
        if n >= head.len() { return false; }
        let peeked = match tokio::time::timeout(Duration::from_secs(5), socket.peek(&mut head[n..])).await {
            Ok(Ok(k)) if k > 0 => k,
            _ => return false,
        };
        n += peeked;
        if head[..n].windows(4).any(|w| w == b"\r\n\r\n") {
            let low = head[..n].to_ascii_lowercase();
            return low.starts_with(b"get ") && low.windows(18).any(|w| w == b"upgrade: websocket");
        }
    }
}

/// Responde o overlay do OBS (GET /overlay) sem depender de arquivo local.
async fn http(mut socket: TcpStream) -> Result<(), String> {
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
    let (status, body, ctype) = if method == "GET" && path == "/overlay" {
        ("200 OK", OVERLAY_HTML, "text/html; charset=utf-8")
    } else if method == "GET" {
        ("404 Not Found", "Não encontrado.\r\n", "text/plain; charset=utf-8")
    } else {
        ("405 Method Not Allowed", "Método não suportado.\r\n", "text/plain; charset=utf-8")
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\r\n{body}",
        body.len()
    );
    let _ = socket.write_all(response.as_bytes()).await;
    let _ = socket.shutdown().await;
    Ok(())
}
