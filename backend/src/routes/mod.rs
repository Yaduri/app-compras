pub mod auth_routes;
pub mod category_routes;
pub mod item_routes;
pub mod list_routes;
pub mod ws_routes;

use axum::{
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "app": "App Compras API",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub fn create_router(state: AppState) -> Router {
    // Configuração de CORS permissiva para desenvolvimento e mobile
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check e rota raiz
        .route("/", get(health_check))
        .route("/health", get(health_check))
        // Categorias de supermercado
        .route("/api/categories", get(category_routes::list_categories))
        // Autenticação
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login))
        .route("/api/auth/demo", post(auth_routes::demo))
        .route("/api/auth/google", post(auth_routes::google_auth))
        .route("/api/auth/me", get(auth_routes::get_me))
        // Listas e Vínculo por Código
        .route("/api/lists", get(list_routes::list_user_lists))
        .route("/api/lists", post(list_routes::create_list))
        .route("/api/lists/join", post(list_routes::join_list))
        .route("/api/lists/{id}", get(list_routes::get_list_detail))
        .route("/api/lists/{id}", put(list_routes::update_list))
        .route("/api/lists/{id}", delete(list_routes::delete_list))
        .route(
            "/api/lists/{id}/suggestions",
            get(list_routes::get_suggestions),
        )
        // Itens da Lista de Compras
        .route("/api/lists/{id}/items", post(item_routes::add_item))
        .route(
            "/api/lists/{id}/items/{item_id}",
            put(item_routes::update_item),
        )
        .route(
            "/api/lists/{id}/items/{item_id}",
            delete(item_routes::delete_item),
        )
        .route(
            "/api/lists/{id}/items/{item_id}/toggle",
            patch(item_routes::toggle_item_purchased),
        )
        // WebSocket em Tempo Real
        .route("/ws", get(ws_routes::ws_handler))
        .layer(cors)
        .with_state(state)
}
