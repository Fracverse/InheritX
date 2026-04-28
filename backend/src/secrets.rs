/// Secrets management abstraction.
///
/// Provides a `SecretsProvider` trait so the application can load secrets from
/// different backends (environment variables for development, AWS Secrets Manager
/// or HashiCorp Vault for production) without changing call-sites.
///
/// The active backend is selected by the `SECRETS_BACKEND` environment variable:
///   - `env`  (default) – reads from environment / `.env` file
///   - `aws`            – reads from AWS Secrets Manager
///
/// Secret rotation is supported: call `rotate_secret` to update a value in the
/// backing store; the in-process cache is invalidated automatically.
use crate::api_error::ApiError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── Trait ─────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait SecretsProvider: Send + Sync {
    /// Retrieve a secret by name.
    async fn get_secret(&self, name: &str) -> Result<String, ApiError>;

    /// Rotate (update) a secret value in the backing store.
    async fn rotate_secret(&self, name: &str, new_value: &str) -> Result<(), ApiError>;
}

// ── Environment backend ───────────────────────────────────────────────────────

/// Reads secrets from environment variables (or a `.env` file via `dotenvy`).
/// Suitable for local development; not recommended for production.
pub struct EnvSecretsProvider {
    cache: RwLock<HashMap<String, String>>,
}

impl EnvSecretsProvider {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for EnvSecretsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretsProvider for EnvSecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, ApiError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(v) = cache.get(name) {
                return Ok(v.clone());
            }
        }

        let value = std::env::var(name).map_err(|_| {
            ApiError::Internal(anyhow::anyhow!(
                "Secret '{}' not found in environment",
                name
            ))
        })?;

        self.cache
            .write()
            .await
            .insert(name.to_string(), value.clone());
        Ok(value)
    }

    async fn rotate_secret(&self, name: &str, new_value: &str) -> Result<(), ApiError> {
        // For env backend, update the process environment and invalidate cache.
        std::env::set_var(name, new_value);
        self.cache.write().await.remove(name);
        tracing::info!(secret = %name, "secret rotated (env backend)");
        Ok(())
    }
}

// ── AWS Secrets Manager backend ───────────────────────────────────────────────

/// Reads secrets from AWS Secrets Manager.
///
/// Requires the following environment variables to be set:
///   - `AWS_REGION`
///   - Standard AWS credential chain (IAM role, `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`, etc.)
///
/// Values are cached in-process; call `rotate_secret` to invalidate the cache
/// after updating a secret in AWS.
pub struct AwsSecretsProvider {
    region: String,
    cache: RwLock<HashMap<String, String>>,
}

impl AwsSecretsProvider {
    pub fn new(region: impl Into<String>) -> Self {
        Self {
            region: region.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SecretsProvider for AwsSecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, ApiError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(v) = cache.get(name) {
                return Ok(v.clone());
            }
        }

        // Call AWS Secrets Manager via the AWS CLI / SDK HTTP API.
        // We use `reqwest` to avoid pulling in the full AWS SDK as a dependency.
        // Build a minimal signed request using the AWS SDK v4 signing process.
        // In production, use the `aws-sdk-secretsmanager` crate for full support.
        // Here we delegate to the AWS CLI via a subprocess for simplicity and to
        // avoid adding a heavy SDK dependency.
        let _ = format!("https://secretsmanager.{}.amazonaws.com", self.region); // endpoint reference
        let output = tokio::process::Command::new("aws")
            .args([
                "secretsmanager",
                "get-secret-value",
                "--secret-id",
                name,
                "--region",
                &self.region,
                "--query",
                "SecretString",
                "--output",
                "text",
            ])
            .output()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("AWS CLI error: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::Internal(anyhow::anyhow!(
                "Failed to retrieve secret '{}' from AWS: {}",
                name,
                stderr
            )));
        }

        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.cache
            .write()
            .await
            .insert(name.to_string(), value.clone());
        Ok(value)
    }

    async fn rotate_secret(&self, name: &str, new_value: &str) -> Result<(), ApiError> {
        let output = tokio::process::Command::new("aws")
            .args([
                "secretsmanager",
                "put-secret-value",
                "--secret-id",
                name,
                "--secret-string",
                new_value,
                "--region",
                &self.region,
            ])
            .output()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("AWS CLI error: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApiError::Internal(anyhow::anyhow!(
                "Failed to rotate secret '{}' in AWS: {}",
                name,
                stderr
            )));
        }

        // Invalidate cache so the next read fetches the new value.
        self.cache.write().await.remove(name);
        tracing::info!(secret = %name, "secret rotated (AWS Secrets Manager)");
        Ok(())
    }
}

// ── Factory ───────────────────────────────────────────────────────────────────

/// Constructs the appropriate `SecretsProvider` based on the `SECRETS_BACKEND`
/// environment variable (`env` or `aws`).
pub fn build_secrets_provider() -> Arc<dyn SecretsProvider> {
    let backend = std::env::var("SECRETS_BACKEND").unwrap_or_else(|_| "env".to_string());
    match backend.as_str() {
        "aws" => {
            let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
            tracing::info!("Using AWS Secrets Manager backend (region: {})", region);
            Arc::new(AwsSecretsProvider::new(region))
        }
        _ => {
            tracing::info!("Using environment variable secrets backend");
            Arc::new(EnvSecretsProvider::new())
        }
    }
}

// ── Startup validation ────────────────────────────────────────────────────────

/// Required secret names that must be present at startup.
const REQUIRED_SECRETS: &[&str] = &["DATABASE_URL", "JWT_SECRET"];

/// Validates that all required secrets are accessible. Fails fast on startup
/// if any secret is missing, preventing the server from starting in a broken state.
pub async fn validate_required_secrets(provider: &dyn SecretsProvider) -> Result<(), ApiError> {
    for name in REQUIRED_SECRETS {
        provider.get_secret(name).await.map_err(|_| {
            ApiError::Internal(anyhow::anyhow!(
                "Required secret '{}' is not available. Check your secrets configuration.",
                name
            ))
        })?;
    }
    tracing::info!("All required secrets validated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_provider_reads_env_var() {
        std::env::set_var("TEST_SECRET_XYZ", "my-value");
        let provider = EnvSecretsProvider::new();
        let val = provider.get_secret("TEST_SECRET_XYZ").await.unwrap();
        assert_eq!(val, "my-value");
    }

    #[tokio::test]
    async fn test_env_provider_missing_secret_errors() {
        let provider = EnvSecretsProvider::new();
        let result = provider.get_secret("__NONEXISTENT_SECRET__").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_env_provider_rotate_updates_value() {
        std::env::set_var("ROTATE_TEST_SECRET", "old-value");
        let provider = EnvSecretsProvider::new();
        provider
            .rotate_secret("ROTATE_TEST_SECRET", "new-value")
            .await
            .unwrap();
        let val = provider.get_secret("ROTATE_TEST_SECRET").await.unwrap();
        assert_eq!(val, "new-value");
    }

    #[tokio::test]
    async fn test_validate_required_secrets_passes() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("JWT_SECRET", "test-secret");
        let provider = EnvSecretsProvider::new();
        assert!(validate_required_secrets(&provider).await.is_ok());
    }
}
