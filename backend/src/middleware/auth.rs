use axum::{
    http::{Request, StatusCode, header::AUTHORIZATION, HeaderMap},
    middleware::Next,
    response::Response,
    body::Body,
};
use uuid::Uuid;
use crate::auth::{self, Claims};
use tracing;

/// Middleware untuk autentikasi JWT
/// 
/// Middleware ini akan memeriksa header Authorization dan menambahkan user_id ke request extensions
/// jika token valid.
pub async fn auth_middleware(
    mut request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // Dapatkan header Authorization
    let token = request.headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer ").map(str::trim))
        .ok_or_else(|| {
            tracing::error!("Missing or invalid Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Validasi token JWT
    let claims = auth::validate_token(token).map_err(|e| {
        tracing::error!("Token validation failed: {:?}", e);
        match e {
            auth::AuthError::ExpiredToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::UNAUTHORIZED,
        }
    })?;

    // Dapatkan user_id dari claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in token");
        StatusCode::UNAUTHORIZED
    })?;

    // Tambahkan user_id ke request extensions
    request.extensions_mut().insert(user_id);

    // Lanjutkan ke handler berikutnya
    Ok(next.run(request).await)
}

/// Helper function untuk mengekstrak user_id dari header request
/// 
/// Fungsi ini digunakan untuk mendapatkan user_id dari token JWT
pub fn get_user_id_from_headers(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ").map(str::trim))
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header".to_string()))?;

    let claims = auth::validate_token(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string()))?;

    Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID in token".to_string()))
}
