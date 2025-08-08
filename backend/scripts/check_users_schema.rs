use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Dapatkan struktur tabel users
    let columns = sqlx::query(
        "SELECT column_name, data_type, is_nullable 
        FROM information_schema.columns 
        WHERE table_name = 'users'"
    )
    .fetch_all(&pool)
    .await?;

    println!("Struktur tabel users:");
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
    
    // Cek juga isi tabel users
    println!("\nMencoba menampilkan isi tabel users...");
    
    // Coba query sederhana dulu
    match sqlx::query("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await {
            Ok(count_row) => {
                let count: i64 = count_row.get(0);
                println!("Jumlah user dalam database: {}", count);
                
                if count > 0 {
                    println!("\nContoh data user (5 pertama):");
                    let users = sqlx::query("SELECT * FROM users LIMIT 5")
                        .fetch_all(&pool)
                        .await?;
                        
                    for (i, user) in users.iter().enumerate() {
                        println!("\nUser #{}:", i + 1);
                        let id: String = user.get("id");
                        println!("ID: {}", id);
                        
                        // Coba ambil email
                        if let Ok(email) = user.try_get::<Option<String>, _>("email") {
                            println!("Email: {}", email.unwrap_or_else(|| "NULL".to_string()));
                        }
                        
                        // Cek kolom password_hash
                        if let Ok(pass_hash) = user.try_get::<Option<String>, _>("password_hash") {
                            println!("Password Hash: {}...", 
                                pass_hash.unwrap_or_default()
                                    .chars().take(10).collect::<String>());
                        }
                        
                        // Cek kolom lain
                        for col in ["nama_lengkap", "no_hp", "role"] {
                            if let Ok(value) = user.try_get::<Option<String>, _>(col) {
                                println!("{}: {}", col, value.unwrap_or_else(|| "NULL".to_string()));
                            }
                        }
                    }
                }
            },
            Err(e) => {
                println!("Gagal mendapatkan jumlah user: {}", e);
            }
        }
    
    Ok(())
}
