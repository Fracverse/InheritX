use inheritx_backend::{create_app, Config};
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::{net::TcpListener, task::JoinHandle};

pub struct TestContext {
    pub client: Client,
    pub base_url: String,
    pub pool: PgPool,
    server_handle: JoinHandle<()>,
}

impl TestContext {
    pub async fn from_env() -> Option<Self> {
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
                eprintln!(
                    "Skipping integration test: unable to connect to DATABASE_URL: {err}"
                );
                return None;
            }
        };

        let config = Config {
            database_url,
            port: 0,
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "test-jwt-secret".to_string()),
        };

        let app = create_app(pool.clone(), config)
            .await
            .expect("failed to create app");

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind test listener");
        let address = listener.local_addr().expect("failed to read local addr");

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test server exited unexpectedly");
        });

        Some(Self {
            client: Client::new(),
            base_url: format!("http://{address}"),
            pool,
            server_handle,
        })
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}