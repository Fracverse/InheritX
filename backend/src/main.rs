#[cfg(feature = "metrics")]
use inheritx_backend::metrics;
use inheritx_backend::{
    create_router, telemetry, AppState, Config, DbManager, InactivityWatchdogConfig,
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
    #[cfg(feature = "metrics")]
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

    // Connect to PostgreSQL and run migrations
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

    if config.kyc_webhook_secret.is_none() {
        warn!("KYC_WEBHOOK_SECRET is not set — /api/kyc/webhook will reject all requests with 503");
    }

    let (kyc_tx, _) = tokio::sync::broadcast::channel(100);
    // Initialize state
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        db_pool: db_pool.clone(),
        kyc_webhook_secret: config.kyc_webhook_secret.clone(),
        apy_config: inheritx_backend::yield_calculator::ApyConfig::from_env(),
        plan_cache: plan_cache.clone(),
        apy_cache: dashmap::DashMap::new(),
        kyc_tx: kyc_tx.clone(),
        stellar_submit: inheritx_backend::stellar_submit::StellarSubmitClient::new(
            config.stellar_horizon_url.clone(),
        ),
    });

    // Start inactivity watchdog
    let inactivity_watchdog = Arc::new(InactivityWatchdogService::new(
        db_pool.clone(),
        plan_cache,
        InactivityWatchdogConfig::from_env(),
    ));
    inactivity_watchdog.start();

    let webhook_dispatcher = Arc::new(inheritx_backend::WebhookDispatcherService::new(
        db_pool.clone(),
    ));
    webhook_dispatcher.start();

    // Periodically refresh DB pool metrics
    #[cfg(feature = "metrics")]
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
