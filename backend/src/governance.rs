use crate::api_error::ApiError;
use crate::notifications::{audit_action, AuditLogService};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalStatus::Active => "active",
            ProposalStatus::Passed => "passed",
            ProposalStatus::Rejected => "rejected",
            ProposalStatus::Executed => "executed",
        }
    }
}

impl std::str::FromStr for ProposalStatus {
    type Err = ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(ProposalStatus::Active),
            "passed" => Ok(ProposalStatus::Passed),
            "rejected" => Ok(ProposalStatus::Rejected),
            "executed" => Ok(ProposalStatus::Executed),
            other => Err(ApiError::BadRequest(format!(
                "Unknown proposal status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Proposal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub proposer_id: Uuid,
    pub status: String, // 'active', 'passed', 'rejected', 'executed'
    pub yes_votes: i32,
    pub no_votes: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub action_type: Option<String>,
    pub action_payload: Option<Value>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub title: String,
    pub description: String,
    pub duration_days: i64,
    /// Optional action to execute when the proposal passes (e.g. "update_parameter").
    pub action_type: Option<String>,
    /// JSON payload for the action (e.g. {"parameter_name": "fee", "parameter_value": "100"}).
    pub action_payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub supports: bool,
}

#[derive(Debug, Deserialize)]
pub struct ParameterUpdateRequest {
    pub parameter_name: String,
    pub parameter_value: String,
}

// ── Delegation types ──────────────────────────────────────────────────────────

/// Active delegation row returned from the DB.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceDelegation {
    pub id: Uuid,
    pub delegator_id: Uuid,
    pub delegate_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for delegating votes.
#[derive(Debug, Deserialize)]
pub struct DelegateVotesRequest {
    /// The user to delegate voting power to.
    pub delegate_id: Uuid,
}

/// Response returned after a successful delegation operation.
#[derive(Debug, Serialize)]
pub struct DelegationResponse {
    pub delegator_id: Uuid,
    pub delegate_id: Option<Uuid>,
    pub message: String,
}

/// Allowed protocol parameters with their validation rules.
///
/// Each entry is `(name, min_value, max_value, description)`.
/// All parameter values are stored as strings but must parse to `u64` within
/// the specified inclusive range.
const ALLOWED_PARAMETERS: &[(&str, u64, u64, &str)] = &[
    (
        "governance_quorum",
        1,
        1_000_000,
        "Minimum votes required for a proposal to pass",
    ),
    (
        "governance_voting_period_days",
        1,
        365,
        "Duration of the voting period in days",
    ),
    (
        "platform_fee_bps",
        0,
        10_000,
        "Platform fee in basis points (0–100%)",
    ),
    (
        "insurance_fund_fee_bps",
        0,
        5_000,
        "Insurance fund contribution in basis points",
    ),
    (
        "loan_liquidation_threshold_bps",
        1,
        10_000,
        "Collateral ratio at which a loan is liquidated (basis points)",
    ),
    (
        "loan_max_duration_days",
        1,
        3_650,
        "Maximum allowed loan duration in days",
    ),
    (
        "loan_min_collateral_bps",
        1,
        100_000,
        "Minimum collateral ratio for new loans (basis points)",
    ),
    (
        "max_beneficiaries_per_plan",
        1,
        100,
        "Maximum number of beneficiaries allowed per inheritance plan",
    ),
    (
        "claim_inactivity_period_days",
        1,
        3_650,
        "Days of inactivity before a claim can be triggered",
    ),
];

pub struct GovernanceService;

impl GovernanceService {
    /// Validate a parameter name/value pair before it is written to the database.
    ///
    /// Rules enforced:
    /// - `parameter_name` must be in the `ALLOWED_PARAMETERS` allowlist.
    /// - `parameter_name` must be non-empty and contain only lowercase letters, digits, and underscores.
    /// - `parameter_value` must be non-empty, parse as a `u64`, and fall within the
    ///   inclusive `[min, max]` range defined for that parameter.
    fn validate_parameter(name: &str, value: &str) -> Result<(), ApiError> {
        // Basic name format check (prevents injection / typos)
        if name.is_empty() {
            return Err(ApiError::BadRequest(
                "parameter_name must not be empty".to_string(),
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(ApiError::BadRequest(format!(
                "parameter_name '{}' contains invalid characters; only lowercase letters, digits, and underscores are allowed",
                name
            )));
        }

        // Allowlist check
        let rule = ALLOWED_PARAMETERS
            .iter()
            .find(|(allowed_name, ..)| *allowed_name == name)
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "Unknown parameter '{}'. Allowed parameters: {}",
                    name,
                    ALLOWED_PARAMETERS
                        .iter()
                        .map(|(n, ..)| *n)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })?;

        let (_, min, max, description) = rule;

        // Value must be non-empty
        if value.is_empty() {
            return Err(ApiError::BadRequest(format!(
                "parameter_value for '{}' must not be empty",
                name
            )));
        }

        // Value must parse as a non-negative integer
        let parsed: u64 = value.trim().parse().map_err(|_| {
            ApiError::BadRequest(format!(
                "parameter_value '{}' for '{}' is not a valid non-negative integer",
                value, name
            ))
        })?;

        // Range check
        if parsed < *min || parsed > *max {
            return Err(ApiError::BadRequest(format!(
                "parameter_value {} for '{}' is out of range [{}, {}] ({})",
                parsed, name, min, max, description
            )));
        }

        Ok(())
    }

    /// Persist terminal statuses for any proposals whose voting window has ended.
    ///
    /// This keeps expired proposals from lingering in `active` state until a manual
    /// finalize call happens.
    async fn refresh_expired_proposals(db: &PgPool) -> Result<(), ApiError> {
        let expired_proposal_ids = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM governance_proposals
            WHERE status = 'active' AND expires_at <= NOW()
            ORDER BY expires_at ASC
            "#,
        )
        .fetch_all(db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("DB error finding expired proposals: {}", e))
        })?;

        if expired_proposal_ids.is_empty() {
            return Ok(());
        }

        info!(
            count = expired_proposal_ids.len(),
            "Auto-finalizing expired governance proposals"
        );

        for proposal_id in expired_proposal_ids {
            Self::finalize_proposal(db, proposal_id).await?;
        }

        Ok(())
    }

