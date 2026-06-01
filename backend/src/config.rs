use crate::api_error::ApiError;
use serde::Deserialize;

/// Per-endpoint rate-limit settings (requests per second + burst allowance).
#[derive(Debug, Deserialize, Clone)]
pub struct EndpointRateLimit {
    /// Sustained requests per second.
    pub per_second: u64,
    /// Maximum burst above the sustained rate.
    pub burst_size: u32,
}

impl EndpointRateLimit {
    fn new(per_second: u64, burst_size: u32) -> Self {
        Self {
            per_second,
            burst_size,
        }
    }
}

/// Configurable rate-limiting settings loaded from environment variables.
#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    /// Global default applied to all routes not listed below.
    pub default_per_second: u64,
    /// Global default burst size.
    pub default_burst_size: u32,
    /// Limit for emergency-access endpoints (grant/revoke).
    pub emergency_per_second: u64,
    pub emergency_burst_size: u32,
    /// Limit for the admin login endpoint.
    pub admin_login_per_second: u64,
    pub admin_login_burst_size: u32,
    /// Comma-separated token values exempt from rate limiting.
    pub bypass_tokens: Vec<String>,
}

impl RateLimitConfig {
    fn load() -> Self {
        let bypass_raw = std::env::var("RATE_LIMIT_BYPASS_TOKENS").unwrap_or_default();
        let bypass_tokens = bypass_raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Self {
            default_per_second: parse_env("RATE_LIMIT_DEFAULT_PER_SECOND", 2),
            default_burst_size: parse_env("RATE_LIMIT_DEFAULT_BURST_SIZE", 5),
            emergency_per_second: parse_env("RATE_LIMIT_EMERGENCY_PER_SECOND", 1),
            emergency_burst_size: parse_env("RATE_LIMIT_EMERGENCY_BURST_SIZE", 2),
            admin_login_per_second: parse_env("RATE_LIMIT_ADMIN_LOGIN_PER_SECOND", 1),
            admin_login_burst_size: parse_env("RATE_LIMIT_ADMIN_LOGIN_BURST_SIZE", 3),
            bypass_tokens,
        }
    }

    pub fn default_limit(&self) -> EndpointRateLimit {
        EndpointRateLimit::new(self.default_per_second, self.default_burst_size)
    }

    pub fn emergency_limit(&self) -> EndpointRateLimit {
        EndpointRateLimit::new(self.emergency_per_second, self.emergency_burst_size)
    }

    pub fn admin_login_limit(&self) -> EndpointRateLimit {
        EndpointRateLimit::new(self.admin_login_per_second, self.admin_login_burst_size)
    }

    /// Returns a permissive config suitable for unit/integration tests.
    pub fn default_for_tests() -> Self {
        Self {
            default_per_second: 1000,
            default_burst_size: 1000,
            emergency_per_second: 1000,
            emergency_burst_size: 1000,
            admin_login_per_second: 1000,
            admin_login_burst_size: 1000,
            bypass_tokens: Vec::new(),
        }
    }
}

/// Database connection pool settings.
///
/// ## Optimal values by environment
///
/// | Setting                      | Development | Staging | Production |
/// |------------------------------|-------------|---------|------------|
/// | `max_connections`            | 5           | 10      | 20–50      |
/// | `min_connections`            | 1           | 2       | 5          |
/// | `acquire_timeout_secs`       | 30          | 30      | 10         |
/// | `idle_timeout_secs`          | 300         | 600     | 600        |
/// | `max_lifetime_secs`          | 900         | 1800    | 1800       |
/// | `connect_retries`            | 3           | 5       | 5          |
/// | `connect_retry_base_delay_secs` | 1        | 2       | 2          |
///
/// ### Guidance
///
/// - **`max_connections`**: Set to roughly `(number_of_cpu_cores * 2) + effective_spindle_count`.
///   For a typical 4-core production host, 10–20 is a good starting point.
///   Exceeding the PostgreSQL `max_connections` server limit will cause connection
///   errors; leave headroom for admin connections and other services.
///
/// - **`min_connections`**: Keep a small warm pool to avoid cold-start latency on
///   the first requests after idle periods. 2–5 is appropriate for most deployments.
///
/// - **`acquire_timeout_secs`**: How long a request waits for a free connection
///   before failing. Lower values (10 s) in production surface pool exhaustion
///   quickly rather than queuing requests indefinitely.
///
/// - **`idle_timeout_secs`**: Connections idle longer than this are closed.
///   600 s (10 min) balances resource usage against reconnection overhead.
///
/// - **`max_lifetime_secs`**: Maximum age of any connection regardless of activity.
///   1800 s (30 min) prevents stale connections after PostgreSQL restarts or
///   network topology changes.
///
/// - **`connect_retries` / `connect_retry_base_delay_secs`**: Controls startup
///   retry behaviour when the database is not yet reachable (e.g. container
///   orchestration startup ordering). Exponential back-off is applied.
///
/// ### Environment variables
///
/// All settings are overridable at runtime:
///
/// ```text
/// DB_POOL_MAX_CONNECTIONS=20
/// DB_POOL_MIN_CONNECTIONS=5
/// DB_POOL_ACQUIRE_TIMEOUT_SECS=10
/// DB_POOL_IDLE_TIMEOUT_SECS=600
/// DB_POOL_MAX_LIFETIME_SECS=1800
/// DB_POOL_CONNECT_RETRIES=5
/// DB_POOL_CONNECT_RETRY_BASE_DELAY_SECS=2
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct DbPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    /// Per-query timeout enforced on the Postgres server (seconds).
    /// Configures `statement_timeout` for each connection.
    pub query_timeout_secs: u64,
    pub connect_retries: u32,
    pub connect_retry_base_delay_secs: u64,
}

