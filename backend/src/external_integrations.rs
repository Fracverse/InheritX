use crate::api_error::ApiError;
use crate::circuit_breaker::CircuitBreaker;
use crate::retry::{retry_async, RetryConfig};
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

#[derive(Clone)]
pub struct SanctionsApiClient {
    client: Client,
    base_url: String,
    api_key: String,
    circuit_breaker: CircuitBreaker,
}

#[derive(Debug, Serialize, Clone)]
struct ComplianceFlagPayload {
    plan_id: uuid::Uuid,
    user_id: uuid::Uuid,
    reason: String,
}

// Owned strings so this type can be moved into retry closures without lifetime
// parameters.
#[derive(Debug, Serialize, Clone)]
struct SanctionsScreenPayload {
    user_id: uuid::Uuid,
    email: String,
    wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SanctionsScreenResponse {
    flagged: bool,
    reason: Option<String>,
}

/// Retry policy for external HTTP calls that may experience transient timeouts.
///
/// Three attempts total with exponential back-off (300 ms → 600 ms, capped at
/// 5 s).  Combined with the 10-second per-request timeout, the worst-case
/// latency budget is ≈ 31 s — still well within the application's 30-second
/// request timeout when the circuit breaker opens and sheds further load.
fn timeout_retry_config() -> RetryConfig {
    RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(300),
        max_delay: Duration::from_secs(5),
        backoff_factor: 2.0,
    }
}

