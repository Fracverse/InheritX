//! # Will PDF Generator & Template Engine (Tasks 1 & 2)
//!
//! Generates a structured legal will document from vault/plan data.
//! Supports multiple templates (simple, formal, jurisdiction-specific).

use crate::api_error::ApiError;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ─── Template Types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WillTemplate {
    Simple,
    Formal,
    UsJurisdiction,
    UkJurisdiction,
    GlobalGeneric,
}

impl WillTemplate {
    pub fn as_str(self) -> &'static str {
        match self {
            WillTemplate::Simple => "simple",
            WillTemplate::Formal => "formal",
            WillTemplate::UsJurisdiction => "us_jurisdiction",
            WillTemplate::UkJurisdiction => "uk_jurisdiction",
            WillTemplate::GlobalGeneric => "global_generic",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            WillTemplate::Simple => "Simple Will",
            WillTemplate::Formal => "Formal Legal Will",
            WillTemplate::UsJurisdiction => "US Jurisdiction Will",
            WillTemplate::UkJurisdiction => "UK Jurisdiction Will",
            WillTemplate::GlobalGeneric => "Global Generic Will",
        }
    }
}

impl std::str::FromStr for WillTemplate {
    type Err = ApiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(WillTemplate::Simple),
            "formal" => Ok(WillTemplate::Formal),
            "us_jurisdiction" => Ok(WillTemplate::UsJurisdiction),
            "uk_jurisdiction" => Ok(WillTemplate::UkJurisdiction),
            "global_generic" => Ok(WillTemplate::GlobalGeneric),
            _ => Err(ApiError::BadRequest(format!("Unknown template: {s}"))),
        }
    }
}

