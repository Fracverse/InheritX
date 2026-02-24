use axum::Router;
use inheritx_backend::{create_app, Config};
use reqwest::StatusCode;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn governor_rate_limit_burst_and_reset() {
    let config = Config {
        database_url: "postgres://localhost/unused".to_string(),
        port: 0,
        jwt_secret: "test_secret".to_string(),
    };
    let db_pool = PgPool::connect_lazy(&config.database_url).expect("lazy pool");
    let app: Router = create_app(db_pool, config).await.expect("create app");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = reqwest::Client::new();
    let url = format!("http://{}/health", addr);
    let mut statuses = Vec::new();
    for _ in 0..6 {
        let resp = client.get(&url).send().await.expect("req");
        statuses.push(resp.status());
    }
    assert!(statuses[..5].iter().all(|s| *s == StatusCode::OK));
    assert_eq!(statuses[5], StatusCode::TOO_MANY_REQUESTS);
    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
    let resp_after = client.get(&url).send().await.expect("req after");
    assert_eq!(resp_after.status(), StatusCode::OK);
}
