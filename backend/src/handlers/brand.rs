use axum::{extract::State, Json, Extension};
use sqlx::PgPool;
use crate::models::brand::Brand;
use uuid::Uuid;
use crate::AppState;


pub async fn get_all_brands(
    State(state): State<AppState>,
) -> Result<Json<Vec<Brand>>, (axum::http::StatusCode, String)> {
    let brands = sqlx::query_as::<_, Brand>("SELECT * FROM brands")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(brands))
}

pub async fn create_brands(
    State(state): State<AppState>,
    Json(payload): Json<Brand>,
) -> Result<Json<Brand>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Brand>(
        "INSERT INTO brands (id, nama, logo_url) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(payload.id)
    .bind(payload.nama)
    .bind(payload.logo_url)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}

pub async fn update_brands(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<Brand>,
) -> Result<Json<Brand>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Brand>(
        "UPDATE brands SET nama = $1, logo_url = $2 WHERE id = $3 RETURNING *"
    )
    .bind(&payload.nama)
    .bind(&payload.logo_url)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}

pub async fn delete_brands(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<Brand>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Brand>(
        "DELETE FROM brands WHERE id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}
