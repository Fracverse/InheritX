use crate::api_error::ApiError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialise the global tracing subscriber.
///
/// Layers installed:
/// - `EnvFilter` — respects `RUST_LOG`; defaults to `inheritx_backend=debug,tower_http=debug`
/// - `fmt` — human-readable log output to stdout
/// - `sentry_tracing` — forwards ERROR and WARN events to Sentry (Issue #424)
pub fn init_tracing() -> Result<(), ApiError> {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inheritx_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        // Forward tracing ERROR/WARN events to Sentry automatically.
        // This is a no-op when no SENTRY_DSN is configured.
        .with(sentry_tracing::layer())
        .try_init();

    Ok(())
}
