use futures_util::{SinkExt, StreamExt};
use shared::ws::{WsClientMessage, WsServerMessage};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

/// Inicia a conexão WebSocket em tempo real para sincronizar a lista ativa
pub async fn start_ws_sync(
    base_http_url: String,
    list_id: Uuid,
    token: Option<String>,
    tx: UnboundedSender<WsServerMessage>,
) {
    let base_ws_url = if base_http_url.starts_with("https://") {
        base_http_url.replace("https://", "wss://")
    } else if base_http_url.starts_with("http://") {
        base_http_url.replace("http://", "ws://")
    } else {
        format!("ws://{}", base_http_url)
    };

    let url_with_query = if let Some(t) = token {
        format!("{}/ws?token={}", base_ws_url.trim_end_matches('/'), t)
    } else {
        format!("{}/ws", base_ws_url.trim_end_matches('/'))
    };

    if let Ok((ws_stream, _)) = connect_async(&url_with_query).await {
        let (mut write, mut read) = ws_stream.split();

        // Inscreve-se no canal da lista atual
        let sub_msg = WsClientMessage::Subscribe { list_id };
        if let Ok(json) = serde_json::to_string(&sub_msg) {
            let _ = write.send(Message::Text(json.into())).await;
        }

        // Drena e encaminha eventos recebidos pelo WebSocket para o canal MPSC do Dioxus
        while let Some(msg_result) = read.next().await {
            if let Ok(msg) = msg_result {
                match msg {
                    Message::Text(text) => {
                        if let Ok(server_msg) = serde_json::from_str::<WsServerMessage>(&text) {
                            if tx.send(server_msg).is_err() {
                                break;
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            } else {
                break;
            }
        }
    }
}