impl DbPoolConfig {
    /// Load pool settings from environment variables, falling back to safe defaults.
    pub fn from_env_or_defaults() -> Self {
        Self::from_env()
    }

    fn from_env() -> Self {
        let get_u64 = |key: &str, default: u64| -> u64 {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        let get_u32 = |key: &str, default: u32| -> u32 {
            std::env::var(key)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };

        Self {
            max_connections: get_u32("DB_POOL_MAX_CONNECTIONS", 10),
            min_connections: get_u32("DB_POOL_MIN_CONNECTIONS", 2),
            acquire_timeout_secs: get_u64("DB_POOL_ACQUIRE_TIMEOUT_SECS", 30),
            idle_timeout_secs: get_u64("DB_POOL_IDLE_TIMEOUT_SECS", 600),
            max_lifetime_secs: get_u64("DB_POOL_MAX_LIFETIME_SECS", 1800),
            query_timeout_secs: get_u64("DB_POOL_QUERY_TIMEOUT_SECS", 15),
            connect_retries: get_u32("DB_POOL_CONNECT_RETRIES", 5),
            connect_retry_base_delay_secs: get_u64("DB_POOL_CONNECT_RETRY_BASE_DELAY_SECS", 2),
        }
    }
}

/// Top-level application configuration loaded from environment variables.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub rate_limit: RateLimitConfig,
    pub db_pool: DbPoolConfig,
}

impl Config {
    pub fn load() -> Result<Self, ApiError> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("DATABASE_URL must be set")))?;

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("PORT must be a valid number")))?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("JWT_SECRET must be set")))?;

        let rate_limit = RateLimitConfig::load();
        let db_pool = DbPoolConfig::from_env();

        Ok(Config {
            database_url,
            port,
            jwt_secret,
            rate_limit,
            db_pool,
        })
    }
}

/// Parse an environment variable as `T`, falling back to `default` on any error.
fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── DbPoolConfig defaults ─────────────────────────────────────────────────

    /// Verify that the default pool settings are within safe operational bounds.
    #[test]
    fn db_pool_defaults_are_sane() {
        // Clear any env overrides that might bleed in from other tests.
        for key in &[
            "DB_POOL_MAX_CONNECTIONS",
            "DB_POOL_MIN_CONNECTIONS",
            "DB_POOL_ACQUIRE_TIMEOUT_SECS",
            "DB_POOL_IDLE_TIMEOUT_SECS",
            "DB_POOL_MAX_LIFETIME_SECS",
            "DB_POOL_QUERY_TIMEOUT_SECS",
            "DB_POOL_CONNECT_RETRIES",
            "DB_POOL_CONNECT_RETRY_BASE_DELAY_SECS",
        ] {
            std::env::remove_var(key);
        }

        let cfg = DbPoolConfig::from_env_or_defaults();

        assert!(
            cfg.max_connections >= 1,
            "max_connections must be at least 1"
        );
        assert!(
            cfg.min_connections <= cfg.max_connections,
            "min_connections must not exceed max_connections"
        );
        assert!(
            cfg.acquire_timeout_secs > 0,
            "acquire_timeout must be positive"
        );
        assert!(cfg.idle_timeout_secs > 0, "idle_timeout must be positive");
        assert!(
            cfg.max_lifetime_secs >= cfg.idle_timeout_secs,
            "max_lifetime should be >= idle_timeout to avoid premature eviction"
        );
        assert!(
            cfg.connect_retries > 0,
            "connect_retries must be at least 1"
        );
        assert!(
            cfg.connect_retry_base_delay_secs > 0,
            "retry base delay must be positive"
        );
    }

    /// Verify that env-var overrides are respected.
    #[test]
    fn db_pool_env_overrides_are_applied() {
        std::env::set_var("DB_POOL_MAX_CONNECTIONS", "42");
        std::env::set_var("DB_POOL_MIN_CONNECTIONS", "7");

        let cfg = DbPoolConfig::from_env_or_defaults();
        assert_eq!(cfg.max_connections, 42);
        assert_eq!(cfg.min_connections, 7);

        std::env::remove_var("DB_POOL_MAX_CONNECTIONS");
        std::env::remove_var("DB_POOL_MIN_CONNECTIONS");
    }

    /// Verify that invalid env-var values fall back to defaults gracefully.
    #[test]
    fn db_pool_invalid_env_falls_back_to_default() {
        std::env::set_var("DB_POOL_MAX_CONNECTIONS", "not_a_number");
        let cfg = DbPoolConfig::from_env_or_defaults();
        // Default is 10; invalid value must not panic and must use the default.
        assert_eq!(cfg.max_connections, 10);
        std::env::remove_var("DB_POOL_MAX_CONNECTIONS");
    }
}
