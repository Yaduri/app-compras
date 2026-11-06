pub mod auth;
pub mod config;
pub mod error;
pub mod hub;
pub mod routes;
pub mod state;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::hub::BroadcastHub;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug".into()),
        )
        .init();

    let config = Arc::new(Config::from_env());

    let pool = if let Some(url) = &config.database_url {
        info!("Iniciando conexão com PostgreSQL / Supabase...");
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(url)
            .await
        {
            Ok(p) => {
                info!("Conexão com PostgreSQL estabelecida com sucesso!");
                Some(p)
            }
            Err(err) => {
                tracing::warn!(
                    "Não foi possível conectar ao banco de dados no momento: {}. Servidor operando em modo de espera.",
                    err
                );
                None
            }
        }
    } else {
        tracing::warn!(
            "Variável DATABASE_URL não encontrada no .env. Configure sua URL do Supabase para persistência."
        );
        None
    };

    let hub = Arc::new(BroadcastHub::new());
    let state = AppState::new(config.clone(), pool, hub);

    let app = routes::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Iniciando API Backend Axum na porta {}", config.port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Falha ao abrir porta do servidor");

    info!("Servidor pronto e escutando em http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
