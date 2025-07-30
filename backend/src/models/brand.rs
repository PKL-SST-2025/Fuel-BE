use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Brand {
    pub id: Uuid,
    pub nama: String,
    pub logo_url: Option<String>,
}