#[cfg(feature = "metrics")]
use inheritx_backend::metrics;
use inheritx_backend::{
    create_router, telemetry, AppState, Config, DbManager, InactivityWatchdogConfig,
    InactivityWatchdogService,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
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
    let config = Config::load().await?;
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

    // Build the Stellar client once and share it between the API and the
    // inactivity watchdog, so both talk to the same network and signer.
    let mut stellar_submit = inheritx_backend::stellar_submit::StellarSubmitClient::new(
        config.stellar_horizon_url.clone(),
    );

    match config.soroban.clone() {
        Some(soroban) => {
            let contract_id = soroban.contract_id.clone();
            match stellar_submit.clone().with_soroban(soroban) {
                Ok(client) => {
                    info!(
                        contract_id = %contract_id,
                        "On-chain inheritance triggering enabled"
                    );
                    stellar_submit = client;
                }
                Err(e) => {
                    // Running with a half-configured signer would silently skip
                    // the on-chain payout, so refuse to start instead.
                    error!("Invalid Soroban configuration: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => warn!(
            "SOROBAN_RPC_URL, INHERITANCE_CONTRACT_ID and STELLAR_SIGNER_SECRET are not all set \
             — expired plans will be marked TRIGGERED without executing the on-chain payout"
        ),
    }

    let (kyc_tx, _) = tokio::sync::broadcast::channel(100);
    let (status_tx, _) = tokio::sync::broadcast::channel(100);
    // Initialize state
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new(
            config.anchor_api_url.clone(),
        )),
        db_pool: db_pool.clone(),
        kyc_webhook_secret: config.kyc_webhook_secret.clone(),
        apy_config: inheritx_backend::yield_calculator::ApyConfig::from_env(),
        plan_cache: plan_cache.clone(),
        plan_statistics_cache_ttl_secs: config.plan_statistics_cache_ttl_secs,
        apy_cache: dashmap::DashMap::new(),
        kyc_tx: kyc_tx.clone(),
        status_tx,
        stellar_submit: stellar_submit.clone(),
    });

    // Shutdown channel — all background tasks watch this for cancellation
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut worker_handles = Vec::new();

    // Start inactivity watchdog
    let inactivity_watchdog = Arc::new(
        InactivityWatchdogService::new(
            db_pool.clone(),
            plan_cache,
            InactivityWatchdogConfig::from_env(),
        )
        .with_stellar(stellar_submit),
    );
    worker_handles.push(inactivity_watchdog.start(shutdown_rx.clone()));

    let webhook_dispatcher = Arc::new(inheritx_backend::WebhookDispatcherService::new(
        db_pool.clone(),
    ));
    worker_handles.push(webhook_dispatcher.start(shutdown_rx.clone()));

    // Start daily yield snapshot service (Issue #1130)
    let snapshot_interval = std::time::Duration::from_secs(
        std::env::var("DAILY_YIELD_SNAPSHOT_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(86_400),
    );
    worker_handles.push(inheritx_backend::yield_calculator::DailyYieldSnapshotService::spawn(
        db_pool.clone(),
        snapshot_interval,
        shutdown_rx.clone(),
    ));

    // Start automated loan liquidation watchdog task (Issue #1119)
    worker_handles.push(inheritx_backend::loan_lifecycle::LoanLiquidationWatchdogService::spawn(
        state.clone(),
        inheritx_backend::loan_lifecycle::LoanLiquidationWatchdogConfig::from_env(),
        shutdown_rx.clone(),
    ));

    // Periodically refresh DB pool metrics
    #[cfg(feature = "metrics")]
    {
        let pool = db_pool.clone();
        let mut rx = shutdown_rx.clone();
        worker_handles.push(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        metrics::update_db_pool_metrics(&pool);
                    }
                    _ = rx.changed() => {
                        info!("DB pool metrics task shutting down");
                        break;
                    }
                }
            }
        }));
    }

    // Create Axum application
    let app = create_router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting rebranded INHERITX backend skeleton on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let (shutdown_notify_tx, mut shutdown_notify_rx) = tokio::sync::broadcast::channel::<()>(1);
    let notify_tx = shutdown_notify_tx.clone();

    // Spawn signal listener for Ctrl+C and SIGTERM
    tokio::spawn(async move {
        shutdown_signal().await;
        let _ = notify_tx.send(());
    });

    let server_handle = tokio::spawn({
        let mut rx = shutdown_notify_tx.subscribe();
        async move {
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = rx.recv().await;
                })
                .await
            {
                error!("Axum server stopped with error: {e}");
            }
        }
    });

    // Wait until shutdown signal triggers (Ctrl+C or SIGTERM)
    let _ = shutdown_notify_rx.recv().await;

    // Signal all background workers to initiate clean shutdown
    info!("Broadcasting stop signal to all background tasks...");
    let _ = shutdown_tx.send(true);

    // Allow 15-second grace window for background tasks and in-flight operations to complete cleanly (Issue #1126)
    const GRACE_PERIOD_SECS: u64 = 15;
    info!("Waiting up to {GRACE_PERIOD_SECS} seconds grace window for in-flight tasks and server to exit cleanly...");

    let grace_wait = async {
        // Wait for Axum in-flight HTTP requests to drain
        let _ = server_handle.await;
        // Wait for all background workers to finish in-flight loops
        for handle in worker_handles {
            let _ = handle.await;
        }
    };

    match tokio::time::timeout(std::time::Duration::from_secs(GRACE_PERIOD_SECS), grace_wait).await {
        Ok(_) => {
            info!("All server connections and background workers completed clean exit within grace window");
        }
        Err(_) => {
            warn!("Grace window ({GRACE_PERIOD_SECS}s) expired before background tasks completed; proceeding with exit");
        }
    }

    // Close database connections safely
    db_pool.close().await;
    info!("Database connections closed. Goodbye.");

    Ok(())
}

/// Waits for SIGTERM (Unix) or CTRL+C (Windows/Unix) to initiate graceful shutdown.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Received SIGINT (Ctrl+C), starting graceful shutdown"); }
        _ = terminate => { info!("Received SIGTERM, starting graceful shutdown"); }
    }
}
