use std::sync::Arc;
use sqlx::PgPool;

use crate::config::SharedConfig;
use crate::error::AppError;
use crate::hub::BroadcastHub;

#[derive(Clone)]
pub struct AppState {
    pub config: SharedConfig,
    pub pool: Option<PgPool>,
    pub hub: Arc<BroadcastHub>,
}

impl AppState {
    pub fn new(config: SharedConfig, pool: Option<PgPool>, hub: Arc<BroadcastHub>) -> Self {
        Self { config, pool, hub }
    }

    /// Retorna a pool do banco de dados ou erro amigável se DATABASE_URL não estiver configurada
    pub fn get_pool(&self) -> Result<&PgPool, AppError> {
        self.pool.as_ref().ok_or_else(|| {
            AppError::Database(
                "Conexão com PostgreSQL não disponível. Configure a variável DATABASE_URL no arquivo .env".to_string(),
            )
        })
    }
}
