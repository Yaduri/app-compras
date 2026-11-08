//! Banco de dados em memória, usado SOMENTE em desenvolvimento local quando não há
//! DATABASE_URL. Em produção o servidor se recusa a iniciar sem o PostgreSQL.
//! As regras de permissão espelham as das rotas com PostgreSQL.

use std::collections::HashMap;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::hash_password_blocking;
use crate::error::AppError;
use shared::categories::CategoryId;
use shared::dto::{FrequentItemResponse, ListDetailResponse, ListMemberInfo, ListSummaryResponse, ListTotals};
use shared::models::{Item, List, ListMember, MemberRole, QuantityUnit, User};

pub const DEMO_EMAIL: &str = "demo@mercado.com";

/// Dados já validados para criar um item
pub struct NewItem {
    pub name: String,
    pub category_id: CategoryId,
    pub quantity: f64,
    pub unit: QuantityUnit,
    pub estimated_price: Option<f64>,
    pub notes: Option<String>,
}

/// Alterações já validadas de um item.
/// `None` = manter; `Some(None)` = limpar; `Some(Some(v))` = definir.
#[derive(Default)]
pub struct ItemPatch {
    pub name: Option<String>,
    pub category_id: Option<CategoryId>,
    pub quantity: Option<f64>,
    pub unit: Option<QuantityUnit>,
    pub estimated_price: Option<Option<f64>>,
    pub actual_price: Option<Option<f64>>,
    pub notes: Option<Option<String>>,
}

#[derive(Default)]
pub struct InMemoryDb {
    users: RwLock<HashMap<Uuid, (User, String)>>, // User e password_hash ("" = conta social)
    lists: RwLock<HashMap<Uuid, List>>,
    members: RwLock<Vec<ListMember>>,
    items: RwLock<HashMap<Uuid, Item>>,
    history: RwLock<HashMap<(Uuid, String), FrequentItemResponse>>,
    /// Equivalente a `users.tokens_valid_after` (segundos Unix)
    tokens_valid_after: RwLock<HashMap<Uuid, i64>>,
}

/// Mesma regra de totais usada nas consultas SQL: preço pago ou, na falta, o estimado
fn compute_totals(items: &[&Item]) -> ListTotals {
    let mut totals = ListTotals {
        total_items: items.len(),
        purchased_items: 0,
        total_planned: 0.0,
        total_cart: 0.0,
    };
    for item in items {
        let price = item.actual_price.or(item.estimated_price).unwrap_or(0.0);
        totals.total_planned += price;
        if item.purchased {
            totals.purchased_items += 1;
            totals.total_cart += price;
        }
    }
    totals
}

impl InMemoryDb {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Instância populada com dados de teste para a experiência Demo local
    pub fn new_seeded() -> Self {
        let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::new_v4());
        let demo_password_hash = hash_password_blocking("demo123456").unwrap_or_default();
        let now = Utc::now();

        let demo_user = User {
            id: demo_user_id,
            email: DEMO_EMAIL.to_string(),
            name: "Usuário Demo".to_string(),
            avatar_url: None,
            created_at: now,
            updated_at: now,
        };

        let mut users = HashMap::new();
        users.insert(demo_user_id, (demo_user, demo_password_hash));

        let demo_list_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap_or_else(|_| Uuid::new_v4());
        let demo_list = List {
            id: demo_list_id,
            name: "Churrasco do Fim de Semana".to_string(),
            description: Some("Lista colaborativa para o churrasco com a família e amigos".to_string()),
            invite_code: "CHUR-RASC".to_string(),
            created_by: demo_user_id,
            created_at: now,
            updated_at: now,
        };

        let mut lists = HashMap::new();
        lists.insert(demo_list_id, demo_list);

        let members = vec![ListMember {
            id: Uuid::new_v4(),
            list_id: demo_list_id,
            user_id: demo_user_id,
            role: MemberRole::Owner,
            joined_at: now,
        }];

