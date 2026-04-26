use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::middleware::{
    request_id_middleware, request_logging_middleware, request_timeout_middleware,
    security_headers_middleware,
};
use uuid::Uuid;

use crate::analytics::analytics_router;
use crate::api_error::ApiError;
use crate::auth::{AuthenticatedAdmin, AuthenticatedUser};
use crate::beneficiary_sync::{BeneficiarySyncService, DocumentBeneficiary};
use crate::collateral_management::{
    AddCollateralRequest, CollateralManagementService, RemoveCollateralRequest,
    SwapCollateralRequest,
};
use crate::config::Config;
use crate::contingent_beneficiary::{
    AddContingentBeneficiaryRequest, ContingentBeneficiaryService, PromoteContingentRequest,
    RemoveContingentBeneficiaryRequest, SetContingencyConditionsRequest,
};
use crate::document_storage::DocumentStorageService;
use crate::governance::{
    CreateProposalRequest, GovernanceService, ParameterUpdateRequest, Proposal, VoteRequest,
};
use crate::insurance_fund::{CreateInsuranceClaimRequest, ProcessInsuranceClaimRequest};
use crate::legacy_content::{ContentListFilters, LegacyContentService};
use crate::loan_lifecycle::{CreateLoanRequest, LoanLifecycleService, LoanListFilters};
use crate::message_access_audit::{MessageAccessAuditService, MessageAuditFilters};
use crate::pagination::PaginationQuery;
use crate::secure_messages::{
    CreateLegacyMessageRequest, LegacyMessageDeliveryService, MessageEncryptionService,
    MessageKeyService,
};
use crate::service::{
    ClaimPlanRequest, CreateEmergencyAccessGrantRequest, CreateEmergencyContactRequest,
    CreatePlanRequest, EmergencyAccessAuditLogFilters, EmergencyAccessService,
    EmergencyAdminService, EmergencyContactService, EmergencySessionService, KycRecord, KycService,
    KycStatus, LoanSimulationRequest, LoanSimulationService, PausePlanRequest, PlanService,
    RevokeEmergencyAccessGrantRequest, RiskOverrideRequest, StartSessionRequest,
    UnpausePlanRequest, UpdateEmergencyContactRequest,
};
use crate::stress_testing::StressTestingEngine;
use crate::will_compliance::{ValidationResult, WillComplianceService};
use crate::will_pdf::{WillDocumentInput, WillPdfService, WillTemplate};
use crate::will_signature::{
    SigningChallengeRequest, SubmitSignatureRequest, WillSignatureService,
};
use crate::will_version::{PaginatedVersions, PaginationParams, WillVersionService};
use crate::witness::{InviteWitnessRequest, WitnessService, WitnessSignRequest};
use crate::yield_service::{DefaultOnChainYieldService, OnChainYieldService};
use base64::Engine as _;

pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub yield_service: Arc<dyn OnChainYieldService>,
    pub stress_testing_engine: Arc<StressTestingEngine>,
    pub insurance_fund_service: Arc<crate::insurance_fund::InsuranceFundService>,
}

