// Example: How to integrate 2FA with plan creation
// This is a reference implementation showing the flow

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::app::AppState;
use crate::two_fa::service::TwoFAService;

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub user_id: Uuid,
    pub plan_name: String,
    pub beneficiaries: Vec<String>,
    pub assets: Vec<String>,
    // 2FA OTP
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePlanResponse {
    pub plan_id: Uuid,
    pub message: String,
}

/// Example handler for creating a plan with 2FA verification
pub async fn create_plan_with_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<Json<CreatePlanResponse>, ApiError> {
    // Step 1: Verify 2FA OTP
    TwoFAService::verify_otp_from_db(&state.db, payload.user_id, &payload.otp).await?;

    // Step 2: If OTP is valid, proceed with plan creation
    let plan_id = Uuid::new_v4();

    // TODO: Implement actual plan creation logic
    // - Store plan in database
    // - Create smart contract
    // - Link beneficiaries
    // - etc.

    Ok(Json(CreatePlanResponse {
        plan_id,
        message: "Plan created successfully".to_string(),
    }))
}

/// Example handler for claiming assets with 2FA verification
#[derive(Debug, Deserialize)]
pub struct ClaimAssetsRequest {
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub claim_code: String,
    // 2FA OTP
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimAssetsResponse {
    pub claim_id: Uuid,
    pub message: String,
}

pub async fn claim_assets_with_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClaimAssetsRequest>,
) -> Result<Json<ClaimAssetsResponse>, ApiError> {
    // Step 1: Verify 2FA OTP
    TwoFAService::verify_otp_from_db(&state.db, payload.user_id, &payload.otp).await?;

    // Step 2: If OTP is valid, proceed with claim
    let claim_id = Uuid::new_v4();

    // TODO: Implement actual claim logic
    // - Verify claim code
    // - Check eligibility
    // - Transfer assets
    // - etc.

    Ok(Json(ClaimAssetsResponse {
        claim_id,
        message: "Assets claimed successfully".to_string(),
    }))
}

// Complete flow example:
//
// 1. User initiates plan creation
//    POST /user/send-2fa { "user_id": "..." }
//
// 2. User receives OTP via email
//
// 3. User submits plan creation with OTP
//    POST /plans/create {
//      "user_id": "...",
//      "plan_name": "My Estate Plan",
//      "beneficiaries": [...],
//      "assets": [...],
//      "otp": "123456"
//    }
//
// 4. Backend verifies OTP and creates plan
