use std::collections::HashMap;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{broadcast::error::RecvError, mpsc};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::auth::authenticate_token;
use crate::routes::list_routes::verify_membership;
use crate::state::AppState;
use shared::ws::{WsClientMessage, WsServerMessage};

/// Sem nenhuma mensagem do cliente por este tempo, a conexão é encerrada
/// (o app envia Ping a cada ~25s)
const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    /// Mantido por compatibilidade com versões antigas do app.
    /// Versões novas enviam o token na mensagem `auth`, fora da URL.
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user_id = match query.token.as_deref() {
        Some(t) => authenticate_token(&state, t).await,
        None => None,
    };

    ws.max_message_size(MAX_MESSAGE_SIZE)
        .on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

async fn is_member(state: &AppState, list_id: Uuid, user_id: Uuid) -> bool {
    match &state.pool {
        Some(pool) => verify_membership(pool, list_id, user_id).await.is_ok(),
        None => state.memory.is_member(list_id, user_id).await,
    }
}

async fn handle_socket(socket: WebSocket, state: AppState, mut user_id: Option<Uuid>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Canal local MPSC que concentra todas as mensagens a serem enviadas ao cliente
    let (client_tx, mut client_rx) = mpsc::channel::<WsServerMessage>(128);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if let Ok(json_text) = serde_json::to_string(&msg) {
                if ws_sender.send(Message::Text(json_text.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Inscrições ativas da conexão: list_id -> tarefa que repassa os eventos
    let mut active_subscriptions: HashMap<Uuid, JoinHandle<()>> = HashMap::new();

    loop {
        let message = match tokio::time::timeout(IDLE_TIMEOUT, ws_receiver.next()).await {
            Ok(Some(Ok(message))) => message,
            // Timeout de inatividade, erro de rede ou fim do stream
            _ => break,
        };

        let text = match message {
            Message::Text(text) => text,
            Message::Close(_) => break,
            // Ping/Pong nativos são respondidos automaticamente pelo axum
            _ => continue,
        };

        let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) else {
            let _ = client_tx
                .send(WsServerMessage::Error { message: "Mensagem inválida".to_string() })
                .await;
            continue;
        };

        match client_msg {
            WsClientMessage::Auth { token } => {
                user_id = authenticate_token(&state, &token).await;
                if user_id.is_none() {
                    let _ = client_tx
                        .send(WsServerMessage::Error { message: "Sessão inválida ou expirada".to_string() })
                        .await;
                }
            }
            WsClientMessage::Subscribe { list_id } => {
                let Some(uid) = user_id else {
                    let _ = client_tx
                        .send(WsServerMessage::Error { message: "Autenticação necessária".to_string() })
                        .await;
                    continue;
                };

                // Só membros da lista recebem os eventos dela
                if !is_member(&state, list_id, uid).await {
                    let _ = client_tx
                        .send(WsServerMessage::Error { message: "Você não é membro desta lista".to_string() })
                        .await;
                    continue;
                }

                if !active_subscriptions.contains_key(&list_id) {
                    let mut broadcast_rx = state.hub.subscribe(list_id).await;
                    let tx_clone = client_tx.clone();

                    let handle = tokio::spawn(async move {
                        loop {
                            match broadcast_rx.recv().await {
                                Ok(msg) => {
                                    if tx_clone.send(msg).await.is_err() {
                                        break;
                                    }
                                }
                                // Cliente lento perdeu eventos: pede para recarregar em vez de encerrar
                                Err(RecvError::Lagged(skipped)) => {
                                    tracing::debug!("Cliente atrasado na lista {} ({} eventos)", list_id, skipped);
                                    if tx_clone.send(WsServerMessage::ResyncRequired { list_id }).await.is_err() {
                                        break;
                                    }
                                }
                                Err(RecvError::Closed) => break,
                            }
                        }
                    });

                    active_subscriptions.insert(list_id, handle);
                    tracing::debug!("Usuário {} inscrito na lista {}", uid, list_id);
                }

                // Confirma sempre (o app usa para saber que está sincronizado)
                let _ = client_tx.send(WsServerMessage::Subscribed { list_id }).await;
            }
            WsClientMessage::Unsubscribe { list_id } => {
                if let Some(handle) = active_subscriptions.remove(&list_id) {
                    handle.abort();
                    let _ = handle.await;
                    state.hub.release_if_unused(list_id).await;
                }
            }
            WsClientMessage::Ping => {
                let _ = client_tx.send(WsServerMessage::Pong).await;
            }
        }
    }

    // Encerra todas as inscrições quando o cliente desconecta
    for (list_id, handle) in active_subscriptions {
        handle.abort();
        let _ = handle.await;
        state.hub.release_if_unused(list_id).await;
    }
    send_task.abort();
    tracing::debug!("Conexão WebSocket finalizada (usuário {:?})", user_id);
}
