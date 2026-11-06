use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::categories::CategoryId;

/// Papel do usuário dentro de uma lista compartilhada
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Member,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Owner => "owner",
            MemberRole::Member => "member",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "owner" => MemberRole::Owner,
            _ => MemberRole::Member,
        }
    }
}

/// Unidades de medida suportadas para os itens da lista
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuantityUnit {
    Un,
    Kg,
    G,
    L,
    Ml,
    Pct,
    Cx,
    Dz,
}

impl QuantityUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuantityUnit::Un => "un",
            QuantityUnit::Kg => "kg",
            QuantityUnit::G => "g",
            QuantityUnit::L => "l",
            QuantityUnit::Ml => "ml",
            QuantityUnit::Pct => "pct",
            QuantityUnit::Cx => "cx",
            QuantityUnit::Dz => "dz",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            QuantityUnit::Un => "unidade(s)",
            QuantityUnit::Kg => "kg",
            QuantityUnit::G => "g",
            QuantityUnit::L => "litro(s)",
            QuantityUnit::Ml => "ml",
            QuantityUnit::Pct => "pacote(s)",
            QuantityUnit::Cx => "caixa(s)",
            QuantityUnit::Dz => "dúzia(s)",
        }
    }

    pub fn from_unit_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "kg" | "quilo" | "quilos" => QuantityUnit::Kg,
            "g" | "grama" | "gramas" => QuantityUnit::G,
            "l" | "litro" | "litros" => QuantityUnit::L,
            "ml" => QuantityUnit::Ml,
            "pct" | "pacote" | "pacotes" => QuantityUnit::Pct,
            "cx" | "caixa" | "caixas" => QuantityUnit::Cx,
            "dz" | "duzia" | "dúzia" | "duzias" => QuantityUnit::Dz,
            _ => QuantityUnit::Un,
        }
    }
}

impl Default for QuantityUnit {
    fn default() -> Self {
        QuantityUnit::Un
    }
}

/// Perfil de usuário
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lista de compras compartilhada
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub invite_code: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Membro vinculado a uma lista
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListMember {
    pub id: Uuid,
    pub list_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

/// Item da lista de compras
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub list_id: Uuid,
    pub name: String,
    pub category_id: CategoryId,
    pub quantity: f64,
    pub unit: QuantityUnit,
    pub estimated_price: Option<f64>,
    pub actual_price: Option<f64>,
    pub purchased: bool,
    pub purchased_at: Option<DateTime<Utc>>,
    pub purchased_by_user_id: Option<Uuid>,
    pub purchased_by_user_name: Option<String>,
    pub added_by_user_id: Uuid,
    pub added_by_user_name: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Histórico acumulado de produtos comprados
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemHistory {
    pub id: Uuid,
    pub list_id: Uuid,
    pub item_name_normalized: String,
    pub category_id: CategoryId,
    pub last_price: Option<f64>,
    pub last_unit: QuantityUnit,
    pub purchase_count: i32,
    pub last_purchased_at: DateTime<Utc>,
}
