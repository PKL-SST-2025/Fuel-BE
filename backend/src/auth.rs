use axum::{
    http::{Request, StatusCode, header},
    response::Response,
    body::Body,
    middleware::Next,
};
use jsonwebtoken::{decode, DecodingKey, Validation, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::env;
use chrono::{Duration, Utc};
use uuid::Uuid;
use tracing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub role: String,
    pub exp: i64,    // expiry time
}

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    MissingToken,
    ExpiredToken,
    InvalidRole,
}

impl Claims {
    pub fn new(user_id: Uuid, role: &str) -> Self {
        let exp = (Utc::now() + Duration::days(30)).timestamp(); // Token berlaku 30 hari
        Self {
            sub: user_id.to_string(),
            role: role.to_string(),
            exp,
        }
    }
}

pub fn create_jwt(user_id: Uuid, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let claims = Claims::new(user_id, role);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    
    let validation = Validation::default();
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Err(AuthError::ExpiredToken),
            _ => Err(AuthError::InvalidToken),
        },
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Middleware untuk memeriksa token JWT
pub async fn auth_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Dapatkan header Authorization
    let token = request.headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer ").map(str::trim))
        .ok_or_else(|| {
            tracing::error!("Missing or invalid Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Validasi token JWT
    let claims = validate_token(token).map_err(|e| {
        tracing::error!("Token validation failed: {:?}", e);
        match e {
            AuthError::ExpiredToken => StatusCode::UNAUTHORIZED,
            _ => StatusCode::UNAUTHORIZED,
        }
    })?;

    // Dapatkan user_id dari claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in token");
        StatusCode::UNAUTHORIZED
    })?;

    tracing::info!("Authenticated user: {}", user_id);

    // Tambahkan user_id ke request extensions
    let (mut parts, body) = request.into_parts();
    parts.extensions.insert(user_id);
    let request = Request::from_parts(parts, body);

    // Lanjutkan ke handler berikutnya
    let response = next.run(request).await;
    Ok(response)
}

// Middleware untuk memeriksa role
pub async fn require_role(
    role: &'static str,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request.extensions()
        .get::<Claims>()
        .ok_or_else(|| {
            tracing::error!("No claims found in request");
            StatusCode::UNAUTHORIZED
        })?;
    
    if claims.role != role && claims.role != "admin" {
        tracing::warn!("Insufficient permissions: required={}, actual={}", role, claims.role);
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(request).await)
}
