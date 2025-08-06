use axum::{
    http::{Request, StatusCode, HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
    body::Body,
};
use uuid::Uuid;

pub async fn auth_middleware<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, (StatusCode, String)> {
    // Get the authorization header
    let user_id = req.headers()
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    if let Some(user_id) = user_id {
        // Add user_id to request extensions
        req.extensions_mut().insert(user_id);
        
        // Convert the response to the expected type
        let response = next.run(req).await;
        Ok(response)
    } else {
        Err((StatusCode::UNAUTHORIZED, "Unauthorized: Missing or invalid token".to_string()))
    }
}

// Helper function to extract user_id from request extensions
pub fn get_user_id_from_headers(headers: &axum::http::HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid or missing user ID".to_string()))
}
