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
//! - `user.id` — decoded from the JWT Bearer token payload; falls back to the
//!   `x-user-id` header for legacy call sites
//! - `plan_id` tag — extracted from URI path patterns `/plans/{uuid}` and
//!   `/loans/lifecycle/{uuid}`
//! - `environment` tag (`RUN_ENV` env var, defaults to `"development"`)
//! - `release` tag (`CARGO_PKG_VERSION` baked in at compile time)

use axum::{extract::Request, middleware::Next, response::Response};
use base64::Engine as _;
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

/// Decode the JWT Bearer payload (without signature verification) to extract
/// `user_id`.  This is intentionally unverified — we only need the claim for
/// observability context, not for any authorization decision.
fn extract_user_id_from_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth = headers.get("Authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    // JWT format: base64url(header).base64url(payload).base64url(signature)
    let payload_b64 = token.splitn(3, '.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .ok()?;
    let payload: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    payload.get("user_id")?.as_str().map(str::to_owned)
}

/// Extract a plan/loan UUID from common URI path patterns.
///
/// Recognises `/plans/…` and `/loans/lifecycle/…` sub-trees, then returns the
/// first UUID-shaped path segment that follows the prefix.  Scanning forward
/// handles routes like `/plans/due-for-claim/:plan_id` where a non-UUID
/// segment precedes the actual ID.
fn extract_plan_id_from_path(path: &str) -> Option<String> {
    for prefix in ["/plans/", "/loans/lifecycle/"] {
        if let Some(rest) = path.find(prefix).map(|i| &path[i + prefix.len()..]) {
            for segment in rest.split('/') {
                if uuid::Uuid::parse_str(segment).is_ok() {
                    return Some(segment.to_owned());
                }
            }
        }
    }
    None
}

/// Axum middleware that enriches the Sentry scope for every request.
///
/// Attaches:
/// - `request_id` tag (from `x-request-id` header)
/// - `user.id` — decoded from the JWT Bearer token payload; falls back to the
///   `x-user-id` header used by legacy call sites
/// - `plan_id` tag — extracted from URI path patterns `/plans/{uuid}` and
///   `/loans/lifecycle/{uuid}`
/// - `http.method` and `http.url` tags
/// - `environment` and `release` are set globally at init time
///
/// Must run **after** `request_id_middleware` so the `x-request-id` header is
/// present.
pub async fn enrich_sentry_context(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();

    // Prefer the user_id embedded in the JWT payload; fall back to the
    // x-user-id header set by legacy / test call sites.
    let user_id = extract_user_id_from_bearer(req.headers()).or_else(|| {
        req.headers()
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    });

    let plan_id = extract_plan_id_from_path(req.uri().path());

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

        if let Some(pid) = &plan_id {
            scope.set_tag("plan_id", pid);
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

    // ── extract_user_id_from_bearer ───────────────────────────────────────────

    fn make_bearer_token(user_id: &str) -> String {
        // Build a minimal JWT with a known user_id claim (header.payload.sig).
        // The payload is base64url({"user_id":"<id>","exp":9999999999}).
        use base64::Engine as _;
        let payload = serde_json::json!({ "user_id": user_id, "exp": 9_999_999_999u64 });
        let payload_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        // header and signature can be arbitrary for this test
        format!("eyJhbGciOiJIUzI1NiJ9.{payload_b64}.fakesig")
    }

    #[test]
    fn bearer_extracts_user_id() {
        let uid = "550e8400-e29b-41d4-a716-446655440000";
        let token = make_bearer_token(uid);

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );

        assert_eq!(extract_user_id_from_bearer(&headers).as_deref(), Some(uid));
    }

    #[test]
    fn bearer_returns_none_for_missing_header() {
        let headers = axum::http::HeaderMap::new();
        assert!(extract_user_id_from_bearer(&headers).is_none());
    }

    #[test]
    fn bearer_returns_none_for_malformed_token() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_static("Bearer not.a.jwt"),
        );
        assert!(extract_user_id_from_bearer(&headers).is_none());
    }

    // ── extract_plan_id_from_path ─────────────────────────────────────────────

    #[test]
    fn path_extracts_plan_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(
            extract_plan_id_from_path(&format!("/api/plans/{uuid}/will/generate")),
            Some(uuid.to_owned())
        );
    }

    #[test]
    fn path_extracts_loan_lifecycle_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440001";
        assert_eq!(
            extract_plan_id_from_path(&format!("/api/loans/lifecycle/{uuid}/repay")),
            Some(uuid.to_owned())
        );
    }

    #[test]
    fn path_returns_none_for_unrelated_routes() {
        assert!(extract_plan_id_from_path("/api/health").is_none());
        assert!(extract_plan_id_from_path("/api/notifications").is_none());
    }

    #[test]
    fn path_scans_past_non_uuid_prefix_segments() {
        // /api/plans/due-for-claim/:plan_id — UUID follows a non-UUID segment.
        let uuid = "550e8400-e29b-41d4-a716-446655440002";
        assert_eq!(
            extract_plan_id_from_path(&format!("/api/plans/due-for-claim/{uuid}")),
            Some(uuid.to_owned())
        );
    }

    #[test]
    fn path_ignores_non_uuid_segments() {
        // No UUID anywhere after the prefix → None.
        assert!(extract_plan_id_from_path("/api/plans/not-a-uuid/foo").is_none());
    }
}
