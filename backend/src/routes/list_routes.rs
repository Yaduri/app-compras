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
use crate::routes::item_routes::{item_from_row, ITEM_COLUMNS, ITEM_JOINS};
use crate::state::AppState;
use crate::validation;
use shared::categories::CategoryId;
use shared::dto::{
    CreateListRequest, FrequentItemResponse, JoinListRequest, ListDetailResponse, ListMemberInfo,
    ListSummaryResponse, ListTotals, UpdateListRequest,
};
use shared::models::{List, MemberRole, QuantityUnit};
use shared::ws::WsServerMessage;

/// Gera um código alfanumérico amigável e legível para convite (ex: 8F2A-K9QX)
pub fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    let mut part = || -> String {
        (0..4)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect()
    };
    let part1 = part();
    let part2 = part();
    format!("{}-{}", part1, part2)
}

/// Verifica se o usuário autenticado pertence à lista e devolve o papel dele
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

async fn verify_owner(pool: &sqlx::PgPool, list_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    match verify_membership(pool, list_id, user_id).await? {
        MemberRole::Owner => Ok(()),
        MemberRole::Member => Err(AppError::Forbidden(
            "Apenas o dono da lista pode fazer isso".to_string(),
        )),
    }
}

/// Atualiza `lists.updated_at` (ordenação do painel). Falha aqui não deve quebrar a operação.
pub async fn touch_list(pool: &sqlx::PgPool, list_id: Uuid) {
    if let Err(err) = sqlx::query("UPDATE lists SET updated_at = NOW() WHERE id = $1")
        .bind(list_id)
        .execute(pool)
        .await
    {
        tracing::warn!("Falha ao atualizar updated_at da lista {}: {}", list_id, err);
    }
}

