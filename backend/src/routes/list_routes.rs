use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use rand::Rng;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::state::AppState;
use shared::categories::CategoryId;
use shared::dto::{
    CreateListRequest, FrequentItemResponse, JoinListRequest, ListDetailResponse, ListMemberInfo,
    ListSummaryResponse, ListTotals, UpdateListRequest,
};
use shared::models::{Item, List, MemberRole, QuantityUnit};
use shared::ws::WsServerMessage;

/// Gera um código alfanumérico amigável e legível para convite (ex: CASA-8F2A)
pub fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    let part1: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    let part2: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("{}-{}", part1, part2)
}

/// Helper para verificar se o usuário autenticado pertence à lista
pub async fn verify_membership(
    pool: &sqlx::PgPool,
    list_id: Uuid,
    user_id: Uuid,
) -> Result<MemberRole, AppError> {
    let row = sqlx::query("SELECT role FROM list_members WHERE list_id = $1 AND user_id = $2")
        .bind(list_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Forbidden("Você não é membro desta lista de compras".to_string()))?;

    let role_str: String = row.try_get("role")?;
    Ok(MemberRole::from_str(&role_str))
}

pub async fn list_user_lists(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ListSummaryResponse>>, AppError> {
    if let Some(pool) = &state.pool {
        // Consulta todas as listas das quais o usuário participa com contadores e totais
        let query = r#"
            SELECT 
                l.id,
                l.name,
                l.description,
                l.invite_code,
                lm.role,
                (SELECT COUNT(*) FROM list_members WHERE list_id = l.id) as member_count,
                (SELECT COUNT(*) FROM items WHERE list_id = l.id) as item_count,
                (SELECT COUNT(*) FROM items WHERE list_id = l.id AND purchased = true) as purchased_count,
                COALESCE((SELECT SUM(COALESCE(actual_price, estimated_price, 0)) FROM items WHERE list_id = l.id), 0)::float8 as total_planned,
                COALESCE((SELECT SUM(COALESCE(actual_price, estimated_price, 0)) FROM items WHERE list_id = l.id AND purchased = true), 0)::float8 as total_cart
            FROM lists l
            INNER JOIN list_members lm ON l.id = lm.list_id
            WHERE lm.user_id = $1
            ORDER BY l.updated_at DESC
        "#;

        let rows = sqlx::query(query).bind(user.id).fetch_all(pool).await?;

        let mut summaries = Vec::with_capacity(rows.len());
        for row in rows {
            let role_str: String = row.try_get("role")?;
            let member_count: i64 = row.try_get("member_count")?;
            let item_count: i64 = row.try_get("item_count")?;
            let purchased_count: i64 = row.try_get("purchased_count")?;
            let total_planned: f64 = row.try_get("total_planned")?;
            let total_cart: f64 = row.try_get("total_cart")?;

            summaries.push(ListSummaryResponse {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                description: row.try_get("description")?,
                invite_code: row.try_get("invite_code")?,
                role: MemberRole::from_str(&role_str),
                member_count: member_count as usize,
                totals: ListTotals {
                    total_items: item_count as usize,
                    purchased_items: purchased_count as usize,
                    total_planned,
                    total_cart,
                },
            });
        }

        Ok(Json(summaries))
    } else {
        let summaries = state.memory.list_user_lists(user.id).await;
        Ok(Json(summaries))
    }
}

pub async fn create_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateListRequest>,
) -> Result<Json<List>, AppError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "O nome da lista não pode ser vazio".to_string(),
        ));
    }

    if let Some(pool) = &state.pool {
        // Gera código de convite único
        let mut invite_code = generate_invite_code();
        for _ in 0..5 {
            let existing = sqlx::query("SELECT id FROM lists WHERE invite_code = $1")
                .bind(&invite_code)
                .fetch_optional(pool)
                .await?;
            if existing.is_none() {
                break;
            }
            invite_code = generate_invite_code();
        }

        let mut tx = pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO lists (name, description, invite_code, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, invite_code, created_by, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&payload.description)
        .bind(&invite_code)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await?;

        let list_id: Uuid = row.try_get("id")?;

        // Adiciona o criador automaticamente como proprietário ('owner')
        sqlx::query(
            "INSERT INTO list_members (list_id, user_id, role) VALUES ($1, $2, 'owner')",
        )
        .bind(list_id)
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Json(List {
            id: list_id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            invite_code: row.try_get("invite_code")?,
            created_by: row.try_get("created_by")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        }))
    } else {
        let invite_code = generate_invite_code();
        let list = state
            .memory
            .create_list(user.id, name, payload.description, invite_code)
            .await;
        Ok(Json(list))
    }
}