pub async fn create_app(
    db: PgPool,
    config: Config,
    prometheus_handle: PrometheusHandle,
) -> Result<Router, ApiError> {
    let price_feed = Arc::new(crate::price_feed::DefaultPriceFeedService::new(
        db.clone(),
        3600,
    ));
    if let Err(e) = price_feed.initialize_defaults().await {
        tracing::warn!("Failed to initialize default price feeds: {}", e);
    }

    let risk_engine = Arc::new(crate::risk_engine::RiskEngine::new(
        db.clone(),
        price_feed.clone(),
        rust_decimal::Decimal::new(12, 1),
    ));
    risk_engine.clone().start();

    let yield_service = Arc::new(DefaultOnChainYieldService::new());

    let stress_testing_engine = Arc::new(StressTestingEngine::new(
        db.clone(),
        price_feed.clone(),
        risk_engine,
    ));

    let insurance_fund_service =
        Arc::new(crate::insurance_fund::InsuranceFundService::new(db.clone()));
    insurance_fund_service.clone().start();

    let state = Arc::new(AppState {
        db: db.clone(),
        config: config.clone(),
        yield_service,
        stress_testing_engine,
        insurance_fund_service,
    });

    // ── Rate limiting (config-driven) ────────────────────────────────────────
    // Limits are read from environment variables via Config::load() so every
    // deployment can tune them without a code change.  Hardcoded fallbacks
    // (2 req/s, burst 5) are preserved as defaults when the variables are absent.
    let rl = &config.rate_limit;

    let mut governor_builder = GovernorConfigBuilder::default();
    governor_builder
        .per_second(rl.default_limit().per_second)
        .burst_size(rl.default_limit().burst_size);
    let mut governor_builder = governor_builder.key_extractor(
        crate::middleware::RateLimitKeyExtractor::new(rl.bypass_tokens.clone()),
    );
    governor_builder.error_handler(crate::middleware::rate_limit_error_response);
    let governor_conf = Arc::new(governor_builder.use_headers().finish().unwrap());

    let emergency_governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(rl.emergency_limit().per_second)
            .burst_size(rl.emergency_limit().burst_size)
            .use_headers()
            .finish()
            .unwrap(),
    );

    let admin_login_governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(rl.admin_login_limit().per_second)
            .burst_size(rl.admin_login_limit().burst_size)
            .use_headers()
            .finish()
            .unwrap(),
    );

    tracing::info!(
        default_rps = rl.default_per_second,
        default_burst = rl.default_burst_size,
        emergency_rps = rl.emergency_per_second,
        admin_login_rps = rl.admin_login_per_second,
        "Rate limiting configuration loaded"
    );

    // ── CORS configuration (Issue #408) ──────────────────────────────────────
    // Allowed origins are read from CORS_ALLOWED_ORIGINS (comma-separated).
    // Falls back to permissive any-origin in development.
    let cors_layer = {
        let allowed_origins_env = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
        if allowed_origins_env.is_empty() {
            CorsLayer::permissive()
        } else {
            let origins: Vec<axum::http::HeaderValue> = allowed_origins_env
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods(AllowMethods::any())
                .allow_headers(AllowHeaders::any())
                .allow_credentials(true)
        }
    };

    // Request timeout (configurable via REQUEST_TIMEOUT_SECS, default 30 s).
    let timeout_secs: u64 = std::env::var("REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let timeout_duration = Duration::from_secs(timeout_secs);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
        // Admin login gets its own, tighter rate limit (brute-force protection).
        .route("/health/db/metrics", get(db_metrics))
        // Prometheus metrics scrape endpoint (Issue #423).
        // Restrict access at the network/ingress layer in production.
        .route("/metrics", get(crate::metrics::metrics_handler))
        .route(
            "/admin/login",
            post(crate::auth::login_admin).layer(GovernorLayer {
                config: admin_login_governor_conf,
            }),
        )
        .route(
            "/api/plans/due-for-claim",
            get(get_all_due_for_claim_plans_user),
        )
        .route(
            "/api/plans/due-for-claim/:plan_id",
            get(get_due_for_claim_plan),
        )
        .route("/api/plans/:plan_id/claim", post(claim_plan))
        .route("/api/plans/:plan_id", get(get_plan))
        .route("/api/plans", post(create_plan))
        .route(
            "/api/messages/legacy",
            post(create_legacy_message).get(list_legacy_messages),
        )
        .route(
            "/api/messages/legacy/vault/:vault_id",
            get(list_vault_legacy_messages),
        )
        .route("/api/admin/messages/keys", get(list_message_keys))
        .route("/api/admin/messages/keys/rotate", post(rotate_message_key))
        .route(
            "/api/admin/messages/delivery/process",
            post(process_legacy_message_delivery),
        )
        .route("/api/admin/messages/audit", get(get_message_audit_logs))
        .route(
            "/api/admin/messages/audit/summary",
            get(get_message_audit_summary),
        )
        .route(
            "/api/admin/messages/audit/search",
            get(search_message_audit_logs),
        )
        .route(
            "/api/messages/:message_id/audit",
            get(get_message_access_history),
        )
        .route(
            "/api/messages/audit/my-activity",
            get(get_my_message_activity),
        )
        .route(
            "/api/emergency/contacts",
            get(list_emergency_contacts).post(create_emergency_contact),
        )
        .route(
            "/api/emergency/contacts/:contact_id",
            put(update_emergency_contact).delete(delete_emergency_contact),
        )
        .route(
            "/api/emergency/access/grants",
            post(create_emergency_access_grant).layer(GovernorLayer {
                config: emergency_governor_conf.clone(),
            }),
        )
        .route(
            "/api/emergency/access/grants/:grant_id/revoke",
            post(revoke_emergency_access_grant).layer(GovernorLayer {
                config: emergency_governor_conf,
            }),
        )
        .route(
            "/api/emergency/access/audit-logs",
            get(list_emergency_access_audit_logs),
        )
        .route(
            "/api/emergency/access/risk-alerts",
            get(list_emergency_access_risk_alerts),
        )
        .route(
            "/api/emergency/access/dashboard",
            get(get_emergency_access_dashboard),
        )
        // Emergency Access Sessions (Issue #306)
        .route(
            "/api/emergency/access/sessions",
            post(start_emergency_session).get(list_active_emergency_sessions),
        )
        .route(
            "/api/emergency/access/sessions/:session_id/heartbeat",
            put(heartbeat_emergency_session),
        )
        .route(
            "/api/emergency/access/sessions/:session_id/end",
            put(end_emergency_session),
        )
        // Loan Simulation endpoints
        .route("/api/loans/simulate", post(simulate_loan))
        .route("/api/loans/simulations", get(get_user_simulations))
        .route("/api/loans/simulations/:simulation_id", get(get_simulation))
        .route("/api/reputation", get(get_user_reputation))
        // ── Loan Lifecycle Tracker ─────────────────────────────────────────────
        .route("/api/loans/lifecycle", post(create_lifecycle_loan))
        .route("/api/loans/lifecycle", get(list_lifecycle_loans))
        .route("/api/loans/lifecycle/summary", get(get_lifecycle_summary))
        .route("/api/loans/lifecycle/:id", get(get_lifecycle_loan))
        .route("/api/loans/lifecycle/:id/repay", post(repay_lifecycle_loan))
        .route(
            "/api/admin/loans/lifecycle/:id/liquidate",
            post(liquidate_lifecycle_loan),
        )
        .route(
            "/api/admin/loans/lifecycle/mark-overdue",
            post(mark_overdue_loans),
        )
        // ── Collateral Management ──────────────────────────────────────────────
        .route(
            "/api/loans/lifecycle/:id/collateral/add",
            post(add_collateral),
        )
        .route(
            "/api/loans/lifecycle/:id/collateral/remove",
            post(remove_collateral),
        )
        .route(
            "/api/loans/lifecycle/:id/collateral/swap",
            post(swap_collateral),
        )
        .route(
            "/api/loans/lifecycle/:id/collateral/value",
            get(get_collateral_value),
        )
        .route(
            "/api/loans/lifecycle/:id/collateral/max-withdrawable",
            get(get_max_withdrawable_collateral),
        )
        .route(
            "/api/loans/lifecycle/:id/collateral/requirements",
            get(get_collateral_requirements),
        )
        .route(
            "/api/admin/plans/due-for-claim",
            get(get_all_due_for_claim_plans_admin),
        )
        .route("/api/admin/kyc/:user_id", get(get_kyc_status))
        .route("/api/admin/kyc/approve", post(approve_kyc))
        .route("/api/admin/kyc/reject", post(reject_kyc))
        // Emergency Admin endpoints (pause/unpause/risk-override)
        .route("/api/admin/emergency/pause", post(pause_plan))
        .route("/api/admin/emergency/unpause", post(unpause_plan))
        .route(
            "/api/admin/emergency/risk-override",
            post(set_risk_override),
        )
        .route("/api/admin/emergency/paused-plans", get(get_paused_plans))
        .route(
            "/api/admin/emergency/risk-override-plans",
            get(get_risk_override_plans),
        )
        // ── Emergency Access (Issue #293) ──────────────────────────────────────
        .route(
            "/api/admin/emergency-access/grant",
            post(grant_emergency_access),
        )
        .route(
            "/api/admin/emergency-access/revoke",
            post(revoke_emergency_access),
        )
        .route(
            "/api/admin/emergency-access/all",
            get(get_all_emergency_access),
        )
        .route(
            "/api/admin/emergency-access/active-sessions",
            get(get_active_emergency_sessions),
        )
        .route(
            "/api/admin/emergency-access/plan/:plan_id",
            get(get_plan_emergency_access),
        )
        // ── Stress Testing Endpoints ──────────────────────────────────────────
        .route(
            "/api/admin/stress-test/price-crash",
            post(simulate_price_crash),
        )
        .route(
            "/api/admin/stress-test/mass-default",
            post(simulate_mass_default),
        )
        .route(
            "/api/admin/stress-test/liquidity-drain",
            post(simulate_liquidity_drain),
        )
        // ── Governance Endpoints ──────────────────────────────────────────────
        .route(
            "/api/admin/governance/proposals",
            post(create_governance_proposal),
        )
        .route("/api/governance/proposals", get(list_governance_proposals))
        .route(
            "/api/governance/proposals/:id/vote",
            post(vote_on_governance_proposal),
        )
        .route(
            "/api/admin/governance/parameters/update",
            post(update_protocol_parameter),
        )
        // ── Insurance Fund Monitoring (Issue #249) ───────────────────────────
        .route(
            "/api/admin/insurance-fund",
            get(get_insurance_fund_dashboard),
        )
        .route("/api/admin/insurance-funds", get(get_all_insurance_funds))
        .route(
            "/api/admin/insurance-fund/:fund_id",
            get(get_insurance_fund),
        )
        .route(
            "/api/admin/insurance-fund/:fund_id/metrics",
            get(get_insurance_fund_metrics_history),
        )
        .route(
            "/api/admin/insurance-fund/:fund_id/transactions",
            get(get_insurance_fund_transactions),
        )
        .route(
            "/api/admin/insurance-fund/:fund_id/claims",
            post(create_insurance_claim).get(get_insurance_claims),
        )
        .route(
            "/api/admin/insurance-fund/claims/:claim_id",
            get(get_insurance_claim),
        )
        .route(
            "/api/admin/insurance-fund/claims/:claim_id/process",
            post(process_insurance_claim),
        )
        .route(
            "/api/admin/insurance-fund/claims/:claim_id/payout",
            post(payout_insurance_claim),
        )
        .merge(analytics_router())
        // ── Will PDF & Template Engine (Tasks 1 & 2) ─────────────────────────
        .route(
            "/api/plans/:plan_id/will/generate",
            post(generate_will_document),
        )
        .route("/api/will/documents/:document_id", get(get_will_document))
        .route(
            "/api/plans/:plan_id/will/documents",
            get(list_will_documents),
        )
        // ── Will Version Management ──────────────────────────────────────────
        .route("/api/plans/:plan_id/will/versions", get(list_will_versions))
        .route(
            "/api/plans/:plan_id/will/versions/active",
            get(get_active_will_version),
        )
        .route(
            "/api/plans/:plan_id/will/versions/:version_number",
            get(get_will_version),
        )
        .route(
            "/api/plans/:plan_id/will/versions/:version_number/finalize",
            put(finalize_will_version),
        )
        // ── Beneficiary Sync (Task 3) ─────────────────────────────────────────
        .route(
            "/api/plans/:plan_id/beneficiaries/sync",
            post(sync_beneficiaries),
        )
        // ── Contingent Beneficiaries ──────────────────────────────────────────
        .route(
            "/api/plans/:plan_id/beneficiaries/contingent",
            post(add_contingent_beneficiary).get(get_contingent_beneficiaries),
        )
        .route(
            "/api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id",
            delete(remove_contingent_beneficiary),
        )
        .route(
            "/api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id/promote",
            post(promote_contingent_beneficiary),
        )
        .route(
            "/api/plans/:plan_id/contingency/conditions",
            post(set_contingency_conditions),
        )
        .route(
            "/api/plans/:plan_id/contingency/config",
            get(get_contingency_config),
        )
        // ── Digital Signature (Task 4) ────────────────────────────────────────
        .route(
            "/api/will/documents/:document_id/sign/challenge",
            post(create_signing_challenge),
        )
        .route("/api/will/sign", post(submit_will_signature))
        .route(
            "/api/will/documents/:document_id/signatures",
            get(get_will_signatures),
        )
        // -- Encrypted Document Storage (Issue #328) --
        .route(
            "/api/will/documents/:document_id/encrypt",
            post(encrypt_document),
        )
        .route(
            "/api/will/documents/:document_id/decrypt",
            get(decrypt_document),
        )
        .route(
            "/api/will/documents/:document_id/backup",
            post(create_document_backup),
        )
        .route(
            "/api/will/documents/:document_id/backups",
            get(list_document_backups),
        )
        // -- Will Compliance Validation (Issue #330) --
        .route("/api/will/validate", post(validate_will_compliance))
        .route("/api/will/jurisdictions", get(list_jurisdictions))
        .route(
            "/api/will/jurisdictions/:jurisdiction",
            get(get_jurisdiction_rules),
        )
        // -- Witness Verification (Issue #331) --------------------------------
        .route(
            "/api/will/documents/:document_id/witnesses",
            post(invite_witness).get(list_witnesses),
        )
        .route(
            "/api/will/documents/:document_id/witnesses/status",
            get(get_witness_status),
        )
        .route(
            "/api/will/witnesses/:witness_id/sign",
            post(sign_as_witness),
        )
        .route(
            "/api/will/witnesses/:witness_id/decline",
            post(decline_witness),
        )
        // -- Legal Document Integrity Check (Issue #332) ----------------------
        .route(
            "/api/will/documents/:document_id/verify",
            get(verify_document_integrity),
        )
        .route(
            "/api/will/documents/:document_id/verify/hash",
            post(verify_document_hash),
        )
        .route(
            "/api/will/documents/:document_id/verify/content",
            post(verify_document_content),
        )
        .route(
            "/api/plans/:plan_id/will/verify-all",
            get(verify_all_document_versions),
        )
        // -- Legal Will Event Logging (Issue #333) ----------------------------
        .route(
            "/api/will/documents/:document_id/events",
            get(get_document_events),
        )
        .route("/api/plans/:plan_id/will/events", get(get_plan_events))
        .route("/api/will/vaults/:vault_id/events", get(get_vault_events))
        .route(
            "/api/plans/:plan_id/will/events/stats",
            get(get_plan_event_stats),
        )
        // -- Legal Document Download API (Issue #334) --------------------------
        .route(
            "/api/will/documents/:document_id/download",
            get(download_will_document),
        )
        .route(
            "/api/plans/:plan_id/will/documents/:version/download",
            get(download_will_document_by_version),
        )
        // -- Legal Will Audit Logs (Issue #335) --------------------------------
        .route("/api/admin/will/audit/logs", get(get_admin_audit_logs))
        .route(
            "/api/admin/will/audit/statistics",
            get(get_admin_audit_statistics),
        )
        .route(
            "/api/admin/will/audit/event-types",
            get(get_admin_event_types),
        )
        .route("/api/admin/will/audit/search", get(search_admin_audit_logs))
        .route("/api/admin/logs", get(get_admin_logs))
        .route("/api/notifications", get(get_notifications))
        .route(
            "/api/admin/will/audit/user/:user_id",
            get(get_user_audit_activity),
        )
        .route(
            "/api/will/audit/plan/:plan_id/summary",
            get(get_plan_audit_summary),
        )
        .route("/api/will/audit/my-activity", get(get_my_audit_activity))
        // -- Legacy Content Upload (Issue #XXX) -------------------------------
        .route("/api/content/upload", post(upload_legacy_content))
        .route("/api/content", get(list_user_content))
        .route(
            "/api/content/:content_id",
            get(get_content_by_id).delete(delete_content),
        )
        .route("/api/content/:content_id/download", get(download_content))
        .route("/api/content/stats", get(get_storage_stats))
        .layer(axum::Extension(config.clone()))
        // ── Middleware stack (Issues #408, #409, #423, #424) ─────────────────
        // track_metrics must be outermost so it captures the full request
        // duration including all inner middleware.
        .layer(middleware::from_fn(crate::metrics::track_metrics))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        // Enrich Sentry scope with request_id and user_id after they are set.
        .layer(middleware::from_fn(
            crate::error_tracking::enrich_sentry_context,
        ))
        .layer(middleware::from_fn(move |req, next| {
            request_timeout_middleware(req, next, timeout_duration)
        }))
        .layer(cors_layer)
        // Inject the Prometheus handle so the /metrics handler can render output.
        .layer(axum::Extension(prometheus_handle))
        .with_state(state);

    // Add price feed routes with separate state
    let price_feed_state = (
        db,
        price_feed as Arc<dyn crate::price_feed::PriceFeedService>,
    );
    let price_routes = Router::new()
        .route(
            "/api/prices/:asset_code",
            get(crate::price_feed_handlers::get_price),
        )
        .route(
            "/api/prices/:asset_code/history",
            get(crate::price_feed_handlers::get_price_history),
        )
        .route(
            "/api/prices/:asset_code/valuation/:amount",
            get(crate::price_feed_handlers::calculate_valuation),
        )
        .route(
            "/api/plans/:plan_id/valuation",
            get(crate::price_feed_handlers::get_plan_valuation),
        )
        .route(
            "/api/admin/prices/register",
            post(crate::price_feed_handlers::register_price_feed),
        )
        .route(
            "/api/admin/prices/:asset_code/update",
            post(crate::price_feed_handlers::update_price),
        )
        .route(
            "/api/admin/prices/:asset_code/fetch",
            post(crate::price_feed_handlers::fetch_and_update_price),
        )
        .route(
            "/api/admin/prices/feeds",
            get(crate::price_feed_handlers::get_active_feeds),
        )
        .with_state(price_feed_state);

    Ok(app
        .merge(price_routes)
        .layer(axum::middleware::from_fn(
            crate::middleware::attach_correlation_id,
        ))
        .layer(axum::middleware::from_fn(
            crate::middleware::log_rate_limit_violations,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(GovernorLayer {
            config: governor_conf,
        }))
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "message": "App is healthy" }))
}

