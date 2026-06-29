use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
use once_cell::sync::Lazy;
use prometheus::{
    histogram_opts, opts, register_gauge, register_histogram_vec, Encoder, Gauge, HistogramVec,
    TextEncoder,
};
use std::time::Instant;

/// Tracks number of in-flight HTTP connections.
pub static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(opts!(
        "inheritx_active_connections",
        "Number of currently active HTTP connections"
    ))
    .expect("failed to register active_connections gauge")
});

/// Per-route request latency histogram (seconds).
/// Labels: method, path, status
pub static REQUEST_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        histogram_opts!(
            "inheritx_http_request_duration_seconds",
            "HTTP request latency in seconds",
            // Buckets tuned for low-latency API; p95/p99 computed by Prometheus from these.
            vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
        ),
        &["method", "path", "status"]
    )
    .expect("failed to register request_latency histogram")
});

/// DB pool size (total connections in the pool).
pub static DB_POOL_SIZE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(opts!(
        "inheritx_db_pool_size",
        "Total connections in the DB pool"
    ))
    .expect("failed to register db_pool_size gauge")
});

/// DB pool idle connections.
pub static DB_POOL_IDLE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(opts!(
        "inheritx_db_pool_idle",
        "Idle connections in the DB pool"
    ))
    .expect("failed to register db_pool_idle gauge")
});

/// Call once at startup to force lazy initialization of all metrics.
pub fn init() {
    Lazy::force(&ACTIVE_CONNECTIONS);
    Lazy::force(&REQUEST_LATENCY);
    Lazy::force(&DB_POOL_SIZE);
    Lazy::force(&DB_POOL_IDLE);
}

/// Updates DB pool gauges from the current sqlx pool state.
pub fn update_db_pool_metrics(pool: &sqlx::PgPool) {
    DB_POOL_SIZE.set(pool.size() as f64);
    DB_POOL_IDLE.set(pool.num_idle() as f64);
}

/// GET /metrics — Prometheus text exposition.
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buf = Vec::new();
    if encoder.encode(&metric_families, &mut buf).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "encoding error").into_response();
    }
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, encoder.format_type())],
        buf,
    )
        .into_response()
}

/// Axum middleware: tracks active connections and records per-route latency.
pub async fn latency_middleware(req: Request, next: Next) -> impl IntoResponse {
    ACTIVE_CONNECTIONS.inc();

    let method = req.method().to_string();
    // Use the matched path pattern (e.g. "/api/plans") to avoid high cardinality.
    let path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());

    let start = Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();
    REQUEST_LATENCY
        .with_label_values(&[&method, &path, &status])
        .observe(elapsed);

    ACTIVE_CONNECTIONS.dec();
    response
}
