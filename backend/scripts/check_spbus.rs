use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::env;
use uuid::Uuid;
use dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Get database URL
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // Create database connection pool
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    println!("Successfully connected to database");

    // 1. Display SPBU table structure
    println!("\n=== SPBU Table Structure ===");
    match sqlx::query(
        "SELECT column_name, data_type, is_nullable 
        FROM information_schema.columns 
        WHERE table_name = 'spbu'"
    )
    .fetch_all(&pool)
    .await {
        Ok(columns) => {
            println!("| {:<20} | {:<20} | {:<10} |", "Column Name", "Data Type", "Nullable");
            println!("|{0:-<22}|{0:-<22}|{0:-<12}|", "");
            
            for col in &columns {
                let column_name: String = col.get(0);
                let data_type: String = col.get(1);
                let is_nullable: String = col.get(2);
                
                println!(
                    "| {:<20} | {:<20} | {:<10} |",
                    column_name, data_type, is_nullable
                );
            }
        },
        Err(e) => {
            println!("Error fetching table structure: {}", e);
            println!("This might be because the SPBU table doesn't exist yet.");
            return Ok(());
        }
    }
    
    // 2. Count total SPBUs
    println!("\n=== SPBU Data ===");
    let count_result: Result<i64, _> = sqlx::query_scalar("SELECT COUNT(*) FROM spbu")
        .fetch_one(&pool)
        .await;
    
    match count_result {
        Ok(count) => {
            if count == 0 {
                println!("No SPBUs found in the database.");
                println!("You may need to run the SPBU migrations and seed some test data first.");
                return Ok(());
            }
            println!("Found {} SPBUs in total.", count);
        },
        Err(e) => {
            println!("Error counting SPBUs: {}", e);
            return Ok(());
        }
    }
    
    // 3. Display sample SPBU data
    println!("\nSample SPBU data (first 5 records):");
    match sqlx::query("SELECT id, nama, alamat FROM spbu LIMIT 5")
        .fetch_all(&pool)
        .await {
            Ok(spbus) => {
                for (i, spbu) in spbus.iter().enumerate() {
                    let id: Uuid = spbu.get("id");
                    let nama: String = spbu.get("nama");
                    let alamat: Option<String> = spbu.try_get("alamat").ok();
                    
                    println!("\nSPBU #{}:", i + 1);
                    println!("ID: {}", id);
                    println!("Nama: {}", nama);
                    println!("Alamat: {}", alamat.unwrap_or_else(|| "Not specified".to_string()));
                }
            },
            Err(e) => {
                println!("Error fetching SPBU data: {}", e);
            }
        }
    
    Ok(())
}