/// Liveness + readiness probe for the database (Issue #420).
///
/// Executes a lightweight `SELECT 1` and returns:
/// - `status`: "ok" | "degraded" | "error"
/// - `latency_ms`: round-trip time for the ping query
/// - `pool`: current pool metrics (size, idle, active, utilisation)
///
/// HTTP 200 → healthy or degraded (caller decides alerting threshold).
/// HTTP 503 → database unreachable.
async fn db_health_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let metrics = crate::db::pool_metrics(&state.db);

    match crate::db::ping(&state.db).await {
        Ok(latency_ms) => {
            // Warn in the response body when utilisation is high so
            // operators can spot saturation before it becomes an outage.
            let status = if metrics.utilisation >= 0.9 {
                "degraded"
            } else {
                "ok"
            };

            Ok(Json(json!({
                "status": status,
                "latency_ms": latency_ms,
                "pool": {
                    "size": metrics.size,
                    "idle": metrics.idle,
                    "active": metrics.active,
                    "max_connections": metrics.max_connections,
                    "utilisation": (metrics.utilisation * 100.0).round() / 100.0,
                }
            })))
        }
        Err(e) => {
            tracing::error!(error = %e, "Database health check failed");
            Err(ApiError::Internal(anyhow::anyhow!(
                "Database health check failed: {}",
                e
            )))
        }
    }
}

/// Exposes raw pool metrics for Prometheus / monitoring scraping (Issue #420).
///
/// Returns the same pool statistics as `/health/db` but without the ping
/// latency, so it is safe to call at high frequency from a metrics collector.
async fn db_metrics(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<Value> {
    let m = crate::db::pool_metrics(&state.db);
    Json(json!({
        "pool_size": m.size,
        "pool_idle": m.idle,
        "pool_active": m.active,
        "pool_max_connections": m.max_connections,
        "pool_utilisation": (m.utilisation * 100.0).round() / 100.0,
    }))
}

async fn create_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreatePlanRequest>,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::create_plan(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": plan
    })))
}

async fn get_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::get_plan_by_id(&state.db, plan_id, user.user_id).await?;
    match plan {
        Some(p) => Ok(Json(json!({
            "status": "success",
            "data": p
        }))),
        None => Err(ApiError::NotFound(format!("Plan {plan_id} not found"))),
    }
}

async fn claim_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<ClaimPlanRequest>,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::claim_plan(&state.db, plan_id, user.user_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "message": "Claim recorded",
        "data": plan
    })))
}

async fn get_due_for_claim_plan(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plan = PlanService::get_due_for_claim_plan_by_id(&state.db, plan_id, user.user_id).await?;

    match plan {
        Some(plan) => Ok(Json(json!({
            "status": "success",
            "data": plan
        }))),
        None => Err(ApiError::NotFound(format!(
            "Plan {plan_id} not found or not due for claim"
        ))),
    }
}

async fn create_legacy_message(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateLegacyMessageRequest>,
) -> Result<Json<Value>, ApiError> {
    let message =
        MessageEncryptionService::create_encrypted_message(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": message })))
}

async fn list_legacy_messages(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let messages = MessageEncryptionService::list_owner_messages(&state.db, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": messages, "count": messages.len() }),
    ))
}

async fn list_vault_legacy_messages(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<i64>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let messages =
        MessageEncryptionService::list_vault_messages(&state.db, user.user_id, vault_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": messages, "count": messages.len() }),
    ))
}

async fn list_message_keys(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let keys = MessageKeyService::list_keys(&state.db).await?;
    Ok(Json(
        json!({ "status": "success", "data": keys, "count": keys.len() }),
    ))
}

async fn rotate_message_key(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let key = MessageKeyService::rotate_active_key(&state.db, admin.admin_id).await?;
    Ok(Json(json!({
        "status": "success",
        "message": "Message encryption key rotated",
        "data": key
    })))
}

async fn process_legacy_message_delivery(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let delivery_service = LegacyMessageDeliveryService::new(state.db.clone());
    let result = delivery_service.process_due_messages().await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

// ─── Message Access Audit Handlers ───────────────────────────────────────────

async fn get_message_audit_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Query(filters): Query<MessageAuditFilters>,
) -> Result<Json<Value>, ApiError> {
    let logs = MessageAccessAuditService::get_logs(&state.db, &filters).await?;
    Ok(Json(
        json!({ "status": "success", "data": logs, "count": logs.len() }),
    ))
}

