use crate::api_error::ApiError;
use crate::circuit_breaker::CircuitBreaker;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone)]
pub struct AnchorIntegrationClient {
    client: Client,
    base_url: String,
    circuit_breaker: CircuitBreaker,
}

#[derive(Clone)]
pub struct ComplianceApiClient {
    client: Client,
    base_url: String,
    circuit_breaker: CircuitBreaker,
}

#[derive(Debug, Serialize)]
struct ComplianceFlagPayload {
    plan_id: uuid::Uuid,
    user_id: uuid::Uuid,
    reason: String,
}

impl AnchorIntegrationClient {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("ANCHOR_INTEGRATION_URL").ok()?;
        let failure_threshold = read_u32("CB_ANCHOR_FAILURE_THRESHOLD", 5);
        let recovery_timeout = read_u64("CB_ANCHOR_RECOVERY_TIMEOUT_SECS", 30);

        Some(Self {
            client: Client::new(),
            base_url,
            circuit_breaker: CircuitBreaker::new(
                "anchor_integration",
                failure_threshold,
                Duration::from_secs(recovery_timeout),
            ),
        })
    }

    pub async fn submit_compliance_flag(
        &self,
        plan_id: uuid::Uuid,
        user_id: uuid::Uuid,
        reason: &str,
    ) -> Result<(), ApiError> {
        let url = format!("{}/v1/compliance/flags", self.base_url.trim_end_matches('/'));
        let payload = ComplianceFlagPayload {
            plan_id,
            user_id,
            reason: reason.to_string(),
        };

        self.circuit_breaker
            .call(|| async {
                let response = self
                    .client
                    .post(&url)
                    .timeout(Duration::from_secs(10))
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| {
                        if e.is_timeout() {
                            ApiError::Timeout
                        } else {
                            ApiError::ExternalService(format!("Anchor integration request failed: {e}"))
                        }
                    })?;

                if !response.status().is_success() {
                    return Err(ApiError::ExternalService(format!(
                        "Anchor integration returned status {}",
                        response.status()
                    )));
                }

                Ok(())
            })
            .await
    }
}

impl ComplianceApiClient {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("COMPLIANCE_API_URL").ok()?;
        let failure_threshold = read_u32("CB_COMPLIANCE_FAILURE_THRESHOLD", 5);
        let recovery_timeout = read_u64("CB_COMPLIANCE_RECOVERY_TIMEOUT_SECS", 30);

        Some(Self {
            client: Client::new(),
            base_url,
            circuit_breaker: CircuitBreaker::new(
                "compliance_api",
                failure_threshold,
                Duration::from_secs(recovery_timeout),
            ),
        })
    }

    pub async fn report_suspicious_activity(
        &self,
        plan_id: uuid::Uuid,
        user_id: uuid::Uuid,
        reason: &str,
    ) -> Result<(), ApiError> {
        let url = format!("{}/v1/suspicious-activity", self.base_url.trim_end_matches('/'));
        let payload = ComplianceFlagPayload {
            plan_id,
            user_id,
            reason: reason.to_string(),
        };

        self.circuit_breaker
            .call(|| async {
                let response = self
                    .client
                    .post(&url)
                    .timeout(Duration::from_secs(10))
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| {
                        if e.is_timeout() {
                            ApiError::Timeout
                        } else {
                            ApiError::ExternalService(format!("Compliance API request failed: {e}"))
                        }
                    })?;

                if !response.status().is_success() {
                    return Err(ApiError::ExternalService(format!(
                        "Compliance API returned status {}",
                        response.status()
                    )));
                }

                Ok(())
            })
            .await
    }
}

fn read_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default)
}

fn read_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}
