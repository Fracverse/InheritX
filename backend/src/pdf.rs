use crate::api_error::ApiError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

struct PlanReportData {
    title: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    asset_code: Option<String>,
    net_amount: Option<Decimal>,
    valuation_usd: Option<Decimal>,
    collateral_ratio: Option<Decimal>,
}

struct BeneficiaryReportEntry {
    name: Option<String>,
    wallet_address: String,
    allocation_percent: Decimal,
    relationship: Option<String>,
    beneficiary_type: String,
}

#[derive(Default)]
struct LendingSummary {
    total_deposits: Decimal,
    total_borrows: Decimal,
    total_repayments: Decimal,
    total_interest_accrued: Decimal,
}

pub struct PlanReportService;

impl PlanReportService {
    pub async fn generate_plan_report(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<u8>, ApiError> {
        crate::service::PlanService::assert_plan_owner(db, plan_id, user_id).await?;

        let plan = fetch_plan_details(db, plan_id).await?;
        let beneficiaries = fetch_beneficiaries(db, plan_id).await?;
        let lending = fetch_lending_summary(db, plan_id).await?;

        let report_text = build_report_text(plan_id, &plan, &beneficiaries, &lending);
        Ok(crate::will_pdf::build_pdf(&report_text))
    }
}

#[derive(sqlx::FromRow)]
struct PlanRow {
    title: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    asset_code: Option<String>,
    net_amount: Option<String>,
    valuation_usd: Option<String>,
    collateral_ratio: Option<Decimal>,
}

async fn fetch_plan_details(db: &PgPool, plan_id: Uuid) -> Result<PlanReportData, ApiError> {
    let row = sqlx::query_as::<_, PlanRow>(
        r#"
        SELECT title, status, created_at, updated_at,
               asset_code,
               net_amount::text,
               valuation_usd::text,
               collateral_ratio
        FROM plans
        WHERE id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("Plan {plan_id} not found")))?;

    Ok(PlanReportData {
        title: row.title,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        asset_code: row.asset_code,
        net_amount: row.net_amount.and_then(|s| s.parse().ok()),
        valuation_usd: row.valuation_usd.and_then(|s| s.parse().ok()),
        collateral_ratio: row.collateral_ratio,
    })
}

#[derive(sqlx::FromRow)]
struct BeneficiaryRow {
    name: Option<String>,
    wallet_address: String,
    allocation_percent: Decimal,
    relationship: Option<String>,
    beneficiary_type: Option<String>,
}

async fn fetch_beneficiaries(
    db: &PgPool,
    plan_id: Uuid,
) -> Result<Vec<BeneficiaryReportEntry>, ApiError> {
    let rows = sqlx::query_as::<_, BeneficiaryRow>(
        r#"
        SELECT name, wallet_address, allocation_percent, relationship,
               COALESCE(beneficiary_type::text, 'primary') as beneficiary_type
        FROM plan_beneficiaries
        WHERE plan_id = $1
        ORDER BY beneficiary_type, priority_order, wallet_address
        "#,
    )
    .bind(plan_id)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| BeneficiaryReportEntry {
            name: r.name,
            wallet_address: r.wallet_address,
            allocation_percent: r.allocation_percent,
            relationship: r.relationship,
            beneficiary_type: r.beneficiary_type.unwrap_or_else(|| "primary".to_string()),
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct LendingSummaryRow {
    total_deposits: Option<Decimal>,
    total_borrows: Option<Decimal>,
    total_repayments: Option<Decimal>,
    total_interest_accrued: Option<Decimal>,
}

async fn fetch_lending_summary(db: &PgPool, plan_id: Uuid) -> Result<LendingSummary, ApiError> {
    let row = sqlx::query_as::<_, LendingSummaryRow>(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN event_type = 'deposit' THEN CAST(amount AS numeric) ELSE 0 END), 0) as total_deposits,
            COALESCE(SUM(CASE WHEN event_type = 'borrow' THEN CAST(amount AS numeric) ELSE 0 END), 0) as total_borrows,
            COALESCE(SUM(CASE WHEN event_type = 'repay' THEN CAST(amount AS numeric) ELSE 0 END), 0) as total_repayments,
            COALESCE(SUM(CASE WHEN event_type = 'interest_accrual' THEN CAST(amount AS numeric) ELSE 0 END), 0) as total_interest_accrued
        FROM lending_events
        WHERE plan_id = $1
        "#,
    )
    .bind(plan_id)
    .fetch_one(db)
    .await?;

    Ok(LendingSummary {
        total_deposits: row.total_deposits.unwrap_or_default(),
        total_borrows: row.total_borrows.unwrap_or_default(),
        total_repayments: row.total_repayments.unwrap_or_default(),
        total_interest_accrued: row.total_interest_accrued.unwrap_or_default(),
    })
}

