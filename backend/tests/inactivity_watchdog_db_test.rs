//! Exercises the inactivity watchdog sweep against a real PostgreSQL instance,
//! which is the only way to check the claim/finalise SQL and the migration
//! chain it depends on.
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` to point at a throwaway database to run
//! these; without it they skip, so `cargo test` still works on a machine with
//! no PostgreSQL. It is deliberately a different variable from `DATABASE_URL`,
//! which other tests expect to be unreachable.

use inheritx_backend::{DbManager, InactivityWatchdogConfig, InactivityWatchdogService, PlanCache};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// A sweep claims every expired plan in the database, not just the one the
/// calling test inserted, so these tests must not run against it at the same
/// time.
fn sweep_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Connects and migrates, or returns `None` when no test database is configured.
async fn database() -> Option<PgPool> {
    let url = std::env::var("WATCHDOG_TEST_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url)
        .await
        .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");

    DbManager::run_migrations(&pool)
        .await
        .expect("migrations must apply cleanly");

    Some(pool)
}

fn service(db: &PgPool) -> InactivityWatchdogService {
    InactivityWatchdogService::new(
        db.clone(),
        PlanCache::disabled(),
        InactivityWatchdogConfig::from_env(),
    )
}

/// Inserts a plan whose inactivity deadline has already passed.
async fn insert_expired_plan(
    db: &PgPool,
    owner: &str,
    status: &str,
    onchain_plan_id: Option<i64>,
    trigger_started_at_offset_secs: Option<i64>,
) -> Uuid {
    let last_ping = chrono::Utc::now().timestamp() - 10_000;

    sqlx::query_scalar(
        r#"
        INSERT INTO plans (
            owner_address, token_address, amount, grace_period,
            grace_period_seconds, last_ping, is_active, status,
            onchain_plan_id, trigger_started_at
        )
        VALUES (
            $1, 'TOKEN', 1000, 1, 1, $2, true, $3, $4,
            CASE WHEN $5::bigint IS NULL
                 THEN NULL
                 ELSE NOW() - make_interval(secs => $5::double precision)
            END
        )
        RETURNING id
        "#,
    )
    .bind(owner)
    .bind(last_ping)
    .bind(status)
    .bind(onchain_plan_id)
    .bind(trigger_started_at_offset_secs)
    .fetch_one(db)
    .await
    .expect("insert plan")
}

async fn plan_state(db: &PgPool, id: Uuid) -> (String, i32, Option<String>) {
    let row =
        sqlx::query("SELECT status, trigger_attempts, last_trigger_error FROM plans WHERE id = $1")
            .bind(id)
            .fetch_one(db)
            .await
            .expect("load plan");

    (
        row.get("status"),
        row.get("trigger_attempts"),
        row.get("last_trigger_error"),
    )
}

async fn cleanup(db: &PgPool, owner: &str) {
    sqlx::query("DELETE FROM plans WHERE owner_address = $1")
        .bind(owner)
        .execute(db)
        .await
        .expect("cleanup");
}

/// Without a Soroban client the sweep keeps its original behaviour: expired
/// plans go straight to TRIGGERED.
#[tokio::test]
async fn sweep_marks_expired_plans_triggered_when_no_chain_is_configured() {
    let Some(db) = database().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let _guard = sweep_lock().lock().await;

    let owner = "TEST_WATCHDOG_NO_CHAIN";
    cleanup(&db, owner).await;
    let plan_id = insert_expired_plan(&db, owner, "ACTIVE", None, None).await;

    let triggered = service(&db).run_once().await.expect("sweep");
    assert!(triggered >= 1, "expected the expired plan to be swept");

    let (status, _, error) = plan_state(&db, plan_id).await;
    assert_eq!(status, "TRIGGERED");
    assert_eq!(error, None);

    cleanup(&db, owner).await;
}

/// A plan with no `onchain_plan_id` cannot be triggered on-chain, so the sweep
/// must record the failure rather than claim the funds were released.
#[tokio::test]
async fn sweep_fails_a_plan_that_has_no_onchain_id() {
    let Some(db) = database().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let _guard = sweep_lock().lock().await;

    let owner = "TEST_WATCHDOG_MISSING_ONCHAIN_ID";
    cleanup(&db, owner).await;
    let plan_id = insert_expired_plan(&db, owner, "ACTIVE", None, None).await;

    let triggered = service(&db)
        .with_stellar(chain_client())
        .run_once()
        .await
        .expect("sweep");
    assert_eq!(triggered, 0, "nothing may be reported as triggered");

    let (status, attempts, error) = plan_state(&db, plan_id).await;
    assert_eq!(status, "TRIGGER_FAILED");
    assert_eq!(attempts, 0, "no submission was attempted");
    assert!(
        error.unwrap_or_default().contains("onchain_plan_id"),
        "the failure reason must name the missing id"
    );

    cleanup(&db, owner).await;
}

/// A plan left in TRIGGERING by a crashed worker is picked up again once it
/// goes stale, instead of being stranded there forever.
#[tokio::test]
async fn sweep_reclaims_a_stale_in_flight_plan() {
    let Some(db) = database().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let _guard = sweep_lock().lock().await;

    let owner = "TEST_WATCHDOG_STALE_IN_FLIGHT";
    cleanup(&db, owner).await;
    // Claimed an hour ago, well past the 15 minute default staleness window.
    let plan_id = insert_expired_plan(&db, owner, "TRIGGERING", None, Some(3_600)).await;

    service(&db)
        .with_stellar(chain_client())
        .run_once()
        .await
        .expect("sweep");

    let (status, _, _) = plan_state(&db, plan_id).await;
    assert_eq!(
        status, "TRIGGER_FAILED",
        "the stale plan must be re-claimed"
    );

    cleanup(&db, owner).await;
}

/// A plan still inside the staleness window belongs to a live worker and must
/// be left alone.
#[tokio::test]
async fn sweep_leaves_a_fresh_in_flight_plan_alone() {
    let Some(db) = database().await else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL is not set");
        return;
    };

    let _guard = sweep_lock().lock().await;

    let owner = "TEST_WATCHDOG_FRESH_IN_FLIGHT";
    cleanup(&db, owner).await;
    let plan_id = insert_expired_plan(&db, owner, "TRIGGERING", None, Some(5)).await;

    service(&db)
        .with_stellar(chain_client())
        .run_once()
        .await
        .expect("sweep");

    let (status, _, _) = plan_state(&db, plan_id).await;
    assert_eq!(status, "TRIGGERING");

    cleanup(&db, owner).await;
}

/// A Soroban-enabled client pointed at an address nothing answers on. The
/// tests above never reach the network: they fail before a transaction is
/// built.
fn chain_client() -> inheritx_backend::stellar_submit::StellarSubmitClient {
    use inheritx_backend::stellar_submit::{
        SorobanConfig, StellarSubmitClient, TESTNET_PASSPHRASE,
    };

    StellarSubmitClient::new("http://127.0.0.1:1".to_string())
        .with_soroban(SorobanConfig {
            rpc_url: "http://127.0.0.1:1".to_string(),
            contract_id: stellar_strkey::Contract([1u8; 32]).to_string(),
            network_passphrase: TESTNET_PASSPHRASE.to_string(),
            signer_secret: stellar_strkey::ed25519::PrivateKey([7u8; 32]).to_string(),
            poll_interval: std::time::Duration::from_millis(10),
            poll_timeout: std::time::Duration::from_secs(1),
        })
        .expect("valid soroban config")
}
