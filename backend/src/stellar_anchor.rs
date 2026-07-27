use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPayoutRequest {
    pub beneficiary_address: String,
    pub beneficiary_name: String,
    pub token: String,
    pub token_amount: f64,
    pub fiat_currency: String,
    pub bank_name: String,
    pub account_number: String,
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
}

pub struct AnchorRegistry {
    payouts: RwLock<HashMap<String, AnchorPayout>>,
}

impl Default for AnchorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AnchorRegistry {
    pub fn new() -> Self {
        Self {
            payouts: RwLock::new(HashMap::new()),
        }
    }

    /// Retrieve exchange rate for a given fiat currency against USD.
    pub fn get_exchange_rate(fiat_currency: &str) -> f64 {
        match fiat_currency.to_uppercase().as_str() {
            "NGN" => 1500.0,
            "KES" => 130.0,
            "BRL" => 5.20,
            "PHP" => 57.50,
            "EUR" => 0.92,
            "USD" => 1.0,
            _ => 1.0,
        }
    }

    /// Simulate creating an anchor payout request with rate matching, fee calculation, and async status update thread.
    pub fn create_payout(self: &Arc<Self>, req: AnchorPayoutRequest) -> AnchorPayout {
        let id = Uuid::new_v4().to_string();
        let exchange_rate = Self::get_exchange_rate(&req.fiat_currency);
        let anchor_fee_usd = 0.50 + (req.token_amount * 0.005);
        let net_token_amount = (req.token_amount - anchor_fee_usd).max(0.0);
        let fiat_amount = (net_token_amount * exchange_rate * 100.0).round() / 100.0;
        let now = Utc::now().to_rfc3339();

        let payout = AnchorPayout {
            id: id.clone(),
            request: req,
            exchange_rate,
            fiat_amount,
            anchor_fee_usd,
            status: AnchorPayoutStatus::Pending,
            created_at: now.clone(),
            updated_at: now,
        };

        if let Ok(mut lock) = self.payouts.write() {
            lock.insert(id.clone(), payout.clone());
        }

        let registry = self.clone();
        let payout_id = id.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            registry.update_status(&payout_id, AnchorPayoutStatus::Processing);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            registry.update_status(&payout_id, AnchorPayoutStatus::Completed);
        });

        payout
    }

    /// Update status of a payout by ID.
    pub fn update_status(&self, id: &str, status: AnchorPayoutStatus) {
        if let Ok(mut lock) = self.payouts.write() {
            if let Some(payout) = lock.get_mut(id) {
                payout.status = status;
                payout.updated_at = Utc::now().to_rfc3339();
            }
        }
    }

    /// Retrieve anchor payout by transaction ID.
    pub fn get_payout(&self, id: &str) -> Option<AnchorPayout> {
        self.payouts.read().ok()?.get(id).cloned()
    }

    /// List all anchor payouts, optionally filtered by beneficiary address.
    pub fn list_payouts(&self, address: Option<String>) -> Vec<AnchorPayout> {
        if let Ok(lock) = self.payouts.read() {
            lock.values()
                .filter(|p| {
                    if let Some(ref addr) = address {
                        &p.request.beneficiary_address == addr
                    } else {
                        true
                    }
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_and_get_payout() {
        let registry = Arc::new(AnchorRegistry::new());
        let req = AnchorPayoutRequest {
            beneficiary_address: "GBN23P...8K9T".to_string(),
            beneficiary_name: "Jane Doe".to_string(),
            token: "USDC".to_string(),
            token_amount: 100.0,
            fiat_currency: "NGN".to_string(),
            bank_name: "Access Bank".to_string(),
            account_number: "0123456789".to_string(),
        };

        let payout = registry.create_payout(req);
        assert!(!payout.id.is_empty());
        assert_eq!(payout.exchange_rate, 1500.0);
        assert_eq!(payout.status, AnchorPayoutStatus::Pending);

        let fetched = registry.get_payout(&payout.id);
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, payout.id);

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        let completed = registry.get_payout(&payout.id).unwrap();
        assert_eq!(completed.status, AnchorPayoutStatus::Completed);
    }

    #[tokio::test]
    async fn test_list_payouts_filtering() {
        let registry = Arc::new(AnchorRegistry::new());
        let req1 = AnchorPayoutRequest {
            beneficiary_address: "ADDR_A".to_string(),
            beneficiary_name: "Alice".to_string(),
            token: "USDC".to_string(),
            token_amount: 50.0,
            fiat_currency: "KES".to_string(),
            bank_name: "KCB".to_string(),
            account_number: "111".to_string(),
        };
        let req2 = AnchorPayoutRequest {
            beneficiary_address: "ADDR_B".to_string(),
            beneficiary_name: "Bob".to_string(),
            token: "USDC".to_string(),
            token_amount: 75.0,
            fiat_currency: "BRL".to_string(),
            bank_name: "Pix".to_string(),
            account_number: "222".to_string(),
        };

        registry.create_payout(req1);
        registry.create_payout(req2);

        let all = registry.list_payouts(None);
        assert_eq!(all.len(), 2);

        let filtered = registry.list_payouts(Some("ADDR_A".to_string()));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].request.beneficiary_name, "Alice");
    }
}
