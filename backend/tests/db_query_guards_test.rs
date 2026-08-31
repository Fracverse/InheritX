//! Verifies the connection-pool query guards configured in `DbManager::create_pool`
//! actually take effect against a real PostgreSQL instance: every pooled
//! connection gets a session-level `statement_timeout`, and Postgres enforces
//! it by cancelling a query that overruns it.
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` to point at a throwaway database to run
//! these; without it they skip, matching `inactivity_watchdog_db_test.rs`.

use inheritx_backend::DbManager;

/// Both tests mutate the process-wide `DB_STATEMENT_TIMEOUT_MS` env var that
/// `create_pool` reads, so they must not run concurrently with each other.
fn env_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Creates a pool with `DB_STATEMENT_TIMEOUT_MS` set to `timeout_ms`, or
/// returns `None` when no test database is configured.
async fn pool_with_statement_timeout(timeout_ms: u64) -> Option<sqlx::PgPool> {
    let url = std::env::var("WATCHDOG_TEST_DATABASE_URL").ok()?;

    std::env::set_var("DB_STATEMENT_TIMEOUT_MS", timeout_ms.to_string());
    let pool = DbManager::create_pool(&url)
        .await
        .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");
    std::env::remove_var("DB_STATEMENT_TIMEOUT_MS");

    Some(pool)
}

#[tokio::test]
async fn statement_timeout_is_applied_to_every_pooled_connection() {
    let _guard = env_lock().lock().await;
    let Some(pool) = pool_with_statement_timeout(2_000).await else {
        return;
    };

    let timeout: String = sqlx::query_scalar("SHOW statement_timeout")
        .fetch_one(&pool)
        .await
        .expect("SHOW statement_timeout must succeed");

    assert_eq!(timeout, "2s");
}

#[tokio::test]
async fn a_query_past_the_statement_timeout_is_cancelled_by_postgres() {
    let _guard = env_lock().lock().await;
    let Some(pool) = pool_with_statement_timeout(200).await else {
        return;
    };

    let result = sqlx::query("SELECT pg_sleep(1)").execute(&pool).await;

    let error = result.expect_err("a query past statement_timeout must fail");
    let db_error = error
        .as_database_error()
        .expect("Postgres must report a database error, not a transport failure");
    // 57014 = query_canceled, Postgres's SQLSTATE for a statement_timeout hit.
    assert_eq!(db_error.code().as_deref(), Some("57014"));
}
