use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

use std::{env, time::Duration};
use tracing::warn;

pub struct DbManager;

impl DbManager {
    pub(crate) fn should_retry_connection_error(error: &str) -> bool {
        let normalized = error.to_lowercase();

        normalized.contains("timeout")
            || normalized.contains("timed out")
            || normalized.contains("connection closed")
            || normalized.contains("connection reset")
            || normalized.contains("connection refused")
            || normalized.contains("broken pipe")
            || normalized.contains("server closed")
            || normalized.contains("temporarily unavailable")
            || normalized.contains("try again")
    }

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

        let max_lifetime: u64 = env::var("DB_MAX_LIFETIME")
            .unwrap_or_else(|_| "1800".to_string())
            .parse()
            .unwrap_or(1800);

        let connect_retries: u32 = env::var("DB_CONNECT_RETRIES")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let connect_retry_delay_secs: u64 = env::var("DB_CONNECT_RETRY_DELAY_SECS")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .unwrap_or(2);

        let connect_options = database_url
            .parse::<PgConnectOptions>()
            .map_err(|error| sqlx::Error::Configuration(error.into()))?;

        let pool_options = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(acquire_timeout))
            .idle_timeout(Duration::from_secs(idle_timeout))
            .max_lifetime(Duration::from_secs(max_lifetime))
            .test_before_acquire(true);

        let mut last_error: Option<sqlx::Error> = None;

        for attempt in 1..=connect_retries {
            match pool_options
                .clone()
                .connect_with(connect_options.clone())
                .await
            {
                Ok(pool) => return Ok(pool),
                Err(error) => {
                    last_error = Some(error);

                    if attempt == connect_retries
                        || !Self::should_retry_connection_error(
                            &last_error.as_ref().unwrap().to_string(),
                        )
                    {
                        return Err(last_error.unwrap());
                    }

                    warn!(
                        attempt,
                        max_retries = connect_retries,
                        error = %last_error.as_ref().unwrap(),
                        "PostgreSQL connection attempt failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_secs(connect_retry_delay_secs)).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            sqlx::Error::Configuration("failed to connect to PostgreSQL".into())
        }))
    }

    /// Runs database migrations
    pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
        let _ = sqlx::query(
            "CREATE OR REPLACE FUNCTION bigint_add_interval(epoch_secs BIGINT, val INTERVAL) \
             RETURNS TIMESTAMP WITH TIME ZONE LANGUAGE sql IMMUTABLE AS $$ \
             SELECT to_timestamp(epoch_secs::double precision) + val; $$;",
        )
        .execute(pool)
        .await;

        let _ = sqlx::query(
            "DO $$ BEGIN \
             CREATE OPERATOR + (LEFTARG = BIGINT, RIGHTARG = INTERVAL, PROCEDURE = bigint_add_interval); \
             EXCEPTION WHEN duplicate_object THEN NULL; END $$;"
        ).execute(pool).await;

        sqlx::migrate!().run(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::DbManager;

    #[test]
    fn retries_transient_connection_errors() {
        assert!(DbManager::should_retry_connection_error(
            "timed out while acquiring a connection"
        ));
        assert!(DbManager::should_retry_connection_error(
            "server closed the connection"
        ));
        assert!(DbManager::should_retry_connection_error(
            "connection refused"
        ));
        assert!(DbManager::should_retry_connection_error(
            "temporary failure, try again later"
        ));
    }

    #[test]
    fn does_not_retry_non_transient_errors() {
        assert!(!DbManager::should_retry_connection_error(
            "permission denied for relation plans"
        ));
        assert!(!DbManager::should_retry_connection_error(
            "invalid password"
        ));
    }
}