async fn get_message_audit_summary(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let summary = MessageAccessAuditService::get_summary(&state.db).await?;
    Ok(Json(json!({ "status": "success", "data": summary })))
}

async fn search_message_audit_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Query(params): Query<SearchAuditParams>,
) -> Result<Json<Value>, ApiError> {
    let limit = params.limit.unwrap_or(100);
    let logs = MessageAccessAuditService::search_logs(&state.db, &params.q, limit).await?;
    Ok(Json(
        json!({ "status": "success", "data": logs, "count": logs.len() }),
    ))
}

#[derive(Debug, serde::Deserialize)]
struct SearchAuditParams {
    q: String,
    limit: Option<i64>,
}

async fn get_message_access_history(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(message_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let logs = MessageAccessAuditService::get_message_logs(&state.db, message_id, None).await?;
    Ok(Json(
        json!({ "status": "success", "data": logs, "count": logs.len() }),
    ))
}

async fn get_my_message_activity(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let activity = MessageAccessAuditService::get_user_activity(&state.db, user.user_id).await?;
    Ok(Json(json!({ "status": "success", "data": activity })))
}

async fn get_all_due_for_claim_plans_user(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Value>, ApiError> {
    let (_page, limit, offset) = pagination.normalize();
    let plans = PlanService::get_all_due_for_claim_plans_for_user_paginated(
        &state.db,
        user.user_id,
        limit as i64,
        offset,
    )
    .await?;
    let total_count =
        PlanService::count_due_for_claim_plans_for_user(&state.db, user.user_id).await?;

    Ok(Json(json!(pagination.create_response(plans, total_count))))
}

async fn get_all_due_for_claim_plans_admin(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Value>, ApiError> {
    let (_page, limit, offset) = pagination.normalize();
    let plans =
        PlanService::get_all_due_for_claim_plans_admin_paginated(&state.db, limit as i64, offset)
            .await?;
    let total_count = PlanService::count_due_for_claim_plans_admin(&state.db).await?;

    Ok(Json(json!(pagination.create_response(plans, total_count))))
}

async fn list_emergency_contacts(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Value>, ApiError> {
    let (_page, limit, offset) = pagination.normalize();
    let contacts = EmergencyContactService::list_for_user_paginated(
        &state.db,
        user.user_id,
        limit as i64,
        offset,
    )
    .await?;
    let total_count = EmergencyContactService::count_for_user(&state.db, user.user_id).await?;
    Ok(Json(json!(
        pagination.create_response(contacts, total_count)
    )))
}

async fn create_emergency_contact(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateEmergencyContactRequest>,
) -> Result<Json<Value>, ApiError> {
    let contact = EmergencyContactService::create_contact(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": contact })))
}

async fn update_emergency_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<UpdateEmergencyContactRequest>,
) -> Result<Json<Value>, ApiError> {
    let contact =
        EmergencyContactService::update_contact(&state.db, user.user_id, contact_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": contact })))
}

async fn delete_emergency_contact(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let result =
        EmergencyContactService::delete_contact(&state.db, user.user_id, contact_id).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn create_emergency_access_grant(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<CreateEmergencyAccessGrantRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAccessService::grant_access(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn revoke_emergency_access_grant(
    State(state): State<Arc<AppState>>,
    Path(grant_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<RevokeEmergencyAccessGrantRequest>,
) -> Result<Json<Value>, ApiError> {
    let result =
        EmergencyAccessService::revoke_access(&state.db, user.user_id, grant_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn list_emergency_access_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<EmergencyAccessAuditLogFilters>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let logs = EmergencyAccessService::list_audit_logs(&state.db, user.user_id, &filters).await?;
    Ok(Json(
        json!({ "status": "success", "data": logs, "count": logs.len() }),
    ))
}

async fn list_emergency_access_risk_alerts(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let alerts = EmergencyAccessService::list_risk_alerts(&state.db, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": alerts, "count": alerts.len() }),
    ))
}

async fn get_emergency_access_dashboard(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let dashboard = EmergencyAccessService::get_dashboard(&state.db, user.user_id).await?;
    Ok(Json(json!({ "status": "success", "data": dashboard })))
}

// ─── Emergency Access Session Handlers (Issue #306) ────────────────────────────

async fn start_emergency_session(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<StartSessionRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = EmergencySessionService::start_session(&state.db, user.user_id, &req).await?;
    Ok(Json(
        json!({ "status": "success", "data": session, "message": "Session started" }),
    ))
}

async fn heartbeat_emergency_session(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let session = EmergencySessionService::heartbeat(&state.db, user.user_id, session_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": session, "message": "Heartbeat recorded" }),
    ))
}

async fn end_emergency_session(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let session = EmergencySessionService::end_session(&state.db, user.user_id, session_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": session, "message": "Session ended" }),
    ))
}

async fn list_active_emergency_sessions(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let sessions = EmergencySessionService::list_active_sessions(&state.db, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": sessions, "count": sessions.len() }),
    ))
}

#[derive(serde::Deserialize)]
pub struct KycUpdateRequest {
    pub user_id: Uuid,
}

async fn get_kyc_status(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Path(user_id): Path<Uuid>,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::get_kyc_status(&state.db, user_id).await?;
    Ok(Json(status))
}

async fn approve_kyc(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(payload): Json<KycUpdateRequest>,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::update_kyc_status(
        &state.db,
        admin.admin_id,
        payload.user_id,
        KycStatus::Approved,
    )
    .await?;
    Ok(Json(status))
}

async fn reject_kyc(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(payload): Json<KycUpdateRequest>,
) -> Result<Json<KycRecord>, ApiError> {
    let status = KycService::update_kyc_status(
        &state.db,
        admin.admin_id,
        payload.user_id,
        KycStatus::Rejected,
    )
    .await?;
    Ok(Json(status))
}

// Loan Simulation Endpoints

/// Preview loan simulation without saving
async fn simulate_loan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<LoanSimulationRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = LoanSimulationService::create_simulation(&state.db, user.user_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": result
    })))
}

/// Get all loan simulations for the current user
async fn get_user_simulations(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let limit = 50; // Default limit
    let simulations =
        LoanSimulationService::get_user_simulations(&state.db, user.user_id, limit).await?;
    Ok(Json(json!({
        "status": "success",
        "data": simulations,
        "count": simulations.len()
    })))
}

/// Get a specific loan simulation by ID
async fn get_simulation(
    State(state): State<Arc<AppState>>,
    Path(simulation_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let simulation =
        LoanSimulationService::get_simulation_by_id(&state.db, simulation_id, user.user_id).await?;
    match simulation {
        Some(sim) => Ok(Json(json!({
            "status": "success",
            "data": sim
        }))),
        None => Err(ApiError::NotFound(format!(
            "Simulation {simulation_id} not found"
        ))),
    }
}

// Reputation Endpoints

/// Get the current user's borrower reputation
async fn get_user_reputation(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let reputation =
        crate::reputation::ReputationService::get_reputation(&state.db, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": reputation
    })))
}

// Loan Lifecycle Endpoints

/// Open a new loan in the `active` state.
///
/// `POST /api/loans/lifecycle`
async fn create_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<CreateLoanRequest>,
) -> Result<Json<Value>, ApiError> {
    // Override user_id from the authenticated token to prevent impersonation.
    req.user_id = user.user_id;
    let record = LoanLifecycleService::create_loan(&state.db, &req).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// List loans, optionally filtered by status.
///
/// `GET /api/loans/lifecycle[?status=active|repaid|overdue|liquidated&page=1&limit=20]`
async fn list_lifecycle_loans(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(mut filters): Query<LoanListFilters>,
) -> Result<Json<Value>, ApiError> {
    // Users may only see their own loans.
    filters.user_id = Some(user.user_id);

    // Extract pagination from filters
    let pagination = PaginationQuery {
        page: filters.page,
        limit: filters.limit,
    };
    let (_page, limit, offset) = pagination.normalize();

    let loans =
        LoanLifecycleService::list_loans_paginated(&state.db, &filters, limit as i64, offset)
            .await?;
    let total_count = LoanLifecycleService::count_loans(&state.db, &filters).await?;
    Ok(Json(json!(pagination.create_response(loans, total_count))))
}

/// Aggregate counts by lifecycle status for the authenticated user.
///
/// `GET /api/loans/lifecycle/summary`
async fn get_lifecycle_summary(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let summary =
        LoanLifecycleService::get_lifecycle_summary(&state.db, Some(user.user_id)).await?;
    Ok(Json(json!({ "status": "success", "data": summary })))
}

/// Fetch a single loan by its UUID.
///
/// `GET /api/loans/lifecycle/:id`
async fn get_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let record = LoanLifecycleService::get_loan(&state.db, id).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Apply a repayment to a loan.  When cumulative repayments reach or exceed
/// the principal the loan transitions to `repaid`.
///
/// `POST /api/loans/lifecycle/:id/repay`
#[derive(serde::Deserialize)]
struct RepayRequest {
    amount: rust_decimal::Decimal,
}

async fn repay_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<RepayRequest>,
) -> Result<Json<Value>, ApiError> {
    let record = LoanLifecycleService::repay_loan(&state.db, id, user.user_id, req.amount).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Admin: forcefully liquidate a loan.
///
/// `POST /api/admin/loans/lifecycle/:id/liquidate`
async fn liquidate_lifecycle_loan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let record = LoanLifecycleService::liquidate_loan(&state.db, id, admin.admin_id).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Admin: sweep all active loans whose due_date has passed and mark them
/// `overdue`.  Designed to be triggered by a cron / background job.
///
/// `POST /api/admin/loans/lifecycle/mark-overdue`
async fn mark_overdue_loans(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let marked_ids = LoanLifecycleService::mark_overdue_loans(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "marked_overdue": marked_ids.len(),
        "loan_ids": marked_ids
    })))
}

