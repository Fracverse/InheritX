//! Retry logic for transient failures.
//!
//! Provides exponential back-off with jitter for operations that can fail
//! transiently (database connection drops, external HTTP calls, etc.).

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuration for a retry policy.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the first).
    pub max_attempts: u32,
    /// Base delay before the first retry.
    pub base_delay: Duration,
    /// Maximum delay between retries (caps the exponential growth).
    pub max_delay: Duration,
    /// Multiplier applied to the delay on each subsequent attempt.
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
        }
    }
}

impl RetryConfig {
    /// Creates a retry config suitable for fast, idempotent database queries.
    pub fn database() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(2),
            backoff_factor: 2.0,
        }
    }

    /// Creates a retry config for external HTTP calls with longer back-off.
    pub fn external_service() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(30),
            backoff_factor: 3.0,
        }
    }

    /// Computes the delay for a given attempt index (0-based).
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exp = self.backoff_factor.powi(attempt as i32);
        let millis = (self.base_delay.as_millis() as f64 * exp) as u64;
        Duration::from_millis(millis).min(self.max_delay)
    }
}

/// Executes an async operation with automatic retry on transient errors.
///
/// `is_transient` receives the error value and must return `true` when the
/// operation should be retried and `false` when the error is permanent.
///
/// # Example
/// ```rust,ignore
/// let result = retry_async(RetryConfig::database(), || async {
///     db_pool.execute("SELECT 1").await.map_err(ApiError::from)
/// }, |e| e.is_transient()).await;
/// ```
pub async fn retry_async<F, Fut, T, E, P>(config: RetryConfig, mut operation: F, is_transient: P) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
    P: Fn(&E) -> bool,
{
    let mut attempt = 0u32;
    loop {
        match operation().await {
            Ok(value) => {
                if attempt > 0 {
                    debug!(attempts = attempt + 1, "Operation succeeded after retries");
                }
                return Ok(value);
            }
            Err(err) => {
                attempt += 1;
                if attempt >= config.max_attempts || !is_transient(&err) {
                    if attempt > 1 {
                        warn!(
                            attempts = attempt,
                            error = ?err,
                            "Operation failed after retries"
                        );
                    }
                    return Err(err);
                }
                let delay = config.delay_for_attempt(attempt - 1);
                warn!(
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    delay_ms = delay.as_millis(),
                    error = ?err,
                    "Transient error, retrying"
                );
                sleep(delay).await;
            }
        }
    }
}
