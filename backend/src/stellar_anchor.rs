use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KycTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

impl KycTier {
    pub fn default_daily_limit_usd(&self) -> f64 {
        match self {
            KycTier::Tier0 => 0.0,
            KycTier::Tier1 => 2_000.0,
            KycTier::Tier2 => 10_000.0, // $10,000 USD/day for Tier 2 KYC
            KycTier::Tier3 => 50_000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum FiatLimitError {
    #[error("Daily fiat payout limit exceeded for beneficiary {beneficiary_address} (Tier: {tier:?}). Limit: ${limit:.2}, last 24h used: ${used_24h:.2}, requested: ${requested:.2}")]
    LimitExceeded {
        beneficiary_address: String,
        tier: KycTier,
        limit: f64,
        used_24h: f64,
        requested: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayoutRequest {
    pub beneficiary_address: String,
    pub beneficiary_name: String,
    pub token: String,
    pub token_amount: f64,
    pub fiat_currency: String,
    pub bank_name: String,
    pub account_number: String,
    #[serde(default)]
    pub user_tier: Option<KycTier>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnchorPayoutStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayout {
    pub id: String,
    pub request: AnchorPayoutRequest,
    pub exchange_rate: f64,
    pub fiat_amount: f64,
    pub anchor_fee_usd: f64,
    pub status: AnchorPayoutStatus,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Deserialize)]
struct AnchorApiResponse {
    id: Option<String>,
    transaction_id: Option<String>,
    status: Option<String>,
    exchange_rate: Option<f64>,
    fiat_amount: Option<f64>,
    fee: Option<f64>,
    message: Option<String>,
}

pub struct AnchorRegistry {
    client: Client,
    api_url: String,
    payout_history: Arc<dashmap::DashMap<String, Vec<AnchorPayout>>>,
    tier_limits: Arc<dashmap::DashMap<KycTier, f64>>,
}

impl AnchorRegistry {
    pub fn new(api_url: String) -> Self {
        let registry = Self {
            client: Client::new(),
            api_url,
            payout_history: Arc::new(dashmap::DashMap::new()),
            tier_limits: Arc::new(dashmap::DashMap::new()),
        };

        registry.tier_limits.insert(KycTier::Tier0, KycTier::Tier0.default_daily_limit_usd());
        registry.tier_limits.insert(KycTier::Tier1, KycTier::Tier1.default_daily_limit_usd());
        registry.tier_limits.insert(KycTier::Tier2, KycTier::Tier2.default_daily_limit_usd());
        registry.tier_limits.insert(KycTier::Tier3, KycTier::Tier3.default_daily_limit_usd());

        registry
    }

    /// Sets a custom daily limit in USD for a specific KYC tier.
    pub fn set_tier_limit(&self, tier: KycTier, limit_usd: f64) {
        self.tier_limits.insert(tier, limit_usd);
    }

    /// Returns the active daily limit in USD for a specific KYC tier.
    pub fn get_tier_limit(&self, tier: KycTier) -> f64 {
        self.tier_limits
            .get(&tier)
            .map(|r| *r)
            .unwrap_or_else(|| tier.default_daily_limit_usd())
    }

    /// Sums the beneficiary's total payout volume over the last 24 hours.
    /// Excludes failed payouts.
    pub fn get_24h_payout_volume(&self, beneficiary_address: &str) -> f64 {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        if let Some(payouts) = self.payout_history.get(beneficiary_address) {
            payouts
                .iter()
                .filter(|p| {
                    if p.status == AnchorPayoutStatus::Failed {
                        return false;
                    }
                    if let Ok(ts) = DateTime::parse_from_rfc3339(&p.created_at) {
                        ts.with_timezone(&Utc) >= cutoff
                    } else {
                        false
                    }
                })
                .map(|p| {
                    if p.fiat_amount > 0.0 {
                        p.fiat_amount
                    } else {
                        p.request.token_amount
                    }
                })
                .sum()
        } else {
            0.0
        }
    }

    /// Verifies whether a requested payout volume would exceed the user's tier limit.
    pub fn check_daily_limit(
        &self,
        beneficiary_address: &str,
        requested_amount: f64,
        tier: KycTier,
    ) -> Result<(), FiatLimitError> {
        let limit = self.get_tier_limit(tier);
        let used_24h = self.get_24h_payout_volume(beneficiary_address);
        if used_24h + requested_amount > limit {
            Err(FiatLimitError::LimitExceeded {
                beneficiary_address: beneficiary_address.to_string(),
                tier,
                limit,
                used_24h,
                requested: requested_amount,
            })
        } else {
            Ok(())
        }
    }

    /// Stores a completed, processing, or pending payout in the in-memory history.
    pub fn record_payout(&self, payout: AnchorPayout) {
        self.payout_history
            .entry(payout.request.beneficiary_address.clone())
            .or_default()
            .push(payout);
    }

    pub async fn create_payout(self: &Arc<Self>, req: AnchorPayoutRequest) -> AnchorPayout {
        let tier = req.user_tier.unwrap_or(KycTier::Tier2);

        // Check 24h cumulative payout limit before initiating off-ramp
        if let Err(limit_err) = self.check_daily_limit(&req.beneficiary_address, req.token_amount, tier) {
            let now = Utc::now().to_rfc3339();
            warn!(
                beneficiary = %req.beneficiary_address,
                tier = ?tier,
                error = %limit_err,
                "Payout rejected: daily fiat tier limit exceeded"
            );
            return AnchorPayout {
                id: String::new(),
                request: req,
                exchange_rate: 1.0,
                fiat_amount: 0.0,
                anchor_fee_usd: 0.0,
                status: AnchorPayoutStatus::Failed,
                created_at: now.clone(),
                updated_at: now,
                error_message: Some(limit_err.to_string()),
            };
        }

        let url = format!("{}/transactions/send", self.api_url.trim_end_matches('/'));

        let payload = serde_json::json!({
            "beneficiary_address": req.beneficiary_address,
            "beneficiary_name": req.beneficiary_name,
            "token": req.token,
            "token_amount": req.token_amount,
            "fiat_currency": req.fiat_currency,
            "bank_name": req.bank_name,
            "account_number": req.account_number,
        });

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<AnchorApiResponse>().await {
                        Ok(api_resp) => {
                            if let Some(msg) = &api_resp.message {
                                warn!(message = %msg, "Anchor API response message");
                            }
                            let status = match api_resp.status.as_deref() {
                                Some("completed") => AnchorPayoutStatus::Completed,
                                Some("processing") | Some("pending") => {
                                    AnchorPayoutStatus::Processing
                                }
                                Some("failed") => AnchorPayoutStatus::Failed,
                                _ => AnchorPayoutStatus::Processing,
                            };

                            let now = Utc::now().to_rfc3339();
                            let fiat_amount = api_resp.fiat_amount.unwrap_or(req.token_amount);
                            let payout = AnchorPayout {
                                id: api_resp.id.or(api_resp.transaction_id).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                request: req,
                                exchange_rate: api_resp.exchange_rate.unwrap_or(1.0),
                                fiat_amount,
                                anchor_fee_usd: api_resp.fee.unwrap_or(0.0),
                                status,
                                created_at: now.clone(),
                                updated_at: now,
                                error_message: None,
                            };

                            if payout.status != AnchorPayoutStatus::Failed {
                                self.record_payout(payout.clone());
                            }

                            payout
                        }
                        Err(e) => {
                            let now = Utc::now().to_rfc3339();
                            warn!(
                                anchor_url = %url,
                                error = %e,
                                "Failed to parse anchor API response"
                            );
                            AnchorPayout {
                                id: String::new(),
                                request: req,
                                exchange_rate: 1.0,
                                fiat_amount: 0.0,
                                anchor_fee_usd: 0.0,
                                status: AnchorPayoutStatus::Failed,
                                created_at: now.clone(),
                                updated_at: now,
                                error_message: Some(format!("Failed to parse anchor response: {}", e)),
                            }
                        }
                    }
                } else {
                    let status_code = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let now = Utc::now().to_rfc3339();
                    error!(
                        anchor_url = %url,
                        status = %status_code,
                        body = %body,
                        "Anchor API returned error"
                    );
                    AnchorPayout {
                        id: String::new(),
                        request: req,
                        exchange_rate: 1.0,
                        fiat_amount: 0.0,
                        anchor_fee_usd: 0.0,
                        status: AnchorPayoutStatus::Failed,
                        created_at: now.clone(),
                        updated_at: now,
                        error_message: Some(format!("Anchor API error {}: {}", status_code, body)),
                    }
                }
            }
            Err(e) => {
                let now = Utc::now().to_rfc3339();
                error!(
                    anchor_url = %url,
                    error = %e,
                    "Failed to reach anchor API"
                );
                AnchorPayout {
                    id: String::new(),
                    request: req,
                    exchange_rate: 1.0,
                    fiat_amount: 0.0,
                    anchor_fee_usd: 0.0,
                    status: AnchorPayoutStatus::Failed,
                    created_at: now.clone(),
                    updated_at: now,
                    error_message: Some(format!("Failed to reach anchor API: {}", e)),
                }
            }
        }
    }

    pub fn get_payout(&self, id: &str) -> Option<AnchorPayout> {
        for entry in self.payout_history.iter() {
            if let Some(payout) = entry.value().iter().find(|p| p.id == id) {
                return Some(payout.clone());
            }
        }
        None
    }

    pub fn list_payouts(&self, address: Option<String>) -> Vec<AnchorPayout> {
        if let Some(addr) = address {
            self.payout_history
                .get(&addr)
                .map(|p| p.clone())
                .unwrap_or_default()
        } else {
            let mut all = Vec::new();
            for entry in self.payout_history.iter() {
                all.extend(entry.value().clone());
            }
            all
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_request(address: &str, amount: f64, tier: Option<KycTier>) -> AnchorPayoutRequest {
        AnchorPayoutRequest {
            beneficiary_address: address.to_string(),
            beneficiary_name: "Alice Doe".to_string(),
            token: "USDC".to_string(),
            token_amount: amount,
            fiat_currency: "USD".to_string(),
            bank_name: "Test Bank".to_string(),
            account_number: "123456789".to_string(),
            user_tier: tier,
        }
    }

    #[test]
    fn test_tier_limits_defaults() {
        assert_eq!(KycTier::Tier0.default_daily_limit_usd(), 0.0);
        assert_eq!(KycTier::Tier1.default_daily_limit_usd(), 2_000.0);
        assert_eq!(KycTier::Tier2.default_daily_limit_usd(), 10_000.0);
        assert_eq!(KycTier::Tier3.default_daily_limit_usd(), 50_000.0);
    }

    #[test]
    fn test_24h_volume_and_limit_rejection() {
        let registry = AnchorRegistry::new("http://localhost:8081".to_string());
        let addr = "GACCOUNTTEST123";

        // Initial 24h volume is 0
        assert_eq!(registry.get_24h_payout_volume(addr), 0.0);

        // Tier 2 limit is $10,000 USD/day
        assert!(registry.check_daily_limit(addr, 5_000.0, KycTier::Tier2).is_ok());

        // Attempting $10,001 on fresh account should exceed Tier 2
        let err = registry.check_daily_limit(addr, 10_001.0, KycTier::Tier2).unwrap_err();
        match err {
            FiatLimitError::LimitExceeded { limit, requested, used_24h, .. } => {
                assert_eq!(limit, 10_000.0);
                assert_eq!(requested, 10_001.0);
                assert_eq!(used_24h, 0.0);
            }
        }

        // Record a successful payout of $6,000
        let now = Utc::now().to_rfc3339();
        let payout1 = AnchorPayout {
            id: "payout-1".to_string(),
            request: dummy_request(addr, 6_000.0, Some(KycTier::Tier2)),
            exchange_rate: 1.0,
            fiat_amount: 6_000.0,
            anchor_fee_usd: 5.0,
            status: AnchorPayoutStatus::Completed,
            created_at: now.clone(),
            updated_at: now,
            error_message: None,
        };
        registry.record_payout(payout1);

        // 24h volume is now 6000
        assert_eq!(registry.get_24h_payout_volume(addr), 6_000.0);

        // An additional $4,000 should be allowed (6000 + 4000 = 10000 <= 10000)
        assert!(registry.check_daily_limit(addr, 4_000.0, KycTier::Tier2).is_ok());

        // An additional $4,001 should be rejected (6000 + 4001 = 10001 > 10000)
        assert!(registry.check_daily_limit(addr, 4_001.0, KycTier::Tier2).is_err());
    }

    #[test]
    fn test_expired_payouts_outside_24h_excluded() {
        let registry = AnchorRegistry::new("http://localhost:8081".to_string());
        let addr = "GACCOUNTTEST456";

        // Record payout from 25 hours ago
        let old_time = (Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
        let old_payout = AnchorPayout {
            id: "payout-old".to_string(),
            request: dummy_request(addr, 9_000.0, Some(KycTier::Tier2)),
            exchange_rate: 1.0,
            fiat_amount: 9_000.0,
            anchor_fee_usd: 5.0,
            status: AnchorPayoutStatus::Completed,
            created_at: old_time.clone(),
            updated_at: old_time,
            error_message: None,
        };
        registry.record_payout(old_payout);

        // Payout outside 24h does not count towards current 24h volume
        assert_eq!(registry.get_24h_payout_volume(addr), 0.0);
        assert!(registry.check_daily_limit(addr, 10_000.0, KycTier::Tier2).is_ok());
    }

    #[test]
    fn test_failed_payouts_do_not_count_towards_limit() {
        let registry = AnchorRegistry::new("http://localhost:8081".to_string());
        let addr = "GACCOUNTTEST789";

        let now = Utc::now().to_rfc3339();
        let failed_payout = AnchorPayout {
            id: "payout-failed".to_string(),
            request: dummy_request(addr, 8_000.0, Some(KycTier::Tier2)),
            exchange_rate: 1.0,
            fiat_amount: 8_000.0,
            anchor_fee_usd: 0.0,
            status: AnchorPayoutStatus::Failed,
            created_at: now.clone(),
            updated_at: now,
            error_message: Some("Bank network timeout".to_string()),
        };
        registry.record_payout(failed_payout);

        assert_eq!(registry.get_24h_payout_volume(addr), 0.0);
    }

    #[tokio::test]
    async fn test_create_payout_rejects_exceeding_tier_limit() {
        let registry = Arc::new(AnchorRegistry::new("http://localhost:8081".to_string()));
        let req = dummy_request("GACCOUNTFAIL", 12_000.0, Some(KycTier::Tier2));

        let payout = registry.create_payout(req).await;
        assert_eq!(payout.status, AnchorPayoutStatus::Failed);
        assert!(payout.error_message.is_some());
        assert!(payout.error_message.unwrap().contains("Daily fiat payout limit exceeded"));
    }
}

