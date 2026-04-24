//! A simple thread-safe circuit breaker for protecting external service calls.
//!
//! The circuit transitions through three states:
//! * **Closed** – normal operation; failures are counted.
//! * **Open**   – requests are rejected immediately to shed load.
//! * **HalfOpen** – one probe request is allowed through to test recovery.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use crate::api_error::ApiError;

/// The state of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Shared inner state stored behind an `Arc` so the breaker can be cheaply cloned.
struct Inner {
    /// Name of the guarded service, used in logs and error messages.
    service_name: String,
    /// Consecutive failures needed to trip the circuit.
    failure_threshold: u32,
    /// How long (seconds) to keep the circuit open before probing.
    recovery_timeout_secs: u64,
    /// Consecutive failures since the last reset.
    failure_count: AtomicU32,
    /// Unix timestamp (seconds) of the first failure in the current window;
    /// also used to mark when the circuit was opened.
    opened_at: AtomicU64,
}

/// A simple circuit breaker that prevents cascading failures.
///
/// # Usage
/// ```rust,ignore
/// let cb = CircuitBreaker::new("price-feed", 5, Duration::from_secs(60));
/// let result = cb.call(|| async { fetch_price().await }).await;
/// ```
#[derive(Clone)]
pub struct CircuitBreaker(Arc<Inner>);

impl CircuitBreaker {
    /// Creates a new circuit breaker.
    ///
    /// * `service_name`      – human-readable name shown in logs/errors.
    /// * `failure_threshold` – consecutive failures before the circuit opens.
    /// * `recovery_timeout`  – how long the circuit stays open before trying again.
    pub fn new(service_name: &str, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self(Arc::new(Inner {
            service_name: service_name.to_string(),
            failure_threshold,
            recovery_timeout_secs: recovery_timeout.as_secs(),
            failure_count: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
        }))
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn state(&self) -> CircuitState {
        let failures = self.0.failure_count.load(Ordering::Acquire);
        if failures < self.0.failure_threshold {
            return CircuitState::Closed;
        }
        // Circuit is open – check if recovery timeout has elapsed.
        let opened_at = self.0.opened_at.load(Ordering::Acquire);
        if Self::now_secs().saturating_sub(opened_at) >= self.0.recovery_timeout_secs {
            CircuitState::HalfOpen
        } else {
            CircuitState::Open
        }
    }

    fn on_success(&self) {
        let prev = self.0.failure_count.swap(0, Ordering::Release);
        if prev >= self.0.failure_threshold {
            info!(service = %self.0.service_name, "Circuit breaker closed after successful probe");
        }
        self.0.opened_at.store(0, Ordering::Release);
    }

    fn on_failure(&self) {
        let failures = self.0.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        if failures == self.0.failure_threshold {
            let now = Self::now_secs();
            self.0.opened_at.store(now, Ordering::Release);
            warn!(
                service = %self.0.service_name,
                failure_threshold = self.0.failure_threshold,
                "Circuit breaker opened after consecutive failures"
            );
        }
    }

    /// Executes `operation` if the circuit is not open.
    ///
    /// Returns `Err(ApiError::CircuitOpen)` without calling `operation` when the
    /// circuit is open.  When in `HalfOpen` state the operation is attempted
    /// once; success closes the circuit, failure re-opens it.
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T, ApiError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ApiError>>,
    {
        match self.state() {
            CircuitState::Open => {
                warn!(service = %self.0.service_name, "Circuit breaker is open, rejecting request");
                Err(ApiError::CircuitOpen(self.0.service_name.clone()))
            }
            CircuitState::HalfOpen => {
                info!(service = %self.0.service_name, "Circuit breaker half-open, sending probe");
                match operation().await {
                    Ok(v) => {
                        self.on_success();
                        Ok(v)
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(e)
                    }
                }
            }
            CircuitState::Closed => match operation().await {
                Ok(v) => {
                    self.on_success();
                    Ok(v)
                }
                Err(e) => {
                    if e.is_transient() {
                        self.on_failure();
                    }
                    Err(e)
                }
            },
        }
    }
}
