use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::config::SharedConfig;
use crate::hub::BroadcastHub;
use crate::memory_db::InMemoryDb;
use crate::rate_limit::RateLimiters;

#[derive(Clone)]
pub struct AppState {
    pub config: SharedConfig,
    /// `None` apenas em desenvolvimento (fallback em memória)
    pub pool: Option<PgPool>,
    pub memory: Arc<InMemoryDb>,
    pub hub: Arc<BroadcastHub>,
    pub limits: Arc<RateLimiters>,
    /// Cliente HTTP compartilhado (reaproveita conexões, com timeout)
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(config: SharedConfig, pool: Option<PgPool>, hub: Arc<BroadcastHub>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        // O banco em memória só é populado quando realmente será usado
        let memory = if pool.is_some() {
            InMemoryDb::empty()
        } else {
            InMemoryDb::new_seeded()
        };

        Self {
            config,
            pool,
            memory: Arc::new(memory),
            hub,
            limits: Arc::new(RateLimiters::new()),
            http,
        }
    }
}
