use crate::api_error::ApiError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialise the global tracing subscriber.
///
/// Layers installed:
/// - `EnvFilter` — respects `RUST_LOG`
/// - `fmt` — structured JSON logs by default (or pretty via `LOG_FORMAT=pretty`)
/// - `sentry_tracing` — forwards ERROR and WARN events to Sentry
pub fn init_tracing() -> Result<(), ApiError> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "inheritx_backend=info,tower_http=info".into());

    let use_json_logs = std::env::var("LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(true);

    if use_json_logs {
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true),
            )
            .with(sentry_tracing::layer())
            .try_init();
    } else {
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .with(sentry_tracing::layer())
            .try_init();
    }

    Ok(())
}