pub async fn get_list_detail(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
) -> Result<Json<ListDetailResponse>, AppError> {
    if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

    // Consulta dados cadastrais da lista
    let list_row = sqlx::query(
        "SELECT id, name, description, invite_code, created_by, created_at, updated_at FROM lists WHERE id = $1",
    )
    .bind(list_id)
    .fetch_one(pool)
    .await?;

    let list = List {
        id: list_row.try_get("id")?,
        name: list_row.try_get("name")?,
        description: list_row.try_get("description")?,
        invite_code: list_row.try_get("invite_code")?,
        created_by: list_row.try_get("created_by")?,
        created_at: list_row.try_get("created_at")?,
        updated_at: list_row.try_get("updated_at")?,
    };

    // Consulta membros da lista
    let member_rows = sqlx::query(
        r#"
        SELECT lm.user_id, lm.role, lm.joined_at, u.name, u.email, u.avatar_url
        FROM list_members lm
        INNER JOIN users u ON lm.user_id = u.id
        WHERE lm.list_id = $1
        ORDER BY lm.joined_at ASC
        "#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;

    let mut members = Vec::with_capacity(member_rows.len());
    for r in member_rows {
        let role_str: String = r.try_get("role")?;
        members.push(ListMemberInfo {
            user_id: r.try_get("user_id")?,
            name: r.try_get("name")?,
            email: r.try_get("email")?,
            avatar_url: r.try_get("avatar_url")?,
            role: MemberRole::from_str(&role_str),
            joined_at: r.try_get("joined_at")?,
        });
    }

    // Consulta itens da lista
    let item_rows = sqlx::query(
        r#"
        SELECT 
            i.id, i.list_id, i.name, i.category_id, 
            i.quantity::float8 as quantity, i.unit,
            i.estimated_price::float8 as estimated_price,
            i.actual_price::float8 as actual_price,
            i.purchased, i.purchased_at,
            i.purchased_by_user_id,
            up.name as purchased_by_user_name,
            i.added_by_user_id,
            ua.name as added_by_user_name,
            i.notes, i.created_at, i.updated_at
        FROM items i
        INNER JOIN users ua ON i.added_by_user_id = ua.id
        LEFT JOIN users up ON i.purchased_by_user_id = up.id
        WHERE i.list_id = $1
        ORDER BY i.purchased ASC, i.created_at DESC
        "#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(item_rows.len());
    let mut total_planned = 0.0;
    let mut total_cart = 0.0;
    let mut purchased_count = 0;

    for r in item_rows {
        let category_id_str: String = r.try_get("category_id")?;
        let unit_str: String = r.try_get("unit")?;
        let purchased: bool = r.try_get("purchased")?;
        let estimated: Option<f64> = r.try_get("estimated_price")?;
        let actual: Option<f64> = r.try_get("actual_price")?;

        let price_value = actual.or(estimated).unwrap_or(0.0);
        total_planned += price_value;
        if purchased {
            purchased_count += 1;
            total_cart += price_value;
        }

        items.push(Item {
            id: r.try_get("id")?,
            list_id: r.try_get("list_id")?,
            name: r.try_get("name")?,
            category_id: CategoryId::from_id_str(&category_id_str),
            quantity: r.try_get("quantity")?,
            unit: QuantityUnit::from_unit_str(&unit_str),
            estimated_price: estimated,
            actual_price: actual,
            purchased,
            purchased_at: r.try_get("purchased_at")?,
            purchased_by_user_id: r.try_get("purchased_by_user_id")?,
            purchased_by_user_name: r.try_get("purchased_by_user_name")?,
            added_by_user_id: r.try_get("added_by_user_id")?,
            added_by_user_name: r.try_get("added_by_user_name")?,
            notes: r.try_get("notes")?,
            created_at: r.try_get("created_at")?,
            updated_at: r.try_get("updated_at")?,
        });
    }

    let totals = ListTotals {
        total_items: items.len(),
        purchased_items: purchased_count,
        total_planned,
        total_cart,
    };

        Ok(Json(ListDetailResponse {
            list,
            members,
            items,
            totals,
        }))
    } else {
        let detail = state.memory.get_list_detail(list_id, user.id).await?;
        Ok(Json(detail))
    }
}

pub async fn update_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
    Json(payload): Json<UpdateListRequest>,
) -> Result<Json<List>, AppError> {
    let pool = state.get_pool()?;
    verify_membership(pool, list_id, user.id).await?;

    let row = sqlx::query(
        r#"
        UPDATE lists
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, name, description, invite_code, created_by, created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(list_id)
    .fetch_one(pool)
    .await?;

    let list = List {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        invite_code: row.try_get("invite_code")?,
        created_by: row.try_get("created_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    };

    // Emite atualização via WebSocket
    state
        .hub
        .send(
            list_id,
            WsServerMessage::ListUpdated {
                list_id,
                name: list.name.clone(),
                description: list.description.clone(),
            },
        )
        .await;

    Ok(Json(list))
}

pub async fn delete_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.get_pool()?;
    let role = verify_membership(pool, list_id, user.id).await?;

    if role != MemberRole::Owner {
        return Err(AppError::Forbidden(
            "Apenas o criador/proprietário pode excluir esta lista".to_string(),
        ));
    }

    sqlx::query("DELETE FROM lists WHERE id = $1")
        .bind(list_id)
        .execute(pool)
        .await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Lista excluída com sucesso"
    })))
}