fn list_from_row(row: &sqlx::postgres::PgRow) -> Result<List, AppError> {
    Ok(List {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        invite_code: row.try_get("invite_code")?,
        created_by: row.try_get("created_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

/// Totais calculados no SQL. Regra: soma o preço pago ou, na falta, o estimado de cada item.
const LIST_TOTALS_SQL: &str = r#"
    (SELECT COUNT(*) FROM list_members WHERE list_id = l.id) AS member_count,
    (SELECT COUNT(*) FROM items WHERE list_id = l.id) AS item_count,
    (SELECT COUNT(*) FROM items WHERE list_id = l.id AND purchased = true) AS purchased_count,
    COALESCE((SELECT SUM(COALESCE(actual_price, estimated_price, 0)) FROM items WHERE list_id = l.id), 0)::float8 AS total_planned,
    COALESCE((SELECT SUM(COALESCE(actual_price, estimated_price, 0)) FROM items WHERE list_id = l.id AND purchased = true), 0)::float8 AS total_cart
"#;

fn summary_from_row(row: &sqlx::postgres::PgRow) -> Result<ListSummaryResponse, AppError> {
    let role_str: String = row.try_get("role")?;
    let member_count: i64 = row.try_get("member_count")?;
    let item_count: i64 = row.try_get("item_count")?;
    let purchased_count: i64 = row.try_get("purchased_count")?;

    Ok(ListSummaryResponse {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        invite_code: row.try_get("invite_code")?,
        role: MemberRole::from_str(&role_str),
        member_count: member_count.max(0) as usize,
        totals: ListTotals {
            total_items: item_count.max(0) as usize,
            purchased_items: purchased_count.max(0) as usize,
            total_planned: row.try_get("total_planned")?,
            total_cart: row.try_get("total_cart")?,
        },
    })
}

pub async fn list_user_lists(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ListSummaryResponse>>, AppError> {
    if let Some(pool) = &state.pool {
        let sql = format!(
            r#"
            SELECT l.id, l.name, l.description, l.invite_code, lm.role, {LIST_TOTALS_SQL}
            FROM lists l
            INNER JOIN list_members lm ON l.id = lm.list_id
            WHERE lm.user_id = $1
            ORDER BY l.updated_at DESC
            "#
        );

        let rows = sqlx::query(&sql).bind(user.id).fetch_all(pool).await?;
        let summaries = rows
            .iter()
            .map(summary_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Json(summaries))
    } else {
        Ok(Json(state.memory.list_user_lists(user.id).await))
    }
}

pub async fn create_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateListRequest>,
) -> Result<Json<List>, AppError> {
    let name = validation::required_text(&payload.name, "O nome da lista", validation::MAX_LIST_NAME)?;
    let description = validation::optional_text(
        payload.description.as_deref(),
        "A descrição",
        validation::MAX_LIST_DESCRIPTION,
    )?;

    if let Some(pool) = &state.pool {
        // Tenta alguns códigos; a restrição UNIQUE do banco é a garantia final
        for attempt in 0..5 {
            let invite_code = generate_invite_code();
            let mut tx = pool.begin().await?;

            let inserted = sqlx::query(
                r#"
                INSERT INTO lists (name, description, invite_code, created_by)
                VALUES ($1, $2, $3, $4)
                RETURNING id, name, description, invite_code, created_by, created_at, updated_at
                "#,
            )
            .bind(&name)
            .bind(&description)
            .bind(&invite_code)
            .bind(user.id)
            .fetch_one(&mut *tx)
            .await;

            let row = match inserted {
                Ok(row) => row,
                Err(err) => {
                    let duplicate = err
                        .as_database_error()
                        .and_then(|e| e.code())
                        .map(|c| c == "23505")
                        .unwrap_or(false);
                    if duplicate && attempt < 4 {
                        continue; // código repetido: tenta outro (a transação é desfeita ao sair)
                    }
                    return Err(err.into());
                }
            };

            let list = list_from_row(&row)?;

            // O criador entra automaticamente como dono ('owner')
            sqlx::query("INSERT INTO list_members (list_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(list.id)
                .bind(user.id)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            return Ok(Json(list));
        }
        Err(AppError::Internal("Não foi possível gerar um código de convite único".to_string()))
    } else {
        let mut invite_code = generate_invite_code();
        while state.memory.invite_code_exists(&invite_code).await {
            invite_code = generate_invite_code();
        }
        let list = state
            .memory
            .create_list(user.id, name, description, invite_code)
            .await;
        Ok(Json(list))
    }
}

pub async fn get_list_detail(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
) -> Result<Json<ListDetailResponse>, AppError> {
    let Some(pool) = &state.pool else {
        let detail = state.memory.get_list_detail(list_id, user.id).await?;
        return Ok(Json(detail));
    };

    verify_membership(pool, list_id, user.id).await?;

    let list_row = sqlx::query(
        "SELECT id, name, description, invite_code, created_by, created_at, updated_at FROM lists WHERE id = $1",
    )
    .bind(list_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Lista não encontrada".to_string()))?;
    let list = list_from_row(&list_row)?;

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

    let items_sql = format!(
        r#"
        SELECT {ITEM_COLUMNS}
        FROM items i
        {ITEM_JOINS}
        WHERE i.list_id = $1
        ORDER BY i.purchased ASC, i.created_at DESC
        "#
    );
    let item_rows = sqlx::query(&items_sql).bind(list_id).fetch_all(pool).await?;

    let items = item_rows
        .iter()
        .map(item_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    let mut totals = ListTotals {
        total_items: items.len(),
        purchased_items: 0,
        total_planned: 0.0,
        total_cart: 0.0,
    };
    for item in &items {
        let price = item.actual_price.or(item.estimated_price).unwrap_or(0.0);
        totals.total_planned += price;
        if item.purchased {
            totals.purchased_items += 1;
            totals.total_cart += price;
        }
    }

    Ok(Json(ListDetailResponse {
        list,
        members,
        items,
        totals,
    }))
}

pub async fn update_list(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
    Json(payload): Json<UpdateListRequest>,
) -> Result<Json<List>, AppError> {
    let name = match payload.name.as_deref() {
        Some(n) => Some(validation::required_text(n, "O nome da lista", validation::MAX_LIST_NAME)?),
        None => None,
    };
    // None = manter; Some(None) = remover; Some(Some(texto)) = definir
    let description = match payload.description.as_deref() {
        Some(d) => Some(validation::optional_text(
            Some(d),
            "A descrição",
            validation::MAX_LIST_DESCRIPTION,
        )?),
        None => None,
    };

    let list = if let Some(pool) = &state.pool {
        verify_owner(pool, list_id, user.id).await?;

        let description_param: Option<String> = description.clone().map(|d| d.unwrap_or_default());
        let row = sqlx::query(
            r#"
            UPDATE lists
            SET name = COALESCE($1, name),
                description = CASE WHEN $2::text IS NULL THEN description ELSE NULLIF($2::text, '') END,
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, name, description, invite_code, created_by, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(description_param)
        .bind(list_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Lista não encontrada".to_string()))?;

        list_from_row(&row)?
    } else {
        state
            .memory
            .update_list(list_id, user.id, name, description)
            .await?
    };

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
    if let Some(pool) = &state.pool {
        verify_owner(pool, list_id, user.id).await?;

        sqlx::query("DELETE FROM lists WHERE id = $1")
            .bind(list_id)
            .execute(pool)
            .await?;
    } else {
        state.memory.delete_list(list_id, user.id).await?;
    }

    // Avisa quem está com a lista aberta
    state
        .hub
        .send(list_id, WsServerMessage::ListDeleted { list_id })
        .await;

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
    // Limita tentativas por usuário para impedir adivinhação de códigos
    state.limits.join.check(&format!("join:{}", user.id))?;

    let code = payload.invite_code.trim().to_uppercase();
    if code.is_empty() || code.len() > validation::MAX_INVITE_CODE {
        return Err(AppError::BadRequest("Código de convite inválido".to_string()));
    }

    let Some(pool) = &state.pool else {
        let (summary, newly_joined) = state.memory.join_by_code(user.id, &code).await?;
        if newly_joined {
            if let Some(member) = state.memory.member_info(summary.id, user.id).await {
                state
                    .hub
                    .send(summary.id, WsServerMessage::MemberJoined { list_id: summary.id, member })
                    .await;
            }
        }
        return Ok(Json(summary));
    };

    let list_id: Uuid = sqlx::query("SELECT id FROM lists WHERE UPPER(invite_code) = $1")
        .bind(&code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Código de convite inválido ou lista não encontrada".to_string()))?
        .try_get("id")?;

    // Insere o membro (se ainda não for). `xmax = 0` indica que a linha acabou de ser criada.
    let joined_row = sqlx::query(
        r#"
        INSERT INTO list_members (list_id, user_id, role)
        VALUES ($1, $2, 'member')
        ON CONFLICT (list_id, user_id) DO UPDATE SET role = list_members.role
        RETURNING joined_at, role, (xmax = 0) AS inserted
        "#,
    )
    .bind(list_id)
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    let joined_at: DateTime<Utc> = joined_row.try_get("joined_at")?;
    let role = MemberRole::from_str(&joined_row.try_get::<String, _>("role")?);
    let inserted: bool = joined_row.try_get("inserted")?;

    if inserted {
        touch_list(pool, list_id).await;

        let avatar_url: Option<String> = sqlx::query("SELECT avatar_url FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_optional(pool)
            .await?
            .map(|r| r.try_get::<Option<String>, _>("avatar_url"))
            .transpose()?
            .flatten();

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
                        avatar_url,
                        role,
                        joined_at,
                    },
                },
            )
            .await;
    }

    let summary_sql = format!(
        r#"
        SELECT l.id, l.name, l.description, l.invite_code, lm.role, {LIST_TOTALS_SQL}
        FROM lists l
        INNER JOIN list_members lm ON l.id = lm.list_id AND lm.user_id = $2
        WHERE l.id = $1
        "#
    );
    let row = sqlx::query(&summary_sql)
        .bind(list_id)
        .bind(user.id)
        .fetch_one(pool)
        .await?;

    Ok(Json(summary_from_row(&row)?))
}

pub async fn get_suggestions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
) -> Result<Json<Vec<FrequentItemResponse>>, AppError> {
    let Some(pool) = &state.pool else {
        return Ok(Json(state.memory.suggested_items(list_id, user.id).await?));
    };

    verify_membership(pool, list_id, user.id).await?;

    let rows = sqlx::query(
        r#"
        SELECT item_name_normalized, category_id, last_price::float8 AS last_price, last_unit, purchase_count
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_code_generation_format() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 9); // 4 + '-' + 4
        let parts: Vec<&str> = code.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 4);
        assert_eq!(parts[1].len(), 4);
        assert!(code
            .chars()
            .filter(|c| *c != '-')
            .all(|c| c.is_ascii_alphanumeric() && !c.is_ascii_lowercase()));
    }
}
