// This file is a placeholder for helper functions and structs.
use axum::{body::Body, http::Request, Router};
use inheritx_backend::{create_app, Config};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tower::ServiceExt;

pub struct TestContext {
    pub app: Router,
    #[allow(dead_code)]
    pub pool: PgPool,
}

impl TestContext {
    #[allow(dead_code)]
    pub async fn from_env() -> Option<Self> {
        // Use a static to ensure tracing is only initialized once
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            let _ = inheritx_backend::telemetry::init_tracing();
        });

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!("Skipping integration test: DATABASE_URL is not set");
                return None;
            }
        };

        let pool = match PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
        {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("Skipping integration test: unable to connect to DATABASE_URL: {err}");
                return None;
            }
        };

        let config = Config {
            database_url,
            port: 0,
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string()),
            db_pool: inheritx_backend::config::DbPoolConfig::from_env_or_defaults(),
        };

        // Run migrations
        inheritx_backend::db::run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        let prometheus_handle = inheritx_backend::get_or_install_recorder();
        let app = create_app(pool.clone(), config, prometheus_handle)
            .await
            .expect("failed to create app");
        Some(Self { app, pool })
    }

    #[allow(dead_code)]
    pub async fn prepare_2fa(&self, user_id: uuid::Uuid, otp: &str) -> String {
        let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

        sqlx::query(
            "INSERT INTO user_2fa (user_id, otp_hash, expires_at) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO UPDATE SET otp_hash = $2, expires_at = $3",
        )
        .bind(user_id)
        .bind(otp_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .unwrap();

        otp.to_string()
    }
}

#[allow(dead_code)]
pub async fn create_test_user(pool: &PgPool, email: &str) -> sqlx::Result<uuid::Uuid> {
    let user_id = uuid::Uuid::new_v4();
    let wallet = format!("G{}", &user_id.to_string().replace("-", "")[..55]);

    sqlx::query(
        "INSERT INTO users (id, email, wallet_address, kyc_status) VALUES ($1, $2, $3, 'approved')",
    )
    .bind(user_id)
    .bind(email)
    .bind(wallet)
    .execute(pool)
    .await?;

    Ok(user_id)
}

#[allow(dead_code)]
pub async fn create_test_admin(pool: &PgPool, email: &str) -> sqlx::Result<uuid::Uuid> {
    let admin_id = uuid::Uuid::new_v4();
    let password_hash = bcrypt::hash("test_password", bcrypt::DEFAULT_COST).unwrap();

    sqlx::query(
        "INSERT INTO admins (id, email, password_hash, status) VALUES ($1, $2, $3, 'active')",
    )
    .bind(admin_id)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(admin_id)
}

/// Helper function to create an admin user and get authentication token
#[allow(dead_code)]
pub async fn create_admin_and_get_token(ctx: &mut TestContext) -> String {
    use uuid::Uuid;

    let admin_id = Uuid::new_v4();
    let email = format!("test_admin_{}@example.com", admin_id);
    let password_hash = bcrypt::hash("testpassword", bcrypt::DEFAULT_COST).unwrap();

    // Create admin user
    sqlx::query(
        r#"
        INSERT INTO admins (id, email, password_hash, role, status)
        VALUES ($1, $2, $3, 'super_admin', 'active')
        ON CONFLICT (id) DO UPDATE SET email = $2, password_hash = $3, role = 'super_admin', status = 'active'
        "#,
    )
    .bind(admin_id)
    .bind(&email)
    .bind(&password_hash)
    .execute(&ctx.pool)
    .await
    .expect("failed to create admin");

    // Login to get token
    let login_request = json!({
        "email": email,
        "password": "testpassword",
    });

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .expect("login request failed");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    json["token"].as_str().unwrap().to_string()
}
