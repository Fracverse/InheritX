use crate::api::{PlanQuery, PlanResponse};
use redis::AsyncCommands;
use std::collections::{HashMap, HashSet};

const CACHE_NAMESPACE: &str = "plans:v1";

#[derive(Debug)]
pub enum CacheError {
    Redis(redis::RedisError),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Redis(err) => write!(f, "{err}"),
            Self::Serialization(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<redis::RedisError> for CacheError {
    fn from(value: redis::RedisError) -> Self {
        Self::Redis(value)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

#[derive(Clone)]
pub enum PlanCache {
    Disabled,
    Redis(RedisPlanCache),
    Memory(std::sync::Arc<tokio::sync::Mutex<MemoryPlanCache>>),
}

#[derive(Clone)]
pub struct RedisPlanCache {
    client: redis::Client,
    ttl_secs: u64,
}

#[derive(Default)]
pub struct MemoryPlanCache {
    values: HashMap<String, String>,
    sets: HashMap<String, HashSet<String>>,
}

impl PlanCache {
    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn from_redis_url(redis_url: Option<&str>, ttl_secs: u64) -> Result<Self, CacheError> {
        match redis_url {
            Some(url) if !url.trim().is_empty() => {
                let client = redis::Client::open(url)?;
                Ok(Self::Redis(RedisPlanCache { client, ttl_secs }))
            }
            _ => Ok(Self::Disabled),
        }
    }

    pub fn memory() -> Self {
        Self::Memory(std::sync::Arc::new(tokio::sync::Mutex::new(
            MemoryPlanCache::default(),
        )))
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    pub async fn get_plans(
        &self,
        query: &PlanQuery,
    ) -> Result<Option<Vec<PlanResponse>>, CacheError> {
        let cache_key = cache_key(query);

        match self {
            Self::Disabled => Ok(None),
            Self::Redis(redis_cache) => {
                let mut conn = redis_cache
                    .client
                    .get_multiplexed_async_connection()
                    .await?;
                let cached: Option<String> = conn.get(cache_key).await?;
                match cached {
                    Some(payload) => Ok(Some(serde_json::from_str(&payload)?)),
                    None => Ok(None),
                }
            }
            Self::Memory(store) => {
                let store = store.lock().await;
                match store.values.get(&cache_key) {
                    Some(payload) => Ok(Some(serde_json::from_str(payload)?)),
                    None => Ok(None),
                }
            }
        }
    }

    pub async fn set_plans(
        &self,
        query: &PlanQuery,
        plans: &[PlanResponse],
    ) -> Result<(), CacheError> {
        let cache_key = cache_key(query);
        let serialized = serde_json::to_string(plans)?;
        let index_keys = query_index_keys(query);

        match self {
            Self::Disabled => Ok(()),
            Self::Redis(redis_cache) => {
                let mut conn = redis_cache
                    .client
                    .get_multiplexed_async_connection()
                    .await?;
                let _: () = conn
                    .set_ex(&cache_key, serialized, redis_cache.ttl_secs)
                    .await?;

                for index_key in index_keys {
                    let _: usize = conn.sadd(&index_key, &cache_key).await?;
                    let _: bool = conn.expire(&index_key, redis_cache.ttl_secs as i64).await?;
                }

                Ok(())
            }
            Self::Memory(store) => {
                let mut store = store.lock().await;
                store.values.insert(cache_key.clone(), serialized);
                for index_key in index_keys {
                    store
                        .sets
                        .entry(index_key)
                        .or_default()
                        .insert(cache_key.clone());
                }
                Ok(())
            }
        }
    }

    pub async fn invalidate_queries(
        &self,
        owner_address: &str,
        beneficiary_addresses: &[String],
    ) -> Result<(), CacheError> {
        let owner = normalize_filter_value(owner_address);
        let beneficiary_addresses: Vec<String> = beneficiary_addresses
            .iter()
            .map(|value| normalize_filter_value(value))
            .collect();

        let mut index_keys: HashSet<String> = HashSet::from([all_queries_index_key()]);
        index_keys.insert(owner_index_key(&owner));

        for beneficiary in &beneficiary_addresses {
            index_keys.insert(beneficiary_index_key(beneficiary));
            index_keys.insert(owner_beneficiary_index_key(&owner, beneficiary));
        }

        match self {
            Self::Disabled => Ok(()),
            Self::Redis(redis_cache) => {
                let mut conn = redis_cache
                    .client
                    .get_multiplexed_async_connection()
                    .await?;
                let mut keys_to_delete: HashSet<String> = HashSet::new();

                for index_key in &index_keys {
                    let members: Vec<String> = conn.smembers(index_key).await?;
                    keys_to_delete.extend(members);
                }

                keys_to_delete.extend(index_keys);

                for key in keys_to_delete {
                    let _: usize = conn.del(key).await?;
                }

                Ok(())
            }
            Self::Memory(store) => {
                let mut store = store.lock().await;
                let mut keys_to_delete: HashSet<String> = HashSet::new();

                for index_key in &index_keys {
                    if let Some(members) = store.sets.get(index_key) {
                        keys_to_delete.extend(members.iter().cloned());
                    }
                }

                for cache_key in keys_to_delete {
                    store.values.remove(&cache_key);
                }

                for index_key in index_keys {
                    store.sets.remove(&index_key);
                }

                Ok(())
            }
        }
    }
}

pub(crate) fn cache_key(query: &PlanQuery) -> String {
    format!(
        "{CACHE_NAMESPACE}:query:owner={}:beneficiary={}",
        normalize_optional_filter(query.owner.as_deref()),
        normalize_optional_filter(query.beneficiary.as_deref()),
    )
}

fn query_index_keys(query: &PlanQuery) -> Vec<String> {
    let owner = query.owner.as_deref().map(normalize_filter_value);
    let beneficiary = query.beneficiary.as_deref().map(normalize_filter_value);

    let mut keys: HashSet<String> = HashSet::from([all_queries_index_key()]);

    if let Some(owner) = owner.as_ref() {
        keys.insert(owner_index_key(owner));
    }

    if let Some(beneficiary) = beneficiary.as_ref() {
        keys.insert(beneficiary_index_key(beneficiary));
    }

    if let (Some(owner), Some(beneficiary)) = (owner.as_ref(), beneficiary.as_ref()) {
        keys.insert(owner_beneficiary_index_key(owner, beneficiary));
    }

    keys.into_iter().collect()
}

fn normalize_optional_filter(value: Option<&str>) -> String {
    value
        .map(normalize_filter_value)
        .unwrap_or_else(|| "all".to_string())
}

fn normalize_filter_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn all_queries_index_key() -> String {
    format!("{CACHE_NAMESPACE}:index:all")
}

fn owner_index_key(owner: &str) -> String {
    format!("{CACHE_NAMESPACE}:index:owner:{owner}")
}

fn beneficiary_index_key(beneficiary: &str) -> String {
    format!("{CACHE_NAMESPACE}:index:beneficiary:{beneficiary}")
}

fn owner_beneficiary_index_key(owner: &str, beneficiary: &str) -> String {
    format!("{CACHE_NAMESPACE}:index:pair:{owner}:{beneficiary}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{BeneficiaryResponse, PlanResponse};
    use chrono::Utc;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn sample_plan(owner: &str, beneficiary: &str) -> PlanResponse {
        PlanResponse {
            id: Uuid::new_v4(),
            owner_address: owner.to_string(),
            token_address: "USDC".to_string(),
            amount: Decimal::from(1000),
            grace_period: 3600,
            grace_period_seconds: 3600,
            earn_yield: true,
            last_ping: 1_718_000_000,
            is_active: true,
            status: "ACTIVE".to_string(),
            yield_rate_bps: 500,
            accrued_yield: 42.5,
            created_at: Utc::now(),
            beneficiaries: vec![BeneficiaryResponse {
                id: Uuid::new_v4(),
                plan_id: Uuid::new_v4(),
                wallet_address: beneficiary.to_string(),
                allocation_bps: 10_000,
                fiat_anchor_info: "bank-usd".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn memory_cache_round_trips_cached_queries() {
        let cache = PlanCache::memory();
        let query = PlanQuery {
            owner: Some("GOWNER".to_string()),
            beneficiary: Some("GBENEFICIARY".to_string()),
        };
        let plans = vec![sample_plan("GOWNER", "GBENEFICIARY")];

        cache.set_plans(&query, &plans).await.unwrap();
        let cached = cache.get_plans(&query).await.unwrap();

        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].owner_address, "GOWNER");
    }

    #[tokio::test]
    async fn invalidation_clears_related_query_variants() {
        let cache = PlanCache::memory();
        let plans = vec![sample_plan("GOWNER", "GBENEFICIARY")];

        let queries = vec![
            PlanQuery {
                owner: None,
                beneficiary: None,
            },
            PlanQuery {
                owner: Some("GOWNER".to_string()),
                beneficiary: None,
            },
            PlanQuery {
                owner: None,
                beneficiary: Some("GBENEFICIARY".to_string()),
            },
            PlanQuery {
                owner: Some("GOWNER".to_string()),
                beneficiary: Some("GBENEFICIARY".to_string()),
            },
        ];

        for query in &queries {
            cache.set_plans(query, &plans).await.unwrap();
        }

        cache
            .invalidate_queries("GOWNER", &[String::from("GBENEFICIARY")])
            .await
            .unwrap();

        for query in &queries {
            assert!(cache.get_plans(query).await.unwrap().is_none());
        }
    }

    #[test]
    fn cache_keys_are_normalized() {
        let query = PlanQuery {
            owner: Some("  GOwner ".to_string()),
            beneficiary: Some(" GBeneficiary ".to_string()),
        };

        assert_eq!(
            cache_key(&query),
            "plans:v1:query:owner=gowner:beneficiary=gbeneficiary"
        );
    }
}
