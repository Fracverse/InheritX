use axum::Router;
use inheritx_backend::{
    config::{Config, DbPoolConfig, RateLimitConfig},
    create_app,
};
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
        rate_limit: RateLimitConfig {
            default_per_second: 2,
            default_burst_size: 5,
            emergency_per_second: 1,
            emergency_burst_size: 2,
            admin_login_per_second: 1,
            admin_login_burst_size: 3,
            bypass_tokens: Vec::new(),
        },
        db_pool: DbPoolConfig::from_env_or_defaults(),
    };
    
    let db_pool = PgPool::connect_lazy(&config.database_url).expect("lazy pool");
    let prometheus_handle = inheritx_backend::get_or_install_recorder();
    let app: Router = create_app(db_pool, config, prometheus_handle).await.expect("create app");
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
    
    // Give the server a small moment to bind and start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/health");
    let mut responses = Vec::new();
    
    for _ in 0..6 {
        let resp = client.get(&url).send().await.expect("req");
        responses.push(resp);
    }
    
    // Verify successful responses (first 5)
    for (i, resp) in responses[..5].iter().enumerate() {
        assert_eq!(resp.status(), StatusCode::OK, "Response {} failed", i);
        
        let headers = resp.headers();
        assert!(headers.contains_key("x-ratelimit-limit"), "Missing x-ratelimit-limit in response {}", i);
        assert!(headers.contains_key("x-ratelimit-remaining"), "Missing x-ratelimit-remaining in response {}", i);
        assert!(headers.contains_key("x-ratelimit-reset"), "Missing x-ratelimit-reset in response {}", i);
        
        let limit = headers.get("x-ratelimit-limit").unwrap().to_str().unwrap().parse::<u32>().unwrap();
        let remaining = headers.get("x-ratelimit-remaining").unwrap().to_str().unwrap().parse::<u32>().unwrap();
        let reset = headers.get("x-ratelimit-reset").unwrap().to_str().unwrap().parse::<u64>().unwrap();
        
        assert_eq!(limit, 5, "Limit mismatch in response {}", i);
        assert!(remaining <= 5, "Remaining was too high in response {}", i);
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(reset >= now, "x-ratelimit-reset should be in the future or now, got {} vs now {}", reset, now);
    }
    
    // Verify rate-limited response (6th)
    let rate_limited_resp = &responses[5];
    assert_eq!(rate_limited_resp.status(), StatusCode::TOO_MANY_REQUESTS);
    
    let headers = rate_limited_resp.headers();
    assert!(headers.contains_key("x-ratelimit-limit"), "Missing x-ratelimit-limit in 429 response");
    assert!(headers.contains_key("x-ratelimit-remaining"), "Missing x-ratelimit-remaining in 429 response");
    assert!(headers.contains_key("x-ratelimit-reset"), "Missing x-ratelimit-reset in 429 response");
    assert!(headers.contains_key("retry-after"), "Missing retry-after in 429 response");
    
    let remaining = headers.get("x-ratelimit-remaining").unwrap().to_str().unwrap().parse::<u32>().unwrap();
    assert_eq!(remaining, 0);
    
    let retry_after = headers.get("retry-after").unwrap().to_str().unwrap().parse::<u32>().unwrap();
    assert!(retry_after > 0);
}
