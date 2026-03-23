use inheritx_backend::{create_app, db, telemetry, Config};
use std::net::SocketAddr;
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
    let price_feed = std::sync::Arc::new(inheritx_backend::DefaultPriceFeedService::new(
        db_pool.clone(),
        3600,
    ));
    if let Err(e) = price_feed.initialize_defaults().await {
        tracing::warn!("Failed to initialize default price feeds: {}", e);
    }

    let risk_engine = std::sync::Arc::new(inheritx_backend::RiskEngine::new(
        db_pool.clone(),
        price_feed,
        rust_decimal::Decimal::new(12, 1), // 1.2 health factor liquidation threshold
    ));
    risk_engine.start();

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
