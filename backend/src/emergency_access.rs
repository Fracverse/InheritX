use crate::api_error::ApiError;
use crate::notifications::{
    audit_action, entity_type, notif_type, AuditLogService, NotificationService,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmergencyAccess {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub granted_by: Uuid,
    pub granted_to: Option<Uuid>,
    pub access_type: String,
    pub reason: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub revocation_reason: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GrantEmergencyAccessRequest {
    pub plan_id: Uuid,
    pub access_type: String, // 'admin_override', 'temporary_access', etc.
    pub reason: String,
    pub expires_in_hours: Option<i64>, // If provided, access expires after N hours
}

#[derive(Debug, Deserialize)]
pub struct RevokeEmergencyAccessRequest {
    pub access_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct EmergencyAccessResponse {
    pub success: bool,
    pub access_id: Uuid,
    pub message: String,
}

// ─── Service ─────────────────────────────────────────────────────────────────

pub struct EmergencyAccessService;

impl EmergencyAccessService {
    /// Grant emergency access to a plan
    pub async fn grant_access(
        pool: &PgPool,
        admin_id: Uuid,
        req: &GrantEmergencyAccessRequest,
    ) -> Result<EmergencyAccessResponse, ApiError> {
        let mut tx = pool.begin().await?;

        // Verify plan exists
        let plan_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM plans WHERE id = $1)")
            .bind(req.plan_id)
            .fetch_one(&mut *tx)
            .await?;

        if !plan_exists {
            return Err(ApiError::NotFound(format!("Plan {} not found", req.plan_id)));
        }

        // Calculate expiration time if provided
        let expires_at = req
            .expires_in_hours
            .map(|hours| Utc::now() + Duration::hours(hours));

        // Insert emergency access record
        let access_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO emergency_access (
                plan_id, granted_by, access_type, reason, expires_at, status
            )
            VALUES ($1, $2, $3, $4, $5, 'active')
            RETURNING id
            "#,
        )
        .bind(req.plan_id)
        .bind(admin_id)
        .bind(&req.access_type)
        .bind(&req.reason)
        .bind(expires_at)
        .fetch_one(&mut *tx)
        .await?;

        // Get plan user for notification
        let plan_user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM plans WHERE id = $1")
            .bind(req.plan_id)
            .fetch_one(&mut *tx)
            .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            audit_action::EMERGENCY_ACCESS_GRANTED,
            Some(req.plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // Notify user
        let expiry_msg = expires_at
            .map(|exp| format!(" This access will expire at {}", exp.format("%Y-%m-%d %H:%M:%S UTC")))
            .unwrap_or_default();

        NotificationService::create(
            &mut tx,
            plan_user_id,
            notif_type::EMERGENCY_ACCESS_GRANTED,
            format!(
                "Emergency access has been granted to your plan by an administrator. Type: {}. Reason: {}.{}",
                req.access_type, req.reason, expiry_msg
            ),
        )
        .await?;

        tx.commit().await?;

        Ok(EmergencyAccessResponse {
            success: true,
            access_id,
            message: "Emergency access granted successfully".to_string(),
        })
    }

    /// Revoke emergency access
    pub async fn revoke_access(
        pool: &PgPool,
        admin_id: Uuid,
        req: &RevokeEmergencyAccessRequest,
    ) -> Result<EmergencyAccessResponse, ApiError> {
        let mut tx = pool.begin().await?;

        // Fetch the access record
        let access: EmergencyAccess = sqlx::query_as(
            "SELECT * FROM emergency_access WHERE id = $1"
        )
        .bind(req.access_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Emergency access {} not found", req.access_id)))?;

        // Check if already revoked
        if access.status == "revoked" {
            return Err(ApiError::BadRequest("Access is already revoked".to_string()));
        }

        // Update access record
        sqlx::query(
            r#"
            UPDATE emergency_access
            SET status = 'revoked',
                revoked_at = NOW(),
                revoked_by = $1,
                revocation_reason = $2,
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(admin_id)
        .bind(&req.reason)
        .bind(req.access_id)
        .execute(&mut *tx)
        .await?;

        // Get plan user for notification
        let plan_user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM plans WHERE id = $1")
            .bind(access.plan_id)
            .fetch_one(&mut *tx)
            .await?;

        // Audit log
        AuditLogService::log(
            &mut *tx,
            Some(admin_id),
            audit_action::EMERGENCY_ACCESS_REVOKED,
            Some(access.plan_id),
            Some(entity_type::PLAN),
        )
        .await?;

        // Notify user
        NotificationService::create(
            &mut tx,
            plan_user_id,
            notif_type::EMERGENCY_ACCESS_REVOKED,
            format!(
                "Emergency access to your plan has been revoked. Reason: {}",
                req.reason
            ),
        )
        .await?;

        tx.commit().await?;

        Ok(EmergencyAccessResponse {
            success: true,
            access_id: req.access_id,
            message: "Emergency access revoked successfully".to_string(),
        })
    }

    /// Get all active emergency access records for a plan
    pub async fn get_active_access_for_plan(
        db: &PgPool,
        plan_id: Uuid,
    ) -> Result<Vec<EmergencyAccess>, ApiError> {
        let records = sqlx::query_as::<_, EmergencyAccess>(
            r#"
            SELECT * FROM emergency_access
            WHERE plan_id = $1 AND status = 'active'
            ORDER BY granted_at DESC
            "#,
        )
        .bind(plan_id)
        .fetch_all(db)
        .await?;

        Ok(records)
    }

    /// Get all emergency access records (admin view)
    pub async fn get_all_access(db: &PgPool) -> Result<Vec<EmergencyAccess>, ApiError> {
        let records = sqlx::query_as::<_, EmergencyAccess>(
            r#"
            SELECT * FROM emergency_access
            ORDER BY granted_at DESC
            "#,
        )
        .fetch_all(db)
        .await?;

        Ok(records)
    }

    /// Check for expiring access and send notifications
    /// This should be called periodically (e.g., every hour)
    pub async fn check_expiring_access(db: &PgPool) -> Result<u64, ApiError> {
        // Find access that expires within the next 24 hours and hasn't been notified yet
        let expiring_access: Vec<(Uuid, Uuid, DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT id, plan_id, expires_at
            FROM emergency_access
            WHERE status = 'active'
              AND expires_at IS NOT NULL
              AND expires_at > NOW()
              AND expires_at <= NOW() + INTERVAL '24 hours'
            "#,
        )
        .fetch_all(db)
        .await?;

        let mut count = 0;

        for (access_id, plan_id, expires_at) in expiring_access {
            // Get plan user
            if let Ok(plan_user_id) = sqlx::query_scalar::<_, Uuid>(
                "SELECT user_id FROM plans WHERE id = $1"
            )
            .bind(plan_id)
            .fetch_one(db)
            .await
            {
                // Create notification
                let mut tx = db.begin().await?;

                if let Err(e) = NotificationService::create(
                    &mut tx,
                    plan_user_id,
                    notif_type::EMERGENCY_ACCESS_EXPIRING,
                    format!(
                        "Emergency access to your plan will expire at {}",
                        expires_at.format("%Y-%m-%d %H:%M:%S UTC")
                    ),
                )
                .await
                {
                    tracing::warn!("Failed to create expiring access notification: {}", e);
                    continue;
                }

                // Audit log
                if let Err(e) = AuditLogService::log(
                    &mut tx,
                    None,
                    audit_action::EMERGENCY_ACCESS_EXPIRED,
                    Some(plan_id),
                    Some(entity_type::PLAN),
                )
                .await
                {
                    tracing::warn!("Failed to log emergency access expiration: {}", e);
                    continue;
                }

                if let Err(e) = tx.commit().await {
                    tracing::warn!("Failed to commit expiring access notification: {}", e);
                    continue;
                }

                count += 1;
            }
        }

        Ok(count)
    }

    /// Mark expired access as expired (should be called periodically)
    pub async fn mark_expired_access(db: &PgPool) -> Result<u64, ApiError> {
        let result = sqlx::query(
            r#"
            UPDATE emergency_access
            SET status = 'expired', updated_at = NOW()
            WHERE status = 'active'
              AND expires_at IS NOT NULL
              AND expires_at <= NOW()
            "#,
        )
        .execute(db)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emergency_access_serializes_correctly() {
        let now = Utc::now();
        let access = EmergencyAccess {
            id: Uuid::new_v4(),
            plan_id: Uuid::new_v4(),
            granted_by: Uuid::new_v4(),
            granted_to: None,
            access_type: "admin_override".to_string(),
            reason: "Risk mitigation".to_string(),
            granted_at: now,
            expires_at: Some(now + Duration::hours(24)),
            revoked_at: None,
            revoked_by: None,
            revocation_reason: None,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_string(&access).expect("Should serialize");
        assert!(json.contains("admin_override"));
        assert!(json.contains("active"));
    }

    #[test]
    fn grant_request_deserializes_correctly() {
        let json = r#"{
            "plan_id": "550e8400-e29b-41d4-a716-446655440000",
            "access_type": "temporary_access",
            "reason": "Emergency override",
            "expires_in_hours": 48
        }"#;

        let req: GrantEmergencyAccessRequest = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(req.access_type, "temporary_access");
        assert_eq!(req.expires_in_hours, Some(48));
    }

    #[test]
    fn revoke_request_deserializes_correctly() {
        let json = r#"{
            "access_id": "550e8400-e29b-41d4-a716-446655440000",
            "reason": "Access no longer needed"
        }"#;

        let req: RevokeEmergencyAccessRequest = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(req.reason, "Access no longer needed");
    }
}
