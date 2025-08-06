use axum::{
    extract::{Path, State, Json},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, FromRow)]
pub struct WishlistResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub spbu_id: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateWishlistRequest {
    pub spbu_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct WishlistWithSpbuResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub spbu_id: Uuid,
    pub spbu_name: String,
    pub spbu_address: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

// Extract user_id from header
fn get_user_id_from_headers(headers: &axum::http::HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid or missing X-User-Id header".to_string()))
}

// Tambah ke wishlist
pub async fn add_to_wishlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWishlistRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = get_user_id_from_headers(&headers)?;
    
    // First check if the SPBU exists
    let spbu_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spbu WHERE id = $1)"
    )
    .bind(payload.spbu_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !spbu_exists {
        return Err((StatusCode::NOT_FOUND, "SPBU not found".to_string()));
    }

    let wishlist = sqlx::query_as::<_, WishlistResponse>(
        r#"
        INSERT INTO wishlists (user_id, spbu_id)
        VALUES ($1, $2)
        RETURNING id, user_id, spbu_id, created_at, updated_at
        "#
    )
    .bind(user_id)
    .bind(payload.spbu_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            (StatusCode::CONFLICT, "SPBU already in wishlist".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    Ok(Json(wishlist))
}

// Hapus dari wishlist
pub async fn remove_from_wishlist(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wishlist_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = get_user_id_from_headers(&headers)?;
    
    let result = sqlx::query(
        "DELETE FROM wishlists WHERE id = $1 AND user_id = $2"
    )
    .bind(wishlist_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Wishlist item not found".to_string()));
    }

    Ok((StatusCode::NO_CONTENT, ()))
}

// Lihat daftar wishlist user
pub async fn get_user_wishlists(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<WishlistWithSpbuResponse>>, (StatusCode, String)> {
    let user_id = get_user_id_from_headers(&headers)?;
    
    let wishlists = sqlx::query_as::<_, WishlistWithSpbuResponse>(
        r#"
        SELECT 
            w.id, 
            w.user_id, 
            w.spbu_id, 
            s.nama as spbu_name,
            s.alamat as spbu_address,
            w.created_at, 
            w.updated_at
        FROM wishlists w
        JOIN spbu s ON w.spbu_id = s.id
        WHERE w.user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(wishlists))
}
