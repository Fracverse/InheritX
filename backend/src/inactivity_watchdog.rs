use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::cache::PlanCache;
use crate::stellar_submit::{
    event_u64_field, find_event, InvocationOutcome, StellarSubmitClient, StellarSubmitError,
};

const DEFAULT_INTERVAL_SECS: u64 = 60 * 60;
const DEFAULT_BATCH_SIZE: i64 = 500;
const WATCHDOG_LOCK_KEY: i64 = 820;

const ACTIVE_STATUS: &str = "ACTIVE";
/// A plan whose on-chain trigger transaction is in flight. Nothing is paid out
/// from this state; it exists so a crashed worker's work can be picked up
/// again without re-triggering plans that already succeeded.
const TRIGGERING_STATUS: &str = "TRIGGERING";
const TRIGGERED_STATUS: &str = "TRIGGERED";
/// The chain refused the trigger, or we could not reach it. Needs an operator.
const TRIGGER_FAILED_STATUS: &str = "TRIGGER_FAILED";

/// Topics of the `InheritanceTriggeredEvent` the inheritance contract publishes
/// from `trigger_inheritance`.
const TRIGGER_EVENT_TOPICS: [&str; 2] = ["INHERIT", "TRIGGER"];

const DEFAULT_MAX_ATTEMPTS: u32 = 3;
const DEFAULT_BACKOFF_MS: u64 = 1_000;
const DEFAULT_MAX_BACKOFF_MS: u64 = 30_000;
const DEFAULT_STALE_AFTER_SECS: u64 = 900;

#[derive(Debug, Clone, Copy)]
pub struct InactivityWatchdogConfig {
    pub interval: Duration,
    pub batch_size: i64,
    pub on_chain: OnChainTriggerConfig,
}

/// Retry policy for the on-chain half of a sweep.
#[derive(Debug, Clone, Copy)]
pub struct OnChainTriggerConfig {
    /// Total submission attempts per plan, per sweep.
    pub max_attempts: u32,
    /// Delay before the second attempt; doubles up to `max_backoff`.
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    /// How long a plan may sit in `TRIGGERING` before another sweep re-claims
    /// it. This is the crash-recovery window, so it should comfortably exceed
    /// the Soroban poll timeout.
    pub stale_after: Duration,
}

impl OnChainTriggerConfig {
    pub fn from_env() -> Self {
        Self {
            max_attempts: parse_env_u64(
                "INACTIVITY_WATCHDOG_ONCHAIN_MAX_ATTEMPTS",
                u64::from(DEFAULT_MAX_ATTEMPTS),
            )
            .clamp(1, u64::from(u32::MAX)) as u32,
            initial_backoff: Duration::from_millis(
                parse_env_u64("INACTIVITY_WATCHDOG_ONCHAIN_BACKOFF_MS", DEFAULT_BACKOFF_MS).max(1),
            ),
            max_backoff: Duration::from_millis(
                parse_env_u64(
                    "INACTIVITY_WATCHDOG_ONCHAIN_MAX_BACKOFF_MS",
                    DEFAULT_MAX_BACKOFF_MS,
                )
                .max(1),
            ),
            stale_after: Duration::from_secs(
                parse_env_u64(
                    "INACTIVITY_WATCHDOG_TRIGGER_STALE_AFTER_SECS",
                    DEFAULT_STALE_AFTER_SECS,
                )
                .max(1),
            ),
        }
    }

    /// Backoff to wait before `attempt` (1-indexed), capped at `max_backoff`.
    fn backoff_for(&self, attempt: u32) -> Duration {
        let factor = 1u32
            .checked_shl(attempt.saturating_sub(1))
            .unwrap_or(u32::MAX);
        self.initial_backoff
            .saturating_mul(factor)
            .min(self.max_backoff)
    }
}

