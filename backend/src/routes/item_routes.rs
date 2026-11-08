use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::memory_db::{ItemPatch, NewItem};
use crate::routes::list_routes::{touch_list, verify_membership};
use crate::state::AppState;
use crate::validation;
use shared::categories::CategoryId;
use shared::dto::{CreateItemRequest, ToggleItemPurchasedRequest, UpdateItemRequest};
use shared::models::{Item, QuantityUnit};
use shared::ws::WsServerMessage;

/// Colunas de um item (tabela/CTE com alias `i`), já com os nomes de quem adicionou/comprou.
/// Usado por todas as consultas para que o item devolvido seja sempre completo.
pub const ITEM_COLUMNS: &str = r#"
    i.id, i.list_id, i.name, i.category_id,
    i.quantity::float8 AS quantity, i.unit,
    i.estimated_price::float8 AS estimated_price,
    i.actual_price::float8 AS actual_price,
    i.purchased, i.purchased_at, i.purchased_by_user_id,
    up.name AS purchased_by_user_name,
    i.added_by_user_id,
    ua.name AS added_by_user_name,
    i.notes, i.created_at, i.updated_at
"#;

pub const ITEM_JOINS: &str = r#"
    LEFT JOIN users ua ON i.added_by_user_id = ua.id
    LEFT JOIN users up ON i.purchased_by_user_id = up.id
"#;

