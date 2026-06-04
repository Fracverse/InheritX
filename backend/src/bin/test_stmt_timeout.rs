use inheritx_backend::db;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this test");

    // Create pool using env-configured settings (including DB_POOL_QUERY_TIMEOUT_SECS).
    let pool = db::create_pool(&database_url).await?;

    println!("Running long query (pg_sleep(10)) — expect cancellation if timeout < 10s");
    let start = Instant::now();

    match sqlx::query("SELECT pg_sleep(10)").execute(&pool).await {
        Ok(_) => println!("Query completed in {:?} — no timeout", start.elapsed()),
        Err(e) => println!("Query failed after {:?}: {}", start.elapsed(), e),
    }

    Ok(())
}