impl InactivityWatchdogConfig {
    pub fn from_env() -> Self {
        let interval_secs =
            parse_env_u64("INACTIVITY_WATCHDOG_INTERVAL_SECS", DEFAULT_INTERVAL_SECS);
        let batch_size = parse_env_i64("INACTIVITY_WATCHDOG_BATCH_SIZE", DEFAULT_BATCH_SIZE).max(1);

        Self {
            interval: Duration::from_secs(interval_secs.max(1)),
            batch_size,
            on_chain: OnChainTriggerConfig::from_env(),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ExpiredPlan {
    id: Uuid,
    owner_address: String,
    inactivity_deadline_at: DateTime<Utc>,
    /// Identifier of this plan inside the Soroban inheritance contract. Plans
    /// created before the contract integration have none and cannot be
    /// triggered on-chain.
    onchain_plan_id: Option<i64>,
}

/// Why a plan could not be triggered on-chain, together with how many
/// submission attempts it cost.
#[derive(Debug)]
struct TriggerFailure {
    reason: String,
    attempts: u32,
}

pub struct InactivityWatchdogService {
    db: PgPool,
    plan_cache: PlanCache,
    config: InactivityWatchdogConfig,
    /// `None` when the deployment has no Soroban signer configured; the sweep
    /// then falls back to updating PostgreSQL only.
    stellar: Option<StellarSubmitClient>,
}

impl InactivityWatchdogService {
    pub fn new(db: PgPool, plan_cache: PlanCache, config: InactivityWatchdogConfig) -> Self {
        Self {
            db,
            plan_cache,
            config,
            stellar: None,
        }
    }

    /// Wires the watchdog to the Stellar client so expired plans unlock funds
    /// on-chain before their PostgreSQL status changes. A client without
    /// Soroban configuration is ignored, with a loud warning — silently
    /// skipping the payout would be worse than saying so.
    pub fn with_stellar(mut self, client: StellarSubmitClient) -> Self {
        if client.soroban_enabled() {
            info!(
                contract_id = client.contract_id().unwrap_or_default(),
                source_account = client.source_account().unwrap_or_default(),
                "Inactivity watchdog will execute inheritance payouts on-chain"
            );
            self.stellar = Some(client);
        } else {
            warn!(
                "Inactivity watchdog has no Soroban configuration — expired plans will be marked \
                 TRIGGERED in PostgreSQL without unlocking funds on-chain"
            );
        }
        self
    }

    pub fn start(self: Arc<Self>, mut shutdown_rx: watch::Receiver<bool>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.config.interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                tokio::select! {
                    _ = interval.tick() => {}
                    _ = shutdown_rx.changed() => {
                        info!("Inactivity watchdog shutting down");
                        return;
                    }
                }

                match self.run_once().await {
                    Ok(count) if count > 0 => {
                        info!("Inactivity watchdog marked {count} plan(s) as triggered");
                    }
                    Ok(_) => {}
                    Err(e) => error!("Inactivity watchdog sweep failed: {e}"),
                }
            }
        });
    }

    /// Runs a single sweep and returns how many plans ended up `TRIGGERED`.
    ///
    /// With a Soroban client configured the sweep is two-phase: expired plans
    /// are first claimed into `TRIGGERING` under an advisory lock, then each is
    /// triggered on-chain outside the transaction. Only a transaction that
    /// lands and emits the contract's `INHERIT/TRIGGER` event promotes a plan
    /// to `TRIGGERED`.
    pub async fn run_once(&self) -> Result<usize, sqlx::Error> {
        let on_chain = self.stellar.is_some();
        let claimed = self.claim_expired_plans(on_chain).await?;

        if claimed.is_empty() {
            return Ok(0);
        }

        let Some(client) = self.stellar.as_ref() else {
            // No signer configured: the claim already wrote TRIGGERED.
            for plan in &claimed {
                self.announce_triggered(plan, None).await;
            }
            return Ok(claimed.len());
        };

        let mut triggered = 0usize;
        for plan in &claimed {
            match self.trigger_on_chain(client, plan).await {
                Ok((tx_hash, attempts)) => {
                    self.mark_triggered(plan, &tx_hash, attempts).await?;
                    self.announce_triggered(plan, Some(&tx_hash)).await;
                    triggered += 1;
                }
                Err(failure) => {
                    self.mark_trigger_failed(plan, &failure).await?;
                    self.announce_trigger_failure(plan, &failure).await;
                }
            }
        }

        Ok(triggered)
    }

