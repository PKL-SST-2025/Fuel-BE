use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub spbu_id: Uuid,
    pub rating: f64,
    pub comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Request/Response structs
#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub spbu_id: Uuid,
    pub rating: f64,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewRequest {
    pub rating: Option<f64>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: Option<String>,  // Made optional to handle NULL values from database
    pub spbu_id: Uuid,
    pub spbu_name: String,
    pub rating: f64,
    pub comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RatingCount {
    pub rating: i32,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct SpbuRatingResponse {
    pub average_rating: f64,
    pub total_reviews: i64,
    pub rating_distribution: Vec<RatingCount>,
}
