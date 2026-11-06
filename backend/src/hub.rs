use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use shared::ws::WsServerMessage;

const CHANNEL_CAPACITY: usize = 128;

#[derive(Clone, Default)]
pub struct BroadcastHub {
    channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsServerMessage>>>>,
}

impl BroadcastHub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Obtém ou inicializa o canal broadcast para uma lista específica
    pub async fn subscribe(&self, list_id: Uuid) -> broadcast::Receiver<WsServerMessage> {
        let mut channels = self.channels.write().await;
        let sender = channels
            .entry(list_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0);

        sender.subscribe()
    }

    /// Emite um evento em tempo real para todos os clientes conectados à lista
    pub async fn send(&self, list_id: Uuid, message: WsServerMessage) {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&list_id) {
            // Se nenhum cliente estiver conectado no momento, ignoramos o resultado
            let _ = sender.send(message);
        }
    }
}
