pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub plan_cache_ttl_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/inheritx".to_string());
        let redis_url = std::env::var("REDIS_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let plan_cache_ttl_secs = std::env::var("PLAN_CACHE_TTL_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(15);

        Ok(Config {
            port,
            database_url,
            redis_url,
            plan_cache_ttl_secs,
        })
    }
}
