use inheritx_backend::{
    create_app, db, telemetry, Config, LegacyMessageDeliveryService, MessageKeyService,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    telemetry::init_tracing()?;

    // Load configuration
    let config = Config::load()?;

    // Initialize database
    let db_pool = db::create_pool(&config.database_url).await?;

    // Run database migrations
    db::run_migrations(&db_pool).await?;

    // Ensure there is always one active message encryption key.
    MessageKeyService::ensure_active_key(&db_pool).await?;

    // Create application
    let app = create_app(db_pool.clone(), config.clone()).await?;

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
