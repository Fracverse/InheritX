use inheritx_backend::{
    create_app, db, error_tracking, metrics, telemetry, Config, LegacyMessageDeliveryService,
    MessageKeyService,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry (tracing subscriber).
    telemetry::init_tracing()?;

    // Initialise Sentry error tracking (Issue #424).
    // The guard MUST be held for the lifetime of the process.
    // Dropping it flushes buffered events and shuts down the client.
    let _sentry_guard = error_tracking::init();

    // Install Prometheus metrics recorder (Issue #423).
    let prometheus_handle = metrics::get_or_install_recorder();

    // Load configuration
    let config = Config::load()?;

    // Initialize database with pool settings from config (Issue #420).
    let db_pool = db::create_pool_with_config(
        &config.database_url,
        &crate::db::DbPoolConfig {
            max_connections: config.db_pool.max_connections,
            min_connections: config.db_pool.min_connections,
            acquire_timeout_secs: config.db_pool.acquire_timeout_secs,
            idle_timeout_secs: config.db_pool.idle_timeout_secs,
            max_lifetime_secs: config.db_pool.max_lifetime_secs,
            connect_retries: config.db_pool.connect_retries,
            connect_retry_base_delay_secs: config.db_pool.connect_retry_base_delay_secs,
        },
    )
    .await?;

    // Run database migrations
    db::run_migrations(&db_pool).await?;

    // Spawn background task that refreshes DB pool gauges every 15 s (Issue #423).
    let pool_metrics_interval: u64 = std::env::var("METRICS_POOL_SCRAPE_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);
    metrics::spawn_pool_metrics_task(db_pool.clone(), pool_metrics_interval);

    // Ensure there is always one active message encryption key.
    MessageKeyService::ensure_active_key(&db_pool).await?;

    // Create application (passes prometheus_handle via Extension).
    let app = create_app(db_pool.clone(), config.clone(), prometheus_handle).await?;

    let compliance_engine = std::sync::Arc::new(inheritx_backend::ComplianceEngine::new(
        db_pool.clone(),
        3,                                     // velocity threshold
        10,                                    // velocity window mins
        rust_decimal::Decimal::new(100000, 0), // $100k volume threshold
    ));
    compliance_engine.start();

    // Initialize Interest Reconciliation Service
    let yield_service = Arc::new(inheritx_backend::DefaultOnChainYieldService::new());
    let interest_reconciliation = Arc::new(inheritx_backend::InterestReconciliationService::new(
        db_pool.clone(),
        yield_service,
        rust_decimal::Decimal::new(1, 2), // 0.01 discrepancy threshold
    ));
    interest_reconciliation.start();

    // Initialize Lending Notification Service
    let lending_notification_service = std::sync::Arc::new(
        inheritx_backend::LendingNotificationService::new(db_pool.clone()),
    );
    lending_notification_service.start();

    // Start legacy message delivery worker.
    let legacy_message_delivery_service =
        Arc::new(LegacyMessageDeliveryService::new(db_pool.clone()));
    legacy_message_delivery_service.start();

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting INHERITX backend server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
