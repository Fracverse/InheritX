use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inheritx_backend=info,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();
    Ok(())
}