fn build_report_text(
    plan_id: Uuid,
    plan: &PlanReportData,
    beneficiaries: &[BeneficiaryReportEntry],
    lending: &LendingSummary,
) -> String {
    let now = Utc::now();
    let mut report = String::new();

    report.push_str(&format!(
        "================================================================\n\
         PLAN REPORT\n\
         ================================================================\n\
         Generated: {now}\n\
         ----------------------------------------------------------------\n\n\
         PLAN INFORMATION\n\
         ----------------\n\
         Title:      {title}\n\
         Status:     {status}\n\
         Created:    {created}\n\
         Updated:    {updated}\n",
        title = plan.title,
        status = plan.status,
        created = plan.created_at,
        updated = plan.updated_at,
    ));

    report.push_str("\nASSETS\n------\n");
    if let Some(asset_code) = &plan.asset_code {
        report.push_str(&format!("Asset Code: {asset_code}\n"));
    }
    if let Some(net_amount) = plan.net_amount {
        report.push_str(&format!("Balance:    {net_amount}\n"));
    }
    if let Some(valuation_usd) = plan.valuation_usd {
        report.push_str(&format!("Valuation:  ${valuation_usd}\n"));
    }
    if let Some(cr) = plan.collateral_ratio {
        report.push_str(&format!("Collateral Ratio: {cr}\n"));
    }
    if plan.asset_code.is_none() && plan.net_amount.is_none() && plan.valuation_usd.is_none() {
        report.push_str("No asset data available.\n");
    }

    report.push_str(&format!(
        "\nYIELD / LENDING SUMMARY\n\
         -----------------------\n\
         Total Deposits:         {deposits}\n\
         Total Borrows:          {borrows}\n\
         Total Repayments:       {repayments}\n\
         Total Interest Accrued: {interest}\n",
        deposits = lending.total_deposits,
        borrows = lending.total_borrows,
        repayments = lending.total_repayments,
        interest = lending.total_interest_accrued,
    ));

    report.push_str("\nBENEFICIARIES\n-------------\n");
    if beneficiaries.is_empty() {
        report.push_str("No beneficiaries registered for this plan.\n");
    } else {
        for (i, b) in beneficiaries.iter().enumerate() {
            let name = b.name.as_deref().unwrap_or("N/A");
            let rel = b.relationship.as_deref().unwrap_or("N/A");
            report.push_str(&format!(
                "{i}. Name:       {name}\n\
                 Wallet:     {wallet}\n\
                 Allocation: {alloc}%\n\
                 Relation:   {rel}\n\
                 Type:       {btype}\n",
                i = i + 1,
                wallet = b.wallet_address,
                alloc = b.allocation_percent,
                btype = b.beneficiary_type,
            ));
        }
    }

    report.push_str(&format!(
        "\n----------------------------------------------------------------\n\
         PLAN ID: {plan_id}\n\
         ================================================================\n\
         End of Report\n\
         ================================================================\n"
    ));

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_plan_data() -> PlanReportData {
        PlanReportData {
            title: "Retirement Fund".to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            asset_code: Some("USDC".to_string()),
            net_amount: Some(dec!(50000.00)),
            valuation_usd: Some(dec!(50000.00)),
            collateral_ratio: Some(dec!(1.50)),
        }
    }

    fn sample_beneficiaries() -> Vec<BeneficiaryReportEntry> {
        vec![
            BeneficiaryReportEntry {
                name: Some("Alice Beneficiary".to_string()),
                wallet_address: "GABCDEF1234567890".to_string(),
                allocation_percent: dec!(60.00),
                relationship: Some("Daughter".to_string()),
                beneficiary_type: "primary".to_string(),
            },
            BeneficiaryReportEntry {
                name: Some("Bob Beneficiary".to_string()),
                wallet_address: "GBOB1234567890ABCD".to_string(),
                allocation_percent: dec!(40.00),
                relationship: Some("Son".to_string()),
                beneficiary_type: "primary".to_string(),
            },
        ]
    }

    fn sample_lending_summary() -> LendingSummary {
        LendingSummary {
            total_deposits: dec!(100000.00),
            total_borrows: dec!(25000.00),
            total_repayments: dec!(10000.00),
            total_interest_accrued: dec!(1500.50),
        }
    }

    #[test]
    fn test_build_report_text_contains_all_sections() {
        let plan_id = Uuid::new_v4();
        let text = build_report_text(
            plan_id,
            &sample_plan_data(),
            &sample_beneficiaries(),
            &sample_lending_summary(),
        );

        assert!(text.contains("PLAN REPORT"));
        assert!(text.contains("Retirement Fund"));
        assert!(text.contains("active"));
        assert!(text.contains("USDC"));
        assert!(text.contains("50000"));
        assert!(text.contains("Alice Beneficiary"));
        assert!(text.contains("Bob Beneficiary"));
        assert!(text.contains("100000"));
        assert!(text.contains("1500.5"));
        assert!(text.contains("Total Deposits"));
        assert!(text.contains("Total Interest Accrued"));
        assert!(text.contains(&plan_id.to_string()));
        assert!(text.contains("End of Report"));
    }

    #[test]
    fn test_build_report_text_no_beneficiaries() {
        let plan_id = Uuid::new_v4();
        let text = build_report_text(plan_id, &sample_plan_data(), &[], &sample_lending_summary());
        assert!(text.contains("No beneficiaries registered for this plan."));
    }

    #[test]
    fn test_build_report_text_no_asset_data() {
        let plan_id = Uuid::new_v4();
        let plan = PlanReportData {
            asset_code: None,
            net_amount: None,
            valuation_usd: None,
            collateral_ratio: None,
            ..sample_plan_data()
        };
        let text = build_report_text(plan_id, &plan, &sample_beneficiaries(), &sample_lending_summary());
        assert!(text.contains("No asset data available"));
    }

    #[test]
    fn test_generated_pdf_is_valid() {
        let plan_id = Uuid::new_v4();
        let text = build_report_text(
            plan_id,
            &sample_plan_data(),
            &sample_beneficiaries(),
            &sample_lending_summary(),
        );
        let pdf = crate::will_pdf::build_pdf(&text);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }
}
