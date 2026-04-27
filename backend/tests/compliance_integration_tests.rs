use crate::compliance::ComplianceEngine;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[sqlx::test]
async fn test_velocity_detection_logic(db: PgPool) {
    // Setup test data
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

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

    // Run compliance scan
    engine.scan_suspicious_activity().await.unwrap();

    // Check if plan was flagged
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

    // Insert plan
    sqlx::query("INSERT INTO plans (id, user_id, is_flagged) VALUES ($1, $2, false)")
        .bind(plan_id)
        .bind(user_id)
        .execute(&db)
        .await
        .unwrap();

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
    // This would test integration with external sanctions screening service
    // For now, placeholder test
    let engine = ComplianceEngine::new(db, 3, 10, dec!(100000));
    assert_eq!(engine.velocity_threshold, 3);
    // TODO: Implement actual sanctions screening test when service is integrated
}

#[sqlx::test]
async fn test_risk_scoring_algorithms(db: PgPool) {
    // Test risk scoring logic
    // Placeholder for risk scoring tests
    let engine = ComplianceEngine::new(db, 3, 10, dec!(100000));
    assert_eq!(engine.volume_threshold, dec!(100000));
    // TODO: Implement risk scoring algorithm tests
}

#[sqlx::test]
async fn test_compliance_violation_scenarios(db: PgPool) {
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    // Insert old plan with no recent activity
    sqlx::query(
        r#"
        INSERT INTO plans (id, user_id, is_flagged, created_at)
        VALUES ($1, $2, false, NOW() - INTERVAL '60 days')
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
    // Test edge cases like exactly at threshold
    let user_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();

    sqlx::query("INSERT INTO plans (id, user_id, is_flagged) VALUES ($1, $2, false)")
        .bind(plan_id)
        .bind(user_id)
        .execute(&db)
        .await
        .unwrap();

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
