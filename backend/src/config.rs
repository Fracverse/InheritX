use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn};
use zeroize::{Zeroize, Zeroizing};

use crate::stellar_submit::SorobanConfig;

pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub plan_cache_ttl_secs: u64,
    /// TTL for the cached `/api/analytics/plan-statistics` response. Kept
    /// separate from `plan_cache_ttl_secs` since the statistics query
    /// aggregates the whole `plans` table and is far more expensive than a
    /// single plan lookup, so admins tolerate a longer staleness window.
    pub plan_statistics_cache_ttl_secs: u64,
    /// Shared secret used to verify HMAC-SHA256 signatures on inbound KYC
    /// provider webhooks. When unset, `/api/kyc/webhook` rejects every request.
    pub kyc_webhook_secret: Option<String>,
    pub stellar_horizon_url: String,
    pub anchor_api_url: String,
    pub fiat_daily_limit_default: rust_decimal::Decimal,
    /// Soroban contract settings used to execute inheritance payouts on-chain.
    /// `None` when the deployment has not configured a signer, in which case
    /// the inactivity watchdog only updates PostgreSQL.
    pub soroban: Option<SorobanConfig>,
}

/// Dynamically fetches secrets from a HashiCorp Vault / Secret Manager HTTP provider.
/// Sensitive buffers are zeroized upon exit.
pub async fn fetch_vault_secrets(
    vault_addr: &str,
    vault_token: &str,
    secret_path: &str,
) -> Result<HashMap<String, String>, anyhow::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!(
        "{}/v1/{}",
        vault_addr.trim_end_matches('/'),
        secret_path.trim_start_matches('/')
    );

    // Protect the token in memory
    let token_header = Zeroizing::new(vault_token.to_string());

    let response = client
        .get(&url)
        .header("X-Vault-Token", token_header.as_str())
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Vault responded with status {status}: {body}"
        ));
    }

    let mut body_text = Zeroizing::new(response.text().await?);
    let val: serde_json::Value = serde_json::from_str(&body_text)?;

    // Zeroize raw JSON string buffer immediately after parsing
    body_text.zeroize();

    // Support KV v2 (data.data), KV v1 (data), or root key-value object
    let secret_obj = if let Some(kv2) = val
        .get("data")
        .and_then(|d| d.get("data"))
        .and_then(|d| d.as_object())
    {
        kv2
    } else if let Some(kv1) = val.get("data").and_then(|d| d.as_object()) {
        kv1
    } else if let Some(root) = val.as_object() {
        root
    } else {
        return Err(anyhow::anyhow!(
            "Unrecognized Vault secret response structure"
        ));
    };

    let mut secrets = HashMap::new();
    for (k, v) in secret_obj {
        if let Some(s) = v.as_str() {
            secrets.insert(k.clone(), s.to_string());
        } else {
            secrets.insert(k.clone(), v.to_string());
        }
    }

    Ok(secrets)
}

/// Helper function to zeroize sensitive string buffers in memory.
pub fn zeroize_sensitive_buffer(buffer: &mut String) {
    buffer.zeroize();
}

/// Helper to substitute a database password into a connection URL.
pub fn inject_password_into_db_url(url: &str, password: &str) -> String {
    if let Ok(mut parsed) = reqwest::Url::parse(url) {
        let _ = parsed.set_password(Some(password));
        parsed.to_string()
    } else {
        url.to_string()
    }
}

