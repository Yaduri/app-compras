use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use shared::categories::CategoryInfo;
use shared::dto::{
    ApiErrorResponse, AuthResponse, CreateItemRequest, CreateListRequest, FrequentItemResponse,
    JoinListRequest, ListDetailResponse, ListSummaryResponse, LoginRequest, RegisterRequest,
    ToggleItemPurchasedRequest, UpdateItemRequest, UpdateListRequest, UserProfile,
};
use shared::models::{Item, List};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ApiClient {
    pub base_url: String,
    pub token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: Option<String>, token: Option<String>) -> Self {
        let url = base_url
            .or_else(|| std::env::var("API_BASE_URL").ok())
            .or_else(|| option_env!("API_BASE_URL").map(|s| s.to_string()))
            .unwrap_or_else(|| "https://app-compras-production.up.railway.app".to_string());
        Self {
            base_url: url,
            token,
        }
    }

    fn client(&self) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_default()
    }

    async fn handle_response<T: for<'de> serde::Deserialize<'de>>(
        res: reqwest::Response,
    ) -> Result<T, String> {
        let status = res.status();
        if status.is_success() {
            res.json::<T>()
                .await
                .map_err(|e| format!("Erro ao decodificar resposta JSON: {}", e))
        } else {
            let err_body = res
                .json::<ApiErrorResponse>()
                .await
                .map(|b| b.message)
                .unwrap_or_else(|_| format!("Erro HTTP {}", status));
            Err(err_body)
        }
    }

    // --- Autenticação ---

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, String> {
        let res = self
            .client()
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn demo(&self) -> Result<AuthResponse, String> {
        let res = self
            .client()
            .post(format!("{}/api/auth/demo", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, String> {
        let res = self
            .client()
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn google_auth(&self, id_token: String) -> Result<AuthResponse, String> {
        let req = shared::dto::GoogleAuthRequest { id_token };
        let res = self
            .client()
            .post(format!("{}/api/auth/google", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn get_me(&self) -> Result<UserProfile, String> {
        let res = self
            .client()
            .get(format!("{}/api/auth/me", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    // --- Listas e Vínculo ---

    pub async fn list_user_lists(&self) -> Result<Vec<ListSummaryResponse>, String> {
        let res = self
            .client()
            .get(format!("{}/api/lists", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn create_list(&self, req: CreateListRequest) -> Result<List, String> {
        let res = self
            .client()
            .post(format!("{}/api/lists", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn join_list(&self, req: JoinListRequest) -> Result<ListSummaryResponse, String> {
        let res = self
            .client()
            .post(format!("{}/api/lists/join", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn get_list_detail(&self, list_id: Uuid) -> Result<ListDetailResponse, String> {
        let res = self
            .client()
            .get(format!("{}/api/lists/{}", self.base_url, list_id))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn update_list(
        &self,
        list_id: Uuid,
        req: UpdateListRequest,
    ) -> Result<List, String> {
        let res = self
            .client()
            .put(format!("{}/api/lists/{}", self.base_url, list_id))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn delete_list(&self, list_id: Uuid) -> Result<(), String> {
        let res = self
            .client()
            .delete(format!("{}/api/lists/{}", self.base_url, list_id))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        if res.status().is_success() {
            Ok(())
        } else {
            let err = res
                .json::<ApiErrorResponse>()
                .await
                .map(|b| b.message)
                .unwrap_or_else(|_| "Erro ao excluir lista".to_string());
            Err(err)
        }
    }

    pub async fn get_suggestions(
        &self,
        list_id: Uuid,
    ) -> Result<Vec<FrequentItemResponse>, String> {
        let res = self
            .client()
            .get(format!("{}/api/lists/{}/suggestions", self.base_url, list_id))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    // --- Itens da Lista ---

    pub async fn add_item(
        &self,
        list_id: Uuid,
        req: CreateItemRequest,
    ) -> Result<Item, String> {
        let res = self
            .client()
            .post(format!("{}/api/lists/{}/items", self.base_url, list_id))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn update_item(
        &self,
        list_id: Uuid,
        item_id: Uuid,
        req: UpdateItemRequest,
    ) -> Result<Item, String> {
        let res = self
            .client()
            .put(format!(
                "{}/api/lists/{}/items/{}",
                self.base_url, list_id, item_id
            ))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn toggle_item_purchased(
        &self,
        list_id: Uuid,
        item_id: Uuid,
        req: ToggleItemPurchasedRequest,
    ) -> Result<Item, String> {
        let res = self
            .client()
            .patch(format!(
                "{}/api/lists/{}/items/{}/toggle",
                self.base_url, list_id, item_id
            ))
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }

    pub async fn delete_item(&self, list_id: Uuid, item_id: Uuid) -> Result<(), String> {
        let res = self
            .client()
            .delete(format!(
                "{}/api/lists/{}/items/{}",
                self.base_url, list_id, item_id
            ))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        if res.status().is_success() {
            Ok(())
        } else {
            let err = res
                .json::<ApiErrorResponse>()
                .await
                .map(|b| b.message)
                .unwrap_or_else(|_| "Erro ao remover item".to_string());
            Err(err)
        }
    }

    // --- Categorias ---

    pub async fn list_categories(&self) -> Result<Vec<CategoryInfo>, String> {
        let res = self
            .client()
            .get(format!("{}/api/categories", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com o servidor: {}", e))?;

        Self::handle_response(res).await
    }
}
