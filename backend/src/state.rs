use std::sync::Arc;
use sqlx::PgPool;

use crate::config::SharedConfig;
use crate::error::AppError;
use crate::hub::BroadcastHub;
use crate::memory_db::InMemoryDb;

#[derive(Clone)]
pub struct AppState {
    pub config: SharedConfig,
    pub pool: Option<PgPool>,
    pub memory: Arc<InMemoryDb>,
    pub hub: Arc<BroadcastHub>,
}

impl AppState {
    pub fn new(config: SharedConfig, pool: Option<PgPool>, hub: Arc<BroadcastHub>) -> Self {
        Self {
            config,
            pool,
            memory: Arc::new(InMemoryDb::new_seeded()),
            hub,
        }
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
