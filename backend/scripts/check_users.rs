use sqlx::postgres::PgPoolOptions;
use std::env;
use std::iter;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database. Checking users...");
    
    let users = sqlx::query!(
        r#"
        SELECT 
            id::text as id,
            email,
            COALESCE(nama_lengkap, 'NULL') as nama_lengkap
        FROM users
        "#
    )
    .fetch_all(&pool)
    .await?;
    
    if users.is_empty() {
        println!("No users found in the database.");
    } else {
        println!("\nFound {} users in the database:", users.len());
        println!("{:<40} | {:<30} | {}", "ID", "Email", "Nama Lengkap");
        println!("{}", std::iter::repeat('-').take(100).collect::<String>());
        
        for user in users {
            println!(
"{:?} | {:?} | {:?}",
                user.id,
                user.email,
                user.nama_lengkap
            );
        }
    }
    
    println!("\nChecking SPBUs for review testing...");
    let spbus = sqlx::query!(
        r#"
        SELECT 
            id::text as id,
            COALESCE(nama, 'NULL') as nama
        FROM spbu 
        LIMIT 5
        "#
    )
    .fetch_all(&pool)
    .await?;
        
    if spbus.is_empty() {
        println!("No SPBUs found in the database.");
    } else {
        println!("\nFound {} SPBUs (showing first 5):", spbus.len());
        println!("{:<40} | {}", "ID", "Nama SPBU");
        println!("{}", iter::repeat('-').take(100).collect::<String>());
            
        for spbu in spbus {
            println!(
"{:?} | {:?}",
                spbu.id,
                spbu.nama
            );
        }
    }
    
    Ok(())
}
