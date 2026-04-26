//! Prometheus metrics — Issue #423
//!
//! # Architecture
//!
//! This module uses the [`metrics`] facade crate together with
//! [`metrics_exporter_prometheus`] as the backend.  The facade provides
//! zero-cost `counter!`, `histogram!`, and `gauge!` macros that are
//! decoupled from the exporter, so the backend can be swapped (e.g. to
//! OpenTelemetry) without touching call-sites.
//!
//! # Metric catalogue
//!
//! ## HTTP request metrics
//! | Name | Type | Labels | Description |
//! |---|---|---|---|
//! | `http_requests_total` | Counter | method, path, status | Total HTTP requests |
//! | `http_request_duration_seconds` | Histogram | method, path, status | Request latency |
//! | `http_requests_in_flight` | Gauge | — | Concurrent requests |
//!
//! ## Database pool metrics
//! | Name | Type | Labels | Description |
//! |---|---|---|---|
//! | `db_pool_size` | Gauge | — | Open connections (idle + active) |
//! | `db_pool_idle` | Gauge | — | Idle connections |
//! | `db_pool_active` | Gauge | — | Checked-out connections |
//! | `db_pool_utilisation` | Gauge | — | active / max_connections |
//! | `db_query_duration_seconds` | Histogram | operation | Query round-trip latency |
//!
//! ## Business metrics
//! | Name | Type | Labels | Description |
//! |---|---|---|---|
//! | `plans_created_total` | Counter | — | Plans created |
//! | `plans_claimed_total` | Counter | — | Plans claimed |
//! | `plans_paused_total` | Counter | — | Plans paused by admin |
//! | `loans_created_total` | Counter | — | Loan lifecycle entries created |
//! | `loans_repaid_total` | Counter | — | Loans repaid |
//! | `loans_liquidated_total` | Counter | — | Loans liquidated |
//! | `emergency_access_grants_total` | Counter | — | Emergency access grants issued |
//! | `kyc_submissions_total` | Counter | status | KYC submissions by outcome |
//! | `messages_created_total` | Counter | — | Legacy messages created |
//! | `will_documents_generated_total` | Counter | — | Will PDFs generated |

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

// ── Registry initialisation ───────────────────────────────────────────────────

/// Install the Prometheus recorder as the global `metrics` backend and return
/// a handle that can render the text exposition format on demand.
///
/// Call this **once** at application startup, before `create_app`.
/// Subsequent calls are safe — they return the handle from the already-installed
/// recorder.
pub fn install_recorder() -> PrometheusHandle {
    // Configure histogram buckets appropriate for a web API:
    //   HTTP latency: 1 ms → 10 s
    //   DB query latency: 100 µs → 1 s
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            metrics_exporter_prometheus::Matcher::Prefix("http_request_duration".to_string()),
            &[
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
        .unwrap()
        .set_buckets_for_metric(
            metrics_exporter_prometheus::Matcher::Prefix("db_query_duration".to_string()),
            &[
                0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
            ],
        )
        .unwrap()
        .install_recorder()
        .expect("Failed to install Prometheus metrics recorder")
}

/// Install the recorder exactly once, returning a clone of the global handle.
///
/// Safe to call from multiple tests or from any code path that may run more
/// than once in the same process.  The first call installs the recorder; all
/// subsequent calls return the already-installed handle.
pub fn get_or_install_recorder() -> PrometheusHandle {
    use std::sync::OnceLock;
    static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
    HANDLE.get_or_init(install_recorder).clone()
}

// ── /metrics handler ─────────────────────────────────────────────────────────

/// Axum handler for `GET /metrics`.
///
/// Returns the full Prometheus text exposition (content-type
/// `text/plain; version=0.0.4`), ready to be scraped by a Prometheus server
/// or compatible tool (Grafana Agent, VictoriaMetrics, etc.).
///
/// The endpoint is intentionally **not** behind authentication in this
/// implementation — restrict access at the network / ingress layer instead
/// (e.g. only expose it on an internal port or behind an IP allowlist).
pub async fn metrics_handler(
    axum::extract::Extension(handle): axum::extract::Extension<PrometheusHandle>,
) -> impl IntoResponse {
    let body = handle.render();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

// ── HTTP request metrics middleware ──────────────────────────────────────────

/// Tower-compatible Axum middleware that records per-request metrics.
///
/// Records:
/// - `http_requests_total`          — counter, labelled by method/path/status
/// - `http_request_duration_seconds` — histogram, labelled by method/path/status
/// - `http_requests_in_flight`       — gauge (incremented on entry, decremented on exit)
///
/// The `path` label is the **matched route pattern** (e.g. `/api/plans/:plan_id`)
/// rather than the raw URI, so high-cardinality UUIDs don't explode the label
/// space.  When the matched pattern is unavailable (e.g. 404), the raw path is
/// used but truncated to 128 characters.
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();

    // Prefer the matched route pattern to avoid cardinality explosion.
    let path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|mp| mp.as_str().to_owned())
        .unwrap_or_else(|| {
            let raw = req.uri().path();
            // Truncate very long paths (e.g. accidental large path segments).
            if raw.len() > 128 {
                raw[..128].to_owned()
            } else {
                raw.to_owned()
            }
        });

    // Increment in-flight gauge.
    metrics::gauge!("http_requests_in_flight").increment(1.0);

    let response = next.run(req).await;

    // Decrement in-flight gauge.
    metrics::gauge!("http_requests_in_flight").decrement(1.0);

    let status = response.status().as_u16().to_string();
    let elapsed = start.elapsed().as_secs_f64();

    let labels = [("method", method), ("path", path), ("status", status)];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_request_duration_seconds", &labels).record(elapsed);

    response
}

