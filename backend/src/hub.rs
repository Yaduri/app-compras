use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use shared::ws::WsServerMessage;

const CHANNEL_CAPACITY: usize = 256;

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

        // Remove canais sem ninguém escutando (evita crescimento indefinido de memória)
        channels.retain(|id, sender| *id == list_id || sender.receiver_count() > 0);

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

    /// Libera o canal de uma lista quando não há mais ninguém inscrito
    pub async fn release_if_unused(&self, list_id: Uuid) {
        let mut channels = self.channels.write().await;
        if channels
            .get(&list_id)
            .map(|sender| sender.receiver_count() == 0)
            .unwrap_or(false)
        {
            channels.remove(&list_id);
        }
    }
}
