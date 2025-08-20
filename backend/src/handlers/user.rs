use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::{Html, Json, IntoResponse},
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use password_hash::SaltString;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

use crate::{AppState, auth::{self, Claims}};

// Handler untuk menampilkan halaman register
pub async fn show_register_form() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Register</title>
        <style>
            body { 
                font-family: Arial, sans-serif; 
                max-width: 500px; 
                margin: 0 auto; 
                padding: 20px; 
            }
            .form-group { 
                margin-bottom: 15px; 
            }
            label { 
                display: block; 
                margin-bottom: 5px; 
            }
            input { 
                width: 100%; 
                padding: 8px; 
                margin-bottom: 10px; 
                box-sizing: border-box;
            }
            button { 
                padding: 10px 15px; 
                background-color: #4CAF50; 
                color: white; 
                border: none; 
                cursor: pointer; 
                width: 100%;
            }
            button:hover { 
                background-color: #45a049; 
            }
            .login-link {
                margin-top: 15px;
                text-align: center;
            }
        </style>
    </head>
    <body>
        <h2>Register</h2>
        <form id="registerForm" onsubmit="event.preventDefault(); submitForm();">
            <div class="form-group">
                <label for="nama_lengkap">Nama Lengkap:</label>
                <input type="text" id="nama_lengkap" name="nama_lengkap" required>
            </div>
            <div class="form-group">
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" required>
            </div>
            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required>
            </div>
            <div class="form-group">
                <label for="no_hp">No HP:</label>
                <input type="text" id="no_hp" name="no_hp" required>
            </div>
            <div class="form-group">
                <label for="jenis_kelamin">Jenis Kelamin (L/P):</label>
                <input type="text" id="jenis_kelamin" name="jenis_kelamin" required>
            </div>
            <div class="form-group">
                <label for="tanggal_lahir">Tanggal Lahir (YYYY-MM-DD):</label>
                <input type="date" id="tanggal_lahir" name="tanggal_lahir" required>
            </div>
            <div class="form-group">
                <label for="foto_profile">Foto Profile (URL):</label>
                <input type="text" id="foto_profile" name="foto_profile" value="default.jpg">
            </div>
            <button type="submit">Register</button>
            <div class="login-link">
                Sudah punya akun? <a href="/login">Login disini</a>
            </div>
        </form>
        
        <script>
        async function submitForm() {
            const form = document.getElementById('registerForm');
            const formData = {
                nama_lengkap: document.getElementById('nama_lengkap').value,
                email: document.getElementById('email').value,
                password: document.getElementById('password').value,
                no_hp: document.getElementById('no_hp').value,
                jenis_kelamin: document.getElementById('jenis_kelamin').value,
                tanggal_lahir: document.getElementById('tanggal_lahir').value,
                foto_profile: document.getElementById('foto_profile').value || 'default.jpg'
            };
            
            try {
                const response = await fetch('/register', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(formData)
                });
                
                const result = await response.json();
                if (response.ok) {
                    alert('Registrasi berhasil! Silakan login.');
                    window.location.href = '/login';
                } else {
                    alert('Gagal mendaftar: ' + (result.message || 'Terjadi kesalahan'));
                }
            } catch (error) {
                console.error('Error:', error);
                alert('Terjadi kesalahan saat mengirim data');
            }
        }
        </script>
    </body>
    </html>
    "#;
    
    Html(html)
}

#[derive(Deserialize)]
pub struct UserPayload {
    pub nama_lengkap: String,
    pub email: String,
    pub password: String,
    pub no_hp: String,
    pub jenis_kelamin: String,
    pub tanggal_lahir: NaiveDate,
    pub foto_profile: String,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "user".to_string()
}

#[derive(Serialize, FromRow)]
pub struct RegisterUserModel {
    pub id: Uuid,
    pub nama_lengkap: Option<String>,
    pub email: String,
    #[serde(skip_serializing)] // Jangan tampilkan password_hash di response
    pub password_hash: String,
    pub no_hp: Option<String>,
    pub foto_profile: Option<String>,
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    let row = sqlx::query!(
        r#"
        INSERT INTO users (
            nama_lengkap,
            email,
            password_hash,
            no_hp,
            foto_profile,
            role,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, nama_lengkap, email, password_hash, no_hp, foto_profile, role, created_at
        "#,
        payload.nama_lengkap,
        payload.email,
        hashed_password,
        payload.no_hp,
        payload.foto_profile,
        payload.role,
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
        role: row.role,
        created_at: row.created_at,
    };

    Ok(Json(result))
}

// GET /users
pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<RegisterUserModel>>, (StatusCode, String)> {
    match sqlx::query_as::<_, RegisterUserModel>(
        r#"
        SELECT id, nama_lengkap, email, password_hash, no_hp, foto_profile, role, created_at 
        FROM users
        "#
    )
    .fetch_all(&state.db)
    .await {
        Ok(users) => Ok(Json(users)),
        Err(e) => {
            tracing::error!("Failed to fetch users: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users".to_string()))
        }
    }
}

// GET /user/:id
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RegisterUserModel>, (StatusCode, String)> {
    let user = sqlx::query_as!(
        RegisterUserModel,
        r#"SELECT id, nama_lengkap, email, password_hash, no_hp, foto_profile, role, created_at FROM users WHERE id = $1"#,
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
        r#"UPDATE users SET nama_lengkap = $1, email = $2, no_hp = $3, foto_profile = $4, role = $5, created_at = $6 WHERE id = $7 RETURNING id, nama_lengkap, email, password_hash, no_hp, foto_profile, role, created_at"#,
        payload.nama_lengkap,
        payload.email,
        payload.no_hp,
        Some(payload.foto_profile),
        payload.role,
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
    pub role: Option<String>,
    pub token: String,
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user = sqlx::query_as!(
        RegisterUserModel,
        r#"SELECT id, nama_lengkap, email, password_hash, no_hp, foto_profile, role, created_at FROM users WHERE email = $1"#,
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

    // Generate JWT token
    let role = user.role.unwrap_or_else(|| "user".to_string());
    let token = auth::create_jwt(user.id, &role)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal membuat token: {}", e)))?;

    let resp = LoginResponse {
        id: user.id,
        email: user.email,
        nama_lengkap: user.nama_lengkap,
        role: Some(role),
        token,
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