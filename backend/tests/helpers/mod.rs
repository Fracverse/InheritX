// This file is a placeholder for helper functions and structs.
use axum::Router;
use inheritx_backend::{create_app, Config};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub struct TestContext {
    pub app: Router,
    #[allow(dead_code)]
    pub pool: PgPool,
}

impl TestContext {
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
        };

        // Run migrations
        inheritx_backend::db::run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        let app = create_app(pool.clone(), config)
            .await
            .expect("failed to create app");
        Some(Self { app, pool })
    }

    #[allow(dead_code)]
    pub async fn prepare_2fa(&self, user_id: uuid::Uuid, otp: &str) -> String {
        let otp_hash = bcrypt::hash(otp, bcrypt::DEFAULT_COST).unwrap();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

        sqlx::query(
            "INSERT INTO user_2fa (user_id, otp_hash, expires_at) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO UPDATE SET otp_hash = $2, expires_at = $3"
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

/// Create a test user and return their ID
pub async fn create_test_user(pool: &PgPool, email: &str) -> sqlx::Result<uuid::Uuid> {
    let user_id = uuid::Uuid::new_v4();
    let wallet = format!("G{}", &user_id.to_string().replace("-", "")[..55]);
    
    sqlx::query(
        "INSERT INTO users (id, email, wallet_address, kyc_status) VALUES ($1, $2, $3, 'approved')"
    )
    .bind(user_id)
    .bind(email)
    .bind(wallet)
    .execute(pool)
    .await?;
    
    Ok(user_id)
}

/// Create a test admin and return their ID
pub async fn create_test_admin(pool: &PgPool, email: &str) -> sqlx::Result<uuid::Uuid> {
    let admin_id = uuid::Uuid::new_v4();
    let password_hash = bcrypt::hash("test_password", bcrypt::DEFAULT_COST).unwrap();
    
    sqlx::query(
        "INSERT INTO admins (id, email, password_hash, status) VALUES ($1, $2, $3, 'active')"
    )
    .bind(admin_id)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await?;
    
    Ok(admin_id)
}
