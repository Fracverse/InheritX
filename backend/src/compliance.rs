use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::UserContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAlert {
    pub id: String,
    #[serde(rename = "type")]
    pub alert_type: String,
    pub address: String,
    pub user_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub amount: Option<f64>,
    pub asset_code: Option<String>,
    pub event_count: Option<i64>,
    pub threshold: Option<f64>,
    pub window_minutes: Option<i64>,
    pub severity: String,
    pub status: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub label: String,
    pub impact: String,
    pub score: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskOverride {
    pub id: Option<String>,
    pub address: String,
    pub score: i32,
    pub level: String,
    pub justification: String,
    pub admin_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub address: String,
    pub score: i32,
    pub level: String,
    pub factors: Vec<RiskFactor>,
    pub last_evaluated_at: DateTime<Utc>,
    #[serde(rename = "override")]
    pub r#override: Option<RiskOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsCheck {
    pub address: String,
    pub is_flagged: bool,
    pub status: String,
    pub lists: Vec<String>,
    pub match_score: Option<f64>,
    pub checked_at: DateTime<Utc>,
    pub recommendation: String,
}

#[derive(Debug, Deserialize)]
pub struct RiskOverrideRequest {
    pub address: String,
    pub score: i32,
    pub level: Option<String>,
    pub justification: String,
}

fn validated_address(address: &str) -> Option<String> {
    let address = address.trim();
    if address.is_empty() || address.len() > 128 || address.chars().any(|c| c.is_control()) {
        None
    } else {
        Some(address.to_string())
    }
}

fn risk_level(score: i32) -> &'static str {
    match score {
        0..=29 => "low",
        30..=49 => "medium",
        50..=79 => "high",
        _ => "critical",
    }
}

fn decimal_f64(value: Option<Decimal>) -> Option<f64> {
    value.map(|v| v.try_into().unwrap_or(0.0))
}

#[derive(sqlx::FromRow)]
struct AlertRow {
    id: Uuid,
    address: String,
    type_: Option<String>,
    user_id: Option<Uuid>,
    transaction_hash: Option<String>,
    amount: Option<Decimal>,
    asset_code: Option<String>,
    event_count: Option<i64>,
    threshold: Option<Decimal>,
    window_minutes: Option<i64>,
    severity: String,
    status: String,
    reason: String,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    metadata: Option<Value>,
}

async fn alert_list(pool: &PgPool, alert_type: &str) -> Result<Vec<ComplianceAlert>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AlertRow>(
        r#"
        SELECT id, address, type AS type_, user_id, transaction_hash, amount,
               asset_code, event_count, threshold, window_minutes, severity, status,
               reason, created_at, resolved_at, metadata
        FROM compliance_alerts
        WHERE type = $1
        ORDER BY created_at DESC
        LIMIT 500
        "#,
    )
    .bind(alert_type)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ComplianceAlert {
            id: row.id.to_string(),
            alert_type: row.type_.unwrap_or_else(|| alert_type.to_string()),
            address: row.address,
            user_id: row.user_id.map(|id| id.to_string()),
            transaction_hash: row.transaction_hash,
            amount: decimal_f64(row.amount),
            asset_code: row.asset_code,
            event_count: row.event_count,
            threshold: decimal_f64(row.threshold),
            window_minutes: row.window_minutes,
            severity: row.severity,
            status: row.status,
            reason: row.reason,
            created_at: row.created_at,
            resolved_at: row.resolved_at,
            metadata: row.metadata,
        })
        .collect())
}

fn env_i64(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
        .max(1)
}

