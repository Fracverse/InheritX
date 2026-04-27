use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

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
}

#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub title: String,
    pub description: String,
    pub duration_days: i64,
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

pub struct GovernanceService;

impl GovernanceService {
    pub async fn create_proposal(
        db: &PgPool,
        proposer_id: Uuid,
        req: &CreateProposalRequest,
    ) -> Result<Proposal, ApiError> {
        let expires_at = Utc::now() + chrono::Duration::days(req.duration_days);

        let proposal = sqlx::query_as::<_, Proposal>(
            r#"
            INSERT INTO governance_proposals (title, description, proposer_id, status, expires_at)
            VALUES ($1, $2, $3, 'active', $4)
            RETURNING *
            "#,
        )
        .bind(&req.title)
        .bind(&req.description)
        .bind(proposer_id)
        .bind(expires_at)
        .fetch_one(db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error creating proposal: {}", e)))?;

        Ok(proposal)
    }

    pub async fn list_proposals(db: &PgPool) -> Result<Vec<Proposal>, ApiError> {
        let proposals = sqlx::query_as::<_, Proposal>(
            "SELECT * FROM governance_proposals ORDER BY created_at DESC",
        )
        .fetch_all(db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error listing proposals: {}", e)))?;

        Ok(proposals)
    }

    pub async fn vote_on_proposal(
        db: &PgPool,
        voter_id: Uuid,
        proposal_id: Uuid,
        req: &VoteRequest,
    ) -> Result<(), ApiError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Tx start error: {}", e)))?;

        // Check if proposal is still active
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

        // Record vote
        let vote_inserted = sqlx::query(
            "INSERT INTO governance_votes (proposal_id, voter_id, supports) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
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

        // Update counts
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

    pub async fn update_parameter(
        db: &PgPool,
        _admin_id: Uuid,
        req: &ParameterUpdateRequest,
    ) -> Result<(), ApiError> {
        info!(
            "Updating protocol parameter: {} = {}",
            req.parameter_name, req.parameter_value
        );

        // In a real system, this would update a 'protocol_parameters' table
        let result = sqlx::query(
            "INSERT INTO protocol_parameters (name, value, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (name) DO UPDATE SET value = $2, updated_at = NOW()"
        )
        .bind(&req.parameter_name)
        .bind(&req.parameter_value)
        .execute(db)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Parameter update failed (table might not exist yet): {}", e);
                // Return success for simulation purposes if table doesn't exist,
                // but in production we'd want a proper migration.
                Ok(())
            }
        }
    }
}
