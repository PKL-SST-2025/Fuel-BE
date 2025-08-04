use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpbuService {
    pub spbu_id: Uuid,
    pub service_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Struct untuk request menambahkan service ke SPBU
#[derive(Debug, Deserialize)]
pub struct AddServiceToSpbuRequest {
    pub service_id: Uuid,
}

// Struct untuk response relasi SPBU-Service
#[derive(Debug, Serialize)]
pub struct SpbuServiceResponse {
    pub spbu_id: Uuid,
    pub service_id: Uuid,
    pub service_name: String,
    pub service_icon_url: Option<String>,
}