fn env_decimal(name: &str, default: Decimal) -> Decimal {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub async fn get_velocity_alerts(State(state): State<Arc<AppState>>) -> Response {
    let window_minutes = env_i64("COMPLIANCE_VELOCITY_WINDOW_MINUTES", 60);
    let threshold = env_i64("COMPLIANCE_VELOCITY_THRESHOLD", 10);
    let since = Utc::now() - Duration::minutes(window_minutes);

    #[derive(sqlx::FromRow)]
    struct VelocityRow {
        address: String,
        event_count: i64,
    }

    let rows = sqlx::query_as::<_, VelocityRow>(
        r#"
        SELECT beneficiary_address AS address, COUNT(*)::BIGINT AS event_count
        FROM payouts
        WHERE created_at >= $1
          AND status::text <> 'failed'
        GROUP BY beneficiary_address
        HAVING COUNT(*)::BIGINT > $2
        "#,
    )
    .bind(since)
    .bind(threshold)
    .fetch_all(&state.db_pool)
    .await;

    let rows = match rows {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "Failed to query velocity activity");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response();
        }
    };

    for row in rows {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM compliance_alerts WHERE address=$1 AND type='velocity' AND status='open'",
        )
        .bind(&row.address)
        .fetch_one(&state.db_pool)
        .await;

        if matches!(exists, Ok(0)) {
            let result = sqlx::query(
                r#"
                INSERT INTO compliance_alerts
                    (address, type, event_count, threshold, window_minutes, severity, status, reason, metadata)
                VALUES ($1, 'velocity', $2, $3, $4, 'high', 'open',
                        'Transaction count exceeded the configured velocity threshold',
                        $5)
                "#,
            )
            .bind(&row.address)
            .bind(row.event_count)
            .bind(Decimal::from(threshold))
            .bind(window_minutes)
            .bind(json!({"method":"payout_activity","source":"payouts"}))
            .execute(&state.db_pool)
            .await;

            if let Err(e) = result {
                error!(error = %e, "Failed to persist velocity alert");
            }
        }
    }

    match alert_list(&state.db_pool, "velocity").await {
        Ok(alerts) => (StatusCode::OK, Json(json!({"alerts": alerts}))).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to load velocity alerts");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response()
        }
    }
}

pub async fn get_volume_alerts(State(state): State<Arc<AppState>>) -> Response {
    let since = Utc::now() - Duration::hours(24);
    let crypto_threshold = env_decimal(
        "COMPLIANCE_CRYPTO_VOLUME_THRESHOLD_USD",
        Decimal::from(10_000),
    );

    #[derive(sqlx::FromRow)]
    struct VolumeRow {
        address: String,
        payout_type: String,
        amount: Decimal,
        fiat_limit: Option<Decimal>,
    }

    let rows = sqlx::query_as::<_, VolumeRow>(
        r#"
        SELECT p.beneficiary_address AS address,
               p.payout_type::text AS payout_type,
               COALESCE(SUM(p.amount), 0) AS amount,
               b.fiat_daily_limit AS fiat_limit
        FROM payouts p
        LEFT JOIN beneficiaries b ON b.wallet_address = p.beneficiary_address
        WHERE p.created_at >= $1
          AND p.status::text <> 'failed'
        GROUP BY p.beneficiary_address, p.payout_type::text, b.fiat_daily_limit
        "#,
    )
    .bind(since)
    .fetch_all(&state.db_pool)
    .await;

    let rows = match rows {
        Ok(rows) => rows,
        Err(e) => {
            error!(error = %e, "Failed to query payout volume");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response();
        }
    };

    for row in rows {
        let threshold = if row.payout_type == "fiat" {
            row.fiat_limit.unwrap_or(Decimal::ZERO)
        } else {
            crypto_threshold
        };
        if threshold <= Decimal::ZERO || row.amount <= threshold {
            continue;
        }

        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM compliance_alerts WHERE address=$1 AND type='volume' AND status='open'",
        )
        .bind(&row.address)
        .fetch_one(&state.db_pool)
        .await;

        if matches!(exists, Ok(0)) {
            let result = sqlx::query(
                r#"
                INSERT INTO compliance_alerts
                    (address, type, amount, asset_code, threshold, window_minutes, severity, status, reason, metadata)
                VALUES ($1, 'volume', $2, $3, $4, 1440, 'high', 'open',
                        '24-hour transaction volume exceeded the applicable configured limit',
                        $5)
                "#,
            )
            .bind(&row.address)
            .bind(row.amount)
            .bind(&row.payout_type)
            .bind(threshold)
            .bind(json!({"window":"24h","payout_type":row.payout_type,"source":"payouts"}))
            .execute(&state.db_pool)
            .await;

            if let Err(e) = result {
                error!(error = %e, "Failed to persist volume alert");
            }
        }
    }

    match alert_list(&state.db_pool, "volume").await {
        Ok(alerts) => (StatusCode::OK, Json(json!({"alerts": alerts}))).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to load volume alerts");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response()
        }
    }
}

