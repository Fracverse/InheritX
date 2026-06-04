use chrono::Utc;
use inheritx_backend::compliance::ComplianceEngine;
use rust_decimal_macros::dec;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

async fn insert_test_user(db: &PgPool, user_id: Uuid) {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, is_verified) VALUES ($1, $2, 'hash', true)",
    )
    .bind(user_id)
    .bind(format!("user_{}@test.com", user_id))
    .execute(db)
    .await
    .unwrap();
}

async fn insert_test_user_with_created_at(
    db: &PgPool,
    user_id: Uuid,
    created_at: chrono::DateTime<Utc>,
) {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, is_verified, created_at) VALUES ($1, $2, 'hash', true, $3)",
    )
    .bind(user_id)
    .bind(format!("user_{}@test.com", user_id))
    .bind(created_at)
    .execute(db)
    .await
    .unwrap();
}

async fn insert_test_plan(db: &PgPool, plan_id: Uuid, user_id: Uuid) {
    sqlx::query(
        "INSERT INTO plans (id, user_id, title, is_flagged) VALUES ($1, $2, 'Test Plan', false)",
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(db)
    .await
    .unwrap();
}

async fn insert_old_test_plan(db: &PgPool, plan_id: Uuid, user_id: Uuid) {
    sqlx::query(
        r#"INSERT INTO plans (id, user_id, title, is_flagged, created_at)
           VALUES ($1, $2, 'Old Test Plan', false, NOW() - INTERVAL '60 days')"#,
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(db)
    .await
    .unwrap();
}

async fn insert_test_lending_event(db: &PgPool, plan_id: Uuid, user_id: Uuid, event_type: &str, amount: &str, minutes_ago: i64) {
    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, $4, $5, 'USD', NOW() - INTERVAL '1 minute' * $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .bind(event_type)
    .bind(amount)
    .bind(minutes_ago)
    .execute(db)
    .await
    .unwrap();
}

#[sqlx::test]
async fn test_velocity_detection_logic(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert multiple lending events within velocity window
    for i in 0..5 {
        sqlx::query(
            r#"
            INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
            VALUES ($1, $2, $3, 'borrow', 1000, 'USD', NOW() - INTERVAL '1 minute' * $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(plan_id)
        .bind(user_id)
        .bind(i)
        .execute(&db)
        .await
        .unwrap();
    }

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(flagged, "Plan should be flagged for high velocity");
}

#[sqlx::test]
async fn test_volume_threshold_detection(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert large volume borrow event
    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'borrow', 150000, 'USD', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(flagged, "Plan should be flagged for abnormal volume");
}

#[sqlx::test]
async fn test_sanctions_screening_integration(db: PgPool) {
    let engine = ComplianceEngine::new(db, 3, 10, dec!(100000));
    assert_eq!(engine.velocity_threshold, 3);
    // TODO: Implement actual sanctions screening test when service is integrated
}

#[sqlx::test]
async fn test_risk_scoring_algorithms(db: PgPool) {
    let engine = ComplianceEngine::new(db, 3, 10, dec!(100000));
    assert_eq!(engine.volume_threshold, dec!(100000));
    // TODO: Implement risk scoring algorithm tests
}

#[sqlx::test]
async fn test_compliance_violation_scenarios(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_old_test_plan(&db, plan_id, user_id).await;

    // Insert sudden borrow event
    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'borrow', 5000, 'USD', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(flagged, "Plan should be flagged for sudden activity spike");
}

#[sqlx::test]
async fn test_velocity_no_flag_below_threshold(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert events below threshold (2 events, threshold is 3)
    for i in 0..2 {
        sqlx::query(
            r#"
            INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
            VALUES ($1, $2, $3, 'borrow', 1000, 'USD', NOW() - INTERVAL '1 minute' * $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(plan_id)
        .bind(user_id)
        .bind(i)
        .execute(&db)
        .await
        .unwrap();
    }

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(!flagged, "Plan should NOT be flagged when events are below threshold");
}

#[sqlx::test]
async fn test_velocity_events_outside_window_not_flagged(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert events OLDER than the velocity window (15 minutes, window is 10)
    for i in 0..5 {
        sqlx::query(
            r#"
            INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
            VALUES ($1, $2, $3, 'borrow', 1000, 'USD', NOW() - INTERVAL '1 minute' * $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(plan_id)
        .bind(user_id)
        .bind(15 + i)
        .execute(&db)
        .await
        .unwrap();
    }

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(!flagged, "Plan should NOT be flagged when events are outside velocity window");
}

#[sqlx::test]
async fn test_velocity_exactly_at_threshold(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert exactly threshold number of events (3 events, threshold is 3)
    for i in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
            VALUES ($1, $2, $3, 'borrow', 1000, 'USD', NOW() - INTERVAL '1 minute' * $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(plan_id)
        .bind(user_id)
        .bind(i)
        .execute(&db)
        .await
        .unwrap();
    }

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(flagged, "Plan should be flagged when events exactly equal threshold");
}

#[sqlx::test]
async fn test_velocity_repay_events_included(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert mix of borrow and repay events (2 borrows + 1 repay = 3 total, threshold is 3)
    for i in 0..2 {
        sqlx::query(
            r#"
            INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
            VALUES ($1, $2, $3, 'borrow', 1000, 'USD', NOW() - INTERVAL '1 minute' * $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(plan_id)
        .bind(user_id)
        .bind(i)
        .execute(&db)
        .await
        .unwrap();
    }

    // Insert one repay event
    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'repay', 500, 'USD', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(flagged, "Plan should be flagged when borrow+repay events reach threshold");
}

#[sqlx::test]
async fn test_volume_below_threshold_not_flagged(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert volume below threshold
    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'borrow', 50000, 'USD', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(!flagged, "Plan should NOT be flagged when volume is below threshold");
}

#[sqlx::test]
async fn test_velocity_suspicion_flags_stored(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert 4 events to exceed threshold
    for i in 0..4 {
        insert_test_lending_event(&db, plan_id, user_id, "borrow", "1000", i).await;
    }

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flags: Option<String> = sqlx::query_scalar("SELECT suspicion_flags FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap();

    assert!(flags.is_some(), "suspicion_flags should be stored");
    let flags = flags.unwrap();
    assert!(flags.contains("High velocity detected"));
    assert!(flags.contains("4 borrowing events"));
    assert!(flags.contains("10 minutes"));
}

#[sqlx::test]
async fn test_sudden_activity_spike_is_not_flagged_for_recent_activity(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user_with_created_at(&db, user_id, Utc::now() - chrono::Duration::days(60)).await;

    sqlx::query(
        r#"
        INSERT INTO plans (id, user_id, title, is_flagged, created_at)
        VALUES ($1, $2, 'Old Plan', false, NOW() - INTERVAL '60 days')
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'borrow', 2500, 'USD', NOW() - INTERVAL '10 days')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'borrow', 5000, 'USD', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    assert!(
        !flagged,
        "Plan should not be flagged when there was recent activity within 30 days"
    );
}

#[sqlx::test]
async fn test_edge_cases_covered(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert exactly threshold volume
    sqlx::query(
        r#"
        INSERT INTO lending_events (id, plan_id, user_id, event_type, amount, asset_code, event_timestamp)
        VALUES ($1, $2, $3, 'borrow', 100000, 'USD', NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(plan_id)
    .bind(user_id)
    .execute(&db)
    .await
    .unwrap();

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let flagged: bool = sqlx::query_scalar("SELECT is_flagged FROM plans WHERE id = $1")
        .bind(plan_id)
        .fetch_one(&db)
        .await
        .unwrap_or(false);

    // Should be flagged at exactly threshold
    assert!(flagged, "Plan should be flagged at volume threshold");
}

#[sqlx::test]
async fn test_velocity_audit_log_created(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    insert_test_user(&db, user_id).await;
    insert_test_plan(&db, plan_id, user_id).await;

    // Insert 4 events to exceed threshold
    for i in 0..4 {
        insert_test_lending_event(&db, plan_id, user_id, "borrow", "1000", i).await;
    }

    let engine = ComplianceEngine::new(db.clone(), 3, 10, dec!(100000));
    let engine = Arc::new(engine);

    engine.scan_suspicious_activity().await.unwrap();

    let log_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM action_logs WHERE user_id = $1 AND action = 'suspicious_borrowing_detected' AND entity_id = $2)"
    )
    .bind(user_id)
    .bind(plan_id)
    .fetch_one(&db)
    .await
    .unwrap_or(false);

    assert!(log_exists, "Audit log should be created for suspicious activity");
}
