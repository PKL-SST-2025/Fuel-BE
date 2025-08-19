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

    println!("Connected to database. Checking fuel prices...\n");

    // Get all fuel prices with SPBU names
    let fuel_prices = sqlx::query!(
        r#"
        SELECT 
            fp.id,
            s.nama as spbu_name,
            fp.fuel_type,
            fp.price,
            fp.created_at
        FROM fuel_prices fp
        JOIN spbu s ON fp.spbu_id = s.id
        ORDER BY s.nama, fp.fuel_type
        "#
    )
    .fetch_all(&pool)
    .await?;

    if fuel_prices.is_empty() {
        println!("No fuel prices found in the database!");
    } else {
        println!("Found {} fuel prices in the database:", fuel_prices.len());
        println!("{:<40} | {:<30} | {:<15} | {}", "SPBU", "Fuel Type", "Price", "Created At");
        println!("{:-<100}", "");
        
        for fp in fuel_prices {
            println!(
                "{:<40} | {:<15} | {:<15.0} | {}",
                fp.spbu_name.unwrap_or_else(|| "Unknown".to_string()), 
                fp.fuel_type.unwrap_or_else(|| "Unknown".to_string()),
                fp.price.unwrap_or_default(),
                fp.created_at.map(|d| d.to_string()).unwrap_or_else(|| "Unknown".to_string())
            );
        }
    }

    Ok(())
}
