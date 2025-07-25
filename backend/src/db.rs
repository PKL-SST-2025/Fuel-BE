use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async fn connect_to_db() -> PgPool {
    let db_url = env::var("DATABASE_URL=postgres://postgres:zelvan08@localhost:5432/fuel_db").expect("DATABASE_URL=postgres://postgres:zelvan08@localhost:5432/fuel_db");
    PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to the database")
}
