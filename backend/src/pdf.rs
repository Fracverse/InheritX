use printpdf::{BuiltinFont, Mm, PdfDocument};

pub struct PlanReportData {
    pub plan_id: String,
    pub owner_address: String,
    pub token_address: String,
    pub amount: String,
    pub status: String,
    pub earn_yield: bool,
    pub yield_rate_bps: i32,
    pub accrued_yield: String,
    pub created_at: String,
    pub grace_period_seconds: i64,
    pub beneficiaries: Vec<BeneficiaryData>,
    pub ping_logs: Vec<PingLogData>,
}

pub struct BeneficiaryData {
    pub wallet_address: String,
    pub allocation_bps: i32,
    pub fiat_anchor_info: String,
}

pub struct PingLogData {
    pub pinged_at: String,
    pub accrued_yield_snapshot: String,
}

/// Generate a PDF audit report as raw bytes. This is synchronous / CPU-bound;
/// callers must run it via `tokio::task::spawn_blocking`.
pub fn generate(data: PlanReportData) -> Result<Vec<u8>, String> {
    let (doc, page, layer) =
        PdfDocument::new("Inheritance Audit Report", Mm(210.0), Mm(297.0), "Content");

    let layer = doc.get_page(page).get_layer(layer);
    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;
    let regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;

    let mut y = 277.0_f32;
    let left = 15.0_f32;
    let line_h = 7.0_f32;
    let section_gap = 4.0_f32;

    macro_rules! line {
        ($font:expr, $size:expr, $text:expr) => {
            layer.use_text($text, $size, Mm(left), Mm(y), &$font);
            y -= line_h;
        };
    }

    // Title
    line!(bold, 16.0, "Inheritance Plan Audit Report");
    y -= section_gap;

    // Plan Details
    line!(bold, 11.0, "Plan Details");
    line!(regular, 9.0, format!("Plan ID:          {}", data.plan_id));
    line!(regular, 9.0, format!("Owner:            {}", data.owner_address));
    line!(regular, 9.0, format!("Token:            {}", data.token_address));
    line!(regular, 9.0, format!("Amount:           {}", data.amount));
    line!(regular, 9.0, format!("Status:           {}", data.status));
    line!(regular, 9.0, format!("Earn Yield:       {}", data.earn_yield));
    line!(regular, 9.0, format!("Yield Rate (bps): {}", data.yield_rate_bps));
    line!(regular, 9.0, format!("Accrued Yield:    {}", data.accrued_yield));
    line!(regular, 9.0, format!("Grace Period (s): {}", data.grace_period_seconds));
    line!(regular, 9.0, format!("Created At:       {}", data.created_at));
    y -= section_gap;

    // Beneficiaries
    line!(bold, 11.0, "Beneficiaries");
    if data.beneficiaries.is_empty() {
        line!(regular, 9.0, "  None");
    } else {
        for (i, b) in data.beneficiaries.iter().enumerate() {
            line!(regular, 9.0, format!("  [{}] Wallet:     {}", i + 1, b.wallet_address));
            line!(regular, 9.0, format!("      Allocation: {} bps ({:.2}%)", b.allocation_bps, b.allocation_bps as f32 / 100.0));
            line!(regular, 9.0, format!("      Fiat Info:  {}", b.fiat_anchor_info));
        }
    }
    y -= section_gap;

    // Ping / Activity Logs
    line!(bold, 11.0, "Activity Log (Pings)");
    if data.ping_logs.is_empty() {
        line!(regular, 9.0, "  No ping activity recorded.");
    } else {
        for (i, p) in data.ping_logs.iter().enumerate() {
            // Start a new page if near bottom
            if y < 20.0 {
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Content");
                let nl = doc.get_page(new_page).get_layer(new_layer);
                // shadow layer for remaining content - can't rebind macro, write directly
                nl.use_text(
                    format!("  [{}] {} | Yield snapshot: {}", i + 1, p.pinged_at, p.accrued_yield_snapshot),
                    9.0,
                    Mm(left),
                    Mm(277.0),
                    &regular,
                );
                y = 270.0;
                continue;
            }
            line!(regular, 9.0, format!("  [{}] {} | Yield snapshot: {}", i + 1, p.pinged_at, p.accrued_yield_snapshot));
        }
    }

    doc.save_to_bytes().map_err(|e| e.to_string())
}
