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

    // Insert old plan with no recent activity
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
