mod handlers;
mod middleware;
mod models;
mod utils;

use axum::{
    routing::{post, get, put, delete},
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use crate::handlers::user::{register_user, get_users, get_user_by_id, update_user_by_id, delete_user_by_id, login_user, forgot_password};
use crate::handlers::brand::{get_all_brands, create_brands, update_brands, delete_brands};
use crate::handlers::spbu::{get_all_spbu, get_spbu_by_id, create_spbu, update_spbu, delete_spbu};
use crate::handlers::service::{get_all_services, create_service, get_service_by_id, update_service, delete_service};
use crate::handlers::spbu_service::{add_service_to_spbu, remove_service_from_spbu, get_services_by_spbu, get_spbus_by_service};
use crate::handlers::wishlist::{add_to_wishlist, remove_from_wishlist, get_user_wishlists};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let app_state = AppState { db: pool };

    let app = Router::new()
        .route("/register", post(register_user))
        .route("/users", get(get_users))
        .route("/user/:id", get(get_user_by_id))
        .route("/user/:id", put(update_user_by_id))
        .route("/user/:id", delete(delete_user_by_id))
        .route("/login", post(login_user))
        .route("/forgot_password", post(forgot_password))
        .route("/brands", get(get_all_brands).post(create_brands))
        .route("/brands/:id", put(update_brands))
        .route("/brands/:id", delete(delete_brands))
        .route("/spbu", get(get_all_spbu).post(create_spbu))
        .route("/spbu/:id", get(get_spbu_by_id).put(update_spbu).delete(delete_spbu))
        // Service CRUD
        .route("/services", get(get_all_services).post(create_service))
        .route("/services/:id", get(get_service_by_id).put(update_service).delete(delete_service))
        // SPBU-Service relationships
        .route("/spbu/:spbu_id/services", get(get_services_by_spbu).post(add_service_to_spbu))
        .route("/spbu/:spbu_id/services/:service_id", delete(remove_service_from_spbu))
        
        // Wishlist endpoints
        .route("/wishlist", 
            post(add_to_wishlist)
                .get(get_user_wishlists)
        )
        .route("/wishlist/:spbu_id", 
            delete(remove_from_wishlist)
        )
        .route("/services/:service_id/spbus", get(get_spbus_by_service))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
