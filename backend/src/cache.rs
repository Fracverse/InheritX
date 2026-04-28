use crate::api_error::ApiError;
use redis::AsyncCommands;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Debug, Clone)]
struct InMemoryCacheEntry {
    value_json: String,
    expires_at_secs: u64,
}

#[derive(Clone)]
enum CacheBackend {
    Redis(redis::aio::ConnectionManager),
    InMemory(Arc<RwLock<HashMap<String, InMemoryCacheEntry>>>),
}

#[derive(Clone)]
pub struct CacheService {
    backend: CacheBackend,
    pub default_ttl_secs: u64,
    pub plans_ttl_secs: u64,
    pub user_profile_ttl_secs: u64,
}

impl CacheService {
    pub async fn from_env() -> Self {
        let default_ttl_secs = read_u64("CACHE_DEFAULT_TTL_SECS", 60);
        let plans_ttl_secs = read_u64("CACHE_PLANS_TTL_SECS", 90);
        let user_profile_ttl_secs = read_u64("CACHE_USER_PROFILE_TTL_SECS", 120);

        let backend = if let Ok(redis_url) = std::env::var("REDIS_URL") {
            if let Ok(client) = redis::Client::open(redis_url) {
                match client.get_connection_manager().await {
                    Ok(conn) => {
                        tracing::info!("Cache backend initialised with Redis");
                        CacheBackend::Redis(conn)
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to initialise Redis cache backend, falling back to in-memory cache");
                        CacheBackend::InMemory(Arc::new(RwLock::new(HashMap::new())))
                    }
                }
            } else {
                warn!("Invalid REDIS_URL provided, falling back to in-memory cache");
                CacheBackend::InMemory(Arc::new(RwLock::new(HashMap::new())))
            }
        } else {
            tracing::info!("REDIS_URL not set, using in-memory cache backend");
            CacheBackend::InMemory(Arc::new(RwLock::new(HashMap::new())))
        };

        Self {
            backend,
            default_ttl_secs,
            plans_ttl_secs,
            user_profile_ttl_secs,
        }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, ApiError> {
        match &self.backend {
            CacheBackend::Redis(manager) => {
                let mut conn = manager.clone();
                let cached: Option<String> = conn.get(key).await.map_err(|e| {
                    ApiError::ExternalService(format!("Redis get failed for key '{key}': {e}"))
                })?;

                match cached {
                    Some(raw) => {
                        metrics::counter!("cache_hits_total", "keyspace" => keyspace(key).to_string())
                            .increment(1);
                        let parsed = serde_json::from_str::<T>(&raw).map_err(|e| {
                            ApiError::Internal(anyhow::anyhow!(
                                "Failed to deserialize cached value for key {}: {}",
                                key,
                                e
                            ))
                        })?;
                        Ok(Some(parsed))
                    }
                    None => {
                        metrics::counter!("cache_misses_total", "keyspace" => keyspace(key).to_string())
                            .increment(1);
                        Ok(None)
                    }
                }
            }
            CacheBackend::InMemory(store) => {
                let now = now_secs();
                let maybe_value = {
                    let guard = store.read().await;
                    guard.get(key).cloned()
                };

                if let Some(entry) = maybe_value {
                    if entry.expires_at_secs > now {
                        metrics::counter!("cache_hits_total", "keyspace" => keyspace(key).to_string())
                            .increment(1);
                        let parsed = serde_json::from_str::<T>(&entry.value_json).map_err(|e| {
                            ApiError::Internal(anyhow::anyhow!(
                                "Failed to deserialize in-memory cached value for key {}: {}",
                                key,
                                e
                            ))
                        })?;
                        return Ok(Some(parsed));
                    }

                    // Expired entry cleanup.
                    let mut guard = store.write().await;
                    guard.remove(key);
                }

                metrics::counter!("cache_misses_total", "keyspace" => keyspace(key).to_string())
                    .increment(1);
                Ok(None)
            }
        }
    }

    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> Result<(), ApiError> {
        let payload = serde_json::to_string(value)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Cache serialize failed: {e}")))?;

        match &self.backend {
            CacheBackend::Redis(manager) => {
                let mut conn = manager.clone();
                conn.set_ex::<_, _, ()>(key, payload, ttl_secs)
                    .await
                    .map_err(|e| {
                        ApiError::ExternalService(format!("Redis set_ex failed for key '{key}': {e}"))
                    })?;
            }
            CacheBackend::InMemory(store) => {
                let expires_at_secs = now_secs().saturating_add(ttl_secs);
                let mut guard = store.write().await;
                guard.insert(
                    key.to_string(),
                    InMemoryCacheEntry {
                        value_json: payload,
                        expires_at_secs,
                    },
                );
            }
        }

        Ok(())
    }

    pub async fn invalidate(&self, key: &str) -> Result<(), ApiError> {
        match &self.backend {
            CacheBackend::Redis(manager) => {
                let mut conn = manager.clone();
                let _: usize = conn.del(key).await.map_err(|e| {
                    ApiError::ExternalService(format!("Redis delete failed for key '{key}': {e}"))
                })?;
            }
            CacheBackend::InMemory(store) => {
                let mut guard = store.write().await;
                guard.remove(key);
            }
        }
        Ok(())
    }

    pub async fn invalidate_prefix(&self, prefix: &str) -> Result<u64, ApiError> {
        match &self.backend {
            CacheBackend::Redis(manager) => {
                let mut conn = manager.clone();
                let pattern = format!("{prefix}*");
                let keys: Vec<String> = conn.keys(pattern).await.map_err(|e| {
                    ApiError::ExternalService(format!("Redis key lookup failed for prefix '{prefix}': {e}"))
                })?;
                let deleted = if keys.is_empty() {
                    0
                } else {
                    conn.del(keys).await.map_err(|e| {
                        ApiError::ExternalService(format!("Redis prefix delete failed: {e}"))
                    })?
                };
                Ok(deleted)
            }
            CacheBackend::InMemory(store) => {
                let mut guard = store.write().await;
                let before = guard.len();
                guard.retain(|k, _| !k.starts_with(prefix));
                Ok((before.saturating_sub(guard.len())) as u64)
            }
        }
    }
}

fn keyspace(key: &str) -> &str {
    key.split(':').next().unwrap_or("default")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn read_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(default)
}
