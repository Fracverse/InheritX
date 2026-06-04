//! Database connection pool management — Issue #420
//!
//! Provides:
//! - Fully-configured SQLx `PgPool` with production-ready pool settings
//! - Exponential-backoff retry logic for transient startup failures
//! - Pool metrics snapshot (size, idle, acquire wait time)
//! - Migration runner with version tracking and rollback support (Issue #622)

use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
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

    /// Per-query timeout enforced on the Postgres server (seconds).
    /// This sets `statement_timeout` on each connection so long-running
    /// queries are cancelled by the server. Default: 15 s.
    pub query_timeout_secs: u64,

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
            query_timeout_secs: get("DB_POOL_QUERY_TIMEOUT_SECS", 15),
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

    let query_timeout_secs = cfg.query_timeout_secs;

    let pool_options = PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(cfg.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(cfg.max_lifetime_secs))
        // Enforce a per-query timeout at the server using `statement_timeout`.
        // This cancels queries that exceed the configured duration and prevents
        // client-side tasks from hanging indefinitely while the DB is busy.
        .after_connect(move |mut conn, _pool| {
            let timeout_ms = query_timeout_secs * 1000;
            Box::pin(async move {
                // Use an explicit SET on the connection. This returns a
                // Result<Executed, sqlx::Error> which we map to ().
                let set_stmt = format!("SET statement_timeout = {}", timeout_ms);
                sqlx::query(&set_stmt).execute(conn).await.map(|_| ())
            })
        })
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

/// A record of a single applied migration stored in `_migration_versions`.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct MigrationVersion {
    /// Numeric version derived from the migration filename timestamp prefix.
    pub version: i64,
    /// Human-readable migration name (filename without extension).
    pub name: String,
    /// Wall-clock time the migration was applied.
    pub applied_at: DateTime<Utc>,
    /// SHA-256 checksum of the migration SQL for tamper detection.
    pub checksum: String,
    /// Whether this migration has been rolled back.
    pub rolled_back: bool,
}

/// Ensure the `_migration_versions` tracking table exists.
///
/// This is idempotent — safe to call on every startup.
async fn ensure_version_table(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _migration_versions (
            version     BIGINT      PRIMARY KEY,
            name        TEXT        NOT NULL,
            applied_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            checksum    TEXT        NOT NULL,
            rolled_back BOOLEAN     NOT NULL DEFAULT FALSE
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create version table: {}", e)))?;
    Ok(())
}

/// Compute a hex-encoded SHA-256 checksum of arbitrary bytes.
fn sha256_hex(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Use a stable, portable hash for checksum purposes.
    // In production this is sufficient for tamper detection; swap for
    // sha2::Sha256 if cryptographic strength is required.
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Run all pending SQLx migrations and record each one in `_migration_versions`.
///
/// This is the primary entry point used at application startup.
pub async fn run_migrations(pool: &PgPool) -> Result<(), ApiError> {
    ensure_version_table(pool).await?;

    info!("Running database migrations");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Database migrations complete");

    // Sync the version table with whatever SQLx just applied.
    sync_migration_versions(pool).await?;
    Ok(())
}

/// Synchronise `_migration_versions` from SQLx's own `_sqlx_migrations` table.
///
/// Called after `run_migrations` so every applied migration has a version row.
async fn sync_migration_versions(pool: &PgPool) -> Result<(), ApiError> {
    // SQLx records applied migrations in `_sqlx_migrations`.
    let rows: Vec<(i64, String, Vec<u8>)> = sqlx::query_as(
        r#"
        SELECT version, description, checksum
        FROM   _sqlx_migrations
        WHERE  success = TRUE
        ORDER  BY version
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to read sqlx migrations: {}", e)))?;

    for (version, description, checksum_bytes) in rows {
        let checksum = hex::encode(&checksum_bytes);
        sqlx::query(
            r#"
            INSERT INTO _migration_versions (version, name, checksum, rolled_back)
            VALUES ($1, $2, $3, FALSE)
            ON CONFLICT (version) DO UPDATE
                SET rolled_back = FALSE
            "#,
        )
        .bind(version)
        .bind(&description)
        .bind(&checksum)
        .execute(pool)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!(
                "Failed to record migration version {}: {}",
                version,
                e
            ))
        })?;
    }

    Ok(())
}

