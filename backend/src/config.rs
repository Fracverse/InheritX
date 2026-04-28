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
#[derive(Debug, Deserialize, Clone)]
pub struct DbPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
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
