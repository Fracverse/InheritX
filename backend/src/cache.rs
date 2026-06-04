//! Caching infrastructure for InheritX.
//!
//! This module provides two layers of caching:
//! 1. **HTTP Response Caching**: Stateless ETag generation and conditional GET helpers.
//! 2. **Application Data Caching**: Redis or In-memory backed service for shared data.

use crate::api_error::ApiError;
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tracing::warn;

// =============================================================================
// Layer 1: HTTP Response Caching (Stateless ETags)
// =============================================================================

/// Compute a strong ETag for any serializable value.
///
/// The ETag is the SHA-256 hash of the canonical JSON representation,
/// Base64URL-encoded (no padding), wrapped in double quotes as required
/// by RFC 7232: `"<hash>"`.
pub fn compute_etag<T: Serialize>(data: &T) -> String {
    let json = serde_json::to_string(data).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let hash = hasher.finalize();
    format!("\"{}\"", URL_SAFE_NO_PAD.encode(hash))
}

/// Extract the raw value of the `If-None-Match` request header, if present.
pub fn parse_if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
}

/// Return `true` when the supplied ETag matches the client's `If-None-Match`
/// header value, meaning the cached response is still fresh.
pub fn etag_matches(etag: &str, if_none_match: &str) -> bool {
    let inm = if_none_match.trim();
    if inm == "*" {
        return true;
    }
    inm.split(',')
        .map(|s| s.trim())
        .any(|candidate| candidate == etag)
}

/// Convenience wrapper: return `true` when the request headers indicate the
/// response is still fresh (i.e. the ETag has not changed).
pub fn is_not_modified(headers: &HeaderMap, etag: &str) -> bool {
    match parse_if_none_match(headers) {
        Some(inm) => etag_matches(etag, &inm),
        None => false,
    }
}

/// `public, max-age=<seconds>, must-revalidate`
pub fn cache_control_public(max_age_secs: u32) -> HeaderValue {
    HeaderValue::from_str(&format!("public, max-age={max_age_secs}, must-revalidate")).unwrap()
}

/// `private, max-age=<seconds>, must-revalidate`
pub fn cache_control_private(max_age_secs: u32) -> HeaderValue {
    HeaderValue::from_str(&format!("private, max-age={max_age_secs}, must-revalidate")).unwrap()
}

/// `no-store`
pub fn cache_control_no_store() -> HeaderValue {
    HeaderValue::from_static("no-store")
}

/// Build a `304 Not Modified` response with the given ETag.
pub fn not_modified_response(etag: &str) -> Response {
    let etag_value = HeaderValue::from_str(etag).unwrap_or_else(|_| HeaderValue::from_static(""));
    (
        StatusCode::NOT_MODIFIED,
        [
            (header::ETAG, etag_value),
            (header::CACHE_CONTROL, cache_control_private(60)),
        ],
    )
        .into_response()
}

/// Build a `304 Not Modified` response with a custom `Cache-Control` header.
pub fn not_modified_response_with_cc(etag: &str, cache_control: HeaderValue) -> Response {
    let etag_value = HeaderValue::from_str(etag).unwrap_or_else(|_| HeaderValue::from_static(""));
    (
        StatusCode::NOT_MODIFIED,
        [
            (header::ETAG, etag_value),
            (header::CACHE_CONTROL, cache_control),
        ],
    )
        .into_response()
}

/// Inject `ETag`, `Cache-Control`, and `Vary: Accept-Encoding` headers into
/// an existing `200 OK` response.
pub fn apply_cache_headers(response: &mut Response, etag: &str, cache_control: HeaderValue) {
    let headers = response.headers_mut();
    if let Ok(etag_value) = HeaderValue::from_str(etag) {
        headers.insert(header::ETAG, etag_value);
    }
    headers.insert(header::CACHE_CONTROL, cache_control);
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
}

// =============================================================================
// Layer 2: Application Data Caching (Shared Service)
// =============================================================================

