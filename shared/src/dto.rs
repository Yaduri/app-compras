use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::categories::CategoryId;
use crate::models::{Item, List, MemberRole, QuantityUnit};

// ==============================================================================
// 1. DTOs de Autenticação e Perfil
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthRequest {
    pub id_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserProfile,
}

// ==============================================================================
// 2. DTOs de Listas e Vínculo por Código
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateListRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateListRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinListRequest {
    /// Código alfanumérico curto da lista (ex: CASA-8F2A)
    pub invite_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMemberInfo {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTotals {
    pub total_items: usize,
    pub purchased_items: usize,
    pub total_planned: f64,
    pub total_cart: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSummaryResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub invite_code: String,
    pub role: MemberRole,
    pub member_count: usize,
    pub totals: ListTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDetailResponse {
    pub list: List,
    pub members: Vec<ListMemberInfo>,
    pub items: Vec<Item>,
    pub totals: ListTotals,
}

// ==============================================================================
// 3. DTOs de Itens da Lista de Compras
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    /// Se omitido, a autocategorização inteligente seleciona a categoria
    pub category_id: Option<CategoryId>,
    pub quantity: Option<f64>,
    pub unit: Option<QuantityUnit>,
    pub estimated_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub category_id: Option<CategoryId>,
    pub quantity: Option<f64>,
    pub unit: Option<QuantityUnit>,
    pub estimated_price: Option<f64>,
    pub actual_price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleItemPurchasedRequest {
    pub purchased: bool,
    pub actual_price: Option<f64>,
}

// ==============================================================================
// 4. DTOs de Histórico e Sugestões Inteligentes
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequentItemResponse {
    pub name: String,
    pub category_id: CategoryId,
    pub last_price: Option<f64>,
    pub last_unit: QuantityUnit,
    pub purchase_count: i32,
}

// ==============================================================================
// 5. DTO Padrão de Resposta de Erro
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub message: String,
}
