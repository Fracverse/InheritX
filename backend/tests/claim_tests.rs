mod helpers;

use chrono::Utc;
use inheritx_backend::auth::UserClaims;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;

fn generate_test_token(user_id: Uuid, email: &str) -> String {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = UserClaims {
        user_id,
        email: email.to_string(),
        exp: expiration,
    };

    // Using the hardcoded secret from auth.rs
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"secret_key_change_in_production"),
    )
    .unwrap()
}

#[tokio::test]
async fn test_claim_before_maturity_returns_400() {
    let Some(test_context) = helpers::TestContext::from_env().await else {
        println!("SKIPPING TEST: no database connection");
        return;
    };
    println!("RUNNING TEST: database connected");

    let pool = test_context.pool.clone();
    let app = test_context.app;

    // 1. Create a test user
    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", user_id);
    let _ = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&email)
        .bind("hashed_password")
        .execute(&pool)
        .await
        .unwrap();

    // 2. Approve KYC
    let _ = sqlx::query("INSERT INTO kyc_status (user_id, status) VALUES ($1, 'approved')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    // 3. Create a plan that is NOT mature (Monthly, created just now)
    let plan_id = Uuid::new_v4();
    let now_ts = Utc::now().timestamp();
    let _ = sqlx::query(
        r#"
        INSERT INTO plans (
            id, user_id, title, description, fee, net_amount, status, 
            distribution_method, contract_created_at, currency_preference
        ) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(plan_id)
    .bind(user_id)
    .bind("Immature Plan")
    .bind("Description")
    .bind("0.00")
    .bind("100.00")
    .bind("pending")
    .bind("Monthly")
    .bind(now_ts)
    .bind("USDC")
    .execute(&pool)
    .await
    .unwrap();

    // 4. Start server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // 5. Attempt to claim
    let token = generate_test_token(user_id, &email);
    let client = reqwest::Client::new();

    let response = client
        .post(format!("http://{}/api/plans/{}/claim", addr, plan_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "beneficiary_email": "beneficiary@example.com"
        }))
        .send()
        .await
        .expect("Failed to send request");

    // 6. Assertions
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body: Value = response.json().await.unwrap();
    assert_eq!(
        body["error"],
        "Bad Request: Plan is not yet mature for claim"
    );
}