impl Config {
    /// Loads configuration dynamically. When `VAULT_ENABLED=true`, secrets are
    /// fetched from the configured Secret Manager / Vault provider, and sensitive
    /// memory buffers are zeroized after initialization.
    pub async fn load() -> Result<Self, anyhow::Error> {
        let vault_enabled = std::env::var("VAULT_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        let mut vault_secrets = HashMap::new();

        if vault_enabled {
            info!("VAULT_ENABLED=true: Fetching dynamic secrets from Secret Manager / Vault provider...");
            let vault_addr =
                std::env::var("VAULT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:8200".to_string());
            let mut vault_token = Zeroizing::new(std::env::var("VAULT_TOKEN").unwrap_or_default());
            let secret_path = std::env::var("VAULT_SECRET_PATH")
                .unwrap_or_else(|_| "secret/data/inheritx".to_string());

            match fetch_vault_secrets(&vault_addr, &vault_token, &secret_path).await {
                Ok(secrets) => {
                    info!(
                        "Successfully retrieved {} secret(s) dynamically from Vault",
                        secrets.len()
                    );
                    vault_secrets = secrets;
                }
                Err(e) => {
                    warn!("Failed to fetch secrets from Vault: {e}. Falling back to environment variables.");
                }
            }

            // Explicitly zeroize the vault token in memory
            vault_token.zeroize();
        }

        Self::build_config(vault_secrets)
    }

    /// Synchronous loader that reads environment variables directly.
    pub fn load_from_env() -> Result<Self, anyhow::Error> {
        Self::build_config(HashMap::new())
    }

    pub fn build_config(mut vault_secrets: HashMap<String, String>) -> Result<Self, anyhow::Error> {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);

        // Database URL: prioritize Vault, fallback to DATABASE_PASSWORD injection or DATABASE_URL env
        let database_url = vault_secrets
            .remove("DATABASE_URL")
            .or_else(|| {
                if let Some(mut db_pass) = vault_secrets
                    .remove("DATABASE_PASSWORD")
                    .or_else(|| vault_secrets.remove("DB_PASSWORD"))
                {
                    let base = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                        "postgres://postgres:postgres@localhost:5432/inheritx".to_string()
                    });
                    let res = inject_password_into_db_url(&base, &db_pass);
                    db_pass.zeroize();
                    Some(res)
                } else {
                    None
                }
            })
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://postgres:postgres@localhost:5432/inheritx".to_string());

        let redis_url = vault_secrets
            .remove("REDIS_URL")
            .or_else(|| std::env::var("REDIS_URL").ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let plan_cache_ttl_secs = std::env::var("PLAN_CACHE_TTL_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(15);

        let plan_statistics_cache_ttl_secs = std::env::var("PLAN_STATISTICS_CACHE_TTL_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(60)
            .max(1);

        let fiat_daily_limit_default = std::env::var("FIAT_DAILY_LIMIT_DEFAULT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .and_then(Decimal::from_f64)
            .unwrap_or(Decimal::ZERO);

        let kyc_webhook_secret = vault_secrets
            .remove("KYC_WEBHOOK_SECRET")
            .or_else(|| std::env::var("KYC_WEBHOOK_SECRET").ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let stellar_horizon_url = std::env::var("STELLAR_HORIZON_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "https://horizon-testnet.stellar.org".to_string());

        let anchor_api_url = std::env::var("ANCHOR_API_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "http://localhost:8081".to_string());

        // Soroban configuration: use vault values if present, else environment
        let soroban = {
            let rpc_url = vault_secrets
                .remove("SOROBAN_RPC_URL")
                .or_else(|| std::env::var("SOROBAN_RPC_URL").ok())
                .filter(|s| !s.trim().is_empty());

            let contract_id = vault_secrets
                .remove("INHERITANCE_CONTRACT_ID")
                .or_else(|| std::env::var("INHERITANCE_CONTRACT_ID").ok())
                .filter(|s| !s.trim().is_empty());

            let signer_secret = vault_secrets
                .remove("STELLAR_SIGNER_SECRET")
                .or_else(|| std::env::var("STELLAR_SIGNER_SECRET").ok())
                .filter(|s| !s.trim().is_empty());

            if let (Some(rpc_url), Some(contract_id), Some(signer_secret)) =
                (rpc_url, contract_id, signer_secret)
            {
                let network_passphrase = vault_secrets
                    .remove("STELLAR_NETWORK_PASSPHRASE")
                    .or_else(|| std::env::var("STELLAR_NETWORK_PASSPHRASE").ok())
                    .unwrap_or_else(|| "Test SDF Network ; September 2015".to_string());

                let poll_interval = std::env::var("SOROBAN_POLL_INTERVAL_MS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_millis)
                    .unwrap_or_else(|| Duration::from_millis(500));

                let poll_timeout = std::env::var("SOROBAN_POLL_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_secs)
                    .unwrap_or_else(|| Duration::from_secs(30));

                Some(SorobanConfig {
                    rpc_url,
                    contract_id,
                    network_passphrase,
                    signer_secret,
                    poll_interval,
                    poll_timeout,
                })
            } else {
                SorobanConfig::from_env()
            }
        };

        // Zeroize remaining secrets in the map
        for (_k, mut v) in vault_secrets {
            v.zeroize();
        }

        Ok(Config {
            port,
            database_url,
            redis_url,
            plan_cache_ttl_secs,
            plan_statistics_cache_ttl_secs,
            kyc_webhook_secret,
            stellar_horizon_url,
            anchor_api_url,
            fiat_daily_limit_default,
            soroban,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroize_sensitive_buffer() {
        let mut sensitive = "super_secret_private_key_S12345".to_string();
        zeroize_sensitive_buffer(&mut sensitive);
        for byte in sensitive.as_bytes() {
            assert_eq!(*byte, 0);
        }
    }

    #[test]
    fn test_zeroizing_wrapper_clears_memory() {
        let mut buffer = Zeroizing::new("db_password_top_secret".to_string());
        assert_eq!(buffer.as_str(), "db_password_top_secret");
        buffer.zeroize();
        for byte in buffer.as_bytes() {
            assert_eq!(*byte, 0);
        }
    }

    #[test]
    fn test_inject_password_into_db_url() {
        let original = "postgres://postgres:oldpass@localhost:5432/inheritx";
        let updated = inject_password_into_db_url(original, "newSecretPass!123");
        assert!(updated.contains(":newSecretPass!123@"));
    }

    #[test]
    fn test_build_config_from_vault_secrets() {
        let mut vault_secrets = HashMap::new();
        vault_secrets.insert(
            "DATABASE_URL".to_string(),
            "postgres://vaultuser:vaultpass@remote:5432/vaultdb".to_string(),
        );
        vault_secrets.insert(
            "KYC_WEBHOOK_SECRET".to_string(),
            "vault-webhook-secret-999".to_string(),
        );

        let config = Config::build_config(vault_secrets).unwrap();
        assert_eq!(
            config.database_url,
            "postgres://vaultuser:vaultpass@remote:5432/vaultdb"
        );
        assert_eq!(
            config.kyc_webhook_secret.as_deref(),
            Some("vault-webhook-secret-999")
        );
    }
}
