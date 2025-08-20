mod auth;
mod handlers;
mod middleware;
mod models;
mod types;
mod utils;

use axum::{
    routing::{post, get, put, delete},
    Router,
    middleware::from_fn,
    http::{Method, header::{self, HeaderValue}, HeaderName},
};
use tower_http::cors::CorsLayer;
use crate::handlers::user::{register_user, get_users, get_user_by_id, update_user_by_id, delete_user_by_id, login_user, forgot_password};
use crate::handlers::brand::{get_all_brands, create_brands, update_brands, delete_brands};
use crate::handlers::spbu::{get_all_spbu, get_spbu_by_id, create_spbu, update_spbu, delete_spbu};
use crate::handlers::service::{get_all_services, create_service, get_service_by_id, update_service, delete_service};
use crate::handlers::spbu_service::{add_service_to_spbu, remove_service_from_spbu, get_services_by_spbu, get_spbus_by_service};
use crate::handlers::wishlist::{add_to_wishlist, remove_from_wishlist, get_user_wishlists};
use crate::handlers::review::{
    create_review, get_review, update_review, delete_review,
    get_spbu_reviews, get_spbu_rating,
};
use crate::handlers::transaction::{
    create_transaction, get_transaction, list_transactions,
    cancel_transaction, process_payment,
};

// Temporary handler to list SPBUs and their fuel prices
use axum::extract::State;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
struct SpbuWithPrices {
    id: uuid::Uuid,
    name: String,
    address: String,
    fuel_prices: Vec<FuelPrice>,
}

#[derive(Serialize)]
struct FuelPrice {
    fuel_type: String,
    price: String,
}

async fn list_spbus_with_prices(State(state): State<AppState>) -> Result<Json<Vec<SpbuWithPrices>>, String> {
    let spbus = sqlx::query!(
        r#"
        SELECT s.id, s.nama as "name", s.alamat as "address", fp.fuel_type, fp.price
        FROM spbu s
        LEFT JOIN fuel_prices fp ON s.id = fp.spbu_id
        ORDER BY s.nama, fp.fuel_type
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    use std::collections::HashMap;
    let mut spbu_map: HashMap<_, SpbuWithPrices> = HashMap::new();

    for row in spbus {
        let spbu = spbu_map.entry(row.id).or_insert_with(|| SpbuWithPrices {
            id: row.id,
            name: row.name,
            address: row.address,
            fuel_prices: Vec::new(),
        });

        spbu.fuel_prices.push(FuelPrice {
            fuel_type: row.fuel_type,
            price: row.price.to_string(),
        });
    }

    let result: Vec<_> = spbu_map.into_values().collect();
    Ok(Json(result))
}
// Auth middleware is now used directly
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    // Inisialisasi logging
    tracing_subscriber::fmt::init();
    
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");
    
    tracing::info!("Database connection established");

    let app_state = AppState { db: pool };

    // Public routes (tidak memerlukan autentikasi)
    let public_routes = Router::new()
        // Route untuk register (tidak perlu autentikasi)
        .route("/register", 
            get(handlers::user::show_register_form)
            .post(register_user)
        )
        // Route untuk mendapatkan list users (tetap di public_routes karena tidak memerlukan autentikasi)
        .route("/users", get(get_users))
        .route("/login", post(login_user))
        .route("/forgot_password", post(forgot_password))
        .route("/brands", get(get_all_brands))
        .route("/spbu", get(get_all_spbu))
        .route("/services", get(get_all_services))
        .route("/services/:id", get(get_service_by_id))
        .route("/spbu/:spbu_id/services", get(get_services_by_spbu))
        .route("/services/:service_id/spbus", get(get_spbus_by_service))
        .route("/spbu/:spbu_id/reviews", get(get_spbu_reviews))
        .route("/spbu/:spbu_id/rating", get(get_spbu_rating))
        .route("/debug/spbus-with-prices", get(list_spbus_with_prices));

    // Protected routes (membutuhkan autentikasi JWT)
    let protected_routes = Router::new()
        // User routes (yang memerlukan autentikasi)
        .route("/user/:id", get(get_user_by_id).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/user/:id", put(update_user_by_id).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/user/:id", delete(delete_user_by_id).layer(from_fn(middleware::auth::auth_middleware)))
        
        // Brand routes
        .route("/brands", post(create_brands).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/brands/:id", put(update_brands).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/brands/:id", delete(delete_brands).layer(from_fn(middleware::auth::auth_middleware)))
        
        // SPBU routes
        .route("/spbu", post(create_spbu).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/spbu/:id", put(update_spbu).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/spbu/:id", delete(delete_spbu).layer(from_fn(middleware::auth::auth_middleware)))
        
        // Service routes
        .route("/services", post(create_service).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/services/:id", put(update_service).layer(from_fn(middleware::auth::auth_middleware)))
        .route("/services/:id", delete(delete_service).layer(from_fn(middleware::auth::auth_middleware)))
        
        // SPBU-Service relationships
        .route(
            "/spbu/:spbu_id/services", 
            post(add_service_to_spbu).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/spbu/:spbu_id/services/:service_id", 
            delete(remove_service_from_spbu).layer(from_fn(middleware::auth::auth_middleware))
        )
        
        // Wishlist endpoints
        .route(
            "/wishlist", 
            post(add_to_wishlist).layer(from_fn(middleware::auth::auth_middleware))
                .get(get_user_wishlists).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/wishlist/:spbu_id", 
            delete(remove_from_wishlist).layer(from_fn(middleware::auth::auth_middleware))
        )
        
        // Review routes
        .route(
            "/reviews", 
            post(create_review).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/reviews/:review_id", 
            get(get_review).layer(from_fn(middleware::auth::auth_middleware))
                .put(update_review).layer(from_fn(middleware::auth::auth_middleware))
                .delete(delete_review).layer(from_fn(middleware::auth::auth_middleware))
        )
        
        // Transaction routes
        .route(
            "/transactions",
            post(create_transaction).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/transactions",
            get(list_transactions).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/transactions/:id",
            get(get_transaction).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/transactions/:id",
            delete(cancel_transaction).layer(from_fn(middleware::auth::auth_middleware))
        )
        .route(
            "/transactions/:id/pay",
            post(process_payment).layer(from_fn(middleware::auth::auth_middleware))
        );

    // Setup CORS
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(
            std::env::var("ALLOWED_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .parse::<HeaderValue>()
                .unwrap()
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
        ])
        .allow_credentials(true);

    // Buat router dengan middleware
    let app = Router::new()
        // Public routes (no auth required)
        .merge(public_routes)
        // Protected routes (require auth)
        .merge(protected_routes.layer(from_fn(auth::auth_middleware)))
        .with_state(app_state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|_| {
        panic!("Failed to bind to port 3001. Is another instance running?");
    });
    println!("Server running on http://{}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
