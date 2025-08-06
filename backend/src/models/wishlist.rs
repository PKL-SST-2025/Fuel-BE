use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Wishlist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub spbu_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Request/Response structs
#[derive(Debug, Deserialize)]
pub struct CreateWishlistRequest {
    pub spbu_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct WishlistResponse {
    pub id: Uuid,
    pub spbu_id: Uuid,
    pub spbu_name: Option<String>,
    pub spbu_address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