    pub async fn create_proposal(
        db: &PgPool,
        proposer_id: Uuid,
        req: &CreateProposalRequest,
    ) -> Result<Proposal, ApiError> {
        if let Some(action_type) = &req.action_type {
            Self::validate_action(action_type, req.action_payload.as_ref())?;
        }

        let expires_at = Utc::now() + chrono::Duration::days(req.duration_days);

        let proposal = sqlx::query_as::<_, Proposal>(
            r#"
            INSERT INTO governance_proposals (
                title, description, proposer_id, status, expires_at, action_type, action_payload
            )
            VALUES ($1, $2, $3, 'active', $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&req.title)
        .bind(&req.description)
        .bind(proposer_id)
        .bind(expires_at)
        .bind(&req.action_type)
        .bind(&req.action_payload)
        .fetch_one(db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error creating proposal: {}", e)))?;

        Ok(proposal)
    }

    pub async fn list_proposals(db: &PgPool) -> Result<Vec<Proposal>, ApiError> {
        Self::refresh_expired_proposals(db).await?;

        let proposals = sqlx::query_as::<_, Proposal>(
            "SELECT * FROM governance_proposals ORDER BY created_at DESC",
        )
        .fetch_all(db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error listing proposals: {}", e)))?;

        Ok(proposals)
    }

    pub async fn get_proposal(db: &PgPool, proposal_id: Uuid) -> Result<Proposal, ApiError> {
        Self::refresh_expired_proposals(db).await?;

        sqlx::query_as::<_, Proposal>("SELECT * FROM governance_proposals WHERE id = $1")
            .bind(proposal_id)
            .fetch_optional(db)
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching proposal: {}", e)))?
            .ok_or_else(|| ApiError::NotFound(format!("Proposal {} not found", proposal_id)))
    }

    pub async fn vote_on_proposal(
        db: &PgPool,
        voter_id: Uuid,
        proposal_id: Uuid,
        req: &VoteRequest,
    ) -> Result<(), ApiError> {
        Self::refresh_expired_proposals(db).await?;

        let mut tx = db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let proposal = sqlx::query_as::<_, Proposal>(
            "SELECT * FROM governance_proposals WHERE id = $1 FOR UPDATE",
        )
        .bind(proposal_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching proposal: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Proposal {} not found", proposal_id)))?;

        if proposal.status != "active" || proposal.expires_at < Utc::now() {
            return Err(ApiError::BadRequest(
                "Proposal is no longer active for voting".to_string(),
            ));
        }

        let vote_inserted = sqlx::query(
            "INSERT INTO governance_votes (proposal_id, voter_id, supports) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(proposal_id)
        .bind(voter_id)
        .bind(req.supports)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error recording vote: {}", e)))?;

        if vote_inserted.rows_affected() == 0 {
            return Err(ApiError::BadRequest(
                "You have already voted on this proposal".to_string(),
            ));
        }

        let query = if req.supports {
            "UPDATE governance_proposals SET yes_votes = yes_votes + 1 WHERE id = $1"
        } else {
            "UPDATE governance_proposals SET no_votes = no_votes + 1 WHERE id = $1"
        };

        sqlx::query(query)
            .bind(proposal_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("DB error updating vote counts: {}", e))
            })?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        Ok(())
    }

    /// Evaluate whether a proposal has passed or been rejected after its voting period.
    pub async fn evaluate_proposal_status(
        db: &PgPool,
        proposal: &Proposal,
    ) -> Result<ProposalStatus, ApiError> {
        let current = proposal.status.parse::<ProposalStatus>()?;

        if current == ProposalStatus::Executed || current == ProposalStatus::Rejected {
            return Ok(current);
        }

        if Utc::now() <= proposal.expires_at {
            return Ok(ProposalStatus::Active);
        }

        let total_votes = proposal.yes_votes + proposal.no_votes;
        let quorum = Self::get_quorum_threshold(db).await?;

        if total_votes >= quorum && proposal.yes_votes > proposal.no_votes {
            Ok(ProposalStatus::Passed)
        } else {
            Ok(ProposalStatus::Rejected)
        }
    }

    /// Finalize an expired proposal by persisting its passed/rejected status.
    pub async fn finalize_proposal(db: &PgPool, proposal_id: Uuid) -> Result<Proposal, ApiError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let proposal = sqlx::query_as::<_, Proposal>(
            "SELECT * FROM governance_proposals WHERE id = $1 FOR UPDATE",
        )
        .bind(proposal_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching proposal: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Proposal {} not found", proposal_id)))?;

        let current = proposal.status.parse::<ProposalStatus>()?;
        if current == ProposalStatus::Executed {
            return Err(ApiError::BadRequest(
                "Proposal has already been executed".to_string(),
            ));
        }

        if current == ProposalStatus::Passed || current == ProposalStatus::Rejected {
            tx.commit()
                .await
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;
            return Ok(proposal);
        }

        let evaluated = Self::evaluate_proposal_status(db, &proposal).await?;
        if evaluated == ProposalStatus::Active {
            return Err(ApiError::BadRequest(
                "Proposal voting period has not ended yet".to_string(),
            ));
        }

        let new_status = evaluated.as_str();
        let updated = sqlx::query_as::<_, Proposal>(
            "UPDATE governance_proposals SET status = $1 WHERE id = $2 RETURNING *",
        )
        .bind(new_status)
        .bind(proposal_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error updating proposal: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        Ok(updated)
    }

    /// Execute a passed proposal, applying its configured action.
    pub async fn execute_proposal(
        db: &PgPool,
        executor_id: Uuid,
        proposal_id: Uuid,
    ) -> Result<Proposal, ApiError> {
        let finalized = Self::finalize_proposal(db, proposal_id).await?;
        let status = finalized.status.parse::<ProposalStatus>()?;

        if status != ProposalStatus::Passed {
            return Err(ApiError::BadRequest(
                "Proposal has not passed and cannot be executed".to_string(),
            ));
        }

        let mut tx = db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let proposal = sqlx::query_as::<_, Proposal>(
            "SELECT * FROM governance_proposals WHERE id = $1 FOR UPDATE",
        )
        .bind(proposal_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching proposal: {}", e)))?;

        if let (Some(action_type), Some(payload)) =
            (&proposal.action_type, &proposal.action_payload)
        {
            Self::apply_action(&mut tx, action_type, payload).await?;
        }

        let executed = sqlx::query_as::<_, Proposal>(
            r#"
            UPDATE governance_proposals
            SET status = 'executed', executed_at = NOW(), executed_by = $1
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(executor_id)
        .bind(proposal_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error executing proposal: {}", e)))?;

        AuditLogService::log(
            &mut *tx,
            None,
            Some(executor_id),
            audit_action::PARAMETER_UPDATE,
            Some(proposal_id),
            Some("governance_proposal"),
            None,
            Some("executed"),
            proposal.action_payload.clone(),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        info!(
            proposal_id = %proposal_id,
            executor_id = %executor_id,
            "Governance proposal executed"
        );

        Ok(executed)
    }

    pub async fn update_parameter(
        db: &PgPool,
        _admin_id: Uuid,
        req: &ParameterUpdateRequest,
    ) -> Result<(), ApiError> {
        // Validate before touching the database
        Self::validate_parameter(&req.parameter_name, &req.parameter_value)?;

        info!(
            "Updating protocol parameter: {} = {}",
            req.parameter_name, req.parameter_value
        );

        let result = sqlx::query(
            "INSERT INTO protocol_parameters (name, value, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (name) DO UPDATE SET value = $2, updated_at = NOW()",
        )
        .bind(&req.parameter_name)
        .bind(&req.parameter_value)
        .execute(db)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Parameter update failed (table might not exist yet): {}", e);
                Ok(())
            }
        }
    }

    fn validate_action(action_type: &str, payload: Option<&Value>) -> Result<(), ApiError> {
        match action_type {
            "update_parameter" => {
                let payload = payload.ok_or_else(|| {
                    ApiError::BadRequest(
                        "action_payload required for update_parameter proposals".to_string(),
                    )
                })?;
                let name = payload
                    .get("parameter_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ApiError::BadRequest(
                            "action_payload.parameter_name is required".to_string(),
                        )
                    })?;
                let value = payload
                    .get("parameter_value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ApiError::BadRequest(
                            "action_payload.parameter_value is required".to_string(),
                        )
                    })?;
                // Delegate to the shared validator for full allowlist + range checks
                Self::validate_parameter(name, value)?;
                Ok(())
            }
            other => Err(ApiError::BadRequest(format!(
                "Unsupported proposal action type: {other}"
            ))),
        }
    }

    async fn apply_action(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        action_type: &str,
        payload: &Value,
    ) -> Result<(), ApiError> {
        match action_type {
            "update_parameter" => {
                let name = payload
                    .get("parameter_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ApiError::BadRequest("Missing parameter_name in action payload".to_string())
                    })?;
                let value = payload
                    .get("parameter_value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ApiError::BadRequest(
                            "Missing parameter_value in action payload".to_string(),
                        )
                    })?;

                // Re-validate at execution time; the parameter rules may have changed
                // since the proposal was created, and this is the last safety gate.
                Self::validate_parameter(name, value)?;

                sqlx::query(
                    "INSERT INTO protocol_parameters (name, value, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (name) DO UPDATE SET value = $2, updated_at = NOW()",
                )
                .bind(name)
                .bind(value)
                .execute(&mut **tx)
                .await
                .map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("Failed to apply parameter update: {}", e))
                })?;
                Ok(())
            }
            other => Err(ApiError::BadRequest(format!(
                "Unsupported proposal action type: {other}"
            ))),
        }
    }

    async fn get_quorum_threshold(db: &PgPool) -> Result<i32, ApiError> {
        let value: Option<String> = sqlx::query_scalar(
            "SELECT value FROM protocol_parameters WHERE name = 'governance_quorum'",
        )
        .fetch_optional(db)
        .await
        .map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to read governance quorum: {}", e))
        })?;

        Ok(value
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(1)
            .max(1))
    }

    // ── Vote Delegation (Issue #649) ──────────────────────────────────────────

    /// Delegate voting power from `delegator_id` to `delegate_id`.
    ///
    /// Rules enforced:
    /// - A user cannot delegate to themselves.
    /// - If an active delegation already exists it is replaced (redelegation).
    /// - The old delegation is removed and the new one is inserted atomically.
    /// - Every change is recorded in `governance_delegation_history`.
    pub async fn delegate_votes(
        db: &PgPool,
        delegator_id: Uuid,
        req: &DelegateVotesRequest,
    ) -> Result<DelegationResponse, ApiError> {
        let delegate_id = req.delegate_id;

        if delegator_id == delegate_id {
            return Err(ApiError::BadRequest(
                "Cannot delegate votes to yourself".to_string(),
            ));
        }

        let mut tx = db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        // Check for an existing delegation so we can decide the audit action.
        let existing: Option<GovernanceDelegation> = sqlx::query_as::<_, GovernanceDelegation>(
            "SELECT * FROM governance_delegations WHERE delegator_id = $1",
        )
        .bind(delegator_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error checking delegation: {}", e)))?;

        let action = if existing.is_some() {
            "redelegated"
        } else {
            "delegated"
        };

        // Upsert the delegation (insert or update on conflict).
        sqlx::query(
            r#"
            INSERT INTO governance_delegations (delegator_id, delegate_id, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (delegator_id)
            DO UPDATE SET delegate_id = EXCLUDED.delegate_id, updated_at = NOW()
            "#,
        )
        .bind(delegator_id)
        .bind(delegate_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error upserting delegation: {}", e)))?;

        // Record in audit history.
        sqlx::query(
            "INSERT INTO governance_delegation_history (delegator_id, delegate_id, action) VALUES ($1, $2, $3)",
        )
        .bind(delegator_id)
        .bind(delegate_id)
        .bind(action)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error recording delegation history: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        info!(
            delegator_id = %delegator_id,
            delegate_id = %delegate_id,
            action = action,
            "Governance vote delegation updated"
        );

        Ok(DelegationResponse {
            delegator_id,
            delegate_id: Some(delegate_id),
            message: format!("Votes successfully {action} to {delegate_id}"),
        })
    }

    /// Remove an active delegation, restoring voting power to the delegator.
    pub async fn undelegate_votes(
        db: &PgPool,
        delegator_id: Uuid,
    ) -> Result<DelegationResponse, ApiError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        let existing: Option<GovernanceDelegation> = sqlx::query_as::<_, GovernanceDelegation>(
            "SELECT * FROM governance_delegations WHERE delegator_id = $1",
        )
        .bind(delegator_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error checking delegation: {}", e)))?;

        let delegation = existing.ok_or_else(|| {
            ApiError::BadRequest("No active delegation found to remove".to_string())
        })?;

        sqlx::query("DELETE FROM governance_delegations WHERE delegator_id = $1")
            .bind(delegator_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("DB error removing delegation: {}", e))
            })?;

        sqlx::query(
            "INSERT INTO governance_delegation_history (delegator_id, delegate_id, action) VALUES ($1, $2, 'undelegated')",
        )
        .bind(delegator_id)
        .bind(delegation.delegate_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error recording undelegation: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx commit error: {}", e)))?;

        info!(delegator_id = %delegator_id, "Governance vote delegation removed");

        Ok(DelegationResponse {
            delegator_id,
            delegate_id: None,
            message: "Delegation removed; voting power restored".to_string(),
        })
    }

    /// Return the current delegation for a user, if one exists.
    pub async fn get_delegation(
        db: &PgPool,
        delegator_id: Uuid,
    ) -> Result<Option<GovernanceDelegation>, ApiError> {
        sqlx::query_as::<_, GovernanceDelegation>(
            "SELECT * FROM governance_delegations WHERE delegator_id = $1",
        )
        .bind(delegator_id)
        .fetch_optional(db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching delegation: {}", e)))
    }

    /// Return all users who have delegated their votes to `delegate_id`.
    pub async fn get_delegators(
        db: &PgPool,
        delegate_id: Uuid,
    ) -> Result<Vec<GovernanceDelegation>, ApiError> {
        sqlx::query_as::<_, GovernanceDelegation>(
            "SELECT * FROM governance_delegations WHERE delegate_id = $1 ORDER BY created_at ASC",
        )
        .bind(delegate_id)
        .fetch_all(db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching delegators: {}", e)))
    }
}
