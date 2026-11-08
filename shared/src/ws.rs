use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::ListMemberInfo;
use crate::models::Item;

/// Mensagens enviadas do Cliente para o Servidor via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Inscreve-se nas atualizações em tempo real de uma lista de compras
    Subscribe { list_id: Uuid },

    /// Cancela a inscrição de uma lista
    Unsubscribe { list_id: Uuid },

    /// Heartbeat para manter a conexão WebSocket ativa
    Ping,

    /// Autentica a conexão com o JWT do usuário. Deve ser a primeira mensagem
    /// enviada (evita expor o token na URL, que costuma aparecer em logs).
    Auth { token: String },
}

/// Mensagens enviadas do Servidor para os Clientes conectados via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// Confirmação de inscrição no canal da lista
    Subscribed { list_id: Uuid },

    /// Notificação de novo item adicionado na lista
    ItemAdded {
        list_id: Uuid,
        item: Item,
        actor_id: Uuid,
        actor_name: String,
    },

    /// Notificação de item modificado (ex: quantidade, observações, preço estimado)
    ItemUpdated {
        list_id: Uuid,
        item: Item,
        actor_id: Uuid,
        actor_name: String,
    },

    /// Notificação de item marcado/desmarcado como comprado no carrinho
    ItemPurchasedToggled {
        list_id: Uuid,
        item_id: Uuid,
        purchased: bool,
        actual_price: Option<f64>,
        actor_id: Uuid,
        actor_name: String,
    },

    /// Notificação de item removido da lista
    ItemDeleted {
        list_id: Uuid,
        item_id: Uuid,
        actor_id: Uuid,
        actor_name: String,
    },

    /// Notificação de alteração cadastral da lista (nome, descrição)
    ListUpdated {
        list_id: Uuid,
        name: String,
        description: Option<String>,
    },

    /// Notificação de novo membro que entrou na lista via código de convite
    MemberJoined {
        list_id: Uuid,
        member: ListMemberInfo,
    },

    /// Notificação de que a lista foi excluída pelo dono
    ListDeleted { list_id: Uuid },

    /// O servidor descartou eventos desta conexão (cliente lento); o cliente
    /// deve recarregar a lista completa para não ficar dessincronizado.
    ResyncRequired { list_id: Uuid },

    /// Resposta de heartbeat
    Pong,

    /// Mensagem de erro do servidor
    Error { message: String },
}
