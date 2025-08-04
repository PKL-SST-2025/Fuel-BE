use axum::{extract::State, Json};
use crate::models::service::Service;
use crate::AppState;
use uuid::Uuid;

// GET all services
pub async fn get_all_services(
    State(state): State<AppState>,
) -> Result<Json<Vec<Service>>, (axum::http::StatusCode, String)> {
    let services = sqlx::query_as::<_, Service>("SELECT * FROM services")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(services))
}

// GET service by id
pub async fn get_service_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Service>, (axum::http::StatusCode, String)> {
    let service = sqlx::query_as::<_, Service>("SELECT * FROM services WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(Json(service))
}

// POST service
pub async fn create_service(
    State(state): State<AppState>,
    Json(payload): Json<Service>,
) -> Result<Json<Service>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Service>(
        "INSERT INTO services (id, nama, icon_url) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(payload.id)
    .bind(payload.nama)
    .bind(payload.icon_url)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}

// PUT service
pub async fn update_service(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<Service>,
) -> Result<Json<Service>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Service>(
        "UPDATE services SET nama = $1, icon_url = $2 WHERE id = $3 RETURNING *"
    )
    .bind(&payload.nama)
    .bind(&payload.icon_url)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}

// DELETE service
pub async fn delete_service(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Service>, (axum::http::StatusCode, String)> {
    let rec = sqlx::query_as::<_, Service>(
        "DELETE FROM services WHERE id = $1 RETURNING *"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rec))
}