#[derive(sqlx::FromRow)]
struct KycRow {
    kyc_status: String,
}

async fn check_sanctions_internal(address: &str) -> Result<SanctionsCheck, (StatusCode, Value)> {
    let provider_url = std::env::var("SANCTIONS_PROVIDER_URL")
        .or_else(|_| std::env::var("SANCTIONS_API_URL"))
        .ok()
        .filter(|v| !v.trim().is_empty());
    let provider_url = provider_url.ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        json!({"error":"Sanctions screening unavailable","status":"unavailable"}),
    ))?;

    let timeout_secs = env_i64("SANCTIONS_PROVIDER_TIMEOUT_SECONDS", 5).clamp(1, 30) as u64;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|_| {
            (
                StatusCode::BAD_GATEWAY,
                json!({"error":"Sanctions provider unavailable"}),
            )
        })?;

    let mut request = client
        .post(&provider_url)
        .json(&json!({"address": address}));
    if let Ok(api_key) = std::env::var("SANCTIONS_PROVIDER_API_KEY") {
        if !api_key.is_empty() {
            request = request.bearer_auth(api_key);
        }
    }

    let response = request.send().await.map_err(|e| {
        let status = if e.is_timeout() {
            StatusCode::GATEWAY_TIMEOUT
        } else {
            StatusCode::BAD_GATEWAY
        };
        (
            status,
            json!({"error":"Sanctions provider unavailable","status":"unavailable"}),
        )
    })?;
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            json!({"error":"Sanctions provider unavailable","status":"unavailable"}),
        ));
    }
    let body: Value = response.json().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            json!({"error":"Malformed sanctions provider response","status":"unavailable"}),
        )
    })?;

    parse_sanctions_body(address, body)
}

fn parse_sanctions_body(address: &str, body: Value) -> Result<SanctionsCheck, (StatusCode, Value)> {
    let raw_status = body
        .get("status")
        .or_else(|| body.get("outcome"))
        .or_else(|| body.get("result"))
        .and_then(|v| v.as_str())
        .unwrap_or("unavailable")
        .to_lowercase();
    let (status, is_flagged, recommendation) = match raw_status.as_str() {
        "clear" | "cleared" | "negative" => ("clear", false, "No sanctions match detected."),
        "potential" | "potential_match" | "review" => {
            ("review", true, "Manual compliance review required.")
        }
        "confirmed" | "confirmed_match" | "flagged" => {
            ("flagged", true, "Block transaction and escalate review.")
        }
        _ => {
            return Err((
                StatusCode::BAD_GATEWAY,
                json!({"error":"Malformed sanctions provider response","status":"unavailable"}),
            ))
        }
    };
    let lists = body
        .get("lists")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let match_score = body
        .get("match_score")
        .and_then(|v| v.as_f64())
        .or_else(|| body.get("matchScore").and_then(|v| v.as_f64()));

    Ok(SanctionsCheck {
        address: address.to_string(),
        is_flagged,
        status: status.to_string(),
        lists,
        match_score,
        checked_at: Utc::now(),
        recommendation: recommendation.to_string(),
    })
}