// ── Database pool metrics updater ─────────────────────────────────────────────

/// Push the current pool snapshot into the Prometheus gauges.
///
/// Call this from a background task (e.g. every 15 s) or from the
/// `/health/db` handler so the gauges stay fresh.
pub fn record_pool_metrics(m: &crate::db::PoolMetrics) {
    metrics::gauge!("db_pool_size").set(m.size as f64);
    metrics::gauge!("db_pool_idle").set(m.idle as f64);
    metrics::gauge!("db_pool_active").set(m.active as f64);
    metrics::gauge!("db_pool_utilisation").set(m.utilisation);
}

/// Record the duration of a database operation.
///
/// `operation` should be a short, low-cardinality label such as
/// `"select_plan"`, `"insert_loan"`, `"ping"`, etc.
pub fn record_db_query(operation: &'static str, elapsed_secs: f64) {
    metrics::histogram!("db_query_duration_seconds", "operation" => operation).record(elapsed_secs);
}

// ── Business metric helpers ───────────────────────────────────────────────────
//
// These thin wrappers keep call-sites clean and ensure consistent label names.
// Import the ones you need in service / handler modules.

/// Increment the plans-created counter.
pub fn inc_plans_created() {
    metrics::counter!("plans_created_total").increment(1);
}

/// Increment the plans-claimed counter.
pub fn inc_plans_claimed() {
    metrics::counter!("plans_claimed_total").increment(1);
}

/// Increment the plans-paused counter.
pub fn inc_plans_paused() {
    metrics::counter!("plans_paused_total").increment(1);
}

/// Increment the loans-created counter.
pub fn inc_loans_created() {
    metrics::counter!("loans_created_total").increment(1);
}

/// Increment the loans-repaid counter.
pub fn inc_loans_repaid() {
    metrics::counter!("loans_repaid_total").increment(1);
}

/// Increment the loans-liquidated counter.
pub fn inc_loans_liquidated() {
    metrics::counter!("loans_liquidated_total").increment(1);
}

/// Increment the emergency-access-grants counter.
pub fn inc_emergency_access_grants() {
    metrics::counter!("emergency_access_grants_total").increment(1);
}

/// Increment the KYC submissions counter.
///
/// `status` should be `"approved"`, `"rejected"`, or `"submitted"`.
pub fn inc_kyc_submissions(status: &str) {
    metrics::counter!("kyc_submissions_total", "status" => status.to_owned()).increment(1);
}

/// Increment the legacy-messages-created counter.
pub fn inc_messages_created() {
    metrics::counter!("messages_created_total").increment(1);
}

/// Increment the will-documents-generated counter.
pub fn inc_will_documents_generated() {
    metrics::counter!("will_documents_generated_total").increment(1);
}

// ── Background pool metrics task ──────────────────────────────────────────────

/// Spawn a background task that refreshes the DB pool gauges every
/// `interval_secs` seconds.
///
/// This ensures the gauges are always up-to-date even when the `/metrics`
/// endpoint is scraped between requests.
pub fn spawn_pool_metrics_task(pool: sqlx::PgPool, interval_secs: u64) {
    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(interval_secs);
        loop {
            let m = crate::db::pool_metrics(&pool);
            record_pool_metrics(&m);
            tokio::time::sleep(interval).await;
        }
    });
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_installs_without_panic() {
        get_or_install_recorder();
    }

    #[test]
    fn business_metric_helpers_do_not_panic() {
        get_or_install_recorder();
        inc_plans_created();
        inc_plans_claimed();
        inc_plans_paused();
        inc_loans_created();
        inc_loans_repaid();
        inc_loans_liquidated();
        inc_emergency_access_grants();
        inc_kyc_submissions("approved");
        inc_messages_created();
        inc_will_documents_generated();
    }

    #[test]
    fn pool_metrics_recording_does_not_panic() {
        get_or_install_recorder();
        let m = crate::db::PoolMetrics {
            size: 3,
            idle: 1,
            active: 2,
            max_connections: 10,
            utilisation: 0.2,
        };
        record_pool_metrics(&m);
    }

    #[test]
    fn db_query_recording_does_not_panic() {
        get_or_install_recorder();
        record_db_query("ping", 0.001);
    }
}
