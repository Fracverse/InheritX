//! Error tracking integration — Issue #424
//!
//! Integrates the Sentry SDK for automatic error capture, panic reporting,
//! and request context enrichment.
//!
//! # Setup
//!
//! Call [`init`] once at application startup (before `create_app`).  It reads
//! configuration from environment variables and returns a [`ClientInitGuard`]
//! that **must be kept alive** for the duration of the process — dropping it
//! flushes and shuts down the Sentry client.
//!
//! ```rust,ignore
//! let _sentry = error_tracking::init();
//! ```
//!
//! # What gets captured automatically
//!
//! - **Panics** — via the `panic` feature; full backtrace attached.
//! - **`tracing` ERROR / WARN spans and events** — via `sentry-tracing`; the
//!   `sentry-tracing` layer is installed in [`crate::telemetry::init_tracing`].
//! - **HTTP 5xx responses** — via [`SentryLayer`] in the Axum middleware stack.
//! - **Explicit captures** — call [`capture_error`] or [`capture_message`]
//!   anywhere in the codebase.
//!
//! # Context enrichment
//!
//! The [`enrich_sentry_context`] Axum middleware runs on every request and
//! attaches:
//! - `request_id` (from the `x-request-id` header set by our middleware)
//! - `http.method`, `http.url`
//! - `user.id` (from the `x-user-id` header set by the auth extractor, if present)
//! - `environment` tag (`RUN_ENV` env var, defaults to `"development"`)
//! - `release` tag (`CARGO_PKG_VERSION` baked in at compile time)

use axum::{extract::Request, middleware::Next, response::Response};
use sentry::ClientInitGuard;
use std::borrow::Cow;

// ── Initialisation ────────────────────────────────────────────────────────────

/// Initialise the Sentry client from environment variables.
///
/// Returns a [`ClientInitGuard`] that must be held for the lifetime of the
/// process.  When dropped, Sentry flushes any buffered events and shuts down.
///
/// If `SENTRY_DSN` is not set or is empty, Sentry is initialised in a
/// **no-op** mode — all API calls become cheap no-ops and no data is sent.
/// This makes it safe to run in development without a DSN configured.
pub fn init() -> ClientInitGuard {
    let dsn = std::env::var("SENTRY_DSN").unwrap_or_default();

    let environment: Cow<'static, str> = std::env::var("RUN_ENV")
        .unwrap_or_else(|_| "development".to_string())
        .into();

    // Sample rate: fraction of transactions to send (0.0–1.0).
    // Defaults to 0.1 (10%) to keep volume manageable; set to 1.0 in staging.
    let traces_sample_rate: f32 = std::env::var("SENTRY_TRACES_SAMPLE_RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.1);

    // Error sample rate: fraction of error events to send (0.0–1.0).
    // Defaults to 1.0 — capture all errors.
    let sample_rate: f32 = std::env::var("SENTRY_SAMPLE_RATE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);

    let release = sentry::release_name!();

    if dsn.is_empty() {
        tracing::info!("SENTRY_DSN not set — error tracking disabled");
    } else {
        tracing::info!(
            environment = %environment,
            traces_sample_rate,
            "Initialising Sentry error tracking",
        );
    }

    sentry::init(sentry::ClientOptions {
        dsn: if dsn.is_empty() {
            None
        } else {
            dsn.parse().ok()
        },
        environment: Some(environment),
        release,
        sample_rate,
        traces_sample_rate,
        // Attach stack traces to all events, not just exceptions.
        attach_stacktrace: true,
        // Send default PII (IP address) — disable if not permitted by your
        // privacy policy by setting SENTRY_SEND_DEFAULT_PII=false.
        send_default_pii: std::env::var("SENTRY_SEND_DEFAULT_PII")
            .map(|v| v != "false")
            .unwrap_or(true),
        // Integrations are enabled via Cargo features:
        //   panic    → PanicIntegration
        //   contexts → ContextIntegration (OS, runtime info)
        ..Default::default()
    })
}

// ── Explicit capture helpers ──────────────────────────────────────────────────

/// Capture an error and send it to Sentry.
///
/// Use this for errors that are caught and handled but still warrant
/// visibility in the error tracker (e.g. unexpected database states,
/// third-party API failures).
///
/// ```rust,ignore
/// if let Err(e) = some_fallible_operation() {
///     error_tracking::capture_error(&e);
/// }
/// ```
pub fn capture_error(err: &dyn std::error::Error) {
    sentry::capture_error(err);
}

/// Capture a plain message at the given level.
///
/// Useful for alerting on business-logic anomalies that aren't Rust errors
/// (e.g. "unexpected plan state transition").
pub fn capture_message(msg: &str, level: sentry::Level) {
    sentry::capture_message(msg, level);
}

/// Capture an [`anyhow::Error`] with its full chain.
pub fn capture_anyhow(err: &anyhow::Error) {
    sentry::integrations::anyhow::capture_anyhow(err);
}

// ── Request context middleware ────────────────────────────────────────────────

/// Axum middleware that enriches the Sentry scope for every request.
///
/// Attaches:
/// - `request_id` tag (from `x-request-id` header)
/// - `user.id` (from `x-user-id` header, set by auth extractors)
/// - `http.method` and `http.url` tags
/// - `environment` and `release` are set globally at init time
///
/// Must run **after** `request_id_middleware` so the header is present.
pub async fn enrich_sentry_context(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();

    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    let method = req.method().to_string();
    let url = req.uri().to_string();

    // Configure the Sentry hub scope for this request.
    sentry::configure_scope(|scope| {
        scope.set_tag("request_id", &request_id);
        scope.set_tag("http.method", &method);
        scope.set_tag("http.url", &url);

        if let Some(uid) = &user_id {
            scope.set_user(Some(sentry::User {
                id: Some(uid.clone()),
                ..Default::default()
            }));
        }
    });

    next.run(req).await
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_without_dsn_is_noop() {
        // Without SENTRY_DSN set, init() should succeed silently.
        // The guard is dropped immediately — that's fine in tests.
        std::env::remove_var("SENTRY_DSN");
        let _guard = init();
        // If we reach here without panic, the no-op path works.
    }

    #[test]
    fn capture_helpers_are_noop_without_dsn() {
        std::env::remove_var("SENTRY_DSN");
        let _guard = init();

        // These should all be no-ops when no DSN is configured.
        let err = anyhow::anyhow!("test error");
        capture_anyhow(&err);
        capture_message("test message", sentry::Level::Warning);
    }

    #[test]
    fn sample_rate_defaults_are_valid() {
        let rate: f32 = std::env::var("SENTRY_SAMPLE_RATE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);
        assert!((0.0..=1.0).contains(&rate));

        let traces: f32 = std::env::var("SENTRY_TRACES_SAMPLE_RATE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.1);
        assert!((0.0..=1.0).contains(&traces));
    }
}