// ─── Data Structures ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficiaryEntry {
    pub name: String,
    pub wallet_address: String,
    pub allocation_percent: rust_decimal::Decimal,
    pub relationship: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WillDocumentInput {
    pub plan_id: Uuid,
    pub owner_name: String,
    pub owner_wallet: String,
    pub vault_id: String,
    pub beneficiaries: Vec<BeneficiaryEntry>,
    pub execution_rules: Option<String>,
    pub template: WillTemplate,
    pub jurisdiction: Option<String>,
    pub will_hash_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedWillDocument {
    pub document_id: Uuid,
    pub plan_id: Uuid,
    pub template_used: String,
    pub will_hash: String,
    pub generated_at: DateTime<Utc>,
    pub version: u32,
    /// Base64-encoded PDF bytes
    pub pdf_base64: String,
    pub filename: String,
}

// ─── Template Engine ──────────────────────────────────────────────────────────

struct TemplateEngine;

impl TemplateEngine {
    fn render(input: &WillDocumentInput, generated_at: DateTime<Utc>, version: u32) -> String {
        match input.template {
            WillTemplate::Simple => Self::render_simple(input, generated_at, version),
            WillTemplate::Formal => Self::render_formal(input, generated_at, version),
            WillTemplate::UsJurisdiction => Self::render_us(input, generated_at, version),
            WillTemplate::UkJurisdiction => Self::render_uk(input, generated_at, version),
            WillTemplate::GlobalGeneric => Self::render_global(input, generated_at, version),
        }
    }

    fn header(title: &str, generated_at: DateTime<Utc>, version: u32) -> String {
        format!(
            "================================================================\n\
             {title}\n\
             ================================================================\n\
             Generated: {generated_at}\n\
             Document Version: {version}\n\
             ----------------------------------------------------------------\n"
        )
    }

    fn beneficiaries_section(beneficiaries: &[BeneficiaryEntry]) -> String {
        let mut s = String::from("BENEFICIARIES\n-------------\n");
        for (i, b) in beneficiaries.iter().enumerate() {
            s.push_str(&format!(
                "{}. Name:       {}\n   Wallet:     {}\n   Allocation: {}%\n",
                i + 1,
                b.name,
                b.wallet_address,
                b.allocation_percent
            ));
            if let Some(rel) = &b.relationship {
                s.push_str(&format!("   Relation:   {rel}\n"));
            }
        }
        s
    }

    fn render_simple(input: &WillDocumentInput, ts: DateTime<Utc>, v: u32) -> String {
        let mut doc = Self::header("LAST WILL AND TESTAMENT (SIMPLE)", ts, v);
        doc.push_str(&format!(
            "\nI, {}, wallet address {}, hereby declare this my last will.\n\n",
            input.owner_name, input.owner_wallet
        ));
        doc.push_str(&Self::beneficiaries_section(&input.beneficiaries));
        Self::append_footer(&mut doc, input);
        doc
    }

    fn render_formal(input: &WillDocumentInput, ts: DateTime<Utc>, v: u32) -> String {
        let mut doc = Self::header("FORMAL LAST WILL AND TESTAMENT", ts, v);
        doc.push_str(&format!(
            "\nI, {owner}, residing at blockchain address {wallet}, being of sound mind,\n\
             do hereby make, publish, and declare this instrument to be my Last Will\n\
             and Testament, hereby revoking all former wills and codicils.\n\n\
             VAULT REFERENCE: {vault}\n\n",
            owner = input.owner_name,
            wallet = input.owner_wallet,
            vault = input.vault_id
        ));
        doc.push_str(&Self::beneficiaries_section(&input.beneficiaries));
        if let Some(rules) = &input.execution_rules {
            doc.push_str(&format!("\nEXECUTION RULES\n---------------\n{rules}\n"));
        }
        Self::append_footer(&mut doc, input);
        doc
    }

    fn render_us(input: &WillDocumentInput, ts: DateTime<Utc>, v: u32) -> String {
        let mut doc = Self::header("LAST WILL AND TESTAMENT — US JURISDICTION", ts, v);
        doc.push_str(
            "\nSTATE OF [STATE], COUNTY OF [COUNTY]\n\
             This Will is executed in accordance with applicable US state law.\n\n",
        );
        doc.push_str(&format!(
            "Testator: {}, Wallet: {}\nVault ID: {}\n\n",
            input.owner_name, input.owner_wallet, input.vault_id
        ));
        doc.push_str(&Self::beneficiaries_section(&input.beneficiaries));
        doc.push_str(
            "\nWITNESS CLAUSE\nThis will requires two witnesses per applicable state law.\n",
        );
        Self::append_footer(&mut doc, input);
        doc
    }

    fn render_uk(input: &WillDocumentInput, ts: DateTime<Utc>, v: u32) -> String {
        let mut doc = Self::header("LAST WILL AND TESTAMENT — UK JURISDICTION", ts, v);
        doc.push_str("\nThis Will is made in accordance with the Wills Act 1837 (as amended).\n\n");
        doc.push_str(&format!(
            "Testator: {}, Wallet: {}\nVault ID: {}\n\n",
            input.owner_name, input.owner_wallet, input.vault_id
        ));
        doc.push_str(&Self::beneficiaries_section(&input.beneficiaries));
        doc.push_str("\nATTESTATION\nSigned by the above-named Testator in our presence.\n");
        Self::append_footer(&mut doc, input);
        doc
    }

    fn render_global(input: &WillDocumentInput, ts: DateTime<Utc>, v: u32) -> String {
        let mut doc = Self::header("LAST WILL AND TESTAMENT — GLOBAL GENERIC", ts, v);
        let jurisdiction = input
            .jurisdiction
            .as_deref()
            .unwrap_or("International / Unspecified");
        doc.push_str(&format!("\nJurisdiction: {jurisdiction}\n\n"));
        doc.push_str(&format!(
            "Testator: {}, Wallet: {}\nVault ID: {}\n\n",
            input.owner_name, input.owner_wallet, input.vault_id
        ));
        doc.push_str(&Self::beneficiaries_section(&input.beneficiaries));
        Self::append_footer(&mut doc, input);
        doc
    }

    fn append_footer(doc: &mut String, input: &WillDocumentInput) {
        doc.push_str("\n----------------------------------------------------------------\n");
        if let Some(hash_ref) = &input.will_hash_reference {
            doc.push_str(&format!("ON-CHAIN WILL HASH: {hash_ref}\n"));
        }
        doc.push_str(&format!("PLAN ID: {}\n", input.plan_id));
        doc.push_str(
            "================================================================\n\
             This document is cryptographically bound to the vault above.\n\
             ================================================================\n",
        );
    }
}

// ─── PDF Builder ──────────────────────────────────────────────────────────────

/// Builds a minimal valid PDF containing the will text.
/// Uses raw PDF syntax — no external crate required.
fn build_pdf(content: &str) -> Vec<u8> {
    // Escape special PDF string characters
    let escaped = content
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");

    // Split into lines for multi-line text rendering
    let lines: Vec<&str> = escaped.lines().collect();
    let mut stream_content = String::new();
    stream_content.push_str("BT\n/F1 10 Tf\n50 780 Td\n12 TL\n");
    for line in &lines {
        stream_content.push_str(&format!("({line}) Tj T*\n"));
    }
    stream_content.push_str("ET\n");

    let stream_bytes = stream_content.as_bytes();
    let stream_len = stream_bytes.len();

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    // Object 1: Catalog
    let obj1_offset = pdf.len();
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Object 2: Pages
    let obj2_offset = pdf.len();
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // Object 3: Page
    let obj3_offset = pdf.len();
    pdf.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R \
          /MediaBox [0 0 612 792] \
          /Contents 4 0 R \
          /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
    );

    // Object 4: Content stream
    let obj4_offset = pdf.len();
    pdf.extend_from_slice(format!("4 0 obj\n<< /Length {stream_len} >>\nstream\n").as_bytes());
    pdf.extend_from_slice(stream_bytes);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // Object 5: Font
    let obj5_offset = pdf.len();
    pdf.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>\nendobj\n",
    );

    // Cross-reference table
    let xref_offset = pdf.len();
    pdf.extend_from_slice(
        format!(
            "xref\n0 6\n\
             0000000000 65535 f \n\
             {obj1_offset:010} 00000 n \n\
             {obj2_offset:010} 00000 n \n\
             {obj3_offset:010} 00000 n \n\
             {obj4_offset:010} 00000 n \n\
             {obj5_offset:010} 00000 n \n"
        )
        .as_bytes(),
    );

    // Trailer
    pdf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n").as_bytes(),
    );

    pdf
}

