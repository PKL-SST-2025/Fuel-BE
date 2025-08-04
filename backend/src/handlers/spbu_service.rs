use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    models::{spbu_service::*, AppState},
    utils::response::{error, success},
};

// Menambahkan service ke SPBU
pub async fn add_service_to_spbu(
    State(state): State<AppState>,
    Path(spbu_id): Path<Uuid>,
    Json(payload): Json<AddServiceToSpbuRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Cek apakah SPBU ada
    let spbu = sqlx::query!("SELECT id FROM spbu WHERE id = $1", spbu_id)
        .fetch_optional(&state.pg_pool)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if spbu.is_none() {
        return Err(error(StatusCode::NOT_FOUND, "SPBU not found"));
    }

    // Cek apakah service ada
    let service = sqlx::query!("SELECT id FROM services WHERE id = $1", payload.service_id)
        .fetch_optional(&state.pg_pool)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if service.is_none() {
        return Err(error(StatusCode::NOT_FOUND, "Service not found"));
    }

    // Tambahkan service ke SPBU
    let result = sqlx::query!(
        r#"
        INSERT INTO spbu_services (spbu_id, service_id)
        VALUES ($1, $2)
        ON CONFLICT (spbu_id, service_id) DO NOTHING
        RETURNING spbu_id, service_id
        "#,
        spbu_id,
        payload.service_id
    )
    .fetch_optional(&state.pg_pool)
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
    let result = sqlx::query!(
        "DELETE FROM spbu_services WHERE spbu_id = $1 AND service_id = $2",
        spbu_id,
        service_id
    )
    .execute(&state.pg_pool)
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
        SpbuServiceResponse,
        r#"
        SELECT 
            ss.spbu_id, 
            ss.service_id,
            s.nama as service_name,
            s.icon_url as service_icon_url
        FROM spbu_services ss
        JOIN services s ON ss.service_id = s.id
        WHERE ss.spbu_id = $1
        "#,
        spbu_id
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(success(services))
}

// Mendapatkan semua SPBU yang memiliki service tertentu
pub async fn get_spbus_by_service(
    State(state): State<AppState>,
    Path(service_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let spbus = sqlx::query!(
        r#"
        SELECT 
            s.id, 
            s.nama, 
            s.alamat,
            s.latitude,
            s.longitude,
            s.rating,
            s.jumlah_pompa,
            s.jumlah_antrian,
            s.foto,
            s.created_at,
            s.updated_at,
            b.nama as brand_nama,
            b.logo_url as brand_logo_url
        FROM spbu_services ss
        JOIN spbu s ON ss.spbu_id = s.id
        JOIN brands b ON s.brand_id = b.id
        WHERE ss.service_id = $1
        "#,
        service_id
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(success(spbus))
}