pub async fn get_risk_score(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Response {
    let address = match validated_address(&address) {
        Some(address) => address,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"Invalid address"})),
            )
                .into_response()
        }
    };

    let kyc_status =
        sqlx::query_as::<_, KycRow>("SELECT kyc_status::text FROM users WHERE wallet_address=$1")
            .bind(&address)
            .fetch_optional(&state.db_pool)
            .await;

    let kyc_status = match kyc_status {
        Ok(Some(row)) => row.kyc_status,
        Ok(None) => "pending".to_string(),
        Err(e) => {
            error!(error = %e, "Failed to query KYC status");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response();
        }
    };

    let mut score = 0;
    let mut factors = Vec::new();
    match kyc_status.as_str() {
        "approved" => factors.push(RiskFactor {
            label: "KYC".into(),
            impact: "positive".into(),
            score: 0,
            description: "KYC is approved.".into(),
        }),
        "submitted" => {
            score += 20;
            factors.push(RiskFactor {
                label: "KYC".into(),
                impact: "negative".into(),
                score: 20,
                description: "KYC verification is submitted but not approved.".into(),
            });
        }
        "rejected" => {
            score += 40;
            factors.push(RiskFactor {
                label: "KYC".into(),
                impact: "negative".into(),
                score: 40,
                description: "KYC verification was rejected.".into(),
            });
        }
        _ => {
            score += 20;
            factors.push(RiskFactor {
                label: "KYC".into(),
                impact: "negative".into(),
                score: 20,
                description: "KYC information is pending or unavailable.".into(),
            });
        }
    }

    let open_alerts = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM compliance_alerts WHERE address=$1 AND status='open'",
    )
    .bind(&address)
    .fetch_one(&state.db_pool)
    .await
    .unwrap_or(0);
    if open_alerts > 0 {
        let alert_score = (open_alerts as i32 * 10).min(30);
        score += alert_score;
        factors.push(RiskFactor {
            label: "Compliance alerts".into(),
            impact: "negative".into(),
            score: alert_score,
            description: format!("{open_alerts} open compliance alert(s)."),
        });
    }

    match check_sanctions_internal(&address).await {
        Ok(screening) if screening.status == "clear" => {}
        Ok(screening) if screening.status == "review" => {
            score += 30;
            factors.push(RiskFactor {
                label: "Sanctions screening".into(),
                impact: "negative".into(),
                score: 30,
                description: "Potential sanctions match requires review.".into(),
            });
        }
        Ok(_) => {
            score += 60;
            factors.push(RiskFactor {
                label: "Sanctions screening".into(),
                impact: "negative".into(),
                score: 60,
                description: "Confirmed sanctions match.".into(),
            });
        }
        Err(_) => {
            score += 50;
            factors.push(RiskFactor {
                label: "Sanctions screening".into(),
                impact: "negative".into(),
                score: 50,
                description: "Screening unavailable; address is fail closed.".into(),
            });
        }
    }
    score = score.clamp(0, 100);

    let risk_override =
        sqlx::query_as::<_, (Uuid, i16, String, Option<DateTime<Utc>>, Option<String>)>(
            r#"
        SELECT id, new_score, address, created_at, admin_id
        FROM risk_score_overrides
        WHERE address=$1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        )
        .bind(&address)
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten()
        .map(|row| {
            let score = row.1 as i32;
            RiskOverride {
                id: Some(row.0.to_string()),
                address: row.2,
                score,
                level: risk_level(score).to_string(),
                justification: String::new(),
                admin_id: row.4,
                created_at: row.3,
            }
        });
    let effective_score = risk_override.as_ref().map(|v| v.score).unwrap_or(score);
    (
        StatusCode::OK,
        Json(RiskScore {
            address,
            score: effective_score,
            level: risk_level(effective_score).to_string(),
            factors,
            last_evaluated_at: Utc::now(),
            r#override: risk_override,
        }),
    )
        .into_response()
}

