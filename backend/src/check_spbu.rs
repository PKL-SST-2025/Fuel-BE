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

    println!("Daftar SPBU yang tersedia:");
    let records = sqlx::query!("SELECT id, nama FROM spbu")
        .fetch_all(&pool)
        .await?;

    for record in records {
        println!("ID: {}, Nama: {}", record.id, record.nama);
    }

    Ok(())
}