// Emergency Access Endpoints (Issue #293)

use crate::emergency_access::{
    EmergencyAccessService as LegacyEmergencyAccessService, GrantEmergencyAccessRequest,
    RevokeEmergencyAccessRequest,
};

/// Admin: Grant emergency access to a plan
///
/// `POST /api/admin/emergency-access/grant`
async fn grant_emergency_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<GrantEmergencyAccessRequest>,
) -> Result<Json<Value>, ApiError> {
    let response =
        LegacyEmergencyAccessService::grant_access(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": response
    })))
}

/// Admin: Revoke emergency access
///
/// `POST /api/admin/emergency-access/revoke`
async fn revoke_emergency_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<RevokeEmergencyAccessRequest>,
) -> Result<Json<Value>, ApiError> {
    let response =
        LegacyEmergencyAccessService::revoke_access(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({
        "status": "success",
        "data": response
    })))
}

/// Admin: Get all emergency access records
///
/// `GET /api/admin/emergency-access/all`
async fn get_all_emergency_access(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let access_records = LegacyEmergencyAccessService::get_all_access(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "data": access_records,
        "count": access_records.len()
    })))
}

/// Admin: Get emergency access records for a specific plan
///
/// `GET /api/admin/emergency-access/plan/:plan_id`
async fn get_plan_emergency_access(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let access_records =
        LegacyEmergencyAccessService::get_active_access_for_plan(&state.db, plan_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": access_records,
        "count": access_records.len()
    })))
}

/// Admin: Get all active emergency access sessions
///
/// `GET /api/admin/emergency-access/active-sessions`
async fn get_active_emergency_sessions(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let active_sessions = LegacyEmergencyAccessService::get_active_sessions(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "data": active_sessions,
        "count": active_sessions.len()
    })))
}

// Emergency Admin Endpoints

async fn pause_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<PausePlanRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAdminService::pause_plan(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn unpause_plan(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<UnpausePlanRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAdminService::unpause_plan(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn set_risk_override(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<RiskOverrideRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = EmergencyAdminService::set_risk_override(&state.db, admin.admin_id, &req).await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn get_paused_plans(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let plans = EmergencyAdminService::get_paused_plans(&state.db).await?;
    Ok(Json(
        json!({ "status": "success", "data": plans, "count": plans.len() }),
    ))
}

async fn get_risk_override_plans(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let plans = EmergencyAdminService::get_risk_override_plans(&state.db).await?;
    Ok(Json(
        json!({ "status": "success", "data": plans, "count": plans.len() }),
    ))
}
// Stress Testing Endpoints

#[derive(serde::Deserialize)]
pub struct PriceCrashRequest {
    pub asset_code: String,
    pub drop_percentage: rust_decimal::Decimal,
}

#[derive(serde::Deserialize)]
pub struct LiquidityDrainRequest {
    pub asset_code: String,
    pub amount: rust_decimal::Decimal,
}

async fn simulate_price_crash(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Json(req): Json<PriceCrashRequest>,
) -> Result<Json<Value>, ApiError> {
    state
        .stress_testing_engine
        .simulate_price_crash(&req.asset_code, req.drop_percentage)
        .await?;
    Ok(Json(
        json!({ "status": "success", "message": "Price crash simulation completed" }),
    ))
}

async fn simulate_mass_default(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    state.stress_testing_engine.simulate_mass_default().await?;
    Ok(Json(
        json!({ "status": "success", "message": "Mass default simulation completed" }),
    ))
}

async fn simulate_liquidity_drain(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Json(req): Json<LiquidityDrainRequest>,
) -> Result<Json<Value>, ApiError> {
    state
        .stress_testing_engine
        .simulate_liquidity_drain(&req.asset_code, req.amount)
        .await?;
    Ok(Json(
        json!({ "status": "success", "message": "Liquidity drain simulation completed" }),
    ))
}

// Governance Endpoints

async fn create_governance_proposal(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<CreateProposalRequest>,
) -> Result<Json<Proposal>, ApiError> {
    let proposal = GovernanceService::create_proposal(&state.db, admin.admin_id, &req).await?;
    Ok(Json(proposal))
}

async fn list_governance_proposals(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Proposal>>, ApiError> {
    let proposals = GovernanceService::list_proposals(&state.db).await?;
    Ok(Json(proposals))
}

async fn vote_on_governance_proposal(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(proposal_id): Path<Uuid>,
    Json(req): Json<VoteRequest>,
) -> Result<Json<Value>, ApiError> {
    GovernanceService::vote_on_proposal(&state.db, user.user_id, proposal_id, &req).await?;
    Ok(Json(
        json!({ "status": "success", "message": "Vote recorded" }),
    ))
}

async fn update_protocol_parameter(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<ParameterUpdateRequest>,
) -> Result<Json<Value>, ApiError> {
    GovernanceService::update_parameter(&state.db, admin.admin_id, &req).await?;
    Ok(Json(
        json!({ "status": "success", "message": "Parameter updated successfully" }),
    ))
}

// ─── Will PDF & Template Engine Handlers (Tasks 1 & 2) ───────────────────────

#[derive(serde::Deserialize)]
struct GenerateWillRequest {
    owner_name: String,
    owner_wallet: String,
    vault_id: String,
    beneficiaries: Vec<crate::will_pdf::BeneficiaryEntry>,
    execution_rules: Option<String>,
    template: Option<String>,
    jurisdiction: Option<String>,
    will_hash_reference: Option<String>,
}

async fn generate_will_document(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<GenerateWillRequest>,
) -> Result<Json<Value>, ApiError> {
    use std::str::FromStr;
    let template = req
        .template
        .as_deref()
        .map(WillTemplate::from_str)
        .transpose()?
        .unwrap_or(WillTemplate::Formal);

    let input = WillDocumentInput {
        plan_id,
        owner_name: req.owner_name,
        owner_wallet: req.owner_wallet,
        vault_id: req.vault_id,
        beneficiaries: req.beneficiaries,
        execution_rules: req.execution_rules,
        template,
        jurisdiction: req.jurisdiction,
        will_hash_reference: req.will_hash_reference,
    };

    let doc = WillPdfService::generate(&state.db, user.user_id, &input).await?;
    Ok(Json(json!({ "status": "success", "data": doc })))
}

async fn get_will_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let doc = WillPdfService::get_document(&state.db, document_id, user.user_id).await?;
    Ok(Json(json!({ "status": "success", "data": doc })))
}

async fn list_will_documents(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let docs = WillPdfService::list_for_plan(&state.db, plan_id, user.user_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": docs, "count": docs.len() }),
    ))
}

// ─── Will Version Handlers ────────────────────────────────────────────────────

async fn list_will_versions(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let (versions, total) =
        WillVersionService::get_all_versions(&state.db, plan_id, user.user_id, page, per_page)
            .await?;
    Ok(Json(json!({
        "status": "success",
        "data": PaginatedVersions { versions, total, page, per_page }
    })))
}

async fn get_active_will_version(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let version = WillVersionService::get_active_version(&state.db, plan_id, user.user_id).await?;
    Ok(Json(json!({ "status": "success", "data": version })))
}

async fn get_will_version(
    State(state): State<Arc<AppState>>,
    Path((plan_id, version_number)): Path<(Uuid, u32)>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let doc =
        WillVersionService::get_version(&state.db, plan_id, user.user_id, version_number).await?;
    Ok(Json(json!({ "status": "success", "data": doc })))
}

async fn finalize_will_version(
    State(state): State<Arc<AppState>>,
    Path((plan_id, version_number)): Path<(Uuid, u32)>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let version =
        WillVersionService::finalize_version(&state.db, plan_id, user.user_id, version_number)
            .await?;
    Ok(Json(json!({ "status": "success", "data": version })))
}

// ─── Beneficiary Sync Handler (Task 3) ───────────────────────────────────────

#[derive(serde::Deserialize)]
struct SyncBeneficiariesRequest {
    document_beneficiaries: Vec<DocumentBeneficiary>,
}

async fn sync_beneficiaries(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Json(req): Json<SyncBeneficiariesRequest>,
) -> Result<Json<Value>, ApiError> {
    let result =
        BeneficiarySyncService::sync_and_validate(&state.db, plan_id, &req.document_beneficiaries)
            .await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

// ─── Digital Signature Handlers (Task 4) ─────────────────────────────────────

async fn create_signing_challenge(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<SigningChallengeRequest>,
) -> Result<Json<Value>, ApiError> {
    // Bind document_id from path
    req.document_id = document_id;
    // Bind wallet from authenticated user's claims if not provided
    if req.wallet_address.is_empty() {
        req.wallet_address = user.email.clone();
    }
    let challenge = WillSignatureService::create_challenge(&state.db, &req).await?;
    Ok(Json(json!({ "status": "success", "data": challenge })))
}

async fn submit_will_signature(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Json(req): Json<SubmitSignatureRequest>,
) -> Result<Json<Value>, ApiError> {
    let record = WillSignatureService::verify_and_store(&state.db, &req).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

async fn get_will_signatures(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let sigs =
        WillSignatureService::get_signatures_for_document(&state.db, document_id, user.user_id)
            .await?;
    Ok(Json(
        json!({ "status": "success", "data": sigs, "count": sigs.len() }),
    ))
}

// -- Encrypted Document Storage Handlers (Issue #328) -------------------------

async fn encrypt_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let doc = WillPdfService::get_document(&state.db, document_id, user.user_id).await?;
    let content_bytes = base64::engine::general_purpose::STANDARD
        .decode(&doc.pdf_base64)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Base64 decode error: {e}")))?;

    DocumentStorageService::store_encrypted(&state.db, user.user_id, document_id, &content_bytes)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Document encrypted successfully",
        "document_id": document_id
    })))
}

