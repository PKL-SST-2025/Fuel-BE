use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use uuid::Uuid;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use password_hash::{SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

use crate::AppState;

#[derive(Deserialize)]
pub struct UserPayload {
    pub nama_lengkap: String,
    pub email: String,
    pub password: String,
    pub no_hp: String,
    pub jenis_kelamin: String,
    pub tanggal_lahir: NaiveDate,
    pub foto_profile: String,
}

#[derive(Serialize, FromRow)]
pub struct RegisterUserModel {
    pub id: Uuid,
    pub nama_lengkap: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub no_hp: Option<String>,
    pub foto_profile: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<UserPayload>,
) -> Result<Json<RegisterUserModel>, (StatusCode, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Hash error: {}", e)))?
        .to_string();

    let now = Utc::now().naive_utc();

    // ...existing code...

    let row = sqlx::query!(
        r#"
        INSERT INTO users (
            nama_lengkap,
            email,
            password_hash,
            no_hp,
            foto_profile,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, nama_lengkap, email, password_hash, no_hp, foto_profile, created_at
        "#,
        payload.nama_lengkap,
        payload.email,
        hashed_password,
        payload.no_hp,
        payload.foto_profile,
        Some(now)
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let result = RegisterUserModel {
        id: row.id,
        nama_lengkap: row.nama_lengkap,
        email: row.email,
        password_hash: row.password_hash,
        no_hp: row.no_hp,
        foto_profile: row.foto_profile,
        created_at: row.created_at,
    };

    Ok(Json(result))
}

use axum::extract::Path;

// GET /users
pub async fn get_users(State(state): State<AppState>) -> Result<Json<Vec<RegisterUserModel>>, (StatusCode, String)> {
    let users = sqlx::query_as!(
        RegisterUserModel,
        r#"SELECT id, nama_lengkap, email, password_hash, no_hp, foto_profile, created_at FROM users"#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
    Ok(Json(users))
}

// GET /user/:id
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RegisterUserModel>, (StatusCode, String)> {
    let user = sqlx::query_as!(
        RegisterUserModel,
        r#"SELECT id, nama_lengkap, email, password_hash, no_hp, foto_profile, created_at FROM users WHERE id = $1"#,
        id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, format!("User not found: {}", e)))?;
    Ok(Json(user))
}

// PUT /user/:id
pub async fn update_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UserPayload>,
) -> Result<Json<RegisterUserModel>, (StatusCode, String)> {
    let now = Utc::now().naive_utc();
    let user = sqlx::query_as!(
        RegisterUserModel,
        r#"UPDATE users SET nama_lengkap = $1, email = $2, no_hp = $3, foto_profile = $4, created_at = $5 WHERE id = $6 RETURNING id, nama_lengkap, email, password_hash, no_hp, foto_profile, created_at"#,
        payload.nama_lengkap,
        payload.email,
        payload.no_hp,
        Some(payload.foto_profile),
        Some(now),
        id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {}", e)))?;
    Ok(Json(user))
}

// DELETE /user/:id
pub async fn delete_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {}", e)))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// LOGIN ENDPOINT
#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub id: Uuid,
    pub email: String,
    pub nama_lengkap: Option<String>,
    pub token: Option<String>, // placeholder, belum JWT
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = sqlx::query_as!(
        RegisterUserModel,
        r#"SELECT id, nama_lengkap, email, password_hash, no_hp, foto_profile, created_at FROM users WHERE email = $1"#,
        payload.email
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let user = match user {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, "Email tidak ditemukan".to_string())),
    };

    // Cek password
    let parsed_hash = match password_hash::PasswordHash::new(&user.password_hash) {
        Ok(h) => h,
        Err(_) => return Err((StatusCode::UNAUTHORIZED, "Hash error".to_string())),
    };
    let argon2 = Argon2::default();
    if argon2.verify_password(payload.password.as_bytes(), &parsed_hash).is_err() {
        return Err((StatusCode::UNAUTHORIZED, "Password salah".to_string()));
    }

    let resp = LoginResponse {
        id: user.id,
        email: user.email,
        nama_lengkap: user.nama_lengkap,
        token: None, // bisa diisi JWT nanti
    };
    Ok(Json(resp))
}

// FORGOT PASSWORD ENDPOINT
#[derive(Deserialize)]
pub struct ForgotPasswordPayload {
    pub email: String,
    pub new_password: String,
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Cari user by email
    let user = sqlx::query!("SELECT id FROM users WHERE email = $1", payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
    let user = match user {
        Some(u) => u,
        None => return Err((StatusCode::NOT_FOUND, "Email tidak ditemukan".to_string())),
    };

    // Hash password baru
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(payload.new_password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Hash error: {}", e)))?
        .to_string();

    // Update password di DB
    sqlx::query!("UPDATE users SET password_hash = $1 WHERE id = $2", hashed_password, user.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
    Ok(StatusCode::OK)
}