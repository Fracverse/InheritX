mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::governance::Proposal;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_governance_list_auto_finalizes_expired_proposals() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let admin_id = helpers::create_test_admin(&ctx.pool, "governance-expiry@test.com")
        .await
        .expect("failed to create test admin");

    let expired_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO governance_proposals (
            id, title, description, proposer_id, status, yes_votes, no_votes, expires_at, created_at
        )
        VALUES ($1, $2, $3, $4, 'active', 1, 0, NOW() - INTERVAL '1 hour', NOW() - INTERVAL '2 hours')
        "#,
    )
    .bind(expired_id)
    .bind("Expired proposal")
    .bind("This proposal should be finalized automatically")
    .bind(admin_id)
    .execute(&ctx.pool)
    .await
    .expect("failed to seed expired proposal");

    let active_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO governance_proposals (
            id, title, description, proposer_id, status, yes_votes, no_votes, expires_at, created_at
        )
        VALUES ($1, $2, $3, $4, 'active', 0, 0, NOW() + INTERVAL '1 day', NOW() - INTERVAL '1 hour')
        "#,
    )
    .bind(active_id)
    .bind("Active proposal")
    .bind("This proposal should remain active")
    .bind(admin_id)
    .execute(&ctx.pool)
    .await
    .expect("failed to seed active proposal");

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/governance/proposals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("failed to read response body");
    let proposals: Vec<Proposal> = serde_json::from_slice(&body).expect("invalid proposal list");

    let expired = proposals
        .iter()
        .find(|proposal| proposal.id == expired_id)
        .expect("expired proposal missing from list");
    assert_eq!(expired.status, "passed");

    let active = proposals
        .iter()
        .find(|proposal| proposal.id == active_id)
        .expect("active proposal missing from list");
    assert_eq!(active.status, "active");

    let stored_expired: Proposal =
        sqlx::query_as("SELECT * FROM governance_proposals WHERE id = $1")
            .bind(expired_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("failed to reload expired proposal");
    assert_eq!(stored_expired.status, "passed");
}