    /// Atomically claims a batch of expired plans, moving them out of `ACTIVE`
    /// so a concurrent worker cannot pick them up as well.
    async fn claim_expired_plans(&self, on_chain: bool) -> Result<Vec<ExpiredPlan>, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let lock_acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(WATCHDOG_LOCK_KEY)
            .fetch_one(&mut *tx)
            .await?;

        if !lock_acquired {
            warn!("Inactivity watchdog lock is held by another worker; skipping sweep");
            tx.commit().await?;
            return Ok(Vec::new());
        }

        let claim_status = if on_chain {
            TRIGGERING_STATUS
        } else {
            TRIGGERED_STATUS
        };

        let expired_plans = sqlx::query_as::<_, ExpiredPlan>(
            r#"
            UPDATE plans
            SET status = $1,
                trigger_started_at = NOW()
            WHERE id IN (
                SELECT p.id
                FROM plans p
                WHERE COALESCE(p.is_active, true) = true
                  AND p.last_ping IS NOT NULL
                  AND p.inactivity_deadline_at <= NOW()
                  AND (
                        COALESCE(p.status, 'ACTIVE') = $4
                        OR (
                             $5::boolean
                             AND p.status = $6
                             AND (
                                   p.trigger_started_at IS NULL
                                   OR p.trigger_started_at
                                      <= NOW() - make_interval(secs => $3::double precision)
                                 )
                           )
                      )
                ORDER BY p.inactivity_deadline_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, owner_address, inactivity_deadline_at, onchain_plan_id
            "#,
        )
        .bind(claim_status)
        .bind(self.config.batch_size)
        .bind(self.config.on_chain.stale_after.as_secs_f64())
        .bind(ACTIVE_STATUS)
        .bind(on_chain)
        .bind(TRIGGERING_STATUS)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(expired_plans)
    }

