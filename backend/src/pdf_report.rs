//! PDF Inheritance Audit Report generator (Issue #825).
//!
//! Call [`build_pdf_bytes`] inside `tokio::task::spawn_blocking` – it is
//! entirely synchronous and must not be called directly on the async runtime.

use crate::api::{BeneficiaryRow, PlanRow};
use chrono::TimeZone as _;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::io::BufWriter;

/// Data bundle for one PDF report.
pub struct ReportData {
    pub plan: PlanRow,
    pub beneficiaries: Vec<BeneficiaryRow>,
    /// Live accrued yield (stored + elapsed since last ping).
    pub accrued_yield: f64,
}

fn fmt_epoch(epoch: i64) -> String {
    chrono::Utc
        .timestamp_opt(epoch, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| epoch.to_string())
}

/// Build and return raw PDF bytes for the given report data.
///
/// **Synchronous** – run inside `tokio::task::spawn_blocking`.
pub fn build_pdf_bytes(data: ReportData) -> Result<Vec<u8>, printpdf::Error> {
    let (doc, page1, layer1) =
        PdfDocument::new("Inheritance Audit Report", Mm(210.0), Mm(297.0), "Main");

    let layer = doc.get_page(page1).get_layer(layer1);
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;

    let lm = Mm(15.0_f32);
    let rc = Mm(110.0_f32);
    let lh = Mm(7.0_f32);
    let mut y = Mm(280.0_f32);

    // ── Title ─────────────────────────────────────────────────────────────
    layer.use_text(
        "InheritX - Inheritance Audit Report",
        18.0_f32,
        lm,
        y,
        &bold,
    );
    y -= lh * 2.0_f32;

    // ── Plan Overview ─────────────────────────────────────────────────────
    layer.use_text("Plan Overview", 13.0_f32, lm, y, &bold);
    y -= lh;

    let plan_id = data.plan.id.to_string();
    let amount_str = data.plan.amount.to_string();
    let yield_rate_str = data.plan.yield_rate_bps.to_string();
    let accrued_str = format!("{:.6}", data.accrued_yield);
    let grace_str = data.plan.grace_period_seconds.to_string();
    let created_str = data.plan.created_at.format("%Y-%m-%d %H:%M UTC").to_string();

    let overview: &[(&str, &str)] = &[
        ("Plan ID:", &plan_id),
        ("Status:", &data.plan.status),
        ("Token:", &data.plan.token_address),
        ("Principal:", &amount_str),
        ("Yield Enabled:", if data.plan.earn_yield { "Yes" } else { "No" }),
        ("Yield Rate (bps):", &yield_rate_str),
        ("Accrued Yield:", &accrued_str),
        ("Grace Period (s):", &grace_str),
        ("Active:", if data.plan.is_active { "Yes" } else { "No" }),
        ("Created At:", &created_str),
    ];

    for (label, value) in overview {
        layer.use_text(*label, 10.0_f32, lm, y, &regular);
        layer.use_text(*value, 10.0_f32, rc, y, &regular);
        y -= lh;
    }
    y -= lh;

    // ── Owner ─────────────────────────────────────────────────────────────
    layer.use_text("Plan Owner", 13.0_f32, lm, y, &bold);
    y -= lh;
    layer.use_text("Wallet Address:", 10.0_f32, lm, y, &regular);
    layer.use_text(
        data.plan.owner_address.as_str(),
        10.0_f32,
        rc,
        y,
        &regular,
    );
    y -= lh * 2.0_f32;

    // ── Activity Log ──────────────────────────────────────────────────────
    layer.use_text("Activity Log", 13.0_f32, lm, y, &bold);
    y -= lh;

    let last_ping_str = if data.plan.last_ping == 0 {
        "Never pinged".to_string()
    } else {
        fmt_epoch(data.plan.last_ping)
    };
    layer.use_text("Last Proof-of-Life:", 10.0_f32, lm, y, &regular);
    layer.use_text(last_ping_str.as_str(), 10.0_f32, rc, y, &regular);
    y -= lh;

    let deadline_str = if data.plan.last_ping > 0 {
        fmt_epoch(data.plan.last_ping + data.plan.grace_period_seconds)
    } else {
        "N/A".to_string()
    };
    layer.use_text("Inactivity Deadline:", 10.0_f32, lm, y, &regular);
    layer.use_text(deadline_str.as_str(), 10.0_f32, rc, y, &regular);
    y -= lh * 2.0_f32;

    // ── Beneficiaries ─────────────────────────────────────────────────────
    layer.use_text("Beneficiaries", 13.0_f32, lm, y, &bold);
    y -= lh;

    layer.use_text("Wallet Address", 9.0_f32, lm, y, &bold);
    layer.use_text("Alloc (bps)", 9.0_f32, Mm(110.0_f32), y, &bold);
    layer.use_text("Alloc (%)", 9.0_f32, Mm(145.0_f32), y, &bold);
    layer.use_text("Fiat Anchor", 9.0_f32, Mm(170.0_f32), y, &bold);
    y -= lh;

    for b in &data.beneficiaries {
        let addr = if b.wallet_address.len() > 28 {
            format!("{}...", &b.wallet_address[..28])
        } else {
            b.wallet_address.clone()
        };
        let anchor = if b.fiat_anchor_info.is_empty() {
            "-".to_string()
        } else if b.fiat_anchor_info.len() > 18 {
            format!("{}...", &b.fiat_anchor_info[..18])
        } else {
            b.fiat_anchor_info.clone()
        };
        let pct = format!("{:.2}%", b.allocation_bps as f64 / 100.0);
        let bps = b.allocation_bps.to_string();

        layer.use_text(addr.as_str(), 9.0_f32, lm, y, &regular);
        layer.use_text(bps.as_str(), 9.0_f32, Mm(110.0_f32), y, &regular);
        layer.use_text(pct.as_str(), 9.0_f32, Mm(145.0_f32), y, &regular);
        layer.use_text(anchor.as_str(), 9.0_f32, Mm(170.0_f32), y, &regular);
        y -= lh;
    }

    y -= lh;
    layer.use_text(
        "Generated automatically by InheritX.",
        7.0_f32,
        lm,
        y,
        &regular,
    );

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf)?;
    buf.into_inner().map_err(|e| {
        printpdf::Error::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn sample_data() -> ReportData {
        ReportData {
            plan: PlanRow {
                id: Uuid::new_v4(),
                owner_address: "GABC1234OWNER".to_string(),
                token_address: "USDC".to_string(),
                amount: Decimal::new(100_000, 2),
                grace_period: 30,
                grace_period_seconds: 2_592_000,
                earn_yield: true,
                last_ping: 1_700_000_000,
                is_active: true,
                status: "ACTIVE".to_string(),
                yield_rate_bps: 500,
                accrued_yield: Decimal::new(5_000, 3),
                created_at: chrono::Utc::now(),
            },
            beneficiaries: vec![
                BeneficiaryRow {
                    id: Uuid::new_v4(),
                    plan_id: Uuid::new_v4(),
                    wallet_address: "GBENEF1WALLET".to_string(),
                    allocation_bps: 6000,
                    fiat_anchor_info: "NGN/bank".to_string(),
                },
                BeneficiaryRow {
                    id: Uuid::new_v4(),
                    plan_id: Uuid::new_v4(),
                    wallet_address: "GBENEF2WALLET".to_string(),
                    allocation_bps: 4000,
                    fiat_anchor_info: String::new(),
                },
            ],
            accrued_yield: 5.0,
        }
    }

    #[test]
    fn test_build_pdf_returns_valid_bytes() {
        let bytes = build_pdf_bytes(sample_data()).expect("PDF generation failed");
        assert!(bytes.starts_with(b"%PDF"), "output is not a valid PDF");
        assert!(bytes.len() > 1024, "PDF suspiciously small");
    }

    #[test]
    fn test_build_pdf_no_beneficiaries() {
        let mut data = sample_data();
        data.beneficiaries.clear();
        let bytes = build_pdf_bytes(data).expect("PDF generation failed");
        assert!(bytes.starts_with(b"%PDF"));
    }
}
