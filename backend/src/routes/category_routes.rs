use axum::Json;
use shared::categories::{CategoryId, CategoryInfo};

pub async fn list_categories() -> Json<Vec<CategoryInfo>> {
    let categories: Vec<CategoryInfo> = CategoryId::all()
        .iter()
        .map(|id| (*id).into())
        .collect();

    Json(categories)
}
