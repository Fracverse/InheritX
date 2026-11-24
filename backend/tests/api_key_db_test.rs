//! Database-backed coverage for the `X-API-Key` rate limit bypass (Issue #1138):
//! only keys whose SHA-256 hash is stored in `api_keys` are accepted.
//!
//! Set `WATCHDOG_TEST_DATABASE_URL` to a throwaway PostgreSQL database to run
//! this; without it the test skips, matching the other `*_db_test.rs` files.

use inheritx_backend::middleware::{hash_api_key, ApiKeyVerifier, PgApiKeyVerifier};

#[tokio::test]
async fn test_pg_verifier_accepts_only_stored_key_hashes() {
    let Ok(url) = std::env::var("WATCHDOG_TEST_DATABASE_URL") else {
        eprintln!("skipping: WATCHDOG_TEST_DATABASE_URL not set");
        return;
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("WATCHDOG_TEST_DATABASE_URL is set but unreachable");
    inheritx_backend::DbManager::run_migrations(&pool)
        .await
        .expect("migrations must apply to the test database");

    let raw_key = format!("db-test-key-{}", uuid::Uuid::new_v4());
    let key_hash = hash_api_key(&raw_key);
    sqlx::query("INSERT INTO api_keys (key_hash, label) VALUES ($1, 'db-test')")
        .bind(&key_hash)
        .execute(&pool)
        .await
        .expect("insert api key hash");

    let verifier = PgApiKeyVerifier::new(pool.clone());
    assert!(verifier.is_valid_hash(&key_hash).await);
    assert!(
        !verifier
            .is_valid_hash(&hash_api_key("not-a-stored-key"))
            .await
    );
    // The raw key itself is never what's stored or matched.
    assert!(!verifier.is_valid_hash(&raw_key).await);

    sqlx::query("DELETE FROM api_keys WHERE key_hash = $1")
        .bind(&key_hash)
        .execute(&pool)
        .await
        .expect("cleanup");
}