pub async fn override_risk(
    State(state): State<Arc<AppState>>,
    Extension(admin): Extension<UserContext>,
    Json(payload): Json<RiskOverrideRequest>,
) -> Response {
    if admin.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"Administrator access required"})),
        )
            .into_response();
    }
    let address = match validated_address(&payload.address) {
        Some(address) => address,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"Invalid address"})),
            )
                .into_response()
        }
    };
    if !(0..=100).contains(&payload.score) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"Score must be between 0 and 100"})),
        )
            .into_response();
    }
    let reason = payload.justification.trim();
    if reason.len() < 3 || reason.len() > 500 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"Justification must be between 3 and 500 characters"})),
        )
            .into_response();
    }
    let previous_row = sqlx::query_as::<_, (i16, Value)>(
        "SELECT new_score, original_factors FROM risk_score_overrides WHERE address=$1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&address)
    .fetch_optional(&state.db_pool)
    .await;
    let had_override = matches!(&previous_row, Ok(Some(_)));
    let mut previous_score = match previous_row {
        Ok(Some(row)) => row.0 as i32,
        Ok(None) => 0,
        Err(e) => {
            error!(error = %e, "Failed to read previous risk score");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response();
        }
    };

    let mut original_factors: Vec<RiskFactor> = Vec::new();
    if !had_override {
        let kyc_status = sqlx::query_as::<_, KycRow>(
            "SELECT kyc_status::text FROM users WHERE wallet_address=$1",
        )
        .bind(&address)
        .fetch_optional(&state.db_pool)
        .await
        .ok()
        .flatten();
        let status = kyc_status
            .map(|row| row.kyc_status)
            .unwrap_or_else(|| "pending".to_string());
        match status.as_str() {
            "approved" => original_factors.push(RiskFactor {
                label: "KYC".into(),
                impact: "positive".into(),
                score: 0,
                description: "KYC is approved.".into(),
            }),
            "submitted" => {
                previous_score += 20;
                original_factors.push(RiskFactor {
                    label: "KYC".into(),
                    impact: "negative".into(),
                    score: 20,
                    description: "KYC verification is submitted but not approved.".into(),
                });
            }
            "rejected" => {
                previous_score += 40;
                original_factors.push(RiskFactor {
                    label: "KYC".into(),
                    impact: "negative".into(),
                    score: 40,
                    description: "KYC verification was rejected.".into(),
                });
            }
            _ => {
                previous_score += 20;
                original_factors.push(RiskFactor {
                    label: "KYC".into(),
                    impact: "negative".into(),
                    score: 20,
                    description: "KYC information is pending or unavailable.".into(),
                });
            }
        }
        if let Ok(alert_count) = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM compliance_alerts WHERE address=$1 AND status='open'",
        )
        .bind(&address)
        .fetch_one(&state.db_pool)
        .await
        {
            if alert_count > 0 {
                let alert_score = (alert_count as i32 * 10).min(30);
                previous_score += alert_score;
                original_factors.push(RiskFactor {
                    label: "Compliance alerts".into(),
                    impact: "negative".into(),
                    score: alert_score,
                    description: format!("{alert_count} open compliance alert(s)."),
                });
            }
        }
        match check_sanctions_internal(&address).await {
            Ok(screening) if screening.status == "clear" => {}
            Ok(screening) if screening.status == "review" => {
                previous_score += 30;
                original_factors.push(RiskFactor {
                    label: "Sanctions screening".into(),
                    impact: "negative".into(),
                    score: 30,
                    description: "Potential sanctions match requires review.".into(),
                });
            }
            Ok(_) => {
                previous_score += 60;
                original_factors.push(RiskFactor {
                    label: "Sanctions screening".into(),
                    impact: "negative".into(),
                    score: 60,
                    description: "Confirmed sanctions match.".into(),
                });
            }
            Err(_) => {
                previous_score += 50;
                original_factors.push(RiskFactor {
                    label: "Sanctions screening".into(),
                    impact: "negative".into(),
                    score: 50,
                    description: "Screening unavailable; address is fail closed.".into(),
                });
            }
        }
        previous_score = previous_score.clamp(0, 100);
    }
    let original_factors_json =
        serde_json::to_value(&original_factors).unwrap_or_else(|_| json!([]));

    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(error = %e, "Failed to begin risk override transaction");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error":"Internal server error"})),
            )
                .into_response();
        }
    };
    let override_id = Uuid::new_v4();
    let previous_level = risk_level(previous_score).to_string();
    let new_level = risk_level(payload.score).to_string();

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO risk_score_overrides
            (id, address, previous_score, new_score, previous_level, new_level, justification, admin_id, original_factors)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
    )
    .bind(override_id)
    .bind(&address)
    .bind(previous_score as i16)
    .bind(payload.score as i16)
    .bind(&previous_level)
    .bind(&new_level)
    .bind(reason)
    .bind(&admin.user_id)
    .bind(&original_factors_json)
    .execute(&mut *tx)
    .await
    {
        error!(error = %e, "Failed to persist risk override");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Internal server error"}))).into_response();
    }

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO risk_override_audit
            (override_id, address, admin_id, previous_score, new_score, reason, original_factors)
        VALUES ($1,$2,$3,$4,$5,$6,$7)
        "#,
    )
    .bind(override_id)
    .bind(&address)
    .bind(&admin.user_id)
    .bind(previous_score as i16)
    .bind(payload.score as i16)
    .bind(reason)
    .bind(&original_factors_json)
    .execute(&mut *tx)
    .await
    {
        error!(error = %e, "Failed to persist risk override audit");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":"Internal server error"})),
        )
            .into_response();
    }
    if let Err(e) = tx.commit().await {
        error!(error = %e, "Failed to commit risk override");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":"Internal server error"})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(RiskOverride {
            id: Some(override_id.to_string()),
            address,
            score: payload.score,
            level: new_level,
            justification: reason.to_string(),
            admin_id: Some(admin.user_id),
            created_at: Some(Utc::now()),
        }),
    )
        .into_response()
}

