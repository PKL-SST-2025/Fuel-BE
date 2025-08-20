use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Daftar user yang terdaftar:");
    let users = sqlx::query!("SELECT id, email, name FROM users")
        .fetch_all(&pool)
        .await?;

    if users.is_empty() {
        println!("Tidak ada user yang terdaftar!");
    } else {
        for user in users {
            println!("ID: {}, Email: {}, Nama: {}", user.id, user.email, user.name);
        }
    }

    Ok(())
}
