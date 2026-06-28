pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub sentry_dsn: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/inheritx".to_string());
        let sentry_dsn = std::env::var("SENTRY_DSN").ok();
        Ok(Config { port, database_url, sentry_dsn })
    }
}