async fn decrypt_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let plaintext =
        DocumentStorageService::retrieve_decrypted(&state.db, user.user_id, document_id).await?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&plaintext);

    Ok(Json(json!({
        "status": "success",
        "document_id": document_id,
        "pdf_base64": encoded
    })))
}

async fn create_document_backup(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let backup =
        DocumentStorageService::create_backup(&state.db, user.user_id, document_id).await?;
    Ok(Json(json!({ "status": "success", "data": backup })))
}

async fn list_document_backups(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let backups =
        DocumentStorageService::list_backups(&state.db, user.user_id, document_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": backups, "count": backups.len() }),
    ))
}

// -- Will Compliance Validation Handlers (Issue #330) --

#[derive(serde::Deserialize)]
struct ValidateWillRequest {
    #[serde(flatten)]
    input: WillDocumentInput,
    witness_count: u32,
}

async fn validate_will_compliance(
    AuthenticatedUser(_user): AuthenticatedUser,
    Json(req): Json<ValidateWillRequest>,
) -> Result<Json<Value>, ApiError> {
    let result: ValidationResult = WillComplianceService::validate(&req.input, req.witness_count);
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn list_jurisdictions(
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let jurisdictions = WillComplianceService::list_supported_jurisdictions();
    Ok(Json(
        json!({ "status": "success", "data": jurisdictions, "count": jurisdictions.len() }),
    ))
}

async fn get_jurisdiction_rules(
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(jurisdiction): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let rules = WillComplianceService::get_jurisdiction_rules(&jurisdiction);
    Ok(Json(json!({ "status": "success", "data": rules })))
}

// -- Witness Verification (Issue #331) ----------------------------------------

async fn invite_witness(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<InviteWitnessRequest>,
) -> Result<Json<Value>, ApiError> {
    let record = WitnessService::invite_witness(
        &state.db,
        user.user_id,
        document_id,
        req.wallet_address,
        req.email,
    )
    .await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

async fn list_witnesses(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let witnesses = WitnessService::get_witnesses(&state.db, user.user_id, document_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": witnesses, "count": witnesses.len() }),
    ))
}

async fn get_witness_status(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let summary = WitnessService::get_witness_status(&state.db, document_id).await?;
    Ok(Json(json!({ "status": "success", "data": summary })))
}

async fn sign_as_witness(
    State(state): State<Arc<AppState>>,
    Path(witness_id): Path<Uuid>,
    Json(req): Json<WitnessSignRequest>,
) -> Result<Json<Value>, ApiError> {
    let record = WitnessService::sign_as_witness(
        &state.db,
        witness_id,
        &req.wallet_address,
        &req.signature_hex,
    )
    .await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

async fn decline_witness(
    State(state): State<Arc<AppState>>,
    Path(witness_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let record = WitnessService::decline_witness(&state.db, witness_id).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

// -- Legal Document Integrity Check (Issue #332) -------------------------------

async fn verify_document_integrity(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let version = params.page; // Reusing page param for version number
    let result = crate::document_verification::DocumentVerificationService::verify_document(
        &state.db,
        document_id,
        version,
    )
    .await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

#[derive(serde::Deserialize)]
struct VerifyHashRequest {
    hash: String,
    version: Option<u32>,
}

async fn verify_document_hash(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    Json(req): Json<VerifyHashRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = crate::document_verification::DocumentVerificationService::verify_hash(
        &state.db,
        document_id,
        req.hash,
        req.version,
    )
    .await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

#[derive(serde::Deserialize)]
struct VerifyContentRequest {
    content: String,
    version: Option<u32>,
}

async fn verify_document_content(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    Json(req): Json<VerifyContentRequest>,
) -> Result<Json<Value>, ApiError> {
    let result = crate::document_verification::DocumentVerificationService::verify_content(
        &state.db,
        document_id,
        req.content,
        req.version,
    )
    .await?;
    Ok(Json(json!({ "status": "success", "data": result })))
}

async fn verify_all_document_versions(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let results = crate::document_verification::DocumentVerificationService::verify_all_versions(
        &state.db, plan_id,
    )
    .await?;
    Ok(Json(
        json!({ "status": "success", "data": results, "count": results.len() }),
    ))
}

// -- Legal Will Event Logging (Issue #333) ------------------------------------

async fn get_document_events(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let events =
        crate::will_events::WillEventService::get_document_events(&state.db, document_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": events, "count": events.len() }),
    ))
}

async fn get_plan_events(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let events = crate::will_events::WillEventService::get_plan_events(&state.db, plan_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": events, "count": events.len() }),
    ))
}

async fn get_vault_events(
    State(state): State<Arc<AppState>>,
    Path(vault_id): Path<String>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let events =
        crate::will_events::WillEventService::get_vault_events(&state.db, &vault_id).await?;
    Ok(Json(
        json!({ "status": "success", "data": events, "count": events.len() }),
    ))
}

async fn get_plan_event_stats(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let stats =
        crate::will_events::WillEventService::get_plan_event_stats(&state.db, plan_id).await?;
    Ok(Json(json!({ "status": "success", "data": stats })))
}

// -- Legal Document Download API (Issue #334) ----------------------------------

/// Download a will document as a PDF file by document ID
///
/// `GET /api/will/documents/:document_id/download`
async fn download_will_document(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<axum::response::Response, ApiError> {
    // Retrieve the document with authentication check
    let doc = WillPdfService::get_document(&state.db, document_id, user.user_id).await?;

    // Decode the base64 PDF content
    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(&doc.pdf_base64)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to decode PDF: {}", e)))?;

    // Emit download event
    let event = crate::will_events::WillEvent::WillDecrypted {
        vault_id: format!("plan_{}", doc.plan_id),
        document_id,
        plan_id: doc.plan_id,
        accessed_by: user.user_id,
        timestamp: chrono::Utc::now(),
    };
    if let Err(e) = crate::will_events::WillEventService::emit(&state.db, event).await {
        tracing::warn!("Failed to emit document download event: {}", e);
    }

    // Build response with proper headers for download
    use axum::body::Body;
    use axum::http::{header, Response, StatusCode};

    let content_disposition = format!("attachment; filename=\"{}\"", doc.filename);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(pdf_bytes))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))
}

/// Download a specific version of a will document by plan ID and version number
///
/// `GET /api/plans/:plan_id/will/documents/:version/download`
async fn download_will_document_by_version(
    State(state): State<Arc<AppState>>,
    Path((plan_id, version)): Path<(Uuid, u32)>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<axum::response::Response, ApiError> {
    // Retrieve the specific version
    let doc = WillVersionService::get_version(&state.db, plan_id, user.user_id, version).await?;

    // Decode the base64 PDF content
    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(&doc.pdf_base64)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to decode PDF: {}", e)))?;

    // Emit download event
    let event = crate::will_events::WillEvent::WillDecrypted {
        vault_id: format!("plan_{}", doc.plan_id),
        document_id: doc.document_id,
        plan_id: doc.plan_id,
        accessed_by: user.user_id,
        timestamp: chrono::Utc::now(),
    };
    if let Err(e) = crate::will_events::WillEventService::emit(&state.db, event).await {
        tracing::warn!("Failed to emit document download event: {}", e);
    }

    // Build response with proper headers for download
    use axum::body::Body;
    use axum::http::{header, Response, StatusCode};

    let content_disposition = format!("attachment; filename=\"{}\"", doc.filename);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(pdf_bytes))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))
}