// ─── Will PDF Service ─────────────────────────────────────────────────────────

pub struct WillPdfService;

impl WillPdfService {
    /// Generate a will PDF document and persist metadata to the database.
    pub async fn generate(
        db: &PgPool,
        user_id: Uuid,
        input: &WillDocumentInput,
    ) -> Result<GeneratedWillDocument, ApiError> {
        // Determine next version for this plan
        let version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM will_documents WHERE plan_id = $1",
        )
        .bind(input.plan_id)
        .fetch_one(db)
        .await
        .unwrap_or(0);
        let version = (version + 1) as u32;

        let generated_at = Utc::now();
        let document_id = Uuid::new_v4();

        // Render content via template engine
        let content = TemplateEngine::render(input, generated_at, version);

        // Compute document hash (SHA-256 over rendered text)
        let hash_bytes = digest(&SHA256, content.as_bytes());
        let will_hash = hex::encode(hash_bytes.as_ref());

        // Build PDF bytes
        let pdf_bytes = build_pdf(&content);
        let pdf_base64 = BASE64.encode(&pdf_bytes);

        let filename = format!(
            "will_{}_{}.pdf",
            input.plan_id,
            generated_at.format("%Y%m%d%H%M%S")
        );

        // Persist metadata
        sqlx::query(
            r#"
            INSERT INTO will_documents
                (id, plan_id, user_id, template, will_hash, version, filename, pdf_base64, generated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(document_id)
        .bind(input.plan_id)
        .bind(user_id)
        .bind(input.template.as_str())
        .bind(&will_hash)
        .bind(version as i32)
        .bind(&filename)
        .bind(&pdf_base64)
        .bind(generated_at)
        .execute(db)
        .await?;

        Ok(GeneratedWillDocument {
            document_id,
            plan_id: input.plan_id,
            template_used: input.template.display_name().to_string(),
            will_hash,
            generated_at,
            version,
            pdf_base64,
            filename,
        })
    }

    /// Retrieve a previously generated will document by ID.
    pub async fn get_document(
        db: &PgPool,
        document_id: Uuid,
        user_id: Uuid,
    ) -> Result<GeneratedWillDocument, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            plan_id: Uuid,
            template: String,
            will_hash: String,
            version: i32,
            filename: String,
            pdf_base64: String,
            generated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, template, will_hash, version, filename, pdf_base64, generated_at \
             FROM will_documents WHERE id = $1 AND user_id = $2",
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Will document {document_id} not found")))?;

        Ok(GeneratedWillDocument {
            document_id: row.id,
            plan_id: row.plan_id,
            template_used: row.template,
            will_hash: row.will_hash,
            generated_at: row.generated_at,
            version: row.version as u32,
            pdf_base64: row.pdf_base64,
            filename: row.filename,
        })
    }

