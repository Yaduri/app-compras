use std::time::Duration;

use futures_util::{Sink, SinkExt, StreamExt};
use shared::ws::{WsClientMessage, WsServerMessage};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

/// Intervalo do heartbeat (o servidor encerra conexões inativas após 90s)
const PING_INTERVAL: Duration = Duration::from_secs(25);
/// Espera máxima entre tentativas de reconexão
const MAX_BACKOFF_SECS: u64 = 30;

/// Eventos entregues à tela
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// Conectado e inscrito na lista
    Connected,
    /// Conexão perdida (o cliente tenta reconectar sozinho)
    Disconnected,
    Message(WsServerMessage),
}

fn ws_url(base_http_url: &str) -> String {
    let base = base_http_url.trim_end_matches('/');
    let base_ws = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else {
        format!("ws://{}", base)
    };
    format!("{}/ws", base_ws)
}

async fn send_msg<S>(sink: &mut S, msg: &WsClientMessage) -> bool
where
    S: Sink<Message> + Unpin,
{
    match serde_json::to_string(msg) {
        Ok(json) => sink.send(Message::Text(json.into())).await.is_ok(),
        Err(_) => false,
    }
}

/// Mantém a sincronização em tempo real da lista, reconectando automaticamente.
/// Termina quando a tela fecha (o receptor do canal é descartado).
pub async fn run_ws_sync(
    base_http_url: String,
    list_id: Uuid,
    token: Option<String>,
    tx: UnboundedSender<WsEvent>,
) {
    let url = ws_url(&base_http_url);
    let mut failures: u32 = 0;

    loop {
        if tx.is_closed() {
            return;
        }

        if let Ok((ws_stream, _)) = connect_async(url.as_str()).await {
            let (mut write, mut read) = ws_stream.split();

            // Autentica pela mensagem (o token não vai na URL) e se inscreve na lista
            let mut ready = true;
            if let Some(t) = &token {
                ready = send_msg(&mut write, &WsClientMessage::Auth { token: t.clone() }).await;
            }
            ready = ready && send_msg(&mut write, &WsClientMessage::Subscribe { list_id }).await;

            if ready {
                let mut ping = tokio::time::interval(PING_INTERVAL);
                ping.tick().await; // o primeiro tick é imediato

                loop {
                    tokio::select! {
                        _ = ping.tick() => {
                            if !send_msg(&mut write, &WsClientMessage::Ping).await {
                                break;
                            }
                        }
                        incoming = read.next() => match incoming {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(server_msg) = serde_json::from_str::<WsServerMessage>(&text) {
                                    let event = match server_msg {
                                        WsServerMessage::Subscribed { .. } => {
                                            failures = 0;
                                            WsEvent::Connected
                                        }
                                        other => WsEvent::Message(other),
                                    };
                                    if tx.send(event).is_err() {
                                        return;
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                            Some(Ok(_)) => {}
                        }
                    }
                }
            }
        }

        if tx.send(WsEvent::Disconnected).is_err() {
            return;
        }

        // Backoff exponencial: 2s, 4s, 8s... até 30s
        failures = failures.saturating_add(1);
        let delay = (1u64 << failures.min(5)).min(MAX_BACKOFF_SECS);
        tokio::time::sleep(Duration::from_secs(delay)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ws_url() {
        assert_eq!(ws_url("https://api.exemplo.com/"), "wss://api.exemplo.com/ws");
        assert_eq!(ws_url("http://127.0.0.1:3000"), "ws://127.0.0.1:3000/ws");
    }
}
