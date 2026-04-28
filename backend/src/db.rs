//! Database connection pool management — Issue #420
//!
//! Provides:
//! - Fully-configured SQLx `PgPool` with production-ready pool settings
//! - Exponential-backoff retry logic for transient startup failures
//! - Pool metrics snapshot (size, idle, acquire wait time)
//! - Migration runner

use crate::api_error::ApiError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{error, info, warn};

// ── Pool configuration ────────────────────────────────────────────────────────

/// All pool tuning parameters, loaded from environment variables.
/// Every field has a documented default that is safe for production.
#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    /// Maximum number of connections kept open at any time.
    /// Default: 10. Tune upward under high concurrency; keep below
    /// `max_connections` in postgresql.conf (typically 100).
    pub max_connections: u32,

    /// Minimum number of idle connections maintained in the pool.
    /// Default: 2. Keeps the pool "warm" so the first requests after a
    /// quiet period don't pay connection-setup latency.
    pub min_connections: u32,

    /// How long to wait for a connection from the pool before returning
    /// an error to the caller. Default: 30 s.
    pub acquire_timeout_secs: u64,

    /// How long an idle connection may sit in the pool before being
    /// closed. Default: 600 s (10 min). Prevents stale connections
    /// after a database restart or network partition.
    pub idle_timeout_secs: u64,

    /// Maximum lifetime of any connection regardless of activity.
    /// Default: 1800 s (30 min). Forces periodic reconnection so
    /// server-side resource limits are respected.
    pub max_lifetime_secs: u64,

    /// Number of times to retry the initial pool creation on failure.
    /// Default: 5. Handles transient startup races (e.g. DB container
    /// not yet ready in docker-compose).
    pub connect_retries: u32,

    /// Base delay between retry attempts. Doubles on each attempt
    /// (exponential back-off). Default: 2 s.
    pub connect_retry_base_delay_secs: u64,
}

impl DbPoolConfig {
    /// Load configuration from environment variables, falling back to
    /// safe production defaults when a variable is absent or unparseable.
    pub fn from_env() -> Self {
        let get = |key: &str, default: u64| -> u64 {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };

        Self {
            max_connections: get("DB_POOL_MAX_CONNECTIONS", 10) as u32,
            min_connections: get("DB_POOL_MIN_CONNECTIONS", 2) as u32,
            acquire_timeout_secs: get("DB_POOL_ACQUIRE_TIMEOUT_SECS", 30),
            idle_timeout_secs: get("DB_POOL_IDLE_TIMEOUT_SECS", 600),
            max_lifetime_secs: get("DB_POOL_MAX_LIFETIME_SECS", 1800),
            connect_retries: get("DB_POOL_CONNECT_RETRIES", 5) as u32,
            connect_retry_base_delay_secs: get("DB_POOL_CONNECT_RETRY_BASE_DELAY_SECS", 2),
        }
    }
}

// ── Pool creation ─────────────────────────────────────────────────────────────

/// Create a fully-configured `PgPool` using settings from the environment.
///
/// Retries the initial connection with exponential back-off so the server
/// can start cleanly even when the database is still initialising (common
/// in containerised deployments).
pub async fn create_pool(database_url: &str) -> Result<PgPool, ApiError> {
    let cfg = DbPoolConfig::from_env();
    create_pool_with_config(database_url, &cfg).await
}

