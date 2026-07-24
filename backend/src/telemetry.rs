use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Output format for console log lines.
///
/// Controlled by the `LOG_FORMAT` environment variable:
/// - `"json"`    – machine-readable JSON (default; ideal for log aggregators)
/// - `"pretty"`  – colourised, human-friendly multi-line output (local dev)
/// - `"compact"` – single-line human-readable without ANSI colours
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
    Compact,
}

impl LogFormat {
    /// Parse from the `LOG_FORMAT` environment variable.
    /// Falls back to [`LogFormat::Json`] for any unrecognised value.
    pub fn from_env() -> Self {
        match std::env::var("LOG_FORMAT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "pretty" => Self::Pretty,
            "compact" => Self::Compact,
            _ => Self::Json,
        }
    }
}

/// Initialise the global `tracing` subscriber.
///
/// The log level is controlled by `RUST_LOG` (defaults to `inheritx_backend=info,info`).
/// The output format is controlled by `LOG_FORMAT` (defaults to `json`).
pub fn init_tracing() -> Result<(), anyhow::Error> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "inheritx_backend=info,info".into());

    let format = LogFormat::from_env();

    match format {
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .init();
        }
    }

    Ok(())
}
