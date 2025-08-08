use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Data user test
    let user_id = Uuid::new_v4();
    let email = "testuser@example.com";
    let password = "password123";
    let nama_lengkap = "Test User";
    let no_hp = "081234567890";
    let role = "user";
    
    // Hash password
    let hashed_password = hash(password, DEFAULT_COST)?;
    
    // Insert user ke database
    let result = sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, nama_lengkap, no_hp, role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, email, nama_lengkap
        "#
    )
    .bind(user_id)
    .bind(email)
    .bind(hashed_password)
    .bind(Some(nama_lengkap))
    .bind(Some(no_hp))
    .bind(Some(role))
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_one(&pool)
    .await?;
    
    let id: Uuid = result.get("id");
    let email: String = result.get("email");
    let nama: Option<String> = result.get("nama_lengkap");
    
    println!("User berhasil dibuat!");
    println!("ID: {}", id);
    println!("Email: {}", email);
    println!("Nama: {}", nama.unwrap_or_else(|| "NULL".to_string()));
    
    Ok(())
}
