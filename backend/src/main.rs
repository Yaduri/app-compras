pub mod auth;
pub mod config;
pub mod error;
pub mod hub;
pub mod memory_db;
pub mod rate_limit;
pub mod routes;
pub mod state;
pub mod validation;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
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

    let config = match Config::from_env() {
        Ok(cfg) => Arc::new(cfg),
        Err(err) => {
            tracing::error!("Configuração inválida: {}", err);
            std::process::exit(1);
        }
    };

    info!(
        "Ambiente: {} | demo: {}",
        if config.is_production { "produção" } else { "desenvolvimento" },
        if config.allow_demo { "ativo" } else { "desativado" }
    );

    let pool = match &config.database_url {
        Some(url) => match connect_database(url).await {
            Ok(pool) => {
                info!("Conexão com PostgreSQL estabelecida com sucesso!");
                Some(pool)
            }
            // Em produção, subir sem banco faria o app perder dados em silêncio
            Err(err) if config.is_production => {
                tracing::error!("Não foi possível conectar ao banco de dados: {}", err);
                std::process::exit(1);
            }
            Err(err) => {
                tracing::warn!(
                    "Não foi possível conectar ao banco ({}). Modo DESENVOLVIMENTO com dados em memória.",
                    err
                );
                None
            }
        },
        None => {
            tracing::warn!(
                "DATABASE_URL não definida. Modo DESENVOLVIMENTO com dados em memória (perdidos ao reiniciar)."
            );
            None
        }
    };

    if let (Some(pool), true) = (&pool, config.run_migrations) {
        if let Err(err) = run_migrations(pool, &config.migrations_dir).await {
            tracing::error!("Falha ao aplicar migrações: {}", err);
            std::process::exit(1);
        }
    }

    let hub = Arc::new(BroadcastHub::new());
    let state = AppState::new(config.clone(), pool, hub);

    let app = routes::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(err) => {
            tracing::error!("Falha ao abrir a porta {}: {}", config.port, err);
            std::process::exit(1);
        }
    };

    info!("Servidor pronto e escutando em http://{}", addr);
    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        tracing::error!("Servidor encerrado com erro: {}", err);
    }
}

async fn connect_database(url: &str) -> Result<PgPool, sqlx::Error> {
    // Desativa statement caching para compatibilidade com PgBouncer / Supabase Transaction Pooler (porta 6543)
    let opts = PgConnectOptions::from_str(url)?.statement_cache_capacity(0);
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_with(opts)
        .await
}

async fn run_migrations(pool: &PgPool, dir: &str) -> Result<(), String> {
    let path = std::path::PathBuf::from(dir);
    let migrator = sqlx::migrate::Migrator::new(path)
        .await
        .map_err(|e| e.to_string())?;
    migrator.run(pool).await.map_err(|e| e.to_string())?;
    info!("Migrações aplicadas a partir de '{}'", dir);
    Ok(())
}

/// Encerra de forma limpa em Ctrl+C ou SIGTERM (enviado pelo Railway em novos deploys)
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Sinal de encerramento recebido, finalizando...");
}