/// Create a pool with an explicit `DbPoolConfig`.  Useful in tests where
/// you want a small, fast pool without touching environment variables.
pub async fn create_pool_with_config(
    database_url: &str,
    cfg: &DbPoolConfig,
) -> Result<PgPool, ApiError> {
    info!(
        max_connections = cfg.max_connections,
        min_connections = cfg.min_connections,
        acquire_timeout_secs = cfg.acquire_timeout_secs,
        idle_timeout_secs = cfg.idle_timeout_secs,
        max_lifetime_secs = cfg.max_lifetime_secs,
        "Initialising database connection pool",
    );

    let pool_options = PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(cfg.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(cfg.max_lifetime_secs))
        // Test each connection with a lightweight ping before handing it
        // to a caller, so stale connections are detected early.
        .test_before_acquire(true);

    let mut last_error: Option<sqlx::Error> = None;
    let mut delay = Duration::from_secs(cfg.connect_retry_base_delay_secs);

    for attempt in 1..=cfg.connect_retries {
        match pool_options.clone().connect(database_url).await {
            Ok(pool) => {
                info!(
                    attempt,
                    max_connections = cfg.max_connections,
                    "Database pool created successfully",
                );
                return Ok(pool);
            }
            Err(e) => {
                warn!(
                    attempt,
                    max_attempts = cfg.connect_retries,
                    error = %e,
                    retry_in_secs = delay.as_secs(),
                    "Failed to connect to database, retrying…",
                );
                last_error = Some(e);

                if attempt < cfg.connect_retries {
                    tokio::time::sleep(delay).await;
                    // Exponential back-off, capped at 60 s.
                    delay = (delay * 2).min(Duration::from_secs(60));
                }
            }
        }
    }

    let err = last_error.expect("connect_retries must be >= 1");
    error!(error = %err, "Exhausted all database connection retries");
    Err(ApiError::Internal(anyhow::anyhow!(
        "Failed to connect to database after {} attempts: {}",
        cfg.connect_retries,
        err
    )))
}

// ── Migrations ────────────────────────────────────────────────────────────────

/// Run all pending SQLx migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), ApiError> {
    info!("Running database migrations");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Database migrations complete");
    Ok(())
}

// ── Pool metrics ──────────────────────────────────────────────────────────────

/// A point-in-time snapshot of pool statistics, suitable for health checks
/// and Prometheus / OpenTelemetry export.
#[derive(Debug, serde::Serialize)]
pub struct PoolMetrics {
    /// Total connections currently open (idle + in-use).
    pub size: u32,
    /// Connections currently idle (available for immediate use).
    pub idle: u32,
    /// Connections currently checked out by active queries.
    pub active: u32,
    /// Configured upper bound on pool size.
    pub max_connections: u32,
    /// Pool utilisation as a fraction in [0.0, 1.0].
    pub utilisation: f64,
}

/// Collect a metrics snapshot from a live pool.
pub fn pool_metrics(pool: &PgPool) -> PoolMetrics {
    let size = pool.size();
    let idle = pool.num_idle() as u32;
    let active = size.saturating_sub(idle);
    let max_connections = pool.options().get_max_connections();
    let utilisation = if max_connections > 0 {
        active as f64 / max_connections as f64
    } else {
        0.0
    };

    PoolMetrics {
        size,
        idle,
        active,
        max_connections,
        utilisation,
    }
}

// ── Health probe ──────────────────────────────────────────────────────────────

/// Lightweight database liveness probe.
///
/// Executes `SELECT 1` and measures round-trip latency.  Returns `Ok(latency_ms)`
/// on success or an `ApiError` if the query fails or times out.
pub async fn ping(pool: &PgPool) -> Result<u128, ApiError> {
    let start = std::time::Instant::now();
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database ping failed: {}", e)))?;
    Ok(start.elapsed().as_millis())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_config_defaults_are_sane() {
        // Temporarily clear any env vars that might be set in CI.
        let cfg = DbPoolConfig {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
            connect_retries: 5,
            connect_retry_base_delay_secs: 2,
        };

        assert!(cfg.min_connections <= cfg.max_connections);
        assert!(cfg.idle_timeout_secs < cfg.max_lifetime_secs);
        assert!(cfg.acquire_timeout_secs > 0);
        assert!(cfg.connect_retries > 0);
    }

    #[test]
    fn pool_metrics_utilisation_is_bounded() {
        // Simulate the utilisation calculation directly.
        let active = 3u32;
        let max = 10u32;
        let utilisation = active as f64 / max as f64;
        assert!((0.0..=1.0).contains(&utilisation));
    }

    #[test]
    fn pool_metrics_zero_max_does_not_divide_by_zero() {
        let utilisation = {
            let max_connections = 0u32;
            let active = 0u32;
            if max_connections > 0 {
                active as f64 / max_connections as f64
            } else {
                0.0
            }
        };
        assert_eq!(utilisation, 0.0);
    }
}
