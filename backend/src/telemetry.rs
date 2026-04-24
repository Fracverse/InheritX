use crate::api_error::ApiError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> Result<(), ApiError> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "inheritx_backend=info,tower_http=info".into());

    if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        let sample_rate = std::env::var("SENTRY_TRACES_SAMPLE_RATE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.1);

        // Leak the guard intentionally to keep Sentry alive for process lifetime.
        let guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: sample_rate,
                ..Default::default()
            },
        ));
        Box::leak(Box::new(guard));
    }

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
            .try_init();
    } else {
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init();
    }

    Ok(())
}
