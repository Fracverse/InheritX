/*
mod helpers;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use inheritx_backend::auth::AdminClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

fn generate_admin_token(admin_id: Uuid) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = AdminClaims {
        admin_id,
        email: format!("admin-{}@example.com", admin_id),
        role: "admin".to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"test-jwt-secret"),
    )
    .expect("Failed to generate admin token")
}

#[tokio::test]
async fn admin_can_fetch_emergency_access_metrics() {
    let Some(ctx) = helpers::TestContext::from_env().await else {
        return;
    };

    let token = generate_admin_token(Uuid::new_v4());

    let response = ctx
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/analytics/emergency-access?range=daily")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read body");
    let json: Value = serde_json::from_slice(&body).expect("Response is not valid JSON");

    assert_eq!(json["status"], "success");
    let data = &json["data"];
    assert!(data.get("totalGrants").is_some(), "missing totalGrants");
    assert!(data.get("activeGrants").is_some(), "missing activeGrants");
    assert!(
        data.get("totalRevocations").is_some(),
        "missing totalRevocations"
    );
    assert!(data.get("totalAlerts").is_some(), "missing totalAlerts");
    assert!(
        data.get("alertsBySeverity").is_some(),
        "missing alertsBySeverity"
    );
    assert!(data.get("grantTrend").is_some(), "missing grantTrend");

    // Check trend structure
    if let Some(trend) = data["grantTrend"].as_array() {
        if !trend.is_empty() {
            assert!(trend[0].get("date").is_some());
            assert!(trend[0].get("count").is_some());
        }
    }
}
*/
