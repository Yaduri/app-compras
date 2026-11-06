use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::auth::verify_jwt;
use crate::state::AppState;
use shared::ws::{WsClientMessage, WsServerMessage};

#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user_id = query
        .token
        .as_deref()
        .and_then(|t| verify_jwt(t, &state.config.jwt_secret).ok())
        .and_then(|c| Uuid::parse_str(&c.sub).ok());

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: Option<Uuid>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Canal local MPSC para concentrar todas as mensagens a serem enviadas ao cliente
    let (client_tx, mut client_rx) = mpsc::channel::<WsServerMessage>(128);

    // Tarefa que drena as mensagens do canal MPSC e envia via WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if let Ok(json_text) = serde_json::to_string(&msg) {
                if ws_sender.send(Message::Text(json_text.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Mapa de inscrições ativas da conexão: list_id -> JoinHandle
    let mut active_subscriptions: HashMap<Uuid, JoinHandle<()>> = HashMap::new();

    // Loop de recepção das mensagens enviadas pelo cliente
    while let Some(Ok(message)) = ws_receiver.next().await {
        match message {
            Message::Text(text) => {
                if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client_msg {
                        WsClientMessage::Subscribe { list_id } => {
                            // Se já estiver inscrito, não duplica
                            if !active_subscriptions.contains_key(&list_id) {
                                let mut broadcast_rx = state.hub.subscribe(list_id).await;
                                let tx_clone = client_tx.clone();

                                let handle = tokio::spawn(async move {
                                    while let Ok(broadcast_msg) = broadcast_rx.recv().await {
                                        if tx_clone.send(broadcast_msg).await.is_err() {
                                            break;
                                        }
                                    }
                                });

                                active_subscriptions.insert(list_id, handle);

                                // Notifica confirmação de inscrição
                                let _ = client_tx.send(WsServerMessage::Subscribed { list_id }).await;
                                tracing::debug!(
                                    "Cliente {:?} conectado e inscrito na lista {}",
                                    user_id,
                                    list_id
                                );
                            }
                        }
                        WsClientMessage::Unsubscribe { list_id } => {
                            if let Some(handle) = active_subscriptions.remove(&list_id) {
                                handle.abort();
                                tracing::debug!(
                                    "Cliente {:?} cancelou inscrição da lista {}",
                                    user_id,
                                    list_id
                                );
                            }
                        }
                        WsClientMessage::Ping => {
                            let _ = client_tx.send(WsServerMessage::Pong).await;
                        }
                    }
                }
            }
            Message::Ping(_bytes) => {
                // WebSocket protocol ping/pong nativo
                tracing::trace!("WebSocket ping nativo recebido");
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    // Limpa todas as inscrições quando o cliente fecha a conexão
    for (_, handle) in active_subscriptions {
        handle.abort();
    }
    send_task.abort();
    tracing::debug!("Conexão WebSocket finalizada para usuário {:?}", user_id);
}
