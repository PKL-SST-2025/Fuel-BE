use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Spbu {
    pub id: Uuid,
    pub nama: String,
    pub alamat: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub brand_id: Option<Uuid>,
    pub rating: Option<f64>,
    pub jumlah_pompa: Option<i32>,
    pub jumlah_antrian: Option<i32>,
    pub foto: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}