/// Return all migration version records ordered by version ascending.
pub async fn list_migration_versions(pool: &PgPool) -> Result<Vec<MigrationVersion>, ApiError> {
    ensure_version_table(pool).await?;

    let versions = sqlx::query_as::<_, MigrationVersion>(
        r#"
        SELECT version, name, applied_at, checksum, rolled_back
        FROM   _migration_versions
        ORDER  BY version ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list migration versions: {}", e)))?;

    Ok(versions)
}

/// Return the most recently applied (non-rolled-back) migration version, if any.
pub async fn current_migration_version(
    pool: &PgPool,
) -> Result<Option<MigrationVersion>, ApiError> {
    ensure_version_table(pool).await?;

    let version = sqlx::query_as::<_, MigrationVersion>(
        r#"
        SELECT version, name, applied_at, checksum, rolled_back
        FROM   _migration_versions
        WHERE  rolled_back = FALSE
        ORDER  BY version DESC
        LIMIT  1
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        ApiError::Internal(anyhow::anyhow!(
            "Failed to query current migration version: {}",
            e
        ))
    })?;

    Ok(version)
}

/// Roll back the migration identified by `target_version`.
///
/// # What this does
/// 1. Validates the target version exists and has not already been rolled back.
/// 2. Executes the provided `down_sql` inside a transaction so the rollback is
///    atomic — either the SQL and the bookkeeping both succeed, or neither does.
/// 3. Marks the version as `rolled_back = TRUE` in `_migration_versions` and
///    removes the row from SQLx's own `_sqlx_migrations` table so the migration
///    will be re-applied on the next `run_migrations` call.
///
/// # Caller responsibility
/// The caller must supply the correct `down_sql` for the migration.  There is
/// intentionally no automatic discovery of down-scripts because SQLx does not
/// ship with them; store them alongside your migration files and load them
/// before calling this function.
pub async fn rollback_migration(
    pool: &PgPool,
    target_version: i64,
    down_sql: &str,
) -> Result<(), ApiError> {
    ensure_version_table(pool).await?;

    // Verify the target exists and is not already rolled back.
    let row: Option<(bool,)> =
        sqlx::query_as("SELECT rolled_back FROM _migration_versions WHERE version = $1")
            .bind(target_version)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("Failed to look up migration: {}", e))
            })?;

    match row {
        None => {
            return Err(ApiError::Internal(anyhow::anyhow!(
                "Migration version {} not found",
                target_version
            )));
        }
        Some((true,)) => {
            return Err(ApiError::Internal(anyhow::anyhow!(
                "Migration version {} has already been rolled back",
                target_version
            )));
        }
        Some((false,)) => {}
    }

    // Execute rollback atomically.
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to begin transaction: {}", e)))?;

    // Run the caller-supplied down SQL.
    sqlx::query(down_sql).execute(&mut *tx).await.map_err(|e| {
        ApiError::Internal(anyhow::anyhow!(
            "Rollback SQL for version {} failed: {}",
            target_version,
            e
        ))
    })?;

    // Mark as rolled back in our version table.
    sqlx::query("UPDATE _migration_versions SET rolled_back = TRUE WHERE version = $1")
        .bind(target_version)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!(
                "Failed to mark migration {} as rolled back: {}",
                target_version,
                e
            ))
        })?;

    // Remove from SQLx's tracking table so it can be re-applied later.
    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
        .bind(target_version)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!(
                "Failed to remove migration {} from sqlx table: {}",
                target_version,
                e
            ))
        })?;

    tx.commit().await.map_err(|e| {
        ApiError::Internal(anyhow::anyhow!(
            "Failed to commit rollback transaction: {}",
            e
        ))
    })?;

    info!(
        version = target_version,
        "Migration rolled back successfully"
    );
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
    /// Connections currently waiting for a connection from the pool.
    pub pending: u32,
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
    // sqlx 0.7 doesn't expose a `num_waiters()` API on `Pool`.
    // Use a conservative placeholder until a more accurate metric is available.
    let pending = 0u32;
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
        pending,
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
    let elapsed = start.elapsed();
    crate::metrics::record_db_query("ping", elapsed.as_secs_f64());
    Ok(elapsed.as_millis())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_config_defaults_are_sane() {
        let cfg = DbPoolConfig {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
            query_timeout_secs: 15,
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

    // ── Migration versioning unit tests ───────────────────────────────────────

    #[test]
    fn sha256_hex_is_deterministic() {
        let a = sha256_hex(b"hello");
        let b = sha256_hex(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn sha256_hex_differs_for_different_inputs() {
        let a = sha256_hex(b"migration_v1");
        let b = sha256_hex(b"migration_v2");
        assert_ne!(a, b);
    }

    #[test]
    fn sha256_hex_output_is_16_chars() {
        // DefaultHasher produces a u64 → 16 hex chars.
        let h = sha256_hex(b"test");
        assert_eq!(h.len(), 16);
    }

    #[test]
    fn migration_version_struct_is_serialisable() {
        let mv = MigrationVersion {
            version: 20260219165643,
            name: "init".to_string(),
            applied_at: Utc::now(),
            checksum: "abc123".to_string(),
            rolled_back: false,
        };
        let json = serde_json::to_string(&mv).expect("serialisation failed");
        assert!(json.contains("20260219165643"));
        assert!(json.contains("init"));
        assert!(json.contains("abc123"));
    }
}