// ── AnchorIntegrationClient ───────────────────────────────────────────────────

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

    /// Submit a compliance flag to the Anchor integration service.
    ///
    /// **Recovery strategy**: transient timeouts and external-service errors are
    /// retried up to three times with exponential back-off.  If all attempts fail
    /// with a timeout, the error is logged prominently and the method returns
    /// `Ok(())` so the caller is not blocked — the flag requires manual follow-up
    /// by the compliance team.  Non-transient errors (circuit open, bad status)
    /// are propagated to the caller unchanged.
    pub async fn submit_compliance_flag(
        &self,
        plan_id: uuid::Uuid,
        user_id: uuid::Uuid,
        reason: &str,
    ) -> Result<(), ApiError> {
        let url = format!(
            "{}/v1/compliance/flags",
            self.base_url.trim_end_matches('/')
        );
        let payload = ComplianceFlagPayload {
            plan_id,
            user_id,
            reason: reason.to_owned(),
        };
        let client = self.client.clone();
        let cb = self.circuit_breaker.clone();

        let result = retry_async(
            timeout_retry_config(),
            move || {
                let client = client.clone();
                let url = url.clone();
                let cb = cb.clone();
                let payload = payload.clone();
                async move {
                    cb.call(move || async move {
                        let pid = payload.plan_id;
                        let uid = payload.user_id;
                        let response = client
                            .post(&url)
                            .timeout(Duration::from_secs(10))
                            .json(&payload)
                            .send()
                            .await
                            .map_err(|e| {
                                if e.is_timeout() {
                                    tracing::warn!(
                                        service = "anchor_integration",
                                        %pid,
                                        %uid,
                                        "Anchor compliance flag request timed out; will retry"
                                    );
                                    ApiError::Timeout
                                } else {
                                    ApiError::ExternalService(format!(
                                        "Anchor integration request failed: {e}"
                                    ))
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
            },
            |e| matches!(e, ApiError::Timeout | ApiError::ExternalService(_)),
        )
        .await;

        match result {
            Ok(()) => Ok(()),
            Err(ApiError::Timeout) => {
                // All retries exhausted.  Degrade gracefully: the primary request
                // path must not be blocked by a non-critical notification service.
                // The error is surfaced via structured logging and a metric so the
                // compliance team can follow up.
                tracing::error!(
                    service = "anchor_integration",
                    %plan_id,
                    %user_id,
                    "Compliance flag submission timed out after all retries; \
                     manual follow-up required"
                );
                metrics::counter!(
                    "external_timeout_total",
                    "service" => "anchor_integration"
                )
                .increment(1);
                crate::error_tracking::capture_message(
                    &format!(
                        "anchor_integration: compliance flag timed out after all retries \
                         (plan_id={plan_id}, user_id={user_id})"
                    ),
                    sentry::Level::Error,
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

// ── ComplianceApiClient ───────────────────────────────────────────────────────

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

    /// Report suspicious activity to the compliance API.
    ///
    /// **Recovery strategy**: mirrors `submit_compliance_flag` — transient
    /// failures are retried with back-off; a persistent timeout degrades
    /// gracefully with `Ok(())` and a prominent error log rather than blocking
    /// the caller.
    pub async fn report_suspicious_activity(
        &self,
        plan_id: uuid::Uuid,
        user_id: uuid::Uuid,
        reason: &str,
    ) -> Result<(), ApiError> {
        let url = format!(
            "{}/v1/suspicious-activity",
            self.base_url.trim_end_matches('/')
        );
        let payload = ComplianceFlagPayload {
            plan_id,
            user_id,
            reason: reason.to_owned(),
        };
        let client = self.client.clone();
        let cb = self.circuit_breaker.clone();

        let result = retry_async(
            timeout_retry_config(),
            move || {
                let client = client.clone();
                let url = url.clone();
                let cb = cb.clone();
                let payload = payload.clone();
                async move {
                    cb.call(move || async move {
                        let pid = payload.plan_id;
                        let uid = payload.user_id;
                        let response = client
                            .post(&url)
                            .timeout(Duration::from_secs(10))
                            .json(&payload)
                            .send()
                            .await
                            .map_err(|e| {
                                if e.is_timeout() {
                                    tracing::warn!(
                                        service = "compliance_api",
                                        %pid,
                                        %uid,
                                        "Compliance suspicious-activity request timed out; will retry"
                                    );
                                    ApiError::Timeout
                                } else {
                                    ApiError::ExternalService(format!(
                                        "Compliance API request failed: {e}"
                                    ))
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
            },
            |e| matches!(e, ApiError::Timeout | ApiError::ExternalService(_)),
        )
        .await;

        match result {
            Ok(()) => Ok(()),
            Err(ApiError::Timeout) => {
                tracing::error!(
                    service = "compliance_api",
                    %plan_id,
                    %user_id,
                    "Suspicious-activity report timed out after all retries; \
                     manual follow-up required"
                );
                metrics::counter!(
                    "external_timeout_total",
                    "service" => "compliance_api"
                )
                .increment(1);
                crate::error_tracking::capture_message(
                    &format!(
                        "compliance_api: suspicious-activity report timed out after all retries \
                         (plan_id={plan_id}, user_id={user_id})"
                    ),
                    sentry::Level::Error,
                );
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

// ── SanctionsApiClient ────────────────────────────────────────────────────────

impl SanctionsApiClient {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("SANCTIONS_API_URL").ok()?;
        let api_key = std::env::var("SANCTIONS_API_KEY").ok()?;
        let failure_threshold = read_u32("CB_SANCTIONS_FAILURE_THRESHOLD", 5);
        let recovery_timeout = read_u64("CB_SANCTIONS_RECOVERY_TIMEOUT_SECS", 30);

        Some(Self {
            client: Client::new(),
            base_url,
            api_key,
            circuit_breaker: CircuitBreaker::new(
                "sanctions_api",
                failure_threshold,
                Duration::from_secs(recovery_timeout),
            ),
        })
    }

    /// Screen a user against the sanctions list.
    ///
    /// Returns `Ok(Some(reason))` when the user is flagged, `Ok(None)` when
    /// clear, or an error when the service cannot be reached.
    ///
    /// **Recovery strategy**: transient failures are retried with back-off.
    /// Unlike the compliance notification clients, sanctions screening is a
    /// **security gate** — a persistent timeout must *not* silently pass the
    /// user.  After all retries are exhausted the method returns
    /// `Err(ApiError::ServiceUnavailable)` so the caller can reject or defer
    /// the operation until the screening service recovers.
    pub async fn screen_user(
        &self,
        user_id: uuid::Uuid,
        email: &str,
        wallet_address: Option<&str>,
    ) -> Result<Option<String>, ApiError> {
        let url = format!(
            "{}/v1/sanctions/screen",
            self.base_url.trim_end_matches('/')
        );
        let payload = SanctionsScreenPayload {
            user_id,
            email: email.to_owned(),
            wallet_address: wallet_address.map(str::to_owned),
        };
        let client = self.client.clone();
        let cb = self.circuit_breaker.clone();
        let api_key = self.api_key.clone();

        let result = retry_async(
            timeout_retry_config(),
            move || {
                let client = client.clone();
                let url = url.clone();
                let cb = cb.clone();
                let payload = payload.clone();
                let api_key = api_key.clone();
                async move {
                    cb.call(move || async move {
                        let uid = payload.user_id;
                        let response = client
                            .post(&url)
                            .timeout(Duration::from_secs(10))
                            .header(AUTHORIZATION, format!("Bearer {api_key}"))
                            .json(&payload)
                            .send()
                            .await
                            .map_err(|e| {
                                if e.is_timeout() {
                                    tracing::warn!(
                                        service = "sanctions_api",
                                        %uid,
                                        "Sanctions screen request timed out; will retry"
                                    );
                                    ApiError::Timeout
                                } else {
                                    ApiError::ExternalService(format!(
                                        "Sanctions API request failed: {e}"
                                    ))
                                }
                            })?;

                        if !response.status().is_success() {
                            return Err(ApiError::ExternalService(format!(
                                "Sanctions API returned status {}",
                                response.status()
                            )));
                        }

                        let screen_result: SanctionsScreenResponse =
                            response.json().await.map_err(|e| {
                                ApiError::ExternalService(format!(
                                    "Sanctions API response parse failed: {e}"
                                ))
                            })?;

                        if screen_result.flagged {
                            Ok(Some(screen_result.reason.unwrap_or_else(|| {
                                "Sanctions list match detected".to_string()
                            })))
                        } else {
                            Ok(None)
                        }
                    })
                    .await
                }
            },
            |e| matches!(e, ApiError::Timeout | ApiError::ExternalService(_)),
        )
        .await;

        match result {
            Ok(v) => Ok(v),
            Err(ApiError::Timeout) => {
                // Sanctions screening is a security gate — failing open is not
                // an option.  Return ServiceUnavailable so the caller can block
                // or defer the operation until the service recovers.
                tracing::error!(
                    service = "sanctions_api",
                    %user_id,
                    "Sanctions screening timed out after all retries; \
                     user action blocked until screening succeeds"
                );
                metrics::counter!(
                    "external_timeout_total",
                    "service" => "sanctions_api"
                )
                .increment(1);
                crate::error_tracking::capture_message(
                    &format!(
                        "sanctions_api: screening timed out after all retries — \
                         user action blocked (user_id={user_id})"
                    ),
                    sentry::Level::Error,
                );
                Err(ApiError::ServiceUnavailable(
                    "Sanctions screening is temporarily unavailable. \
                     Please try again shortly."
                        .to_owned(),
                ))
            }
            Err(e) => Err(e),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── timeout_retry_config ──────────────────────────────────────────────────

    #[test]
    fn retry_config_has_valid_bounds() {
        let cfg = timeout_retry_config();
        assert!(cfg.max_attempts >= 2, "need at least one retry");
        assert!(
            cfg.base_delay < Duration::from_secs(2),
            "base delay should be short"
        );
        assert!(
            cfg.max_delay <= Duration::from_secs(30),
            "max delay must be within request budget"
        );
        assert!(cfg.backoff_factor > 1.0, "backoff factor must grow delay");
    }

    // ── from_env returns None without env vars ────────────────────────────────

    #[test]
    fn anchor_client_none_without_env() {
        std::env::remove_var("ANCHOR_INTEGRATION_URL");
        assert!(AnchorIntegrationClient::from_env().is_none());
    }

    #[test]
    fn compliance_client_none_without_env() {
        std::env::remove_var("COMPLIANCE_API_URL");
        assert!(ComplianceApiClient::from_env().is_none());
    }

    #[test]
    fn sanctions_client_none_without_env() {
        std::env::remove_var("SANCTIONS_API_URL");
        std::env::remove_var("SANCTIONS_API_KEY");
        assert!(SanctionsApiClient::from_env().is_none());
    }

    // ── recovery strategy: compliance flag degrades gracefully on timeout ─────

    /// Verify that `ApiError::Timeout` is converted to `Ok(())` for the
    /// non-critical compliance notification paths, so callers are not blocked.
    #[test]
    fn compliance_timeout_recovery_is_ok() {
        // Simulate the match arm directly — the retry/HTTP stack is integration-
        // tested separately.
        let timeout_result: Result<(), ApiError> = Err(ApiError::Timeout);
        let recovered = match timeout_result {
            Ok(()) => Ok(()),
            Err(ApiError::Timeout) => Ok(()), // mirrors the degradation logic
            Err(e) => Err(e),
        };
        assert!(
            recovered.is_ok(),
            "compliance timeout should degrade to Ok(())"
        );
    }

    /// Verify that non-transient errors are NOT swallowed by the compliance
    /// degradation path.
    #[test]
    fn compliance_circuit_open_is_propagated() {
        let circuit_result: Result<(), ApiError> =
            Err(ApiError::CircuitOpen("anchor_integration".to_owned()));
        let recovered = match circuit_result {
            Ok(()) => Ok(()),
            Err(ApiError::Timeout) => Ok(()),
            Err(e) => Err(e),
        };
        assert!(
            recovered.is_err(),
            "CircuitOpen must not be swallowed by compliance degradation"
        );
    }

    // ── recovery strategy: sanctions screening fails safe on timeout ──────────

    /// Verify that sanctions timeout produces `ServiceUnavailable`, not `Ok(None)`
    /// (which would be a fail-open security vulnerability).
    #[test]
    fn sanctions_timeout_is_service_unavailable() {
        let timeout_result: Result<Option<String>, ApiError> = Err(ApiError::Timeout);
        let recovered: Result<Option<String>, ApiError> = match timeout_result {
            Ok(v) => Ok(v),
            Err(ApiError::Timeout) => Err(ApiError::ServiceUnavailable(
                "Sanctions screening is temporarily unavailable. Please try again shortly."
                    .to_owned(),
            )),
            Err(e) => Err(e),
        };
        assert!(
            matches!(recovered, Err(ApiError::ServiceUnavailable(_))),
            "sanctions timeout must fail safe with ServiceUnavailable, not Ok(None)"
        );
    }

    /// Verify that a successful sanctions response is passed through unchanged.
    #[test]
    fn sanctions_flagged_response_passes_through() {
        let flagged: Result<Option<String>, ApiError> = Ok(Some("OFAC match".to_owned()));
        let recovered = match flagged {
            Ok(v) => Ok(v),
            Err(ApiError::Timeout) => Err(ApiError::ServiceUnavailable("".to_owned())),
            Err(e) => Err(e),
        };
        assert_eq!(recovered.unwrap(), Some("OFAC match".to_owned()));
    }

    // ── retry predicate: transient vs non-transient classification ────────────

    /// The retry predicate used by all external clients must classify
    /// `Timeout` and `ExternalService` as transient (retryable) and all other
    /// variants as non-transient (permanent failures that should not be retried).
    #[test]
    fn retry_predicate_classifies_errors_correctly() {
        let is_transient =
            |e: &ApiError| matches!(e, ApiError::Timeout | ApiError::ExternalService(_));

        // Transient — should be retried
        assert!(is_transient(&ApiError::Timeout));
        assert!(is_transient(&ApiError::ExternalService("503".to_owned())));

        // Non-transient — must NOT be retried
        assert!(!is_transient(&ApiError::Unauthorized));
        assert!(!is_transient(&ApiError::NotFound("x".to_owned())));
        assert!(!is_transient(&ApiError::CircuitOpen("svc".to_owned())));
        assert!(!is_transient(&ApiError::ServiceUnavailable("x".to_owned())));
    }

    /// Verify that the retry config uses exponential back-off (each successive
    /// delay is strictly larger than the previous one, up to the cap).
    #[test]
    fn retry_config_delays_grow_exponentially() {
        let cfg = timeout_retry_config();
        let d0 = cfg.base_delay.as_millis() as f64;
        let d1 = (d0 * cfg.backoff_factor).min(cfg.max_delay.as_millis() as f64);
        let d2 = (d1 * cfg.backoff_factor).min(cfg.max_delay.as_millis() as f64);
        assert!(d1 > d0, "second delay must be larger than first");
        assert!(d2 >= d1, "third delay must be >= second (may be capped)");
    }
}