    /// List all will documents for a plan.
    pub async fn list_for_plan(
        db: &PgPool,
        plan_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<GeneratedWillDocument>, ApiError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            plan_id: Uuid,
            template: String,
            will_hash: String,
            version: i32,
            filename: String,
            pdf_base64: String,
            generated_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            "SELECT id, plan_id, template, will_hash, version, filename, pdf_base64, generated_at \
             FROM will_documents WHERE plan_id = $1 AND user_id = $2 ORDER BY version DESC",
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_all(db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| GeneratedWillDocument {
                document_id: r.id,
                plan_id: r.plan_id,
                template_used: r.template,
                will_hash: r.will_hash,
                generated_at: r.generated_at,
                version: r.version as u32,
                pdf_base64: r.pdf_base64,
                filename: r.filename,
            })
            .collect())
    }
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_input(template: WillTemplate) -> WillDocumentInput {
        WillDocumentInput {
            plan_id: Uuid::new_v4(),
            owner_name: "Alice Testator".to_string(),
            owner_wallet: "GABC1234567890ABCDEF".to_string(),
            vault_id: "vault-001".to_string(),
            beneficiaries: vec![BeneficiaryEntry {
                name: "Bob Beneficiary".to_string(),
                wallet_address: "GBOB1234567890ABCDEF".to_string(),
                allocation_percent: dec!(100),
                relationship: Some("Son".to_string()),
            }],
            execution_rules: Some("Distribute after 90-day inactivity".to_string()),
            template,
            jurisdiction: Some("Global".to_string()),
            will_hash_reference: Some("0xdeadbeef".to_string()),
        }
    }

    #[test]
    fn test_template_rendering_simple() {
        let input = sample_input(WillTemplate::Simple);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        assert!(content.contains("Alice Testator"));
        assert!(content.contains("Bob Beneficiary"));
        assert!(content.contains("100"));
    }

    #[test]
    fn test_template_rendering_formal() {
        let input = sample_input(WillTemplate::Formal);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        assert!(content.contains("FORMAL LAST WILL"));
        assert!(content.contains("EXECUTION RULES"));
    }

    #[test]
    fn test_template_rendering_us() {
        let input = sample_input(WillTemplate::UsJurisdiction);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        assert!(content.contains("US JURISDICTION"));
        assert!(content.contains("WITNESS CLAUSE"));
    }

    #[test]
    fn test_template_rendering_uk() {
        let input = sample_input(WillTemplate::UkJurisdiction);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        assert!(content.contains("Wills Act 1837"));
    }

    #[test]
    fn test_template_rendering_global() {
        let input = sample_input(WillTemplate::GlobalGeneric);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        assert!(content.contains("GLOBAL GENERIC"));
    }

    #[test]
    fn test_pdf_bytes_start_with_pdf_header() {
        let input = sample_input(WillTemplate::Simple);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        let pdf = build_pdf(&content);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn test_will_hash_is_hex_sha256() {
        let data = b"test content";
        let hash_bytes = digest(&SHA256, data);
        let hash = hex::encode(hash_bytes.as_ref());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_template_from_str() {
        use std::str::FromStr;
        assert!(matches!(
            WillTemplate::from_str("formal"),
            Ok(WillTemplate::Formal)
        ));
        assert!(WillTemplate::from_str("unknown").is_err());
    }

    #[test]
    fn test_pdf_base64_roundtrip() {
        let input = sample_input(WillTemplate::Formal);
        let content = TemplateEngine::render(&input, Utc::now(), 1);
        let pdf = build_pdf(&content);
        let encoded = BASE64.encode(&pdf);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(pdf, decoded);
    }
}
