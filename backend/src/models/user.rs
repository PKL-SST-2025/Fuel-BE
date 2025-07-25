use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub nama_lengkap: Option<String>, // ← ini sebelumnya String, harus Option<String>
    pub no_hp: String,
    pub foto_profile: Option<String>,
    pub bio: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
