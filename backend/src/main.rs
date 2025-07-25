mod handlers;
mod utils;

use axum::{routing::{post, get, put, delete}, Router};
use crate::handlers::user::{register_user, get_users, get_user_by_id, update_user_by_id, delete_user_by_id, login_user, forgot_password};
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
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
