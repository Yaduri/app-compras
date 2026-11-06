use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::hash_password;
use crate::error::AppError;
use shared::categories::CategoryId;
use shared::dto::{
    CreateItemRequest, FrequentItemResponse, ListDetailResponse, ListMemberInfo,
    ListSummaryResponse, ListTotals,
};
use shared::models::{Item, List, ListMember, MemberRole, QuantityUnit, User};

/// Banco de dados em memória para desenvolvimento e modo Demo
pub struct InMemoryDb {
    pub users: Arc<RwLock<HashMap<Uuid, (User, String)>>>, // User e password_hash
    pub lists: Arc<RwLock<HashMap<Uuid, List>>>,
    pub members: Arc<RwLock<Vec<ListMember>>>,
    pub items: Arc<RwLock<HashMap<Uuid, Item>>>,
}

impl InMemoryDb {
    /// Cria uma nova instância populada com dados de teste para a experiência Demo
    pub fn new_seeded() -> Self {
        let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let demo_password_hash = hash_password("demo123456").unwrap_or_default();

        let demo_user = User {
            id: demo_user_id,
            email: "demo@mercado.com".to_string(),
            name: "Usuário Demo".to_string(),
            avatar_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut users = HashMap::new();
        users.insert(demo_user_id, (demo_user, demo_password_hash));

        let demo_list_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let demo_list = List {
            id: demo_list_id,
            name: "Churrasco do Fim de Semana".to_string(),
            description: Some("Lista colaborativa para o churrasco com a família e amigos".to_string()),
            invite_code: "CHURRAS".to_string(),
            created_by: demo_user_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut lists = HashMap::new();
        lists.insert(demo_list_id, demo_list);

        let members = vec![ListMember {
            id: Uuid::new_v4(),
            list_id: demo_list_id,
            user_id: demo_user_id,
            role: MemberRole::Owner,
            joined_at: Utc::now(),
        }];

        // Itens de exemplo
        let sample_items: Vec<(&str, CategoryId, f64, QuantityUnit, Option<f64>, bool, Option<f64>)> = vec![
            (
                "Picanha Bovina 1.5kg",
                CategoryId::Butcher,
                1.5,
                QuantityUnit::Kg,
                Some(115.00),
                false,
                None,
            ),
            (
                "Cerveja Puro Malte 6x",
                CategoryId::Beverages,
                6.0,
                QuantityUnit::Un,
                Some(36.00),
                true,
                Some(34.80),
            ),
            (
                "Pão de Alho Especial 400g",
                CategoryId::Bakery,
                1.0,
                QuantityUnit::Pct,
                Some(16.50),
                false,
                None,
            ),
            (
                "Carvão Vegetal 5kg",
                CategoryId::Cleaning,
                1.0,
                QuantityUnit::Pct,
                Some(24.90),
                false,
                None,
            ),
            (
                "Tomate, Cebola e Pimentão para Vinagrete",
                CategoryId::Produce,
                1.0,
                QuantityUnit::Kg,
                Some(14.00),
                true,
                Some(13.50),
            ),
        ];

        let mut items = HashMap::new();
        for (name, category_id, qty, unit, est_price, purchased, act_price) in sample_items {
            let item_id = Uuid::new_v4();
            let item = Item {
                id: item_id,
                list_id: demo_list_id,
                name: name.to_string(),
                category_id,
                quantity: qty,
                unit,
                estimated_price: est_price,
                actual_price: act_price,
                purchased,
                purchased_at: if purchased { Some(Utc::now()) } else { None },
                purchased_by_user_id: if purchased { Some(demo_user_id) } else { None },
                purchased_by_user_name: if purchased { Some("Usuário Demo".to_string()) } else { None },
                added_by_user_id: demo_user_id,
                added_by_user_name: Some("Usuário Demo".to_string()),
                notes: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            items.insert(item_id, item);
        }

        Self {
            users: Arc::new(RwLock::new(users)),
            lists: Arc::new(RwLock::new(lists)),
            members: Arc::new(RwLock::new(members)),
            items: Arc::new(RwLock::new(items)),
        }
    }

    // --- Autenticação ---

    pub async fn find_user_by_email(&self, email: &str) -> Option<(User, String)> {
        let users = self.users.read().await;
        users
            .values()
            .find(|(u, _)| u.email.eq_ignore_ascii_case(email))
            .cloned()
    }

    pub async fn create_user(&self, name: String, email: String, password_hash: String) -> Result<User, AppError> {
        let mut users = self.users.write().await;
        if users.values().any(|(u, _)| u.email.eq_ignore_ascii_case(&email)) {
            return Err(AppError::Conflict("Já existe uma conta com este e-mail".to_string()));
        }

        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            email,
            name,
            avatar_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        users.insert(user_id, (user.clone(), password_hash));
        Ok(user)
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Option<User> {
        let users = self.users.read().await;
        users.get(&id).map(|(u, _)| u.clone())
    }

    // --- Listas ---

    pub async fn list_user_lists(&self, user_id: Uuid) -> Vec<ListSummaryResponse> {
        let members = self.members.read().await;
        let lists = self.lists.read().await;
        let items = self.items.read().await;

        let user_list_memberships: Vec<_> = members.iter().filter(|m| m.user_id == user_id).collect();
        let mut summaries = Vec::new();

        for m in user_list_memberships {
            if let Some(list) = lists.get(&m.list_id) {
                let member_count = members.iter().filter(|mb| mb.list_id == list.id).count() as i64;
                let list_items: Vec<_> = items.values().filter(|it| it.list_id == list.id).collect();
                let total_items = list_items.len() as i64;
                let purchased_items = list_items.iter().filter(|it| it.purchased).count() as i64;

                let estimated_total: f64 = list_items.iter().filter_map(|it| it.estimated_price).sum();
                let actual_total: f64 = list_items
                    .iter()
                    .filter(|it| it.purchased)
                    .filter_map(|it| it.actual_price.or(it.estimated_price))
                    .sum();

                summaries.push(ListSummaryResponse {
                    id: list.id,
                    name: list.name.clone(),
                    description: list.description.clone(),
                    invite_code: list.invite_code.clone(),
                    role: m.role,
                    member_count: member_count as usize,
                    totals: ListTotals {
                        total_items: total_items as usize,
                        purchased_items: purchased_items as usize,
                        total_planned: estimated_total,
                        total_cart: actual_total,
                    },
                });
            }
        }

        summaries.sort_by(|a, b| b.name.cmp(&a.name));
        summaries
    }

    pub async fn create_list(&self, user_id: Uuid, name: String, description: Option<String>, invite_code: String) -> List {
        let list_id = Uuid::new_v4();
        let now = Utc::now();

        let list = List {
            id: list_id,
            name: name.clone(),
            description: description.clone(),
            invite_code: invite_code.clone(),
            created_by: user_id,
            created_at: now,
            updated_at: now,
        };

        {
            let mut lists = self.lists.write().await;
            lists.insert(list_id, list.clone());
        }

        {
            let mut members = self.members.write().await;
            members.push(ListMember {
                id: Uuid::new_v4(),
                list_id,
                user_id,
                role: MemberRole::Owner,
                joined_at: now,
            });
        }

        list
    }

    pub async fn get_list_detail(&self, list_id: Uuid, user_id: Uuid) -> Result<ListDetailResponse, AppError> {
        let members = self.members.read().await;
        let _my_membership = members
            .iter()
            .find(|m| m.list_id == list_id && m.user_id == user_id)
            .ok_or_else(|| AppError::Forbidden("Você não é membro desta lista".to_string()))?;

        let lists = self.lists.read().await;
        let list = lists
            .get(&list_id)
            .ok_or_else(|| AppError::NotFound("Lista não encontrada".to_string()))?;

        let users = self.users.read().await;
        let list_members_info: Vec<ListMemberInfo> = members
            .iter()
            .filter(|m| m.list_id == list_id)
            .map(|m| {
                let user_name = users.get(&m.user_id).map(|(u, _)| u.name.clone()).unwrap_or_else(|| "Membro".to_string());
                let user_email = users.get(&m.user_id).map(|(u, _)| u.email.clone()).unwrap_or_default();
                ListMemberInfo {
                    user_id: m.user_id,
                    name: user_name,
                    email: user_email,
                    avatar_url: None,
                    role: m.role,
                    joined_at: m.joined_at,
                }
            })
            .collect();

        let items = self.items.read().await;
        let mut list_items: Vec<Item> = items.values().filter(|it| it.list_id == list_id).cloned().collect();
        list_items.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let total_items = list_items.len();
        let purchased_items = list_items.iter().filter(|it| it.purchased).count();
        let estimated_total: f64 = list_items.iter().filter_map(|it| it.estimated_price).sum();
        let actual_total: f64 = list_items
            .iter()
            .filter(|it| it.purchased)
            .filter_map(|it| it.actual_price.or(it.estimated_price))
            .sum();

        Ok(ListDetailResponse {
            list: list.clone(),
            members: list_members_info,
            items: list_items,
            totals: ListTotals {
                total_items,
                purchased_items,
                total_planned: estimated_total,
                total_cart: actual_total,
            },
        })
    }

    pub async fn join_by_code(&self, user_id: Uuid, code: &str) -> Result<ListSummaryResponse, AppError> {
        let code_clean = code.trim().to_uppercase();
        let lists = self.lists.read().await;
        let list = lists
            .values()
            .find(|l| l.invite_code.eq_ignore_ascii_case(&code_clean))
            .cloned()
            .ok_or_else(|| AppError::NotFound("Código de convite inválido ou lista inexistente".to_string()))?;

        drop(lists);

        {
            let mut members = self.members.write().await;
            if !members.iter().any(|m| m.list_id == list.id && m.user_id == user_id) {
                members.push(ListMember {
                    id: Uuid::new_v4(),
                    list_id: list.id,
                    user_id,
                    role: MemberRole::Member,
                    joined_at: Utc::now(),
                });
            }
        }

        let user_lists = self.list_user_lists(user_id).await;
        user_lists
            .into_iter()
            .find(|l| l.id == list.id)
            .ok_or_else(|| AppError::Internal("Erro ao obter lista vinculada".to_string()))
    }

    // --- Itens ---

    pub async fn create_item(&self, list_id: Uuid, user_id: Uuid, user_name: Option<String>, req: CreateItemRequest) -> Result<Item, AppError> {
        let members = self.members.read().await;
        if !members.iter().any(|m| m.list_id == list_id && m.user_id == user_id) {
            return Err(AppError::Forbidden("Você não é membro desta lista".to_string()));
        }
        drop(members);

        let category_id = req.category_id.unwrap_or_else(|| CategoryId::detect_from_name(&req.name));
        let item_id = Uuid::new_v4();
        let now = Utc::now();

        let item = Item {
            id: item_id,
            list_id,
            name: req.name.trim().to_string(),
            category_id,
            quantity: req.quantity.unwrap_or(1.0),
            unit: req.unit.unwrap_or_default(),
            estimated_price: req.estimated_price,
            actual_price: None,
            purchased: false,
            purchased_at: None,
            purchased_by_user_id: None,
            purchased_by_user_name: None,
            added_by_user_id: user_id,
            added_by_user_name: user_name,
            notes: req.notes,
            created_at: now,
            updated_at: now,
        };

        {
            let mut items = self.items.write().await;
            items.insert(item_id, item.clone());
        }

        {
            let mut lists = self.lists.write().await;
            if let Some(list) = lists.get_mut(&list_id) {
                list.updated_at = now;
            }
        }

        Ok(item)
    }

    pub async fn toggle_item(&self, item_id: Uuid, user_id: Uuid, user_name: Option<String>, purchased: bool, actual_price: Option<f64>) -> Result<Item, AppError> {
        let mut items = self.items.write().await;
        let item = items
            .get_mut(&item_id)
            .ok_or_else(|| AppError::NotFound("Item não encontrado".to_string()))?;

        item.purchased = purchased;
        item.updated_at = Utc::now();

        if purchased {
            item.purchased_at = Some(Utc::now());
            item.purchased_by_user_id = Some(user_id);
            item.purchased_by_user_name = user_name;
            if actual_price.is_some() {
                item.actual_price = actual_price;
            } else if item.actual_price.is_none() {
                item.actual_price = item.estimated_price;
            }
        } else {
            item.purchased_at = None;
            item.purchased_by_user_id = None;
            item.purchased_by_user_name = None;
        }

        let updated_item = item.clone();
        let list_id = item.list_id;
        drop(items);

        {
            let mut lists = self.lists.write().await;
            if let Some(list) = lists.get_mut(&list_id) {
                list.updated_at = Utc::now();
            }
        }

        Ok(updated_item)
    }

    pub async fn delete_item(&self, item_id: Uuid, user_id: Uuid) -> Result<Item, AppError> {
        let mut items = self.items.write().await;
        let item = items
            .get(&item_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Item não encontrado".to_string()))?;

        let members = self.members.read().await;
        if !members.iter().any(|m| m.list_id == item.list_id && m.user_id == user_id) {
            return Err(AppError::Forbidden("Você não é membro desta lista".to_string()));
        }
        drop(members);

        items.remove(&item_id);
        Ok(item)
    }

    pub async fn suggested_items(&self, _list_id: Uuid) -> Vec<FrequentItemResponse> {
        vec![
            FrequentItemResponse {
                name: "Café Torrado e Moído 500g".to_string(),
                category_id: CategoryId::Grocery,
                purchase_count: 8,
                last_price: Some(19.90),
                last_unit: QuantityUnit::Pct,
            },
            FrequentItemResponse {
                name: "Leite Integral 1L".to_string(),
                category_id: CategoryId::Dairy,
                purchase_count: 7,
                last_price: Some(5.80),
                last_unit: QuantityUnit::L,
            },
            FrequentItemResponse {
                name: "Arroz Branco Tipo 1 5kg".to_string(),
                category_id: CategoryId::Grocery,
                purchase_count: 5,
                last_price: Some(28.50),
                last_unit: QuantityUnit::Pct,
            },
            FrequentItemResponse {
                name: "Detergente Líquido 500ml".to_string(),
                category_id: CategoryId::Cleaning,
                purchase_count: 5,
                last_price: Some(2.99),
                last_unit: QuantityUnit::Un,
            },
            FrequentItemResponse {
                name: "Banana Prata (kg)".to_string(),
                category_id: CategoryId::Produce,
                purchase_count: 4,
                last_price: Some(7.50),
                last_unit: QuantityUnit::Kg,
            },
        ]
    }
}

