use reqwest::Method;
use serde::de::DeserializeOwned;
use shared::categories::CategoryInfo;
use shared::dto::{
    ApiErrorResponse, AuthResponse, CreateItemRequest, CreateListRequest, FrequentItemResponse,
    JoinListRequest, ListDetailResponse, ListSummaryResponse, LoginRequest, RegisterRequest,
    ToggleItemPurchasedRequest, UpdateItemRequest, UpdateListRequest, UserProfile,
};
use shared::models::{Item, List};
use uuid::Uuid;

/// URL padrão da API (pode ser trocada com API_BASE_URL na compilação ou execução)
const DEFAULT_API_BASE_URL: &str = "https://app-compras-production.up.railway.app";
/// URL do projeto Supabase usado no login com Google (pode ser trocada com SUPABASE_URL na compilação)
const DEFAULT_SUPABASE_URL: &str = "https://xfzcsdtsjundbhhxqwjm.supabase.co";

pub fn default_base_url() -> String {
    std::env::var("API_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| option_env!("API_BASE_URL").map(|s| s.to_string()))
        .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

pub fn supabase_url() -> String {
    option_env!("SUPABASE_URL")
        .unwrap_or(DEFAULT_SUPABASE_URL)
        .trim_end_matches('/')
        .to_string()
}

/// Codifica um valor para uso seguro em query string
pub fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Erro de uma chamada à API, com o status HTTP quando houver resposta do servidor
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: Option<u16>,
    pub message: String,
}

impl ApiError {
    fn network(err: reqwest::Error) -> Self {
        let message = if err.is_timeout() {
            "O servidor demorou para responder. Tente novamente.".to_string()
        } else {
            "Sem conexão com o servidor. Verifique sua internet e tente novamente.".to_string()
        };
        Self { status: None, message }
    }

    /// Token ausente, inválido ou expirado: o app deve voltar para a tela de login
    pub fn is_unauthorized(&self) -> bool {
        self.status == Some(401)
    }

    /// Sem acesso à lista ou lista inexistente
    pub fn is_forbidden_or_missing(&self) -> bool {
        matches!(self.status, Some(403) | Some(404))
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Cliente HTTP único: reaproveita conexões TLS e aplica timeout
#[cfg(not(target_arch = "wasm32"))]
fn http() -> reqwest::Client {
    use std::sync::OnceLock;
    use std::time::Duration;
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(20))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default()
        })
        .clone()
}

#[cfg(target_arch = "wasm32")]
fn http() -> reqwest::Client {
    reqwest::Client::new()
}