pub fn item_from_row(row: &PgRow) -> Result<Item, AppError> {
    let category_id: String = row.try_get("category_id")?;
    let unit: String = row.try_get("unit")?;
    Ok(Item {
        id: row.try_get("id")?,
        list_id: row.try_get("list_id")?,
        name: row.try_get("name")?,
        category_id: CategoryId::from_id_str(&category_id),
        quantity: row.try_get("quantity")?,
        unit: QuantityUnit::from_unit_str(&unit),
        estimated_price: row.try_get("estimated_price")?,
        actual_price: row.try_get("actual_price")?,
        purchased: row.try_get("purchased")?,
        purchased_at: row.try_get("purchased_at")?,
        purchased_by_user_id: row.try_get("purchased_by_user_id")?,
        purchased_by_user_name: row.try_get("purchased_by_user_name")?,
        added_by_user_id: row.try_get("added_by_user_id")?,
        added_by_user_name: row.try_get("added_by_user_name")?,
        notes: row.try_get("notes")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn validate_new_item(payload: CreateItemRequest) -> Result<NewItem, AppError> {
    let name = validation::required_text(&payload.name, "O nome do item", validation::MAX_ITEM_NAME)?;
    let category_id = payload
        .category_id
        .unwrap_or_else(|| CategoryId::detect_from_name(&name));
    Ok(NewItem {
        category_id,
        quantity: validation::quantity(payload.quantity.unwrap_or(1.0))?,
        unit: payload.unit.unwrap_or_default(),
        estimated_price: validation::price(payload.estimated_price)?,
        notes: validation::optional_text(payload.notes.as_deref(), "As observações", validation::MAX_NOTES)?,
        name,
    })
}

fn validate_patch(payload: UpdateItemRequest) -> Result<ItemPatch, AppError> {
    let name = match payload.name.as_deref() {
        Some(n) => Some(validation::required_text(n, "O nome do item", validation::MAX_ITEM_NAME)?),
        None => None,
    };
    let quantity = match payload.quantity {
        Some(q) => Some(validation::quantity(q)?),
        None => None,
    };
    let estimated_price = if payload.clear_estimated_price {
        Some(None)
    } else {
        validation::price(payload.estimated_price)?.map(Some)
    };
    let actual_price = if payload.clear_actual_price {
        Some(None)
    } else {
        validation::price(payload.actual_price)?.map(Some)
    };
    let notes = match payload.notes.as_deref() {
        // Some("") limpa; Some(texto) define
        Some(n) => Some(validation::optional_text(Some(n), "As observações", validation::MAX_NOTES)?),
        None => None,
    };
    Ok(ItemPatch {
        name,
        category_id: payload.category_id,
        quantity,
        unit: payload.unit,
        estimated_price,
        actual_price,
        notes,
    })
}

pub async fn add_item(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
    Json(payload): Json<CreateItemRequest>,
) -> Result<Json<Item>, AppError> {
    let new_item = validate_new_item(payload)?;

    let item = if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        let sql = format!(
            r#"
            WITH i AS (
                INSERT INTO items (
                    list_id, name, category_id, quantity, unit, estimated_price, added_by_user_id, notes
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
            )
            SELECT {ITEM_COLUMNS} FROM i {ITEM_JOINS}
            "#
        );

        let row = sqlx::query(&sql)
            .bind(list_id)
            .bind(&new_item.name)
            .bind(new_item.category_id.as_str())
            .bind(new_item.quantity)
            .bind(new_item.unit.as_str())
            .bind(new_item.estimated_price)
            .bind(user.id)
            .bind(&new_item.notes)
            .fetch_one(pool)
            .await?;

        touch_list(pool, list_id).await;
        item_from_row(&row)?
    } else {
        state
            .memory
            .create_item(list_id, user.id, Some(user.name.clone()), new_item)
            .await?
    };

    state
        .hub
        .send(
            list_id,
            WsServerMessage::ItemAdded {
                list_id,
                item: item.clone(),
                actor_id: user.id,
                actor_name: user.name,
            },
        )
        .await;

    Ok(Json(item))
}

pub async fn update_item(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((list_id, item_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateItemRequest>,
) -> Result<Json<Item>, AppError> {
    let patch = validate_patch(payload)?;

    let item = if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        let clear_estimated = matches!(patch.estimated_price, Some(None));
        let clear_actual = matches!(patch.actual_price, Some(None));
        let estimated = patch.estimated_price.flatten();
        let actual = patch.actual_price.flatten();
        // NULL = manter; '' = limpar; texto = definir
        let notes: Option<String> = patch.notes.map(|n| n.unwrap_or_default());

        let sql = format!(
            r#"
            WITH i AS (
                UPDATE items
                SET
                    name = COALESCE($1, name),
                    category_id = COALESCE($2, category_id),
                    quantity = COALESCE($3, quantity),
                    unit = COALESCE($4, unit),
                    estimated_price = CASE WHEN $5 THEN NULL ELSE COALESCE($6, estimated_price) END,
                    actual_price = CASE WHEN $7 THEN NULL ELSE COALESCE($8, actual_price) END,
                    notes = CASE WHEN $9::text IS NULL THEN notes ELSE NULLIF($9::text, '') END,
                    updated_at = NOW()
                WHERE id = $10 AND list_id = $11
                RETURNING *
            )
            SELECT {ITEM_COLUMNS} FROM i {ITEM_JOINS}
            "#
        );

        let row = sqlx::query(&sql)
            .bind(&patch.name)
            .bind(patch.category_id.map(|c| c.as_str()))
            .bind(patch.quantity)
            .bind(patch.unit.map(|u| u.as_str()))
            .bind(clear_estimated)
            .bind(estimated)
            .bind(clear_actual)
            .bind(actual)
            .bind(notes)
            .bind(item_id)
            .bind(list_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Item não encontrado".to_string()))?;

        touch_list(pool, list_id).await;
        item_from_row(&row)?
    } else {
        state.memory.update_item(list_id, item_id, user.id, patch).await?
    };

    state
        .hub
        .send(
            list_id,
            WsServerMessage::ItemUpdated {
                list_id,
                item: item.clone(),
                actor_id: user.id,
                actor_name: user.name,
            },
        )
        .await;

    Ok(Json(item))
}

pub async fn toggle_item_purchased(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((list_id, item_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ToggleItemPurchasedRequest>,
) -> Result<Json<Item>, AppError> {
    let actual_price = validation::price(payload.actual_price)?;

    let item = if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        let sql = format!(
            r#"
            WITH i AS (
                UPDATE items
                SET
                    purchased = $1,
                    purchased_at = CASE WHEN $1 THEN NOW() ELSE NULL END,
                    purchased_by_user_id = CASE WHEN $1 THEN $2 ELSE NULL END,
                    actual_price = CASE WHEN $1 THEN COALESCE($3, actual_price, estimated_price) ELSE actual_price END,
                    updated_at = NOW()
                WHERE id = $4 AND list_id = $5
                RETURNING *
            )
            SELECT {ITEM_COLUMNS} FROM i {ITEM_JOINS}
            "#
        );

        let row = sqlx::query(&sql)
            .bind(payload.purchased)
            .bind(user.id)
            .bind(actual_price)
            .bind(item_id)
            .bind(list_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Item não encontrado".to_string()))?;

        touch_list(pool, list_id).await;
        item_from_row(&row)?
    } else {
        state
            .memory
            .toggle_item(
                list_id,
                item_id,
                user.id,
                Some(user.name.clone()),
                payload.purchased,
                actual_price,
            )
            .await?
    };

    state
        .hub
        .send(
            list_id,
            WsServerMessage::ItemPurchasedToggled {
                list_id,
                item_id,
                purchased: item.purchased,
                actual_price: item.actual_price,
                actor_id: user.id,
                actor_name: user.name,
            },
        )
        .await;

    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((list_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        let result = sqlx::query("DELETE FROM items WHERE id = $1 AND list_id = $2")
            .bind(item_id)
            .bind(list_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Item não encontrado".to_string()));
        }
        touch_list(pool, list_id).await;
    } else {
        state.memory.delete_item(list_id, item_id, user.id).await?;
    }

    state
        .hub
        .send(
            list_id,
            WsServerMessage::ItemDeleted {
                list_id,
                item_id,
                actor_id: user.id,
                actor_name: user.name,
            },
        )
        .await;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Item removido com sucesso"
    })))
}