pub async fn join_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<JoinListRequest>,
) -> Result<Json<ListSummaryResponse>, AppError> {
    if let Some(pool) = &state.pool {
        let code = payload.invite_code.trim().to_uppercase();

        let list_row = sqlx::query(
            "SELECT id, name, description, invite_code FROM lists WHERE UPPER(invite_code) = $1",
        )
        .bind(&code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Código de convite inválido ou lista não encontrada".to_string()))?;

        let list_id: Uuid = list_row.try_get("id")?;

        // Insere o membro na lista (se ainda não for membro)
        let joined_row = sqlx::query(
            r#"
            INSERT INTO list_members (list_id, user_id, role)
            VALUES ($1, $2, 'member')
            ON CONFLICT (list_id, user_id) DO UPDATE SET role = list_members.role
            RETURNING joined_at, role
            "#,
        )
        .bind(list_id)
        .bind(user.id)
        .fetch_one(pool)
        .await?;

        let joined_at: DateTime<Utc> = joined_row.try_get("joined_at")?;
        let role_str: String = joined_row.try_get("role")?;

        // Notifica outros membros via WebSocket
        state
            .hub
            .send(
                list_id,
                WsServerMessage::MemberJoined {
                    list_id,
                    member: ListMemberInfo {
                        user_id: user.id,
                        name: user.name.clone(),
                        email: user.email.clone(),
                        avatar_url: None,
                        role: MemberRole::from_str(&role_str),
                        joined_at,
                    },
                },
            )
            .await;

        // Retorna o sumário da lista vinculada
        let counts = sqlx::query(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM list_members WHERE list_id = $1) as member_count,
                (SELECT COUNT(*) FROM items WHERE list_id = $1) as item_count,
                (SELECT COUNT(*) FROM items WHERE list_id = $1 AND purchased = true) as purchased_count,
                COALESCE((SELECT SUM(COALESCE(actual_price, estimated_price, 0)) FROM items WHERE list_id = $1), 0)::float8 as total_planned,
                COALESCE((SELECT SUM(COALESCE(actual_price, estimated_price, 0)) FROM items WHERE list_id = $1 AND purchased = true), 0)::float8 as total_cart
            "#,
        )
        .bind(list_id)
        .fetch_one(pool)
        .await?;

        let member_count: i64 = counts.try_get("member_count")?;
        let item_count: i64 = counts.try_get("item_count")?;
        let purchased_count: i64 = counts.try_get("purchased_count")?;
        let total_planned: f64 = counts.try_get("total_planned")?;
        let total_cart: f64 = counts.try_get("total_cart")?;

        Ok(Json(ListSummaryResponse {
            id: list_id,
            name: list_row.try_get("name")?,
            description: list_row.try_get("description")?,
            invite_code: list_row.try_get("invite_code")?,
            role: MemberRole::Member,
            member_count: member_count as usize,
            totals: ListTotals {
                total_items: item_count as usize,
                purchased_items: purchased_count as usize,
                total_planned,
                total_cart,
            },
        }))
    } else {
        let summary = state.memory.join_by_code(user.id, &payload.invite_code).await?;
        Ok(Json(summary))
    }
}

pub async fn get_suggestions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
) -> Result<Json<Vec<FrequentItemResponse>>, AppError> {
    if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        let rows = sqlx::query(
            r#"
            SELECT item_name_normalized, category_id, last_price::float8 as last_price, last_unit, purchase_count
            FROM item_history
            WHERE list_id = $1
            ORDER BY purchase_count DESC, last_purchased_at DESC
            LIMIT 20
            "#,
        )
        .bind(list_id)
        .fetch_all(pool)
        .await?;

        let mut suggestions = Vec::with_capacity(rows.len());
        for r in rows {
            let cat_str: String = r.try_get("category_id")?;
            let unit_str: String = r.try_get("last_unit")?;
            suggestions.push(FrequentItemResponse {
                name: r.try_get("item_name_normalized")?,
                category_id: CategoryId::from_id_str(&cat_str),
                last_price: r.try_get("last_price")?,
                last_unit: QuantityUnit::from_unit_str(&unit_str),
                purchase_count: r.try_get("purchase_count")?,
            });
        }

        Ok(Json(suggestions))
    } else {
        let suggestions = state.memory.suggested_items(list_id).await;
        Ok(Json(suggestions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_code_generation_format() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 9); // 4 + '-' + 4
        assert!(code.contains('-'));
        let parts: Vec<&str> = code.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 4);
        assert_eq!(parts[1].len(), 4);
        assert!(parts[0].chars().all(|c| c.is_ascii_alphanumeric() && !c.is_ascii_lowercase()));
        assert!(parts[1].chars().all(|c| c.is_ascii_alphanumeric() && !c.is_ascii_lowercase()));
    }
}
