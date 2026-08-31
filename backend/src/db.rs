use log::LevelFilter;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};

use std::{env, time::Duration};
use tracing::warn;

/// Default SQL `statement_timeout` (ms) applied to every pooled connection.
/// Caps how long any single query — web request or background job — may hold
/// locks before Postgres cancels it, so a runaway query can no longer exhaust
/// the connection pool.
const DEFAULT_STATEMENT_TIMEOUT_MS: u64 = 10_000;

/// Default threshold (ms) above which a query execution is logged as slow.
const DEFAULT_SLOW_QUERY_WARN_MS: u64 = 500;

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

    /// Applies the per-connection query guards: a hard `statement_timeout` so
    /// no single query can hold locks indefinitely, and slow-statement
    /// logging so queries approaching that limit show up before they start
    /// timing out.
    fn apply_query_guards(
        connect_options: PgConnectOptions,
        statement_timeout_ms: u64,
        slow_query_warn_ms: u64,
    ) -> PgConnectOptions {
        connect_options
            // Applied as a session-level GUC on every connection the pool
            // opens, so it covers all web requests as well as background jobs
            // (e.g. the inactivity watchdog) that share this pool.
            .options([("statement_timeout", statement_timeout_ms.to_string())])
            .log_slow_statements(LevelFilter::Warn, Duration::from_millis(slow_query_warn_ms))
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

        let statement_timeout_ms: u64 = env::var("DB_STATEMENT_TIMEOUT_MS")
            .unwrap_or_else(|_| DEFAULT_STATEMENT_TIMEOUT_MS.to_string())
            .parse()
            .unwrap_or(DEFAULT_STATEMENT_TIMEOUT_MS);

        let slow_query_warn_ms: u64 = env::var("DB_SLOW_QUERY_WARN_MS")
            .unwrap_or_else(|_| DEFAULT_SLOW_QUERY_WARN_MS.to_string())
            .parse()
            .unwrap_or(DEFAULT_SLOW_QUERY_WARN_MS);

        let connect_options = database_url
            .parse::<PgConnectOptions>()
            .map_err(|error| sqlx::Error::Configuration(error.into()))?;
        let connect_options =
            Self::apply_query_guards(connect_options, statement_timeout_ms, slow_query_warn_ms);

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
    use super::{DbManager, DEFAULT_SLOW_QUERY_WARN_MS, DEFAULT_STATEMENT_TIMEOUT_MS};
    use sqlx::postgres::PgConnectOptions;

    #[test]
    fn applies_statement_timeout_as_a_session_level_startup_option() {
        let base = "postgres://user:pass@localhost/inheritx"
            .parse::<PgConnectOptions>()
            .expect("valid connection URL");

        let configured = DbManager::apply_query_guards(
            base,
            DEFAULT_STATEMENT_TIMEOUT_MS,
            DEFAULT_SLOW_QUERY_WARN_MS,
        );

        assert_eq!(configured.get_options(), Some("-c statement_timeout=10000"));
    }

    #[test]
    fn honours_a_custom_statement_timeout() {
        let base = "postgres://user:pass@localhost/inheritx"
            .parse::<PgConnectOptions>()
            .expect("valid connection URL");

        let configured = DbManager::apply_query_guards(base, 2_500, DEFAULT_SLOW_QUERY_WARN_MS);

        assert_eq!(configured.get_options(), Some("-c statement_timeout=2500"));
    }

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
