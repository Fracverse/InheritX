use crate::api_error::ApiError;
use crate::app::AppState;
use crate::auth::AuthenticatedUser;
use crate::events::{EventService, EventType, LendingEvent};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Query parameters for event listing
#[derive(Debug, Deserialize)]
pub struct EventQueryParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub event_type: Option<String>,
}

fn default_limit() -> i64 {
    50
}

/// Response for event listing
#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub events: Vec<LendingEvent>,
    pub total: usize,
    pub limit: i64,
    pub offset: i64,
}

/// Get events for the authenticated user
pub async fn get_user_events(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(params): Query<EventQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    let event_type = if let Some(et_str) = params.event_type {
        Some(parse_event_type(&et_str)?)
    } else {
        None
    };

    let events = EventService::get_user_events(
        &state.db,
        user.user_id,
        event_type,
        params.limit.min(100),
        params.offset,
    )
    .await?;

    let total = events.len();

    Ok((
        StatusCode::OK,
        Json(EventListResponse {
            events,
            total,
            limit: params.limit,
            offset: params.offset,
        }),
    ))
}

/// Get events for a specific plan
pub async fn get_plan_events(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(plan_id): Path<Uuid>,
    Query(params): Query<EventQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify user owns the plan
    let plan =
        crate::service::PlanService::get_plan_by_id(&state.db, plan_id, user.user_id).await?;
    if plan.is_none() {
        return Err(ApiError::NotFound(format!("Plan {} not found", plan_id)));
    }

    let event_type = if let Some(et_str) = params.event_type {
        Some(parse_event_type(&et_str)?)
    } else {
        None
    };

    let events = EventService::get_plan_events(
        &state.db,
        plan_id,
        event_type,
        params.limit.min(100),
        params.offset,
    )
    .await?;

    let total = events.len();

    Ok((
        StatusCode::OK,
        Json(EventListResponse {
            events,
            total,
            limit: params.limit,
            offset: params.offset,
        }),
    ))
}

/// Get events by transaction hash
pub async fn get_events_by_transaction(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(transaction_hash): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let events = EventService::get_by_transaction_hash(&state.db, &transaction_hash).await?;

    Ok((StatusCode::OK, Json(events)))
}

/// Parse event type string to EventType enum
fn parse_event_type(s: &str) -> Result<EventType, ApiError> {
    match s.to_lowercase().as_str() {
        "deposit" => Ok(EventType::Deposit),
        "borrow" => Ok(EventType::Borrow),
        "repay" => Ok(EventType::Repay),
        "liquidation" => Ok(EventType::Liquidation),
        "interest_accrual" => Ok(EventType::InterestAccrual),
        _ => Err(ApiError::BadRequest(format!(
            "Invalid event type: {}. Valid types: deposit, borrow, repay, liquidation, interest_accrual",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_type() {
        assert!(matches!(
            parse_event_type("deposit").unwrap(),
            EventType::Deposit
        ));
        assert!(matches!(
            parse_event_type("BORROW").unwrap(),
            EventType::Borrow
        ));
        assert!(matches!(
            parse_event_type("Repay").unwrap(),
            EventType::Repay
        ));
        assert!(matches!(
            parse_event_type("liquidation").unwrap(),
            EventType::Liquidation
        ));
        assert!(matches!(
            parse_event_type("interest_accrual").unwrap(),
            EventType::InterestAccrual
        ));
        assert!(parse_event_type("invalid").is_err());
    }

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 50);
    }
}
