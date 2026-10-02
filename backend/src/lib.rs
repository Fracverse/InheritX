pub mod api;
pub mod auth;
pub mod cache;
pub mod config;
pub mod db;
pub mod inactivity_watchdog;

pub mod kyc_webhook;
pub mod loan_lifecycle;
#[cfg(feature = "metrics")]
pub mod metrics;
pub mod middleware;
pub mod password;

#[cfg(feature = "pdf")]
pub mod pdf;

pub mod stellar_anchor;
pub mod stellar_submit;
pub mod telemetry;
pub mod webhooks;
pub mod ws;
pub mod xdr;
pub mod yield_calculator;

pub use api::{create_router, AppState, PlanResponse};
pub use cache::PlanCache;
pub use config::Config;
pub use db::DbManager;
pub use inactivity_watchdog::{InactivityWatchdogConfig, InactivityWatchdogService};
pub use webhooks::WebhookDispatcherService;

pub mod soroban_events;
pub use soroban_events::{SorobanDomainEvent, SorobanEventIndexerService, SorobanEventParser};

/// Async alert notification dispatcher for Telegram and Discord bot webhooks.
pub async fn send_community_alert(endpoint_url: &str, alert_text: &str) -> Result<(), String> {
    if endpoint_url.is_empty() || alert_text.is_empty() {
        return Ok(());
    }
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "text": alert_text,
        "content": alert_text,
    });
    let res = client
        .post(endpoint_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to dispatch alert: {e}"))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Alert webhook returned status: {}", res.status()))
    }
}
