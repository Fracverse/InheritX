mod helpers;

use reqwest::StatusCode;

#[tokio::test]
async fn health_db_returns_200_when_database_connected() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    let response = test_context
        .client
        .get(format!("{}/health/db", test_context.base_url))
        .send()
        .await
        .expect("request to /health/db failed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_db_returns_500_when_database_is_unavailable() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        return;
    };

    test_context.pool.close().await;

    let response = test_context
        .client
        .get(format!("{}/health/db", test_context.base_url))
        .send()
        .await
        .expect("request to /health/db failed");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}