        let sample_items: Vec<(&str, CategoryId, f64, QuantityUnit, Option<f64>, bool, Option<f64>)> = vec![
            ("Picanha Bovina", CategoryId::Butcher, 1.5, QuantityUnit::Kg, Some(115.00), false, None),
            ("Cerveja Puro Malte", CategoryId::Beverages, 6.0, QuantityUnit::Un, Some(36.00), true, Some(34.80)),
            ("Pão de Alho Especial 400g", CategoryId::Bakery, 1.0, QuantityUnit::Pct, Some(16.50), false, None),
            ("Carvão Vegetal 5kg", CategoryId::Grocery, 1.0, QuantityUnit::Pct, Some(24.90), false, None),
            ("Tomate, Cebola e Pimentão", CategoryId::Produce, 1.0, QuantityUnit::Kg, Some(14.00), true, Some(13.50)),
        ];

        let mut items = HashMap::new();
        for (name, category_id, qty, unit, est_price, purchased, act_price) in sample_items {
            let item_id = Uuid::new_v4();
            items.insert(
                item_id,
                Item {
                    id: item_id,
                    list_id: demo_list_id,
                    name: name.to_string(),
                    category_id,
                    quantity: qty,
                    unit,
                    estimated_price: est_price,
                    actual_price: act_price,
                    purchased,
                    purchased_at: if purchased { Some(now) } else { None },
                    purchased_by_user_id: if purchased { Some(demo_user_id) } else { None },
                    purchased_by_user_name: if purchased { Some("Usuário Demo".to_string()) } else { None },
                    added_by_user_id: demo_user_id,
                    added_by_user_name: Some("Usuário Demo".to_string()),
                    notes: None,
                    created_at: now,
                    updated_at: now,
                },
            );
        }

