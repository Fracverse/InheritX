use crate::api_error::ApiError;
use serde::Deserialize;

/// Top-level application configuration loaded from environment variables.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    /// Database connection pool tuning (Issue #420).
    pub db_pool: DbPoolConfig,
}

/// Database connection pool settings.
///
/// All fields are optional in the environment — sensible production defaults
/// are applied when a variable is absent.  See `env.example` for the full
/// list of variable names and their meanings.
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

        // Load pool config from env, falling back to safe defaults.
        let db_pool = DbPoolConfig::from_env();

        Ok(Config {
            database_url,
            port,
            jwt_secret,
            db_pool,
        })
    }
}

impl DbPoolConfig {
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
