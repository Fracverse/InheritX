use inheritx_backend::{create_router, AppState};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[tokio::test]
async fn test_router_compiles() {
    let state = Arc::new(AppState {
        anchor: Arc::new(inheritx_backend::stellar_anchor::AnchorRegistry::new()),
        kyc_tx: tokio::sync::broadcast::channel(16).0,
        db_pool: PgPoolOptions::new()
            .connect_lazy("postgres://postgres:password@localhost/test")
            .unwrap(),
        kyc_webhook_secret: None,
    });

    let _app = create_router(state);

    // Router created successfully!
}
