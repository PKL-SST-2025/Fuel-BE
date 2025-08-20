use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::models::service::Service;
use crate::models::spbu::Spbu;

#[derive(Debug, Deserialize)]
pub struct AddServiceToSpbuRequest {
    pub service_id: Uuid,
}

// Helper functions for response
fn success<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "success",
        "data": data
    }))
}

fn error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    let error_response = serde_json::json!({
        "status": "error",
        "message": message.into()
    });
    (status, Json(error_response))
}

// Menambahkan service ke SPBU
pub async fn add_service_to_spbu(
    State(state): State<AppState>,
    Path(spbu_id): Path<Uuid>,
    Json(payload): Json<AddServiceToSpbuRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Cek apakah SPBU ada
    let spbu: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM spbu WHERE id = $1")
        .bind(spbu_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if spbu.is_none() {
        return Err(error(StatusCode::NOT_FOUND, "SPBU not found"));
    }

    // Cek apakah service ada
    let service: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM services WHERE id = $1")
        .bind(payload.service_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if service.is_none() {
        return Err(error(StatusCode::NOT_FOUND, "Service not found"));
    }

    // Tambahkan service ke SPBU
    let result: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        INSERT INTO spbu_services (spbu_id, service_id)
        VALUES ($1, $2)
        ON CONFLICT (spbu_id, service_id) DO NOTHING
        RETURNING spbu_id, service_id
        "#
    )
    .bind(spbu_id)
    .bind(payload.service_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.is_none() {
        return Err(error(StatusCode::CONFLICT, "Service already added to SPBU"));
    }

    Ok(success("Service added to SPBU successfully"))
}

// Menghapus service dari SPBU
pub async fn remove_service_from_spbu(
    State(state): State<AppState>,
    Path((spbu_id, service_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM spbu_services WHERE spbu_id = $1 AND service_id = $2"
    )
    .bind(spbu_id)
    .bind(service_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if exists.is_none() {
        return Err(error(
            StatusCode::NOT_FOUND,
            "Service not found in this SPBU",
        ));
    }

    let result = sqlx::query(
        "DELETE FROM spbu_services WHERE spbu_id = $1 AND service_id = $2"
    )
    .bind(spbu_id)
    .bind(service_id)
    .execute(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(error(
            StatusCode::NOT_FOUND,
            "Service not found in this SPBU",
        ));
    }

    Ok(success("Service removed from SPBU successfully"))
}

// Mendapatkan semua service dari SPBU tertentu
pub async fn get_services_by_spbu(
    State(state): State<AppState>,
    Path(spbu_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let services = sqlx::query_as!(
        Service,
        "SELECT s.* FROM services s 
         JOIN spbu_services ss ON s.id = ss.service_id 
         WHERE ss.spbu_id = $1",
        spbu_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(success(services))
}

// Mendapatkan semua SPBU yang memiliki service tertentu
pub async fn get_spbus_by_service(
    State(state): State<AppState>,
    Path(service_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let spbus = sqlx::query_as!(
        Spbu,
        "SELECT s.* FROM spbu s 
         JOIN spbu_services ss ON s.id = ss.spbu_id 
         WHERE ss.service_id = $1",
        service_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(success(spbus))
}