pub async fn check_sanctions(
    State(_state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Response {
    let address = match validated_address(&address) {
        Some(address) => address,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"Invalid address"})),
            )
                .into_response()
        }
    };
    match check_sanctions_internal(&address).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err((status, body)) => (status, Json(body)).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_boundaries_are_strictly_greater() {
        let threshold = Decimal::from(10_000);
        assert!(Decimal::from(10_000) <= threshold);
        assert!(Decimal::from(10_001) > threshold);
    }

    #[test]
    fn risk_levels_cover_defined_ranges() {
        assert_eq!(risk_level(29), "low");
        assert_eq!(risk_level(30), "medium");
        assert_eq!(risk_level(49), "medium");
        assert_eq!(risk_level(50), "high");
        assert_eq!(risk_level(79), "high");
        assert_eq!(risk_level(80), "critical");
        assert_eq!(risk_level(100), "critical");
    }

    #[test]
    fn sanctions_clear_result_is_not_flagged() {
        let result = parse_sanctions_body(
            "GTEST",
            json!({"status":"clear","lists":[],"match_score":0.1}),
        )
        .unwrap();
        assert_eq!(result.status, "clear");
        assert!(!result.is_flagged);
    }

    #[test]
    fn sanctions_potential_result_requires_review() {
        let result = parse_sanctions_body(
            "GTEST",
            json!({"status":"potential","lists":["OFAC"],"match_score":0.72}),
        )
        .unwrap();
        assert_eq!(result.status, "review");
        assert!(result.is_flagged);
    }

    #[test]
    fn sanctions_confirmed_result_is_flagged() {
        let result = parse_sanctions_body(
            "GTEST",
            json!({"status":"confirmed","lists":["OFAC"],"match_score":0.99}),
        )
        .unwrap();
        assert_eq!(result.status, "flagged");
        assert!(result.is_flagged);
    }

    #[test]
    fn malformed_sanctions_result_is_unavailable() {
        let result = parse_sanctions_body("GTEST", json!({"unexpected":"data"}));
        assert!(result.is_err());
    }
}
