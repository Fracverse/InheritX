use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::cache::PlanCache;

const DEFAULT_INTERVAL_SECS: u64 = 60 * 60;
const DEFAULT_BATCH_SIZE: i64 = 500;
const WATCHDOG_LOCK_KEY: i64 = 820;
const CLAIMABLE_STATUS: &str = "CLAIMABLE";

#[derive(Debug, Clone, Copy)]
pub struct InactivityWatchdogConfig {
    pub interval: Duration,
    pub batch_size: i64,
}

impl InactivityWatchdogConfig {
    pub fn from_env() -> Self {
        let interval_secs =
            parse_env_u64("INACTIVITY_WATCHDOG_INTERVAL_SECS", DEFAULT_INTERVAL_SECS);
        let batch_size = parse_env_i64("INACTIVITY_WATCHDOG_BATCH_SIZE", DEFAULT_BATCH_SIZE).max(1);

        Self {
            interval: Duration::from_secs(interval_secs.max(1)),
            batch_size,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ExpiredPlan {
    id: Uuid,
    owner_address: String,
    user_id: Uuid,
    title: String,
    inactivity_deadline_at: DateTime<Utc>,
}

pub struct InactivityWatchdogService {
    db: PgPool,
    plan_cache: PlanCache,
    config: InactivityWatchdogConfig,
}

impl InactivityWatchdogService {
    pub fn new(db: PgPool, plan_cache: PlanCache, config: InactivityWatchdogConfig) -> Self {
        Self {
            db,
            plan_cache,
            config,
        }
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.config.interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                match self.run_once().await {
                    Ok(count) if count > 0 => {
                        info!("Inactivity watchdog marked {count} plan(s) as claimable");
                    }
                    Ok(_) => {}
                    Err(e) => error!("Inactivity watchdog sweep failed: {e}"),
                }
            }
        });
    }

    pub async fn run_once(&self) -> Result<usize, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let lock_acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
            .bind(WATCHDOG_LOCK_KEY)
            .fetch_one(&mut *tx)
            .await?;

        if !lock_acquired {
            warn!("Inactivity watchdog lock is held by another worker; skipping sweep");
            tx.commit().await?;
            return Ok(0);
        }

        let expired_plans = sqlx::query_as::<_, ExpiredPlan>(
            r#"
            UPDATE plans
            SET status = $1,
                updated_at = NOW()
            WHERE id IN (
                SELECT p.id
                FROM plans p
                WHERE COALESCE(p.is_active, true) = true
                  AND p.status <> $1
                  AND p.last_ping IS NOT NULL
                  AND p.inactivity_deadline_at <= NOW()
                ORDER BY p.inactivity_deadline_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, owner_address, user_id, title, inactivity_deadline_at
            "#,
        )
        .bind(CLAIMABLE_STATUS)
        .bind(self.config.batch_size)
        .fetch_all(&mut *tx)
        .await?;

        for plan in &expired_plans {
            let beneficiary_addresses: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT wallet_address
                FROM beneficiaries
                WHERE plan_id = $1
                "#,
            )
            .bind(plan.id)
            .fetch_all(&mut *tx)
            .await?;

            if let Err(err) = self
                .plan_cache
                .invalidate_queries(&plan.owner_address, &beneficiary_addresses)
                .await
            {
                warn!(
                    plan_id = %plan.id,
                    error = %err,
                    "Failed to invalidate Redis plan cache for claimable plan"
                );
            }
        }

        for plan in &expired_plans {
            warn!(
                plan_id = %plan.id,
                user_id = %plan.user_id,
                title = %plan.title,
                inactivity_deadline_at = %plan.inactivity_deadline_at,
                "Plan marked claimable by inactivity watchdog"
            );
        }

        tx.commit().await?;
        Ok(expired_plans.len())
    }
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
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn config_uses_safe_defaults() {
        let _guard = env_lock();
        std::env::remove_var("INACTIVITY_WATCHDOG_INTERVAL_SECS");
        std::env::remove_var("INACTIVITY_WATCHDOG_BATCH_SIZE");

        let config = InactivityWatchdogConfig::from_env();

        assert_eq!(config.interval, Duration::from_secs(DEFAULT_INTERVAL_SECS));
        assert_eq!(config.batch_size, DEFAULT_BATCH_SIZE);
    }

    #[test]
    fn config_applies_env_overrides() {
        let _guard = env_lock();
        std::env::set_var("INACTIVITY_WATCHDOG_INTERVAL_SECS", "30");
        std::env::set_var("INACTIVITY_WATCHDOG_BATCH_SIZE", "25");

        let config = InactivityWatchdogConfig::from_env();

        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.batch_size, 25);

        std::env::remove_var("INACTIVITY_WATCHDOG_INTERVAL_SECS");
        std::env::remove_var("INACTIVITY_WATCHDOG_BATCH_SIZE");
    }

    #[test]
    fn config_rejects_zero_values() {
        let _guard = env_lock();
        std::env::set_var("INACTIVITY_WATCHDOG_INTERVAL_SECS", "0");
        std::env::set_var("INACTIVITY_WATCHDOG_BATCH_SIZE", "0");

        let config = InactivityWatchdogConfig::from_env();

        assert_eq!(config.interval, Duration::from_secs(1));
        assert_eq!(config.batch_size, 1);

        std::env::remove_var("INACTIVITY_WATCHDOG_INTERVAL_SECS");
        std::env::remove_var("INACTIVITY_WATCHDOG_BATCH_SIZE");
    }
}
