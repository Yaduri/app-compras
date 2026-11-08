pub mod auth_routes;
pub mod category_routes;
pub mod item_routes;
pub mod list_routes;
pub mod ws_routes;

use axum::{
    response::Html,
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

async fn root_handler() -> Html<&'static str> {
    Html(r##"<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>App Compras</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background-color: #0f172a;
      color: #f8fafc;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
      text-align: center;
      padding: 20px;
      box-sizing: border-box;
    }
    .card {
      background: #1e293b;
      padding: 32px 24px;
      border-radius: 20px;
      max-width: 380px;
      width: 100%;
      border: 1px solid #334155;
      box-shadow: 0 10px 30px rgba(0,0,0,0.5);
    }
    .btn {
      display: inline-block;
      margin-top: 16px;
      background: #ffffff;
      color: #0f172a;
      text-decoration: none;
      font-weight: 700;
      padding: 12px 24px;
      border-radius: 9999px;
      font-size: 0.95rem;
    }
    .spinner {
      width: 36px;
      height: 36px;
      border: 3px solid rgba(255,255,255,0.2);
      border-top-color: #ffffff;
      border-radius: 50%;
      animation: spin 1s infinite linear;
      margin: 0 auto 16px auto;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
  </style>
</head>
<body>
  <div class="card">
    <div id="loading">
      <div class="spinner"></div>
      <h3 style="margin: 0 0 6px 0;">Autenticando...</h3>
      <p style="color: #94a3b8; font-size: 0.88rem; margin: 0;">Retornando para o App Compras...</p>
    </div>
    <div id="action" style="display: none;">
      <h3 style="color: #10b981; margin: 0 0 6px 0;">Login Concluído!</h3>
      <p style="color: #94a3b8; font-size: 0.88rem; margin: 0;">Toque no botão abaixo para abrir o app:</p>
      <a id="deep-link-btn" href="#" class="btn">Abrir no App Compras</a>
    </div>
  </div>
  <script>
    // Repassa ao app apenas um access_token bem formado (sem caracteres que possam ser
    // interpretados como código ou parâmetros extras)
    const raw = (window.location.hash || window.location.search || "").replace(/^[#?]/, "");
    const params = new URLSearchParams(raw);
    const accessToken = params.get("access_token") || "";
    if (/^[A-Za-z0-9._-]{20,8192}$/.test(accessToken)) {
      const targetUri = "appcompras://auth#access_token=" + accessToken;
      document.getElementById("deep-link-btn").href = targetUri;
      document.getElementById("action").style.display = "block";
      document.getElementById("loading").style.display = "none";
      // Remove o token da barra de endereços/histórico do navegador
      history.replaceState(null, "", window.location.pathname);
      setTimeout(function() {
        window.location.href = targetUri;
      }, 300);
    } else {
      document.getElementById("loading").innerHTML = "<h3 style='margin: 0 0 6px 0;'>App Compras API</h3><p style='color: #10b981; margin: 0;'>Servidor online</p>";
    }
  </script>
</body>
</html>"##)
}

pub fn create_router(state: AppState) -> Router {
    // CORS aberto é seguro aqui: a API usa token Bearer no cabeçalho (sem cookies),
    // então outros sites não conseguem agir em nome do usuário.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check e rota raiz
        .route("/", get(root_handler))
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
