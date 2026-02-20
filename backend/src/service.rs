use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api_error::ApiError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InheritancePlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub fee: f64,
    pub net_amount: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn get_user_plan_by_id(
    pool: &PgPool,
    user_id: Uuid,
    plan_id: Uuid,
) -> Result<InheritancePlan, ApiError> {
    let plan = sqlx::query_as::<_, InheritancePlan>(
        r#"
		SELECT
			id,
			user_id,
			title,
			description,
			fee::double precision AS fee,
			net_amount::double precision AS net_amount,
			status,
			created_at,
			updated_at
		FROM plans
		WHERE id = $1 AND user_id = $2
		"#,
    )
    .bind(plan_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    plan.ok_or_else(|| ApiError::NotFound("Plan not found".to_string()))
}

pub async fn get_all_user_plans(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<InheritancePlan>, ApiError> {
    let plans = sqlx::query_as::<_, InheritancePlan>(
        r#"
		SELECT
			id,
			user_id,
			title,
			description,
			fee::double precision AS fee,
			net_amount::double precision AS net_amount,
			status,
			created_at,
			updated_at
		FROM plans
		WHERE user_id = $1
		ORDER BY created_at DESC
		"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(plans)
}

pub async fn get_all_user_pending_plans(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<InheritancePlan>, ApiError> {
    let plans = sqlx::query_as::<_, InheritancePlan>(
        r#"
		SELECT
			id,
			user_id,
			title,
			description,
			fee::double precision AS fee,
			net_amount::double precision AS net_amount,
			status,
			created_at,
			updated_at
		FROM plans
		WHERE user_id = $1 AND status = 'pending'
		ORDER BY created_at DESC
		"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(plans)
}

pub async fn get_all_admin_plans(pool: &PgPool) -> Result<Vec<InheritancePlan>, ApiError> {
    let plans = sqlx::query_as::<_, InheritancePlan>(
        r#"
		SELECT
			id,
			user_id,
			title,
			description,
			fee::double precision AS fee,
			net_amount::double precision AS net_amount,
			status,
			created_at,
			updated_at
		FROM plans
		ORDER BY created_at DESC
		"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(plans)
}

pub async fn get_all_admin_pending_plans(pool: &PgPool) -> Result<Vec<InheritancePlan>, ApiError> {
    let plans = sqlx::query_as::<_, InheritancePlan>(
        r#"
		SELECT
			id,
			user_id,
			title,
			description,
			fee::double precision AS fee,
			net_amount::double precision AS net_amount,
			status,
			created_at,
			updated_at
		FROM plans
		WHERE status = 'pending'
		ORDER BY created_at DESC
		"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(plans)
}
