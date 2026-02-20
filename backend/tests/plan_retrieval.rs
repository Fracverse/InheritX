use axum::{body::Body, http::Request};
use inheritx_backend::{create_app, db, Config};
use serde_json::Value;
use sqlx::PgPool;
use tower05::util::ServiceExt;
use uuid::Uuid;

async fn setup_db_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let pool = db::create_pool(&database_url).await.ok()?;
    db::run_migrations(&pool).await.ok()?;
    Some(pool)
}

async fn seed_test_data(pool: &PgPool) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let user_1 = Uuid::new_v4();
    let user_2 = Uuid::new_v4();
    let admin_1 = Uuid::new_v4();

    let user_1_pending_plan = Uuid::new_v4();
    let user_1_claimed_plan = Uuid::new_v4();
    let user_2_pending_plan = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3), ($4, $5, $6)",
    )
    .bind(user_1)
    .bind(format!("plan-test-{}@example.com", user_1))
    .bind("hash")
    .bind(user_2)
    .bind(format!("plan-test-{}@example.com", user_2))
    .bind("hash")
    .execute(pool)
    .await
    .expect("users should insert");

    sqlx::query("INSERT INTO admins (id, email, password_hash, role) VALUES ($1, $2, $3, $4)")
        .bind(admin_1)
        .bind(format!("admin-test-{}@example.com", admin_1))
        .bind("hash")
        .bind("super-admin")
        .execute(pool)
        .await
        .expect("admin should insert");

    sqlx::query(
        r#"
        INSERT INTO plans (id, user_id, title, description, fee, net_amount, status)
        VALUES
            ($1, $2, 'Pending Plan A', 'pending plan', 1.0, 100.0, 'pending'),
            ($3, $4, 'Claimed Plan B', 'claimed plan', 1.0, 200.0, 'claimed'),
            ($5, $6, 'Pending Plan C', 'pending plan', 1.0, 300.0, 'pending')
        "#,
    )
    .bind(user_1_pending_plan)
    .bind(user_1)
    .bind(user_1_claimed_plan)
    .bind(user_1)
    .bind(user_2_pending_plan)
    .bind(user_2)
    .execute(pool)
    .await
    .expect("plans should insert");

    (
        user_1,
        user_2,
        admin_1,
        user_1_pending_plan,
        user_1_claimed_plan,
        user_2_pending_plan,
    )
}

async fn cleanup_seed_data(pool: &PgPool, user_1: Uuid, user_2: Uuid, admin_1: Uuid) {
    sqlx::query("DELETE FROM plans WHERE user_id = $1 OR user_id = $2")
        .bind(user_1)
        .bind(user_2)
        .execute(pool)
        .await
        .expect("plans cleanup should succeed");

    sqlx::query("DELETE FROM admins WHERE id = $1")
        .bind(admin_1)
        .execute(pool)
        .await
        .expect("admins cleanup should succeed");

    sqlx::query("DELETE FROM users WHERE id = $1 OR id = $2")
        .bind(user_1)
        .bind(user_2)
        .execute(pool)
        .await
        .expect("users cleanup should succeed");
}

#[tokio::test]
async fn retrieves_plans_by_user_and_admin_scopes() {
    let Some(pool) = setup_db_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set or DB unavailable");
        return;
    };

    let (user_1, user_2, admin_1, user_1_pending_plan, _user_1_claimed_plan, user_2_pending_plan) =
        seed_test_data(&pool).await;

    let app = create_app(
        pool.clone(),
        Config {
            database_url: std::env::var("DATABASE_URL").unwrap_or_default(),
            port: 0,
        },
    )
    .await
    .expect("app should build");

    let user_all = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/plans")
                .header("x-user-id", user_1.to_string())
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(user_all.status(), 200);
    let user_all_body = axum::body::to_bytes(user_all.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let user_all_json: Value = serde_json::from_slice(&user_all_body).expect("json should parse");
    assert_eq!(user_all_json.as_array().expect("array expected").len(), 2);

    let user_pending = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/plans/pending")
                .header("x-user-id", user_1.to_string())
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(user_pending.status(), 200);
    let user_pending_body = axum::body::to_bytes(user_pending.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let user_pending_json: Value =
        serde_json::from_slice(&user_pending_body).expect("json should parse");
    assert_eq!(user_pending_json.as_array().expect("array expected").len(), 1);
    assert_eq!(
        user_pending_json
            .as_array()
            .expect("array expected")[0]["status"],
        "pending"
    );

    let single_plan = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/plans/{}", user_1_pending_plan))
                .header("x-user-id", user_1.to_string())
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(single_plan.status(), 200);

    let foreign_plan = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/plans/{}", user_2_pending_plan))
                .header("x-user-id", user_1.to_string())
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(foreign_plan.status(), 404);

    let admin_all = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/plans")
                .header("x-admin-id", admin_1.to_string())
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(admin_all.status(), 200);
    let admin_all_body = axum::body::to_bytes(admin_all.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let admin_all_json: Value = serde_json::from_slice(&admin_all_body).expect("json should parse");
    assert_eq!(admin_all_json.as_array().expect("array expected").len(), 3);

    let admin_pending = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/plans/pending")
                .header("x-admin-id", admin_1.to_string())
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(admin_pending.status(), 200);
    let admin_pending_body = axum::body::to_bytes(admin_pending.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let admin_pending_json: Value =
        serde_json::from_slice(&admin_pending_body).expect("json should parse");
    assert_eq!(admin_pending_json.as_array().expect("array expected").len(), 2);

    cleanup_seed_data(&pool, user_1, user_2, admin_1).await;
}

#[tokio::test]
async fn rejects_unauthorized_plan_access() {
    let Some(pool) = setup_db_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set or DB unavailable");
        return;
    };

    let app = create_app(
        pool,
        Config {
            database_url: std::env::var("DATABASE_URL").unwrap_or_default(),
            port: 0,
        },
    )
    .await
    .expect("app should build");

    let no_user_header = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/plans")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(no_user_header.status(), 401);

    let no_admin_header = app
        .oneshot(
            Request::builder()
                .uri("/admin/plans")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    assert_eq!(no_admin_header.status(), 401);
}