#[derive(Clone, Debug)]
pub struct ApiClient {
    pub base_url: String,
    pub token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: Option<String>, token: Option<String>) -> Self {
        Self {
            base_url: base_url
                .map(|u| u.trim_end_matches('/').to_string())
                .unwrap_or_else(default_base_url),
            token,
        }
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let builder = http().request(method, format!("{}{}", self.base_url, path));
        match &self.token {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    async fn error_from(res: reqwest::Response) -> ApiError {
        let status = res.status();
        let message = res
            .json::<ApiErrorResponse>()
            .await
            .map(|b| b.message)
            .unwrap_or_else(|_| match status.as_u16() {
                401 => "Sua sessão expirou. Entre novamente.".to_string(),
                403 => "Você não tem permissão para isso.".to_string(),
                404 => "Não encontrado.".to_string(),
                429 => "Muitas tentativas. Aguarde um pouco.".to_string(),
                s if s >= 500 => "O servidor está com problemas. Tente novamente em instantes.".to_string(),
                s => format!("Erro inesperado (HTTP {}).", s),
            });
        ApiError { status: Some(status.as_u16()), message }
    }

    async fn send_json<T: DeserializeOwned>(builder: reqwest::RequestBuilder) -> Result<T, ApiError> {
        let res = builder.send().await.map_err(ApiError::network)?;
        if res.status().is_success() {
            res.json::<T>().await.map_err(|_| ApiError {
                status: None,
                message: "Resposta inesperada do servidor. Atualize o app.".to_string(),
            })
        } else {
            Err(Self::error_from(res).await)
        }
    }

    async fn send_empty(builder: reqwest::RequestBuilder) -> Result<(), ApiError> {
        let res = builder.send().await.map_err(ApiError::network)?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(Self::error_from(res).await)
        }
    }

    // --- Autenticação ---

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, ApiError> {
        Self::send_json(self.request(Method::POST, "/api/auth/login").json(&req)).await
    }

    pub async fn demo(&self) -> Result<AuthResponse, ApiError> {
        Self::send_json(self.request(Method::POST, "/api/auth/demo")).await
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, ApiError> {
        Self::send_json(self.request(Method::POST, "/api/auth/register").json(&req)).await
    }

    pub async fn google_auth(&self, id_token: String) -> Result<AuthResponse, ApiError> {
        let req = shared::dto::GoogleAuthRequest { id_token };
        Self::send_json(self.request(Method::POST, "/api/auth/google").json(&req)).await
    }

    pub async fn get_me(&self) -> Result<UserProfile, ApiError> {
        Self::send_json(self.request(Method::GET, "/api/auth/me")).await
    }

    // --- Listas e Vínculo ---

    pub async fn list_user_lists(&self) -> Result<Vec<ListSummaryResponse>, ApiError> {
        Self::send_json(self.request(Method::GET, "/api/lists")).await
    }

    pub async fn create_list(&self, req: CreateListRequest) -> Result<List, ApiError> {
        Self::send_json(self.request(Method::POST, "/api/lists").json(&req)).await
    }

    pub async fn join_list(&self, req: JoinListRequest) -> Result<ListSummaryResponse, ApiError> {
        Self::send_json(self.request(Method::POST, "/api/lists/join").json(&req)).await
    }

    pub async fn get_list_detail(&self, list_id: Uuid) -> Result<ListDetailResponse, ApiError> {
        Self::send_json(self.request(Method::GET, &format!("/api/lists/{}", list_id))).await
    }

    pub async fn update_list(&self, list_id: Uuid, req: UpdateListRequest) -> Result<List, ApiError> {
        Self::send_json(self.request(Method::PUT, &format!("/api/lists/{}", list_id)).json(&req)).await
    }

    pub async fn delete_list(&self, list_id: Uuid) -> Result<(), ApiError> {
        Self::send_empty(self.request(Method::DELETE, &format!("/api/lists/{}", list_id))).await
    }

    pub async fn get_suggestions(&self, list_id: Uuid) -> Result<Vec<FrequentItemResponse>, ApiError> {
        Self::send_json(self.request(Method::GET, &format!("/api/lists/{}/suggestions", list_id))).await
    }

    // --- Itens da Lista ---

    pub async fn add_item(&self, list_id: Uuid, req: CreateItemRequest) -> Result<Item, ApiError> {
        Self::send_json(self.request(Method::POST, &format!("/api/lists/{}/items", list_id)).json(&req)).await
    }

    pub async fn update_item(
        &self,
        list_id: Uuid,
        item_id: Uuid,
        req: UpdateItemRequest,
    ) -> Result<Item, ApiError> {
        let path = format!("/api/lists/{}/items/{}", list_id, item_id);
        Self::send_json(self.request(Method::PUT, &path).json(&req)).await
    }

    pub async fn toggle_item_purchased(
        &self,
        list_id: Uuid,
        item_id: Uuid,
        req: ToggleItemPurchasedRequest,
    ) -> Result<Item, ApiError> {
        let path = format!("/api/lists/{}/items/{}/toggle", list_id, item_id);
        Self::send_json(self.request(Method::PATCH, &path).json(&req)).await
    }

    pub async fn delete_item(&self, list_id: Uuid, item_id: Uuid) -> Result<(), ApiError> {
        let path = format!("/api/lists/{}/items/{}", list_id, item_id);
        Self::send_empty(self.request(Method::DELETE, &path)).await
    }

    // --- Categorias ---

    pub async fn list_categories(&self) -> Result<Vec<CategoryInfo>, ApiError> {
        Self::send_json(self.request(Method::GET, "/api/categories")).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_urls() {
        assert_eq!(url_encode("https://a.b/c?d=e"), "https%3A%2F%2Fa.b%2Fc%3Fd%3De");
    }
}
