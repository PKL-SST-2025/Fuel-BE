use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::models::review::{CreateReviewRequest, Review, ReviewResponse, SpbuRatingResponse, UpdateReviewRequest, RatingCount};
use crate::AppState;

// Helper functions for response
fn success<T: Serialize>(data: T) -> Json<serde_json::Value> {
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

// Buat review baru
pub async fn create_review(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Validasi rating
    if payload.rating < 1.0 || payload.rating > 5.0 {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "Rating must be between 1 and 5",
        ));
    }

    // Dapatkan user_id dari header
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Invalid or missing X-User-Id header"))?;

    // Cek apakah SPBU ada
    let spbu_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spbu WHERE id = $1)")
        .bind(payload.spbu_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !spbu_exists {
        return Err(error(StatusCode::NOT_FOUND, "SPBU not found"));
    }

    // Buat review
    let review = sqlx::query_as!(
        crate::models::review::Review,
        r#"
        INSERT INTO reviews (user_id, spbu_id, rating, comment)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, spbu_id, rating, comment, created_at, updated_at
        "#,
        user_id,
        payload.spbu_id,
        payload.rating,
        payload.comment
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            error(StatusCode::CONFLICT, "You have already reviewed this SPBU")
        } else {
            error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Dapatkan nama user dan SPBU untuk response
    let response = get_review_with_details(&state, review.id).await?;

    Ok((StatusCode::CREATED, success(response)))
}

// Dapatkan detail review
pub async fn get_review(
    State(state): State<AppState>,
    Path(review_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let review = get_review_with_details(&state, review_id).await?;
    Ok(success(review))
}

// Update review
pub async fn update_review(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(review_id): Path<Uuid>,
    Json(payload): Json<UpdateReviewRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Validasi rating jika ada
    if let Some(rating) = payload.rating {
        if rating < 1.0 || rating > 5.0 {
            return Err(error(
                StatusCode::BAD_REQUEST,
                "Rating must be between 1 and 5",
            ));
        }
    }

    // Dapatkan user_id dari header
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Invalid or missing X-User-Id header"))?;

    // Update review
    let review = sqlx::query_as!(
        crate::models::review::Review,
        r#"
        UPDATE reviews
        SET 
            rating = COALESCE($1, rating),
            comment = COALESCE($2, comment),
            updated_at = NOW()
        WHERE id = $3 AND user_id = $4
        RETURNING id, user_id, spbu_id, rating, comment, created_at, updated_at
        "#,
        payload.rating,
        payload.comment,
        review_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let review = review.ok_or_else(|| error(StatusCode::NOT_FOUND, "Review not found"))?;
    let response = get_review_with_details(&state, review.id).await?;

    Ok(success(response))
}

// Hapus review
pub async fn delete_review(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(review_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Dapatkan user_id dari header
    let user_id = headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "Invalid or missing X-User-Id header"))?;

    // Hapus review
    let result = sqlx::query!(
        "DELETE FROM reviews WHERE id = $1 AND user_id = $2",
        review_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(error(StatusCode::NOT_FOUND, "Review not found"));
    }

    Ok((StatusCode::NO_CONTENT, ()))
}

// Dapatkan semua review untuk SPBU tertentu
pub async fn get_spbu_reviews(
    State(state): State<AppState>,
    Path(spbu_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Cek apakah SPBU ada
    let spbu_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spbu WHERE id = $1)")
        .bind(spbu_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !spbu_exists {
        return Err(error(StatusCode::NOT_FOUND, "SPBU not found"));
    }

    // Dapatkan semua review untuk SPBU ini
    let reviews = sqlx::query_as!(
        ReviewResponse,
        r#"
        SELECT 
            r.id,
            r.user_id,
            u.nama_lengkap as user_name,  -- Menggunakan nama_lengkap dari tabel users
            r.spbu_id,
            s.nama as spbu_name,
            r.rating,
            r.comment,
            r.created_at,
            r.updated_at
        FROM reviews r
        JOIN users u ON r.user_id = u.id
        JOIN spbu s ON r.spbu_id = s.id
        WHERE r.spbu_id = $1
        ORDER BY r.created_at DESC
        "#,
        spbu_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(success(reviews))
}

// Dapatkan rating summary untuk SPBU
pub async fn get_spbu_rating(
    State(state): State<AppState>,
    Path(spbu_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Cek apakah SPBU ada
    let spbu_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spbu WHERE id = $1)")
        .bind(spbu_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !spbu_exists {
        return Err(error(StatusCode::NOT_FOUND, "SPBU not found"));
    }

    // Dapatkan rating rata-rata dan total review
    let rating_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_reviews,
            AVG(rating) as average_rating
        FROM reviews
        WHERE spbu_id = $1
        "#,
        spbu_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rating_dist = sqlx::query!(
        r#"
        SELECT 
            rating,
            COUNT(*) as count
        FROM reviews
        WHERE spbu_id = $1
        GROUP BY rating
        ORDER BY rating
        "#,
        spbu_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rating_distribution: Vec<RatingCount> = rating_dist
        .into_iter()
        .map(|r| RatingCount {
            rating: r.rating as i32,
            count: r.count.unwrap_or(0) as i32,
        })
        .collect();

    let rating_summary = SpbuRatingResponse {
        average_rating: rating_stats.average_rating.unwrap_or(0.0) as f64,
        total_reviews: rating_stats.total_reviews.unwrap_or(0) as i64,
        rating_distribution,
    };

    Ok(success(rating_summary))
}

// Helper function untuk mendapatkan detail review dengan nama user dan SPBU
async fn get_review_with_details(
    state: &AppState,
    review_id: Uuid,
) -> Result<ReviewResponse, (StatusCode, Json<serde_json::Value>)> {
    let review = sqlx::query_as!(
        ReviewResponse,
        r#"
        SELECT 
            r.id,
            r.user_id,
            u.nama_lengkap as user_name,  -- Menggunakan nama_lengkap dari tabel users
            r.spbu_id,
            s.nama as spbu_name,
            r.rating,
            r.comment,
            r.created_at,
            r.updated_at
        FROM reviews r
        JOIN users u ON r.user_id = u.id
        JOIN spbu s ON r.spbu_id = s.id
        WHERE r.id = $1
        "#,
        review_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    review.ok_or_else(|| error(StatusCode::NOT_FOUND, "Review not found"))
}
