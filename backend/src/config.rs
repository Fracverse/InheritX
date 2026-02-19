use crate::api_error::ApiError;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub email: EmailConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
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

        let email = EmailConfig {
            smtp_host: std::env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: std::env::var("SMTP_USERNAME")
                .unwrap_or_else(|_| "".to_string()),
            smtp_password: std::env::var("SMTP_PASSWORD")
                .unwrap_or_else(|_| "".to_string()),
            from_email: std::env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@inheritx.com".to_string()),
        };

        Ok(Config {
            database_url,
            port,
            email,
        })
    }
}