// -- Legal Will Audit Logs (Issue #335) ----------------------------------------

use crate::will_audit::{AuditLogFilters, WillAuditService};

/// Admin: Get audit logs with filters
///
/// `GET /api/admin/will/audit/logs?document_id=...&plan_id=...&user_id=...&event_type=...&start_date=...&end_date=...&limit=...&offset=...`
async fn get_admin_audit_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Query(filters): Query<AuditLogFilters>,
) -> Result<Json<Value>, ApiError> {
    let logs = WillAuditService::get_audit_logs(&state.db, &filters).await?;
    Ok(Json(json!({
        "status": "success",
        "data": logs,
        "count": logs.len()
    })))
}

/// Admin: Get audit statistics
///
/// `GET /api/admin/will/audit/statistics`
async fn get_admin_audit_statistics(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let stats = WillAuditService::get_admin_statistics(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "data": stats
    })))
}

/// Admin: Get all event types
///
/// `GET /api/admin/will/audit/event-types`
async fn get_admin_event_types(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let event_types = WillAuditService::get_event_types(&state.db).await?;
    Ok(Json(json!({
        "status": "success",
        "data": event_types,
        "count": event_types.len()
    })))
}

/// Admin: Search audit logs
///
/// `GET /api/admin/will/audit/search?q=...&limit=...`
#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i64>,
}

async fn search_admin_audit_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, ApiError> {
    let limit = query.limit.unwrap_or(100);
    let logs = WillAuditService::search_audit_logs(&state.db, &query.q, limit).await?;
    Ok(Json(json!({
        "status": "success",
        "data": logs,
        "count": logs.len()
    })))
}

/// Admin: Get user audit activity
///
/// `GET /api/admin/will/audit/user/:user_id`
async fn get_user_audit_activity(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let activity = WillAuditService::get_user_activity_summary(&state.db, user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": activity
    })))
}

/// User: Get plan audit summary
///
/// `GET /api/will/audit/plan/:plan_id/summary`
async fn get_plan_audit_summary(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let summary = WillAuditService::get_plan_audit_summary(&state.db, plan_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": summary
    })))
}

/// User: Get my audit activity
///
/// `GET /api/will/audit/my-activity`
async fn get_my_audit_activity(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let activity = WillAuditService::get_user_activity_summary(&state.db, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": activity
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Insurance Fund Monitoring (Issue #249)
// ─────────────────────────────────────────────────────────────────────────────

/// Admin: Get insurance fund dashboard
///
/// `GET /api/admin/insurance-fund`
async fn get_insurance_fund_dashboard(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let fund = state.insurance_fund_service.get_primary_fund().await?;
    let dashboard = state.insurance_fund_service.get_dashboard(fund.id).await?;

    Ok(Json(json!({
        "status": "success",
        "data": dashboard
    })))
}

/// Admin: Get all insurance funds
///
/// `GET /api/admin/insurance-funds`
async fn get_all_insurance_funds(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let funds = state.insurance_fund_service.get_all_funds().await?;

    Ok(Json(json!({
        "status": "success",
        "data": funds,
        "count": funds.len()
    })))
}

/// Admin: Get insurance fund by ID
///
/// `GET /api/admin/insurance-fund/:fund_id`
async fn get_insurance_fund(
    State(state): State<Arc<AppState>>,
    Path(fund_id): Path<Uuid>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let fund = state.insurance_fund_service.get_fund_by_id(fund_id).await?;

    Ok(Json(json!({
        "status": "success",
        "data": fund
    })))
}

/// Admin: Get insurance fund metrics history
///
/// `GET /api/admin/insurance-fund/:fund_id/metrics?days=30`
#[derive(serde::Deserialize)]
struct MetricsHistoryQuery {
    days: Option<i64>,
}

async fn get_insurance_fund_metrics_history(
    State(state): State<Arc<AppState>>,
    Path(fund_id): Path<Uuid>,
    Query(query): Query<MetricsHistoryQuery>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let days = query.days.unwrap_or(30);
    let history = state
        .insurance_fund_service
        .get_metrics_history(fund_id, days)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "data": history,
        "count": history.len()
    })))
}

/// Admin: Get insurance fund transactions
///
/// `GET /api/admin/insurance-fund/:fund_id/transactions?limit=50`
#[derive(serde::Deserialize)]
struct TransactionsQuery {
    limit: Option<i64>,
}

async fn get_insurance_fund_transactions(
    State(state): State<Arc<AppState>>,
    Path(fund_id): Path<Uuid>,
    Query(query): Query<TransactionsQuery>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let limit = query.limit.unwrap_or(50);

    let transactions = sqlx::query_as::<_, crate::insurance_fund::InsuranceFundTransaction>(
        "SELECT * FROM insurance_fund_transactions WHERE fund_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(fund_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching transactions: {}", e)))?;

    Ok(Json(json!({
        "status": "success",
        "data": transactions,
        "count": transactions.len()
    })))
}

/// Admin: Get insurance claims
///
/// `GET /api/admin/insurance-fund/:fund_id/claims?status=pending&limit=50`
#[derive(serde::Deserialize)]
struct ClaimsQuery {
    status: Option<String>,
    limit: Option<i64>,
}

async fn get_insurance_claims(
    State(state): State<Arc<AppState>>,
    Path(fund_id): Path<Uuid>,
    Query(query): Query<ClaimsQuery>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let limit = query.limit.unwrap_or(50);

    let query_builder = if let Some(status) = &query.status {
        sqlx::query_as::<_, crate::insurance_fund::InsuranceClaim>(
            "SELECT * FROM insurance_claims WHERE fund_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3",
        )
        .bind(fund_id)
        .bind(status)
        .bind(limit)
    } else {
        sqlx::query_as::<_, crate::insurance_fund::InsuranceClaim>(
            "SELECT * FROM insurance_claims WHERE fund_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(fund_id)
        .bind(limit)
    };

    let claims = query_builder
        .fetch_all(&state.db)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching claims: {}", e)))?;

    Ok(Json(json!({
        "status": "success",
        "data": claims,
        "count": claims.len()
    })))
}

/// Admin: Get insurance claim by ID
///
/// `GET /api/admin/insurance-fund/claims/:claim_id`
async fn get_insurance_claim(
    State(state): State<Arc<AppState>>,
    Path(claim_id): Path<Uuid>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    let claim = sqlx::query_as::<_, crate::insurance_fund::InsuranceClaim>(
        "SELECT * FROM insurance_claims WHERE id = $1",
    )
    .bind(claim_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error fetching claim: {}", e)))?
    .ok_or_else(|| ApiError::NotFound(format!("Insurance claim {} not found", claim_id)))?;

    Ok(Json(json!({
        "status": "success",
        "data": claim
    })))
}

/// Admin: Create insurance claim
///
/// `POST /api/admin/insurance-fund/:fund_id/claims`
async fn create_insurance_claim(
    State(state): State<Arc<AppState>>,
    Path(fund_id): Path<Uuid>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<CreateInsuranceClaimRequest>,
) -> Result<Json<Value>, ApiError> {
    let claim = state
        .insurance_fund_service
        .create_claim(fund_id, admin.admin_id, &req)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "data": claim
    })))
}

/// Admin: Process insurance claim (approve/reject)
///
/// `POST /api/admin/insurance-fund/claims/:claim_id/process`
async fn process_insurance_claim(
    State(state): State<Arc<AppState>>,
    Path(claim_id): Path<Uuid>,
    AuthenticatedAdmin(admin): AuthenticatedAdmin,
    Json(req): Json<ProcessInsuranceClaimRequest>,
) -> Result<Json<Value>, ApiError> {
    let claim = state
        .insurance_fund_service
        .process_claim(claim_id, admin.admin_id, &req)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "data": claim
    })))
}

/// Admin: Payout approved insurance claim
///
/// `POST /api/admin/insurance-fund/claims/:claim_id/payout`
async fn payout_insurance_claim(
    State(state): State<Arc<AppState>>,
    Path(claim_id): Path<Uuid>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
) -> Result<Json<Value>, ApiError> {
    state.insurance_fund_service.payout_claim(claim_id).await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Claim paid out successfully"
    })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy Content Handlers (Issue #XXX)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct UploadContentRequest {
    pub original_filename: String,
    pub content_type: String,
    pub description: Option<String>,
}

/// User: Upload legacy content
///
/// `POST /api/content/upload`
async fn upload_legacy_content(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<UploadContentRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate the content type
    LegacyContentService::validate_content_type(&req.content_type)?;

    // For now, we'll create a metadata record. Full implementation would handle file upload.
    let metadata = crate::legacy_content::UploadMetadata {
        original_filename: req.original_filename,
        content_type: req.content_type.clone(),
        file_size: 0, // Would be set from actual file upload
        description: req.description,
    };

    let storage_path =
        LegacyContentService::generate_storage_path(user.user_id, &metadata.original_filename);
    let file_hash = "pending".to_string(); // Would be calculated from file content

    let content = LegacyContentService::create_content_record(
        &state.db,
        user.user_id,
        &metadata,
        storage_path,
        file_hash,
    )
    .await?;

    Ok(Json(json!({
        "status": "success",
        "data": content
    })))
}

