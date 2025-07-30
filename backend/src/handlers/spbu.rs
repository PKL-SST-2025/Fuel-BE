use axum::{extract::State, Json};
use crate::models::spbu::Spbu;
use crate::AppState;
use uuid::Uuid;

// GET all SPBU
pub async fn get_all_spbu(
    State(state): State<AppState>,
) -> Result<Json<Vec<Spbu>>, (axum::http::StatusCode, String)> {
    let spbus = sqlx::query_as::<_, Spbu>("SELECT * FROM spbu")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(spbus))
}

// GET SPBU by id
pub async fn get_spbu_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Spbu>, (axum::http::StatusCode, String)> {
    let spbu = sqlx::query_as::<_, Spbu>("SELECT * FROM spbu WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(spbu))
}

// POST SPBU
pub async fn create_spbu(
    State(state): State<AppState>,
    Json(payload): Json<Spbu>,
) -> Result<Json<Spbu>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Spbu>(
        "INSERT INTO spbu (id, nama, alamat, latitude, longitude, brand_id, rating, jumlah_pompa, jumlah_antrian, foto, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) RETURNING *"
    )
    .bind(payload.id)
    .bind(payload.nama)
    .bind(payload.alamat)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(payload.brand_id)
    .bind(payload.rating)
    .bind(payload.jumlah_pompa)
    .bind(payload.jumlah_antrian)
    .bind(payload.foto)
    .bind(payload.created_at)
    .bind(payload.updated_at)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}

// PUT SPBU
pub async fn update_spbu(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<Spbu>,
) -> Result<Json<Spbu>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Spbu>(
        "UPDATE spbu SET nama = $1, alamat = $2, latitude = $3, longitude = $4, brand_id = $5, rating = $6, jumlah_pompa = $7, jumlah_antrian = $8, foto = $9, updated_at = $10 WHERE id = $11 RETURNING *"
    )
    .bind(&payload.nama)
    .bind(&payload.alamat)
    .bind(&payload.latitude)
    .bind(&payload.longitude)
    .bind(&payload.brand_id)
    .bind(&payload.rating)
    .bind(&payload.jumlah_pompa)
    .bind(&payload.jumlah_antrian)
    .bind(&payload.foto)
    .bind(&payload.updated_at)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}

// DELETE SPBU
pub async fn delete_spbu(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Spbu>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Spbu>(
        "DELETE FROM spbu WHERE id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}
