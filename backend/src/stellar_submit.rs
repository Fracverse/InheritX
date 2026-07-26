use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StellarSubmitError {
    #[error("failed to reach the Stellar network: {0}")]
    Network(String),
    #[error("Stellar network rejected the transaction")]
    Rejected(Value),
}

/// Thin client for submitting already-validated, signed transaction XDR to
/// a Stellar Horizon instance.
#[derive(Clone)]
pub struct StellarSubmitClient {
    client: Client,
    horizon_url: String,
}

impl StellarSubmitClient {
    pub fn new(horizon_url: String) -> Self {
        Self {
            client: Client::new(),
            horizon_url,
        }
    }

    pub async fn submit(&self, xdr_base64: &str) -> Result<Value, StellarSubmitError> {
        let url = format!("{}/transactions", self.horizon_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .form(&[("tx", xdr_base64)])
            .send()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        let success = response.status().is_success();
        let body: Value = response
            .json()
            .await
            .map_err(|e| StellarSubmitError::Network(e.to_string()))?;

        if success {
            Ok(body)
        } else {
            Err(StellarSubmitError::Rejected(body))
        }
    }

    /// Health-check: probes the Stellar Horizon root endpoint to verify
    /// network reachability of the RPC node.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/", self.horizon_url.trim_end_matches('/'));
        self.client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