/// User: List legacy content
///
/// `GET /api/content?content_type_prefix=video&limit=50&offset=0`
async fn list_user_content(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(filters): Query<ContentListFilters>,
) -> Result<Json<Value>, ApiError> {
    let contents =
        LegacyContentService::list_user_content(&state.db, user.user_id, &filters).await?;
    Ok(Json(json!({
        "status": "success",
        "data": contents,
        "count": contents.len()
    })))
}

/// User: Get content by ID
///
/// `GET /api/content/:content_id`
async fn get_content_by_id(
    State(state): State<Arc<AppState>>,
    Path(content_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let content =
        LegacyContentService::get_content_by_id(&state.db, content_id, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": content
    })))
}

/// User: Delete content (soft delete)
///
/// `DELETE /api/content/:content_id`
async fn delete_content(
    State(state): State<Arc<AppState>>,
    Path(content_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    LegacyContentService::delete_content(&state.db, content_id, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "message": "Content deleted successfully"
    })))
}

/// User: Download content
///
/// `GET /api/content/:content_id/download`
async fn download_content(
    State(state): State<Arc<AppState>>,
    Path(content_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<axum::response::Response, ApiError> {
    let content =
        LegacyContentService::get_content_by_id(&state.db, content_id, user.user_id).await?;

    // In a full implementation, this would read from the FileStorageService
    // For now, return a placeholder response
    use axum::body::Body;
    use axum::http::{header, Response, StatusCode};

    let content_disposition = format!("attachment; filename=\"{}\"", content.original_filename);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &content.content_type)
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
        .body(Body::empty())
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to build response: {}", e)))
}

/// User: Get storage statistics
///
/// `GET /api/content/stats`
async fn get_storage_stats(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let stats = LegacyContentService::get_user_storage_stats(&state.db, user.user_id).await?;
    Ok(Json(json!({
        "status": "success",
        "data": stats
    })))
}

/// User: Get notifications
async fn get_notifications(
    State(state): State<Arc<AppState>>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Value>, ApiError> {
    let (page, limit, _offset) = pagination.normalize();
    let notifications = crate::notifications::NotificationService::list_for_user_paginated(
        &state.db,
        user.user_id,
        page,
        limit,
    )
    .await?;
    let total_count =
        crate::notifications::NotificationService::count_for_user(&state.db, user.user_id).await?;

    Ok(Json(json!(
        pagination.create_response(notifications, total_count)
    )))
}

/// Admin: Get audit logs
async fn get_admin_logs(
    State(state): State<Arc<AppState>>,
    AuthenticatedAdmin(_admin): AuthenticatedAdmin,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Value>, ApiError> {
    let (page, limit, _offset) = pagination.normalize();
    let logs =
        crate::notifications::AuditLogService::list_all_paginated(&state.db, page, limit).await?;
    let total_count = crate::notifications::AuditLogService::count_all(&state.db).await?;

    Ok(Json(json!(pagination.create_response(logs, total_count))))
}

// ─────────────────────────────────────────────────────────────────────────────
// Collateral Management Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// Add collateral to an existing active loan.
///
/// `POST /api/loans/lifecycle/:id/collateral/add`
async fn add_collateral(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<AddCollateralRequest>,
) -> Result<Json<Value>, ApiError> {
    req.loan_id = id;
    req.user_id = user.user_id;
    let record = CollateralManagementService::add_collateral(&state.db, &req).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Remove collateral from an existing active loan.
///
/// `POST /api/loans/lifecycle/:id/collateral/remove`
async fn remove_collateral(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<RemoveCollateralRequest>,
) -> Result<Json<Value>, ApiError> {
    req.loan_id = id;
    req.user_id = user.user_id;
    let price_feed = Arc::new(crate::price_feed::DefaultPriceFeedService::new(
        state.db.clone(),
        3600,
    ));
    let record =
        CollateralManagementService::remove_collateral(&state.db, price_feed, &req).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Swap collateral type for an existing active loan.
///
/// `POST /api/loans/lifecycle/:id/collateral/swap`
async fn swap_collateral(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<SwapCollateralRequest>,
) -> Result<Json<Value>, ApiError> {
    req.loan_id = id;
    req.user_id = user.user_id;
    let price_feed = Arc::new(crate::price_feed::DefaultPriceFeedService::new(
        state.db.clone(),
        3600,
    ));
    let record = CollateralManagementService::swap_collateral(&state.db, price_feed, &req).await?;
    Ok(Json(json!({ "status": "success", "data": record })))
}

/// Get current collateral value in USD for a loan.
///
/// `GET /api/loans/lifecycle/:id/collateral/value`
async fn get_collateral_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let price_feed = Arc::new(crate::price_feed::DefaultPriceFeedService::new(
        state.db.clone(),
        3600,
    ));
    let info = CollateralManagementService::get_collateral_value(&state.db, price_feed, id).await?;
    Ok(Json(json!({ "status": "success", "data": info })))
}

/// Get maximum withdrawable collateral amount while maintaining health factor >= 150%.
///
/// `GET /api/loans/lifecycle/:id/collateral/max-withdrawable`
async fn get_max_withdrawable_collateral(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let price_feed = Arc::new(crate::price_feed::DefaultPriceFeedService::new(
        state.db.clone(),
        3600,
    ));
    let info =
        CollateralManagementService::get_max_withdrawable_collateral(&state.db, price_feed, id)
            .await?;
    Ok(Json(json!({ "status": "success", "data": info })))
}

/// Get required collateral amount for a given loan.
///
/// `GET /api/loans/lifecycle/:id/collateral/requirements`
async fn get_collateral_requirements(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let price_feed = Arc::new(crate::price_feed::DefaultPriceFeedService::new(
        state.db.clone(),
        3600,
    ));
    let reqs =
        CollateralManagementService::get_required_collateral(&state.db, price_feed, id).await?;
    Ok(Json(json!({ "status": "success", "data": reqs })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Contingent Beneficiary Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// Add a contingent beneficiary to a plan.
///
/// `POST /api/plans/:plan_id/beneficiaries/contingent`
async fn add_contingent_beneficiary(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<AddContingentBeneficiaryRequest>,
) -> Result<Json<Value>, ApiError> {
    req.plan_id = plan_id;
    let beneficiary =
        ContingentBeneficiaryService::add_contingent_beneficiary(&state.db, user.user_id, &req)
            .await?;
    Ok(Json(json!({ "status": "success", "data": beneficiary })))
}

/// Remove a contingent beneficiary from a plan.
///
/// `DELETE /api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id`
async fn remove_contingent_beneficiary(
    State(state): State<Arc<AppState>>,
    Path((_plan_id, beneficiary_id)): Path<(Uuid, Uuid)>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let req = RemoveContingentBeneficiaryRequest { beneficiary_id };
    ContingentBeneficiaryService::remove_contingent_beneficiary(&state.db, user.user_id, &req)
        .await?;
    Ok(Json(
        json!({ "status": "success", "message": "Contingent beneficiary removed" }),
    ))
}

/// Get all contingent beneficiaries for a plan.
///
/// `GET /api/plans/:plan_id/beneficiaries/contingent`
async fn get_contingent_beneficiaries(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let beneficiaries =
        ContingentBeneficiaryService::get_contingent_beneficiaries(&state.db, plan_id).await?;
    Ok(Json(json!({ "status": "success", "data": beneficiaries })))
}

/// Promote a contingent beneficiary to primary.
///
/// `POST /api/plans/:plan_id/beneficiaries/contingent/:beneficiary_id/promote`
async fn promote_contingent_beneficiary(
    State(state): State<Arc<AppState>>,
    Path((_plan_id, beneficiary_id)): Path<(Uuid, Uuid)>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(req): Json<PromoteContingentRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut promote_req = req;
    promote_req.beneficiary_id = beneficiary_id;
    let promoted =
        ContingentBeneficiaryService::promote_contingent(&state.db, user.user_id, &promote_req)
            .await?;
    Ok(Json(json!({ "status": "success", "data": promoted })))
}

/// Set contingency conditions for a plan.
///
/// `POST /api/plans/:plan_id/contingency/conditions`
async fn set_contingency_conditions(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(mut req): Json<SetContingencyConditionsRequest>,
) -> Result<Json<Value>, ApiError> {
    req.plan_id = plan_id;
    ContingentBeneficiaryService::set_contingency_conditions(&state.db, user.user_id, &req).await?;
    Ok(Json(
        json!({ "status": "success", "message": "Contingency conditions set" }),
    ))
}

/// Get contingency configuration for a plan.
///
/// `GET /api/plans/:plan_id/contingency/config`
async fn get_contingency_config(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let config = ContingentBeneficiaryService::get_or_create_config(&state.db, plan_id).await?;
    Ok(Json(json!({ "status": "success", "data": config })))
}
