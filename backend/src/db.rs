use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use std::{env, time::Duration};

pub struct DbManager;

impl DbManager {
    /// Creates a PostgreSQL connection pool
    pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
        let max_connections: u32 = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let min_connections: u32 = env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .unwrap_or(2);

        let acquire_timeout: u64 = env::var("DB_ACQUIRE_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let idle_timeout: u64 = env::var("DB_IDLE_TIMEOUT")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .unwrap_or(600);

        PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(acquire_timeout))
            .idle_timeout(Duration::from_secs(idle_timeout))
            .connect(database_url)
            .await
    }

    /// Runs database migrations
    pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!().run(pool).await
    }
}