    /// Submits `trigger_inheritance` for one plan, retrying transient failures
    /// with exponential backoff. Returns the transaction hash and the number of
    /// attempts it took.
    async fn trigger_on_chain(
        &self,
        client: &StellarSubmitClient,
        plan: &ExpiredPlan,
    ) -> Result<(String, u32), TriggerFailure> {
        let Some(onchain_plan_id) = plan.onchain_plan_id else {
            return Err(TriggerFailure {
                reason: "plan has no onchain_plan_id; it cannot be triggered on-chain".to_string(),
                attempts: 0,
            });
        };

        let onchain_plan_id = u64::try_from(onchain_plan_id).map_err(|_| TriggerFailure {
            reason: format!("onchain_plan_id {onchain_plan_id} is not a valid contract plan id"),
            attempts: 0,
        })?;

        let retry = self.config.on_chain;
        let mut last_error = String::new();

        for attempt in 1..=retry.max_attempts {
            match client.trigger_inheritance(onchain_plan_id).await {
                Ok(outcome) => {
                    return match verify_trigger_event(client, &outcome, onchain_plan_id) {
                        // The chain accepted the call, so retrying cannot help
                        // whether or not the event was where we expected it.
                        Ok(()) => Ok((outcome.tx_hash, attempt)),
                        Err(reason) => Err(TriggerFailure {
                            reason,
                            attempts: attempt,
                        }),
                    };
                }
                Err(error) => {
                    last_error = error.to_string();

                    if !is_retryable(&error) {
                        return Err(TriggerFailure {
                            reason: last_error,
                            attempts: attempt,
                        });
                    }

                    if attempt < retry.max_attempts {
                        let backoff = retry.backoff_for(attempt);
                        warn!(
                            plan_id = %plan.id,
                            onchain_plan_id,
                            attempt,
                            backoff_ms = backoff.as_millis(),
                            error = %last_error,
                            "On-chain inheritance trigger failed; retrying"
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(TriggerFailure {
            reason: last_error,
            attempts: retry.max_attempts,
        })
    }

    async fn mark_triggered(
        &self,
        plan: &ExpiredPlan,
        tx_hash: &str,
        attempts: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE plans
            SET status = $2,
                trigger_tx_hash = $3,
                trigger_attempts = COALESCE(trigger_attempts, 0) + $4,
                last_trigger_error = NULL
            WHERE id = $1
            "#,
        )
        .bind(plan.id)
        .bind(TRIGGERED_STATUS)
        .bind(tx_hash)
        .bind(attempts as i32)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn mark_trigger_failed(
        &self,
        plan: &ExpiredPlan,
        failure: &TriggerFailure,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE plans
            SET status = $2,
                trigger_attempts = COALESCE(trigger_attempts, 0) + $3,
                last_trigger_error = $4
            WHERE id = $1
            "#,
        )
        .bind(plan.id)
        .bind(TRIGGER_FAILED_STATUS)
        .bind(failure.attempts as i32)
        .bind(&failure.reason)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Invalidates cached plan queries and publishes `plan.triggered`.
    async fn announce_triggered(&self, plan: &ExpiredPlan, tx_hash: Option<&str>) {
        // Only a real submission counts towards the on-chain series; the
        // no-signer fallback never touched the chain.
        #[cfg(feature = "metrics")]
        if tx_hash.is_some() {
            crate::metrics::WATCHDOG_ONCHAIN_TRIGGERS
                .with_label_values(&["success"])
                .inc();
        }

        let beneficiary_addresses: Vec<String> = match sqlx::query_scalar(
            r#"
            SELECT wallet_address
            FROM beneficiaries
            WHERE plan_id = $1
            "#,
        )
        .bind(plan.id)
        .fetch_all(&self.db)
        .await
        {
            Ok(addresses) => addresses,
            Err(err) => {
                warn!(
                    plan_id = %plan.id,
                    error = %err,
                    "Failed to load beneficiaries for triggered plan"
                );
                Vec::new()
            }
        };

        if let Err(err) = self
            .plan_cache
            .invalidate_queries(&plan.owner_address, &beneficiary_addresses)
            .await
        {
            warn!(
                plan_id = %plan.id,
                error = %err,
                "Failed to invalidate Redis plan cache for triggered plan"
            );
        }

        warn!(
            plan_id = %plan.id,
            inactivity_deadline_at = %plan.inactivity_deadline_at,
            trigger_tx_hash = tx_hash.unwrap_or("none"),
            "Plan marked triggered by inactivity watchdog"
        );

        let payload = serde_json::json!({
            "plan_id": plan.id,
            "owner_address": plan.owner_address,
            "inactivity_deadline_at": plan.inactivity_deadline_at,
            "onchain_plan_id": plan.onchain_plan_id,
            "trigger_tx_hash": tx_hash,
        });

        if let Err(e) =
            crate::WebhookDispatcherService::enqueue_event(&self.db, "plan.triggered", &payload)
                .await
        {
            warn!("Failed to enqueue webhook for plan.triggered: {:?}", e);
        }
    }

    /// Alerting path: a plan reached its deadline but the funds were not
    /// unlocked. Operators need to know immediately.
    async fn announce_trigger_failure(&self, plan: &ExpiredPlan, failure: &TriggerFailure) {
        #[cfg(feature = "metrics")]
        crate::metrics::WATCHDOG_ONCHAIN_TRIGGERS
            .with_label_values(&["failure"])
            .inc();

        error!(
            plan_id = %plan.id,
            onchain_plan_id = plan.onchain_plan_id.unwrap_or_default(),
            owner_address = %plan.owner_address,
            inactivity_deadline_at = %plan.inactivity_deadline_at,
            attempts = failure.attempts,
            error = %failure.reason,
            "On-chain inheritance trigger failed; plan needs manual intervention"
        );

        let payload = serde_json::json!({
            "plan_id": plan.id,
            "owner_address": plan.owner_address,
            "inactivity_deadline_at": plan.inactivity_deadline_at,
            "onchain_plan_id": plan.onchain_plan_id,
            "attempts": failure.attempts,
            "error": failure.reason,
        });

        if let Err(e) = crate::WebhookDispatcherService::enqueue_event(
            &self.db,
            "plan.trigger_failed",
            &payload,
        )
        .await
        {
            warn!("Failed to enqueue webhook for plan.trigger_failed: {:?}", e);
        }
    }
}

/// Confirms the contract really triggered *this* plan, by looking for the
/// `INHERIT/TRIGGER` event it publishes and checking the plan id it carries.
/// A transaction that succeeded without that event did not do what we asked.
fn verify_trigger_event(
    client: &StellarSubmitClient,
    outcome: &InvocationOutcome,
    expected_plan_id: u64,
) -> Result<(), String> {
    let contract = client
        .contract()
        .ok_or_else(|| "Soroban contract is not configured".to_string())?;

    let event = find_event(&outcome.events, contract, &TRIGGER_EVENT_TOPICS).ok_or_else(|| {
        format!(
            "transaction {} succeeded but emitted no INHERIT/TRIGGER event",
            outcome.tx_hash
        )
    })?;

    match event_u64_field(event, "plan_id") {
        Some(plan_id) if plan_id == expected_plan_id => Ok(()),
        Some(plan_id) => Err(format!(
            "transaction {} triggered plan {plan_id}, expected {expected_plan_id}",
            outcome.tx_hash
        )),
        None => Err(format!(
            "transaction {} emitted an INHERIT/TRIGGER event without a plan_id",
            outcome.tx_hash
        )),
    }
}

/// Whether another attempt could plausibly succeed. A rejection by the network
/// or the contract will be rejected identically next time, so only transport
/// and inclusion failures are worth retrying.
fn is_retryable(error: &StellarSubmitError) -> bool {
    matches!(
        error,
        StellarSubmitError::Network(_)
            | StellarSubmitError::Rpc(_)
            | StellarSubmitError::Timeout { .. }
    )
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_env_i64(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stellar_submit::{SorobanConfig, TESTNET_PASSPHRASE};
    use std::sync::{Mutex, OnceLock};
    use stellar_xdr::{
        ContractEvent, ContractEventBody, ContractEventType, ContractEventV0, ContractId,
        ExtensionPoint, Hash, ScMap, ScMapEntry, ScSymbol, ScVal,
    };

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    const ON_CHAIN_ENV: [&str; 4] = [
        "INACTIVITY_WATCHDOG_ONCHAIN_MAX_ATTEMPTS",
        "INACTIVITY_WATCHDOG_ONCHAIN_BACKOFF_MS",
        "INACTIVITY_WATCHDOG_ONCHAIN_MAX_BACKOFF_MS",
        "INACTIVITY_WATCHDOG_TRIGGER_STALE_AFTER_SECS",
    ];

    fn clear_on_chain_env() {
        for key in ON_CHAIN_ENV {
            std::env::remove_var(key);
        }
    }

    fn signer_secret() -> String {
        stellar_strkey::ed25519::PrivateKey([7u8; 32]).to_string()
    }

    /// A client pointed at contract id `[1u8; 32]`, with no network behind it —
    /// enough to exercise the event verification path.
    fn test_client() -> StellarSubmitClient {
        StellarSubmitClient::new("https://horizon-testnet.stellar.org".to_string())
            .with_soroban(SorobanConfig {
                rpc_url: "https://soroban-testnet.stellar.org".to_string(),
                contract_id: stellar_strkey::Contract([1u8; 32]).to_string(),
                network_passphrase: TESTNET_PASSPHRASE.to_string(),
                signer_secret: signer_secret(),
                poll_interval: Duration::from_millis(10),
                poll_timeout: Duration::from_secs(1),
            })
            .expect("valid soroban config")
    }

    fn symbol(value: &str) -> ScVal {
        ScVal::Symbol(ScSymbol(value.try_into().unwrap()))
    }

    fn event(contract: [u8; 32], topics: &[&str], plan_id: Option<u64>) -> ContractEvent {
        let mut entries = vec![ScMapEntry {
            key: symbol("triggered_at"),
            val: ScVal::U64(1_700_000_000),
        }];
        if let Some(plan_id) = plan_id {
            entries.insert(
                0,
                ScMapEntry {
                    key: symbol("plan_id"),
                    val: ScVal::U64(plan_id),
                },
            );
        }

        ContractEvent {
            ext: ExtensionPoint::V0,
            contract_id: Some(ContractId(Hash(contract))),
            type_: ContractEventType::Contract,
            body: ContractEventBody::V0(ContractEventV0 {
                topics: topics
                    .iter()
                    .map(|topic| symbol(topic))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                data: ScVal::Map(Some(ScMap(entries.try_into().unwrap()))),
            }),
        }
    }

    fn outcome(events: Vec<ContractEvent>) -> InvocationOutcome {
        InvocationOutcome {
            tx_hash: "abc123".to_string(),
            events,
            return_value: None,
        }
    }

    #[test]
    fn config_uses_safe_defaults() {
        let _guard = env_lock();
        std::env::remove_var("INACTIVITY_WATCHDOG_INTERVAL_SECS");
        std::env::remove_var("INACTIVITY_WATCHDOG_BATCH_SIZE");
        clear_on_chain_env();

        let config = InactivityWatchdogConfig::from_env();

        assert_eq!(config.interval, Duration::from_secs(DEFAULT_INTERVAL_SECS));
        assert_eq!(config.batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(config.on_chain.max_attempts, DEFAULT_MAX_ATTEMPTS);
        assert_eq!(
            config.on_chain.initial_backoff,
            Duration::from_millis(DEFAULT_BACKOFF_MS)
        );
        assert_eq!(
            config.on_chain.stale_after,
            Duration::from_secs(DEFAULT_STALE_AFTER_SECS)
        );
    }

    #[test]
    fn config_applies_env_overrides() {
        let _guard = env_lock();
        std::env::set_var("INACTIVITY_WATCHDOG_INTERVAL_SECS", "30");
        std::env::set_var("INACTIVITY_WATCHDOG_BATCH_SIZE", "25");
        std::env::set_var("INACTIVITY_WATCHDOG_ONCHAIN_MAX_ATTEMPTS", "7");
        std::env::set_var("INACTIVITY_WATCHDOG_ONCHAIN_BACKOFF_MS", "250");

        let config = InactivityWatchdogConfig::from_env();

        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.batch_size, 25);
        assert_eq!(config.on_chain.max_attempts, 7);
        assert_eq!(config.on_chain.initial_backoff, Duration::from_millis(250));

        std::env::remove_var("INACTIVITY_WATCHDOG_INTERVAL_SECS");
        std::env::remove_var("INACTIVITY_WATCHDOG_BATCH_SIZE");
        clear_on_chain_env();
    }

    #[test]
    fn config_rejects_zero_values() {
        let _guard = env_lock();
        std::env::set_var("INACTIVITY_WATCHDOG_INTERVAL_SECS", "0");
        std::env::set_var("INACTIVITY_WATCHDOG_BATCH_SIZE", "0");
        std::env::set_var("INACTIVITY_WATCHDOG_ONCHAIN_MAX_ATTEMPTS", "0");
        std::env::set_var("INACTIVITY_WATCHDOG_ONCHAIN_BACKOFF_MS", "0");
        std::env::set_var("INACTIVITY_WATCHDOG_TRIGGER_STALE_AFTER_SECS", "0");

        let config = InactivityWatchdogConfig::from_env();

        assert_eq!(config.interval, Duration::from_secs(1));
        assert_eq!(config.batch_size, 1);
        assert_eq!(config.on_chain.max_attempts, 1);
        assert_eq!(config.on_chain.initial_backoff, Duration::from_millis(1));
        assert_eq!(config.on_chain.stale_after, Duration::from_secs(1));

        std::env::remove_var("INACTIVITY_WATCHDOG_INTERVAL_SECS");
        std::env::remove_var("INACTIVITY_WATCHDOG_BATCH_SIZE");
        clear_on_chain_env();
    }

    #[test]
    fn backoff_doubles_and_is_capped() {
        let retry = OnChainTriggerConfig {
            max_attempts: 6,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_millis(500),
            stale_after: Duration::from_secs(900),
        };

        assert_eq!(retry.backoff_for(1), Duration::from_millis(100));
        assert_eq!(retry.backoff_for(2), Duration::from_millis(200));
        assert_eq!(retry.backoff_for(3), Duration::from_millis(400));
        assert_eq!(retry.backoff_for(4), Duration::from_millis(500));
        // A large attempt count must saturate rather than overflow the shift.
        assert_eq!(retry.backoff_for(64), Duration::from_millis(500));
    }

    #[test]
    fn only_transport_failures_are_retried() {
        assert!(is_retryable(&StellarSubmitError::Network("down".into())));
        assert!(is_retryable(&StellarSubmitError::Rpc("busy".into())));
        assert!(is_retryable(&StellarSubmitError::Timeout {
            hash: "abc".into()
        }));

        assert!(!is_retryable(&StellarSubmitError::TransactionFailed {
            hash: "abc".into(),
            detail: "already triggered".into(),
        }));
        assert!(!is_retryable(&StellarSubmitError::Simulation(
            "unauthorized".into()
        )));
        assert!(!is_retryable(&StellarSubmitError::NotConfigured));
    }

    #[test]
    fn a_matching_trigger_event_verifies() {
        let client = test_client();
        let outcome = outcome(vec![event([1u8; 32], &["INHERIT", "TRIGGER"], Some(42))]);
        assert_eq!(verify_trigger_event(&client, &outcome, 42), Ok(()));
    }

    #[test]
    fn a_transaction_without_the_trigger_event_does_not_verify() {
        let client = test_client();
        let outcome = outcome(vec![event([1u8; 32], &["LOAN", "FREEZE"], Some(42))]);
        let error = verify_trigger_event(&client, &outcome, 42).unwrap_err();
        assert!(error.contains("no INHERIT/TRIGGER event"), "{error}");
    }

    #[test]
    fn an_event_from_another_contract_does_not_verify() {
        let client = test_client();
        let outcome = outcome(vec![event([9u8; 32], &["INHERIT", "TRIGGER"], Some(42))]);
        assert!(verify_trigger_event(&client, &outcome, 42).is_err());
    }

    #[test]
    fn an_event_for_a_different_plan_does_not_verify() {
        let client = test_client();
        let outcome = outcome(vec![event([1u8; 32], &["INHERIT", "TRIGGER"], Some(7))]);
        let error = verify_trigger_event(&client, &outcome, 42).unwrap_err();
        assert!(error.contains("triggered plan 7"), "{error}");
    }

    #[test]
    fn an_event_without_a_plan_id_does_not_verify() {
        let client = test_client();
        let outcome = outcome(vec![event([1u8; 32], &["INHERIT", "TRIGGER"], None)]);
        let error = verify_trigger_event(&client, &outcome, 42).unwrap_err();
        assert!(error.contains("without a plan_id"), "{error}");
    }

    #[tokio::test]
    async fn a_client_without_soroban_is_not_attached() {
        let pool = PgPool::connect_lazy("postgres://postgres:postgres@localhost/inheritx")
            .expect("lazy pool");
        let service = InactivityWatchdogService::new(
            pool,
            PlanCache::disabled(),
            InactivityWatchdogConfig::from_env(),
        )
        .with_stellar(StellarSubmitClient::new(
            "https://horizon-testnet.stellar.org".to_string(),
        ));

        assert!(service.stellar.is_none());
    }
}
