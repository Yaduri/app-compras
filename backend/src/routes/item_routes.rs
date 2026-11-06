use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::Row;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::routes::list_routes::verify_membership;
use crate::state::AppState;
use shared::categories::CategoryId;
use shared::dto::{CreateItemRequest, ToggleItemPurchasedRequest, UpdateItemRequest};
use shared::models::{Item, QuantityUnit};
use shared::ws::WsServerMessage;

pub async fn add_item(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(list_id): Path<Uuid>,
    Json(payload): Json<CreateItemRequest>,
) -> Result<Json<Item>, AppError> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("O nome do item é obrigatório".to_string()));
    }

    let item = if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        // Autocategorização inteligente automática se a categoria não foi especificada manualmente
        let category = payload
            .category_id
            .unwrap_or_else(|| CategoryId::detect_from_name(&name));

        let quantity = payload.quantity.unwrap_or(1.0);
        let unit = payload.unit.unwrap_or_default();

        let row = sqlx::query(
            r#"
            INSERT INTO items (
                list_id, name, category_id, quantity, unit, estimated_price, added_by_user_id, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, list_id, name, category_id, quantity::float8 as quantity, unit,
                estimated_price::float8 as estimated_price, actual_price::float8 as actual_price,
                purchased, purchased_at, purchased_by_user_id, added_by_user_id, notes,
                created_at, updated_at
            "#,
        )
        .bind(list_id)
        .bind(&name)
        .bind(category.as_str())
        .bind(quantity)
        .bind(unit.as_str())
        .bind(payload.estimated_price)
        .bind(user.id)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        let cat_id_str: String = row.try_get("category_id")?;
        let unit_str: String = row.try_get("unit")?;

        Item {
            id: row.try_get("id")?,
            list_id: row.try_get("list_id")?,
            name: row.try_get("name")?,
            category_id: CategoryId::from_id_str(&cat_id_str),
            quantity: row.try_get("quantity")?,
            unit: QuantityUnit::from_unit_str(&unit_str),
            estimated_price: row.try_get("estimated_price")?,
            actual_price: row.try_get("actual_price")?,
            purchased: row.try_get("purchased")?,
            purchased_at: row.try_get("purchased_at")?,
            purchased_by_user_id: row.try_get("purchased_by_user_id")?,
            purchased_by_user_name: None,
            added_by_user_id: row.try_get("added_by_user_id")?,
            added_by_user_name: Some(user.name.clone()),
            notes: row.try_get("notes")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        }
    } else {
        state
            .memory
            .create_item(list_id, user.id, Some(user.name.clone()), payload)
            .await?
    };

    // Emite evento em tempo real para todos os membros conectados à lista
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
    let pool = state.get_pool()?;
    verify_membership(pool, list_id, user.id).await?;

    let cat_str = payload.category_id.map(|c| c.as_str().to_string());
    let unit_str = payload.unit.map(|u| u.as_str().to_string());

    let row = sqlx::query(
        r#"
        UPDATE items
        SET 
            name = COALESCE($1, name),
            category_id = COALESCE($2, category_id),
            quantity = COALESCE($3, quantity),
            unit = COALESCE($4, unit),
            estimated_price = COALESCE($5, estimated_price),
            actual_price = COALESCE($6, actual_price),
            notes = COALESCE($7, notes),
            updated_at = NOW()
        WHERE id = $8 AND list_id = $9
        RETURNING 
            id, list_id, name, category_id, quantity::float8 as quantity, unit,
            estimated_price::float8 as estimated_price, actual_price::float8 as actual_price,
            purchased, purchased_at, purchased_by_user_id, added_by_user_id, notes,
            created_at, updated_at
        "#,
    )
    .bind(&payload.name)
    .bind(&cat_str)
    .bind(payload.quantity)
    .bind(&unit_str)
    .bind(payload.estimated_price)
    .bind(payload.actual_price)
    .bind(&payload.notes)
    .bind(item_id)
    .bind(list_id)
    .fetch_one(pool)
    .await?;

    let cat_id_ret: String = row.try_get("category_id")?;
    let unit_ret: String = row.try_get("unit")?;

    let item = Item {
        id: row.try_get("id")?,
        list_id: row.try_get("list_id")?,
        name: row.try_get("name")?,
        category_id: CategoryId::from_id_str(&cat_id_ret),
        quantity: row.try_get("quantity")?,
        unit: QuantityUnit::from_unit_str(&unit_ret),
        estimated_price: row.try_get("estimated_price")?,
        actual_price: row.try_get("actual_price")?,
        purchased: row.try_get("purchased")?,
        purchased_at: row.try_get("purchased_at")?,
        purchased_by_user_id: row.try_get("purchased_by_user_id")?,
        purchased_by_user_name: None,
        added_by_user_id: row.try_get("added_by_user_id")?,
        added_by_user_name: None,
        notes: row.try_get("notes")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    };

    // Emite atualização para a sala WebSocket
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
    let item = if let Some(pool) = &state.pool {
        verify_membership(pool, list_id, user.id).await?;

        let row = if payload.purchased {
            sqlx::query(
                r#"
                UPDATE items
                SET 
                    purchased = true,
                    purchased_at = NOW(),
                    purchased_by_user_id = $1,
                    actual_price = COALESCE($2, actual_price, estimated_price),
                    updated_at = NOW()
                WHERE id = $3 AND list_id = $4
                RETURNING 
                    id, list_id, name, category_id, quantity::float8 as quantity, unit,
                    estimated_price::float8 as estimated_price, actual_price::float8 as actual_price,
                    purchased, purchased_at, purchased_by_user_id, added_by_user_id, notes,
                    created_at, updated_at
                "#,
            )
            .bind(user.id)
            .bind(payload.actual_price)
            .bind(item_id)
            .bind(list_id)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query(
                r#"
                UPDATE items
                SET 
                    purchased = false,
                    purchased_at = NULL,
                    purchased_by_user_id = NULL,
                    updated_at = NOW()
                WHERE id = $1 AND list_id = $2
                RETURNING 
                    id, list_id, name, category_id, quantity::float8 as quantity, unit,
                    estimated_price::float8 as estimated_price, actual_price::float8 as actual_price,
                    purchased, purchased_at, purchased_by_user_id, added_by_user_id, notes,
                    created_at, updated_at
                "#,
            )
            .bind(item_id)
            .bind(list_id)
            .fetch_one(pool)
            .await?
        };

        let cat_id_ret: String = row.try_get("category_id")?;
        let unit_ret: String = row.try_get("unit")?;

        Item {
            id: row.try_get("id")?,
            list_id: row.try_get("list_id")?,
            name: row.try_get("name")?,
            category_id: CategoryId::from_id_str(&cat_id_ret),
            quantity: row.try_get("quantity")?,
            unit: QuantityUnit::from_unit_str(&unit_ret),
            estimated_price: row.try_get("estimated_price")?,
            actual_price: row.try_get("actual_price")?,
            purchased: row.try_get("purchased")?,
            purchased_at: row.try_get("purchased_at")?,
            purchased_by_user_id: row.try_get("purchased_by_user_id")?,
            purchased_by_user_name: if payload.purchased {
                Some(user.name.clone())
            } else {
                None
            },
            added_by_user_id: row.try_get("added_by_user_id")?,
            added_by_user_name: None,
            notes: row.try_get("notes")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        }
    } else {
        state
            .memory
            .toggle_item(
                item_id,
                user.id,
                Some(user.name.clone()),
                payload.purchased,
                payload.actual_price,
            )
            .await?
    };

    // Emite atualização do status comprado para os membros
    state
        .hub
        .send(
            list_id,
            WsServerMessage::ItemPurchasedToggled {
                list_id,
                item_id,
                purchased: payload.purchased,
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
    } else {
        state.memory.delete_item(item_id, user.id).await?;
    }

    // Emite exclusão de item via WebSocket
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
