use inheritx_backend::{create_app, db, telemetry, Config};
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

    // Create application
    let app = create_app(db_pool.clone(), config.clone()).await?;

    // Initialize Price Feed and Risk Engine
    let price_feed = Arc::new(inheritx_backend::DefaultPriceFeedService::new(
        db_pool.clone(),
        3600,
    ));
    if let Err(e) = price_feed.initialize_defaults().await {
        tracing::warn!("Failed to initialize default price feeds: {}", e);
    }

    let risk_engine = Arc::new(inheritx_backend::RiskEngine::new(
        db_pool.clone(),
        price_feed,
        rust_decimal::Decimal::new(12, 1), // 1.2 health factor liquidation threshold
    ));
    risk_engine.start();

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

    // Initialize Emergency Access Background Job (Issue #293)
    inheritx_backend::emergency_access_jobs::EmergencyAccessJobService::start(Arc::new(
        db_pool.clone(),
    ));

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
