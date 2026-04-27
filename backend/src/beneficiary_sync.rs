//! # Beneficiary Sync Service (Task 3)
//!
//! Validates that beneficiaries in a legal will document match the
//! smart contract vault data, blocking document generation on mismatch.

use crate::api_error::ApiError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractBeneficiary {
    pub wallet_address: String,
    pub allocation_percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentBeneficiary {
    pub wallet_address: String,
    pub allocation_percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Matched,
    Mismatched,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MismatchDetail {
    pub wallet_address: String,
    pub field: String,
    pub contract_value: String,
    pub document_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficiarySyncResult {
    pub plan_id: Uuid,
    pub status: SyncStatus,
    pub mismatches: Vec<MismatchDetail>,
    pub contract_count: usize,
    pub document_count: usize,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

// ─── Service ──────────────────────────────────────────────────────────────────

pub struct BeneficiarySyncService;

impl BeneficiarySyncService {
    /// Fetch beneficiaries stored in the DB for a plan (representing contract state).
    pub async fn fetch_contract_beneficiaries(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<ContractBeneficiary>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            wallet_address: String,
            allocation_percent: Decimal,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT wallet_address, allocation_percent \
             FROM plan_beneficiaries WHERE plan_id = $1 ORDER BY wallet_address",
        )
        .bind(plan_id)
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ContractBeneficiary {
                wallet_address: r.wallet_address,
                allocation_percent: r.allocation_percent,
            })
            .collect())
    }

    /// Compare document beneficiaries against contract beneficiaries.
    /// Returns a sync result — callers should block generation if status is Mismatched.
    pub fn validate(
        plan_id: Uuid,
        contract: &[ContractBeneficiary],
        document: &[DocumentBeneficiary],
    ) -> BeneficiarySyncResult {
        let mut mismatches = Vec::new();
        let checked_at = chrono::Utc::now();

        // Count mismatch
        if contract.len() != document.len() {
            mismatches.push(MismatchDetail {
                wallet_address: "N/A".to_string(),
                field: "beneficiary_count".to_string(),
                contract_value: contract.len().to_string(),
                document_value: document.len().to_string(),
            });
        }

        // Build lookup maps (normalise addresses to lowercase)
        use std::collections::HashMap;
        let contract_map: HashMap<String, &ContractBeneficiary> = contract
            .iter()
            .map(|b| (b.wallet_address.to_lowercase(), b))
            .collect();

        let document_map: HashMap<String, &DocumentBeneficiary> = document
            .iter()
            .map(|b| (b.wallet_address.to_lowercase(), b))
            .collect();

        // Check every contract entry exists in document with matching allocation
        for (addr, cb) in &contract_map {
            match document_map.get(addr) {
                None => mismatches.push(MismatchDetail {
                    wallet_address: cb.wallet_address.clone(),
                    field: "wallet_address".to_string(),
                    contract_value: cb.wallet_address.clone(),
                    document_value: "MISSING".to_string(),
                }),
                Some(db_entry) => {
                    if (cb.allocation_percent - db_entry.allocation_percent).abs()
                        > Decimal::new(1, 4)
                    {
                        mismatches.push(MismatchDetail {
                            wallet_address: cb.wallet_address.clone(),
                            field: "allocation_percent".to_string(),
                            contract_value: cb.allocation_percent.to_string(),
                            document_value: db_entry.allocation_percent.to_string(),
                        });
                    }
                }
            }
        }

        // Check for extra document entries not in contract
        for (addr, db_entry) in &document_map {
            if !contract_map.contains_key(addr) {
                mismatches.push(MismatchDetail {
                    wallet_address: db_entry.wallet_address.clone(),
                    field: "wallet_address".to_string(),
                    contract_value: "MISSING".to_string(),
                    document_value: db_entry.wallet_address.clone(),
                });
            }
        }

        let status = if mismatches.is_empty() {
            SyncStatus::Matched
        } else {
            SyncStatus::Mismatched
        };

        BeneficiarySyncResult {
            plan_id,
            status,
            mismatches,
            contract_count: contract.len(),
            document_count: document.len(),
            checked_at,
        }
    }

    /// Full sync check: fetch from DB and validate against provided document list.
    /// Returns `Err` if mismatched (blocks document generation).
    pub async fn sync_and_validate(
        db: &PgPool,
        plan_id: Uuid,
        document_beneficiaries: &[DocumentBeneficiary],
    ) -> Result<BeneficiarySyncResult, ApiError> {
        let contract_beneficiaries = Self::fetch_contract_beneficiaries(db, plan_id).await?;

        let result = Self::validate(plan_id, &contract_beneficiaries, document_beneficiaries);

        if result.status == SyncStatus::Mismatched {
            return Err(ApiError::BadRequest(format!(
                "Beneficiary mismatch detected for plan {plan_id}: {} issue(s) found",
                result.mismatches.len()
            )));
        }

        Ok(result)
    }
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn contract(addr: &str, alloc: Decimal) -> ContractBeneficiary {
        ContractBeneficiary {
            wallet_address: addr.to_string(),
            allocation_percent: alloc,
        }
    }

    fn document(addr: &str, alloc: Decimal) -> DocumentBeneficiary {
        DocumentBeneficiary {
            wallet_address: addr.to_string(),
            allocation_percent: alloc,
        }
    }

    #[test]
    fn test_matching_beneficiaries_passes() {
        let plan_id = Uuid::new_v4();
        let c = vec![contract("GABC", dec!(60)), contract("GDEF", dec!(40))];
        let d = vec![document("GABC", dec!(60)), document("GDEF", dec!(40))];
        let result = BeneficiarySyncService::validate(plan_id, &c, &d);
        assert_eq!(result.status, SyncStatus::Matched);
        assert!(result.mismatches.is_empty());
    }

    #[test]
    fn test_count_mismatch_detected() {
        let plan_id = Uuid::new_v4();
        let c = vec![contract("GABC", dec!(100))];
        let d = vec![document("GABC", dec!(50)), document("GDEF", dec!(50))];
        let result = BeneficiarySyncService::validate(plan_id, &c, &d);
        assert_eq!(result.status, SyncStatus::Mismatched);
        assert!(!result.mismatches.is_empty());
    }

    #[test]
    fn test_allocation_mismatch_detected() {
        let plan_id = Uuid::new_v4();
        let c = vec![contract("GABC", dec!(60))];
        let d = vec![document("GABC", dec!(40))];
        let result = BeneficiarySyncService::validate(plan_id, &c, &d);
        assert_eq!(result.status, SyncStatus::Mismatched);
        let m = &result.mismatches[0];
        assert_eq!(m.field, "allocation_percent");
    }

    #[test]
    fn test_missing_wallet_in_document_detected() {
        let plan_id = Uuid::new_v4();
        let c = vec![contract("GABC", dec!(100))];
        let d = vec![document("GXYZ", dec!(100))];
        let result = BeneficiarySyncService::validate(plan_id, &c, &d);
        assert_eq!(result.status, SyncStatus::Mismatched);
    }

    #[test]
    fn test_case_insensitive_address_matching() {
        let plan_id = Uuid::new_v4();
        let c = vec![contract("GABC", dec!(100))];
        let d = vec![document("gabc", dec!(100))];
        let result = BeneficiarySyncService::validate(plan_id, &c, &d);
        assert_eq!(result.status, SyncStatus::Matched);
    }

    #[test]
    fn test_empty_both_sides_matches() {
        let plan_id = Uuid::new_v4();
        let result = BeneficiarySyncService::validate(plan_id, &[], &[]);
        assert_eq!(result.status, SyncStatus::Matched);
    }
}