        Self {
            users: RwLock::new(users),
            lists: RwLock::new(lists),
            members: RwLock::new(members),
            items: RwLock::new(items),
            history: RwLock::new(HashMap::new()),
            tokens_valid_after: RwLock::new(HashMap::new()),
        }
    }

    // --- Usuários ---

    pub async fn find_user_by_email(&self, email: &str) -> Option<(User, String)> {
        let users = self.users.read().await;
        users
            .values()
            .find(|(u, _)| u.email.eq_ignore_ascii_case(email))
            .cloned()
    }

    /// Remove a senha da conta (passa a ser só social) e, se havia senha, invalida as
    /// sessões abertas com ela. Ver `google_auth`.
    pub async fn clear_password(&self, user_id: Uuid) {
        let had_password = match self.users.write().await.get_mut(&user_id) {
            Some((_, hash)) if !hash.is_empty() => {
                hash.clear();
                true
            }
            _ => false,
        };
        if had_password {
            self.tokens_valid_after
                .write()
                .await
                .insert(user_id, Utc::now().timestamp());
        }
    }

    /// `None` se o usuário não existe
    pub async fn tokens_valid_after(&self, user_id: Uuid) -> Option<i64> {
        if !self.users.read().await.contains_key(&user_id) {
            return None;
        }
        Some(self.tokens_valid_after.read().await.get(&user_id).copied().unwrap_or(0))
    }

    pub async fn create_user(
        &self,
        name: String,
        email: String,
        password_hash: String,
        avatar_url: Option<String>,
    ) -> Result<User, AppError> {
        let mut users = self.users.write().await;
        if users.values().any(|(u, _)| u.email.eq_ignore_ascii_case(&email)) {
            return Err(AppError::Conflict("Já existe uma conta cadastrada com este e-mail".to_string()));
        }

        let now = Utc::now();
        let user = User {
            id: Uuid::new_v4(),
            email,
            name,
            avatar_url,
            created_at: now,
            updated_at: now,
        };

        users.insert(user.id, (user.clone(), password_hash));
        Ok(user)
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Option<User> {
        let users = self.users.read().await;
        users.get(&id).map(|(u, _)| u.clone())
    }

    // --- Permissões ---

    pub async fn role_of(&self, list_id: Uuid, user_id: Uuid) -> Option<MemberRole> {
        let members = self.members.read().await;
        members
            .iter()
            .find(|m| m.list_id == list_id && m.user_id == user_id)
            .map(|m| m.role)
    }

    pub async fn is_member(&self, list_id: Uuid, user_id: Uuid) -> bool {
        self.role_of(list_id, user_id).await.is_some()
    }

    async fn require_member(&self, list_id: Uuid, user_id: Uuid) -> Result<MemberRole, AppError> {
        self.role_of(list_id, user_id)
            .await
            .ok_or_else(|| AppError::Forbidden("Você não é membro desta lista de compras".to_string()))
    }

    async fn require_owner(&self, list_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        match self.require_member(list_id, user_id).await? {
            MemberRole::Owner => Ok(()),
            MemberRole::Member => Err(AppError::Forbidden(
                "Apenas o dono da lista pode fazer isso".to_string(),
            )),
        }
    }

    async fn touch_list(&self, list_id: Uuid) {
        let mut lists = self.lists.write().await;
        if let Some(list) = lists.get_mut(&list_id) {
            list.updated_at = Utc::now();
        }
    }

    // --- Listas ---

    pub async fn list_user_lists(&self, user_id: Uuid) -> Vec<ListSummaryResponse> {
        let members = self.members.read().await;
        let lists = self.lists.read().await;
        let items = self.items.read().await;

        let mut summaries: Vec<(chrono::DateTime<Utc>, ListSummaryResponse)> = members
            .iter()
            .filter(|m| m.user_id == user_id)
            .filter_map(|m| {
                let list = lists.get(&m.list_id)?;
                let list_items: Vec<&Item> = items.values().filter(|it| it.list_id == list.id).collect();
                Some((
                    list.updated_at,
                    ListSummaryResponse {
                        id: list.id,
                        name: list.name.clone(),
                        description: list.description.clone(),
                        invite_code: list.invite_code.clone(),
                        role: m.role,
                        member_count: members.iter().filter(|mb| mb.list_id == list.id).count(),
                        totals: compute_totals(&list_items),
                    },
                ))
            })
            .collect();

        summaries.sort_by(|a, b| b.0.cmp(&a.0));
        summaries.into_iter().map(|(_, s)| s).collect()
    }

    pub async fn invite_code_exists(&self, code: &str) -> bool {
        let lists = self.lists.read().await;
        lists.values().any(|l| l.invite_code.eq_ignore_ascii_case(code))
    }

    pub async fn create_list(&self, user_id: Uuid, name: String, description: Option<String>, invite_code: String) -> List {
        let now = Utc::now();
        let list = List {
            id: Uuid::new_v4(),
            name,
            description,
            invite_code,
            created_by: user_id,
            created_at: now,
            updated_at: now,
        };

        self.lists.write().await.insert(list.id, list.clone());
        self.members.write().await.push(ListMember {
            id: Uuid::new_v4(),
            list_id: list.id,
            user_id,
            role: MemberRole::Owner,
            joined_at: now,
        });

        list
    }

    pub async fn get_list_detail(&self, list_id: Uuid, user_id: Uuid) -> Result<ListDetailResponse, AppError> {
        self.require_member(list_id, user_id).await?;

        let list = self
            .lists
            .read()
            .await
            .get(&list_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Lista não encontrada".to_string()))?;

        let members_info: Vec<ListMemberInfo> = {
            let members = self.members.read().await;
            let users = self.users.read().await;
            let mut info: Vec<ListMemberInfo> = members
                .iter()
                .filter(|m| m.list_id == list_id)
                .map(|m| {
                    let user = users.get(&m.user_id).map(|(u, _)| u);
                    ListMemberInfo {
                        user_id: m.user_id,
                        name: user.map(|u| u.name.clone()).unwrap_or_else(|| "Membro".to_string()),
                        email: user.map(|u| u.email.clone()).unwrap_or_default(),
                        avatar_url: user.and_then(|u| u.avatar_url.clone()),
                        role: m.role,
                        joined_at: m.joined_at,
                    }
                })
                .collect();
            info.sort_by(|a, b| a.joined_at.cmp(&b.joined_at));
            info
        };

        let items = self.items.read().await;
        let mut list_items: Vec<Item> = items.values().filter(|it| it.list_id == list_id).cloned().collect();
        // Mesma ordem do SQL: pendentes primeiro, mais recentes no topo
        list_items.sort_by(|a, b| a.purchased.cmp(&b.purchased).then(b.created_at.cmp(&a.created_at)));
        let totals = compute_totals(&list_items.iter().collect::<Vec<_>>());

        Ok(ListDetailResponse {
            list,
            members: members_info,
            items: list_items,
            totals,
        })
    }

    pub async fn update_list(
        &self,
        list_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> Result<List, AppError> {
        self.require_owner(list_id, user_id).await?;
        let mut lists = self.lists.write().await;
        let list = lists
            .get_mut(&list_id)
            .ok_or_else(|| AppError::NotFound("Lista não encontrada".to_string()))?;
        if let Some(n) = name {
            list.name = n;
        }
        if let Some(d) = description {
            list.description = d;
        }
        list.updated_at = Utc::now();
        Ok(list.clone())
    }

    pub async fn delete_list(&self, list_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.require_owner(list_id, user_id).await?;
        self.lists.write().await.remove(&list_id);
        self.members.write().await.retain(|m| m.list_id != list_id);
        self.items.write().await.retain(|_, item| item.list_id != list_id);
        self.history.write().await.retain(|(lid, _), _| *lid != list_id);
        Ok(())
    }

    /// Retorna o resumo da lista e se o usuário acabou de entrar (true) ou já era membro
    pub async fn join_by_code(&self, user_id: Uuid, code: &str) -> Result<(ListSummaryResponse, bool), AppError> {
        let code_clean = code.trim().to_uppercase();
        let list_id = self
            .lists
            .read()
            .await
            .values()
            .find(|l| l.invite_code.eq_ignore_ascii_case(&code_clean))
            .map(|l| l.id)
            .ok_or_else(|| AppError::NotFound("Código de convite inválido ou lista não encontrada".to_string()))?;

        let newly_joined = {
            let mut members = self.members.write().await;
            if members.iter().any(|m| m.list_id == list_id && m.user_id == user_id) {
                false
            } else {
                members.push(ListMember {
                    id: Uuid::new_v4(),
                    list_id,
                    user_id,
                    role: MemberRole::Member,
                    joined_at: Utc::now(),
                });
                true
            }
        };

        let summary = self
            .list_user_lists(user_id)
            .await
            .into_iter()
            .find(|l| l.id == list_id)
            .ok_or_else(|| AppError::Internal("Erro ao obter lista vinculada".to_string()))?;

        Ok((summary, newly_joined))
    }

    pub async fn member_info(&self, list_id: Uuid, user_id: Uuid) -> Option<ListMemberInfo> {
        let members = self.members.read().await;
        let member = members.iter().find(|m| m.list_id == list_id && m.user_id == user_id)?;
        let users = self.users.read().await;
        let (user, _) = users.get(&user_id)?;
        Some(ListMemberInfo {
            user_id,
            name: user.name.clone(),
            email: user.email.clone(),
            avatar_url: user.avatar_url.clone(),
            role: member.role,
            joined_at: member.joined_at,
        })
    }

    // --- Itens ---

    pub async fn create_item(
        &self,
        list_id: Uuid,
        user_id: Uuid,
        user_name: Option<String>,
        new_item: NewItem,
    ) -> Result<Item, AppError> {
        self.require_member(list_id, user_id).await?;

        let now = Utc::now();
        let item = Item {
            id: Uuid::new_v4(),
            list_id,
            name: new_item.name,
            category_id: new_item.category_id,
            quantity: new_item.quantity,
            unit: new_item.unit,
            estimated_price: new_item.estimated_price,
            actual_price: None,
            purchased: false,
            purchased_at: None,
            purchased_by_user_id: None,
            purchased_by_user_name: None,
            added_by_user_id: user_id,
            added_by_user_name: user_name,
            notes: new_item.notes,
            created_at: now,
            updated_at: now,
        };

        self.items.write().await.insert(item.id, item.clone());
        self.touch_list(list_id).await;
        Ok(item)
    }

    pub async fn update_item(
        &self,
        list_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
        patch: ItemPatch,
    ) -> Result<Item, AppError> {
        self.require_member(list_id, user_id).await?;

        let updated = {
            let mut items = self.items.write().await;
            let item = items
                .get_mut(&item_id)
                .filter(|it| it.list_id == list_id)
                .ok_or_else(|| AppError::NotFound("Item não encontrado".to_string()))?;

            if let Some(v) = patch.name {
                item.name = v;
            }
            if let Some(v) = patch.category_id {
                item.category_id = v;
            }
            if let Some(v) = patch.quantity {
                item.quantity = v;
            }
            if let Some(v) = patch.unit {
                item.unit = v;
            }
            if let Some(v) = patch.estimated_price {
                item.estimated_price = v;
            }
            if let Some(v) = patch.actual_price {
                item.actual_price = v;
            }
            if let Some(v) = patch.notes {
                item.notes = v;
            }
            item.updated_at = Utc::now();
            item.clone()
        };

        self.touch_list(list_id).await;
        Ok(updated)
    }

    pub async fn toggle_item(
        &self,
        list_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
        user_name: Option<String>,
        purchased: bool,
        actual_price: Option<f64>,
    ) -> Result<Item, AppError> {
        self.require_member(list_id, user_id).await?;

        let (updated, became_purchased) = {
            let mut items = self.items.write().await;
            let item = items
                .get_mut(&item_id)
                .filter(|it| it.list_id == list_id)
                .ok_or_else(|| AppError::NotFound("Item não encontrado".to_string()))?;

            let became_purchased = purchased && !item.purchased;
            let now = Utc::now();
            item.purchased = purchased;
            item.updated_at = now;

            if purchased {
                item.purchased_at = Some(now);
                item.purchased_by_user_id = Some(user_id);
                item.purchased_by_user_name = user_name;
                item.actual_price = actual_price.or(item.actual_price).or(item.estimated_price);
            } else {
                item.purchased_at = None;
                item.purchased_by_user_id = None;
                item.purchased_by_user_name = None;
            }
            (item.clone(), became_purchased)
        };

        // Histórico para sugestões (equivalente ao trigger do PostgreSQL)
        if became_purchased {
            let key = (list_id, updated.name.trim().to_lowercase());
            let mut history = self.history.write().await;
            let entry = history.entry(key.clone()).or_insert(FrequentItemResponse {
                name: key.1.clone(),
                category_id: updated.category_id,
                last_price: None,
                last_unit: updated.unit,
                purchase_count: 0,
            });
            entry.purchase_count += 1;
            entry.last_unit = updated.unit;
            entry.last_price = updated.actual_price.or(updated.estimated_price).or(entry.last_price);
        }

        self.touch_list(list_id).await;
        Ok(updated)
    }

    pub async fn delete_item(&self, list_id: Uuid, item_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.require_member(list_id, user_id).await?;
        {
            let mut items = self.items.write().await;
            match items.get(&item_id) {
                Some(item) if item.list_id == list_id => {
                    items.remove(&item_id);
                }
                _ => return Err(AppError::NotFound("Item não encontrado".to_string())),
            }
        }
        self.touch_list(list_id).await;
        Ok(())
    }

    pub async fn suggested_items(&self, list_id: Uuid, user_id: Uuid) -> Result<Vec<FrequentItemResponse>, AppError> {
        self.require_member(list_id, user_id).await?;
        let history = self.history.read().await;
        let mut suggestions: Vec<FrequentItemResponse> = history
            .iter()
            .filter(|((lid, _), _)| *lid == list_id)
            .map(|(_, v)| v.clone())
            .collect();
        suggestions.sort_by(|a, b| b.purchase_count.cmp(&a.purchase_count));
        suggestions.truncate(20);
        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn non_member_cannot_touch_list() {
        let db = InMemoryDb::empty();
        let owner = Uuid::new_v4();
        let stranger = Uuid::new_v4();
        let list = db.create_list(owner, "Mercado".into(), None, "AAAA-BBBB".into()).await;

        assert!(db.get_list_detail(list.id, stranger).await.is_err());
        assert!(db.delete_list(list.id, stranger).await.is_err());

        let item = db
            .create_item(
                list.id,
                owner,
                None,
                NewItem {
                    name: "Leite".into(),
                    category_id: CategoryId::Dairy,
                    quantity: 1.0,
                    unit: QuantityUnit::Un,
                    estimated_price: Some(5.0),
                    notes: None,
                },
            )
            .await
            .unwrap();
        assert!(db.toggle_item(list.id, item.id, stranger, None, true, None).await.is_err());
        assert!(db.delete_item(list.id, item.id, stranger).await.is_err());
    }

    #[tokio::test]
    async fn member_cannot_delete_list() {
        let db = InMemoryDb::empty();
        let owner = Uuid::new_v4();
        let member = Uuid::new_v4();
        let list = db.create_list(owner, "Mercado".into(), None, "CCCC-DDDD".into()).await;
        let (_, joined) = db.join_by_code(member, "cccc-dddd").await.unwrap();
        assert!(joined);
        assert!(db.delete_list(list.id, member).await.is_err());
        assert!(db.delete_list(list.id, owner).await.is_ok());
    }
}
