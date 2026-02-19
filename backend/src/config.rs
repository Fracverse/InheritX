use crate::api_error::ApiError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
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

        Ok(Config { database_url, port })
    }
}