#[derive(Debug, Clone)]
struct InMemoryCacheEntry {
    value_json: String,
    expires_at_secs: u64,
}

#[derive(Clone)]
enum CacheBackend {
    Redis(Box<redis::aio::ConnectionManager>),
    InMemory(Arc<RwLock<HashMap<String, InMemoryCacheEntry>>>),
}

#[derive(Clone)]
pub struct CacheService {
    backend: CacheBackend,
    key_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
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
                        CacheBackend::Redis(Box::new(conn))
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
            key_locks: Arc::new(Mutex::new(HashMap::new())),
            default_ttl_secs,
            plans_ttl_secs,
            user_profile_ttl_secs,
        }
    }

    async fn lock_for_key(&self, key: &str) -> Arc<Mutex<()>> {
        let mut locks = self.key_locks.lock().await;
        locks
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
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
                        ApiError::ExternalService(format!(
                            "Redis set_ex failed for key '{key}': {e}"
                        ))
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

    /// Get a cached JSON value or compute it once, coalescing concurrent misses.
    pub async fn get_or_set_json<T, F, Fut>(
        &self,
        key: &str,
        ttl_secs: u64,
        compute: F,
    ) -> Result<T, ApiError>
    where
        T: Serialize + DeserializeOwned + Clone + Send + 'static,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, ApiError>> + Send,
    {
        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }

        let key_lock = self.lock_for_key(key).await;
        let _guard = key_lock.lock().await;

        if let Some(cached) = self.get_json::<T>(key).await? {
            return Ok(cached);
        }

        let value = compute().await?;
        self.set_json(key, &value, ttl_secs).await?;
        Ok(value)
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
                    ApiError::ExternalService(format!(
                        "Redis key lookup failed for prefix '{prefix}': {e}"
                    ))
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

    /// Invalidate all user-related caches when a user is modified.
    pub async fn invalidate_user_caches(&self, user_id: &str) -> Result<u64, ApiError> {
        self.invalidate_prefix(&format!("user:{user_id}:")).await
    }

    /// Invalidate all plan-related caches when a plan is modified.
    pub async fn invalidate_plan_caches(&self, plan_id: &str) -> Result<u64, ApiError> {
        self.invalidate_prefix(&format!("plan:{plan_id}:")).await
    }

    /// Invalidate all loan-related caches when a loan is modified.
    pub async fn invalidate_loan_caches(&self, loan_id: &str) -> Result<u64, ApiError> {
        self.invalidate_prefix(&format!("loan:{loan_id}:")).await
    }

    /// Invalidate all notification caches for a user.
    pub async fn invalidate_notification_caches(&self, user_id: &str) -> Result<u64, ApiError> {
        self.invalidate_prefix(&format!("notifications:{user_id}:"))
            .await
    }

    /// Invalidate all audit log caches.
    pub async fn invalidate_audit_log_caches(&self) -> Result<u64, ApiError> {
        self.invalidate_prefix("audit_logs:").await
    }

    /// Invalidate all collateral-related caches.
    pub async fn invalidate_collateral_caches(&self, user_id: &str) -> Result<u64, ApiError> {
        self.invalidate_prefix(&format!("collateral:{user_id}:"))
            .await
    }

    /// Invalidate price feed caches.
    pub async fn invalidate_price_caches(&self) -> Result<u64, ApiError> {
        self.invalidate_prefix("prices:").await
    }

    /// Invalidate yield caches.
    pub async fn invalidate_yield_caches(&self) -> Result<u64, ApiError> {
        self.invalidate_prefix("yields:").await
    }

    /// Invalidate all caches (used for system-wide updates).
    pub async fn invalidate_all(&self) -> Result<u64, ApiError> {
        match &self.backend {
            CacheBackend::Redis(manager) => {
                let mut conn = manager.clone();
                let keys: Vec<String> = conn.keys("*").await.map_err(|e| {
                    ApiError::ExternalService(format!("Redis key lookup failed: {e}"))
                })?;
                let deleted = if keys.is_empty() {
                    0
                } else {
                    conn.del(keys).await.map_err(|e| {
                        ApiError::ExternalService(format!("Redis flush failed: {e}"))
                    })?
                };
                Ok(deleted)
            }
            CacheBackend::InMemory(store) => {
                let mut guard = store.write().await;
                let count = guard.len() as u64;
                guard.clear();
                Ok(count)
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

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn etag_is_deterministic() {
        let data = json!({ "id": 1, "name": "test" });
        let e1 = compute_etag(&data);
        let e2 = compute_etag(&data);
        assert_eq!(e1, e2, "same data must produce the same ETag");
        assert!(e1.starts_with('"') && e1.ends_with('"'));
    }

    #[test]
    fn different_data_produces_different_etag() {
        let a = json!({ "id": 1 });
        let b = json!({ "id": 2 });
        assert_ne!(compute_etag(&a), compute_etag(&b));
    }

    #[test]
    fn etag_matches_exact() {
        let etag = compute_etag(&json!({ "x": 42 }));
        assert!(etag_matches(&etag, &etag));
    }

    #[test]
    fn etag_matches_wildcard() {
        let etag = compute_etag(&json!({ "x": 42 }));
        assert!(etag_matches(&etag, "*"));
    }

    #[test]
    fn etag_no_match_on_stale() {
        let etag_new = compute_etag(&json!({ "x": 42 }));
        let etag_old = compute_etag(&json!({ "x": 1 }));
        assert!(!etag_matches(&etag_new, &etag_old));
    }

    #[test]
    fn etag_matches_multi_value() {
        let etag = compute_etag(&json!({ "x": 42 }));
        let stale = compute_etag(&json!({ "x": 1 }));
        let header = format!("{stale}, {etag}");
        assert!(etag_matches(&etag, &header));
    }

    #[test]
    fn is_not_modified_true_on_match() {
        let data = json!({ "id": 99 });
        let etag = compute_etag(&data);
        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, HeaderValue::from_str(&etag).unwrap());
        assert!(is_not_modified(&headers, &etag));
    }

    #[test]
    fn is_not_modified_false_when_no_header() {
        let etag = compute_etag(&json!({ "id": 99 }));
        let headers = HeaderMap::new();
        assert!(!is_not_modified(&headers, &etag));
    }

    #[test]
    fn cache_control_public_format() {
        let v = cache_control_public(300);
        assert_eq!(v.to_str().unwrap(), "public, max-age=300, must-revalidate");
    }

    #[test]
    fn cache_control_private_format() {
        let v = cache_control_private(60);
        assert_eq!(v.to_str().unwrap(), "private, max-age=60, must-revalidate");
    }

    #[test]
    fn cache_control_no_store_format() {
        let v = cache_control_no_store();
        assert_eq!(v.to_str().unwrap(), "no-store");
    }

    #[test]
    fn apply_cache_headers_injects_all_three() {
        let mut response = StatusCode::OK.into_response();
        let etag = compute_etag(&json!({ "v": 1 }));
        apply_cache_headers(&mut response, &etag, cache_control_private(120));

        let hdrs = response.headers();
        assert!(hdrs.contains_key(header::ETAG));
        assert!(hdrs.contains_key(header::CACHE_CONTROL));
        assert!(hdrs.contains_key(header::VARY));
    }

    #[tokio::test]
    async fn get_or_set_json_coalesces_concurrent_misses() {
        let cache = Arc::new(CacheService::from_env().await);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..5 {
            let cache = cache.clone();
            let counter = counter.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_set_json("test:stampede", 60, || async {
                        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Ok(json!({ "value": 42 }))
                    })
                    .await
            }));
        }

        for handle in handles {
            let value: serde_json::Value = handle.await.unwrap().unwrap();
            assert_eq!(value["value"], 42);
        }

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
