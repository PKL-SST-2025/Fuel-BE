use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;
    
    println!("\n=== Checking fuel_prices table ===");
    
    // Check if table exists
    let table_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'fuel_prices'
        )
        "#
    )
    .fetch_one(&pool)
    .await?;
    
    if !table_exists {
        println!("Table 'fuel_prices' does not exist!");
        return Ok(());
    }
    
    // Get table structure
    println!("\n=== fuel_prices Table Structure ===");
    let columns = sqlx::query(
        r#"
        SELECT column_name, data_type, is_nullable
        FROM information_schema.columns
        WHERE table_name = 'fuel_prices'
        ORDER BY ordinal_position
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    println!("| {:<20} | {:<20} | {:<10} |", "Column Name", "Data Type", "Nullable");
    println!("|{:-<22}|{:-<22}|{:-<12}|", "", "", "");
    
    for row in columns {
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");
        println!("| {:<20} | {:<20} | {:<10} |", column_name, data_type, is_nullable);
    }
    
    // Get data count
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fuel_prices")
        .fetch_one(&pool)
        .await?;
    
    println!("\nFound {} records in fuel_prices table.", count);
    
    if count > 0 {
        // Get sample data
        println!("\nSample fuel_prices data (first 5 records):");
        let prices = sqlx::query(
            r#"
            SELECT * FROM fuel_prices 
            ORDER BY spbu_id, fuel_type
            LIMIT 5
            "#
        )
        .fetch_all(&pool)
        .await?;
        
        for (i, row) in prices.iter().enumerate() {
            println!("\nFuel Price #{}:", i + 1);
            println!("ID: {}", row.get::<uuid::Uuid, _>("id"));
            println!("SPBU ID: {}", row.get::<uuid::Uuid, _>("spbu_id"));
            println!("Fuel Type: {}", row.get::<String, _>("fuel_type"));
            println!("Price: {}", row.get::<sqlx::types::BigDecimal, _>("price"));
            println!("Created At: {}", row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"));
        }
    }
    
    // Get list of SPBUs with their fuel prices
    println!("\n=== SPBUs and Their Fuel Prices ===");
    let spbus_with_prices = sqlx::query(
        r#"
        SELECT 
            s.id as spbu_id, 
            s.nama as spbu_name,
            fp.fuel_type,
            fp.price
        FROM spbu s
        LEFT JOIN fuel_prices fp ON s.id = fp.spbu_id
        ORDER BY s.nama, fp.fuel_type
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    if spbus_with_prices.is_empty() {
        println!("No SPBUs with fuel prices found!");
    } else {
        for row in spbus_with_prices {
            let spbu_id: uuid::Uuid = row.get("spbu_id");
            let spbu_name: String = row.get("spbu_name");
            let fuel_type: Option<String> = row.get("fuel_type");
            let price: Option<sqlx::types::BigDecimal> = row.get("price");
            
            println!("\nSPBU: {} ({})", spbu_name, spbu_id);
            if let (Some(ft), Some(p)) = (fuel_type, price) {
                println!("  - {}: {}", ft, p);
            } else {
                println!("  No fuel prices available");
            }
        }
    }
    
    Ok(())
}
