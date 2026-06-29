use inheritx_backend::{
    create_router, metrics, telemetry, AppState, Config, DbManager, InactivityWatchdogConfig,
    InactivityWatchdogService,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing logging
    telemetry::init_tracing()?;

    // Initialize Prometheus metrics
    metrics::init();

    //loading the .env
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::load()?;
    let plan_cache = inheritx_backend::PlanCache::from_redis_url(
        config.redis_url.as_deref(),
        config.plan_cache_ttl_secs,
    )
    .unwrap_or_else(|error| {
        warn!("Redis cache disabled due to invalid configuration: {error}");
        inheritx_backend::PlanCache::disabled()
    });

    // Attempt to connect to PostgreSQL stub/real
    let db_pool = match DbManager::create_pool(&config.database_url).await {
        Ok(pool) => {
            info!("Successfully connected to PostgreSQL database.");

            if let Err(e) = DbManager::run_migrations(&pool).await {
                warn!("Failed to run database migrations: {:?}", e);
            }

            pool
        }

        Err(e) => {
            error!(
                "Failed to connect to PostgreSQL database ({}): {:?}",
                config.database_url, e
            );

            std::process::exit(1);
        }
    };

    // Initialize state skeleton
    let (kyc_tx, _) = tokio::sync::broadcast::channel(100);
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool: db_pool.clone(),
        kyc_tx,
        kyc_webhook_secret: std::env::var("KYC_WEBHOOK_SECRET").ok(),
        apy_config: inheritx_backend::yield_calculator::ApyConfig::from_env(),
        plan_cache: plan_cache.clone(),
    });

    let inactivity_watchdog = Arc::new(InactivityWatchdogService::new(
        db_pool.clone(),
        plan_cache,
        InactivityWatchdogConfig::from_env(),
    ));
    inactivity_watchdog.start();

    // Periodically refresh DB pool metrics
    {
        let pool = db_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
            loop {
                interval.tick().await;
                metrics::update_db_pool_metrics(&pool);
            }
        });
    }

    // Create Axum application
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting rebranded INHERITX backend skeleton on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
