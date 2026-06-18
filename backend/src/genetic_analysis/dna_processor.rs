use super::errors::GeneticError;
use super::types::*;
use ring::digest::{digest, SHA256};
use std::collections::HashMap;
use uuid::Uuid;

/// Parses raw DNA data (23andMe/Ancestry-style text or synthetic binary) into structured SNPs.
pub struct DNAProcessor;

impl DNAProcessor {
    pub fn process(
        &self,
        raw_data: &[u8],
        privacy_level: PrivacyLevel,
    ) -> Result<ProcessedDNAData, GeneticError> {
        if raw_data.is_empty() {
            return Err(GeneticError::InvalidInput(
                "DNA data cannot be empty".into(),
            ));
        }

        let text = std::str::from_utf8(raw_data).ok();
        let snp_data = if let Some(content) = text {
            if content.contains('\t') || content.contains("rs") {
                Self::parse_text_format(content)?
            } else {
                Self::parse_binary_format(raw_data)?
            }
        } else {
            Self::parse_binary_format(raw_data)?
        };

        if snp_data.is_empty() {
            return Err(GeneticError::ProcessingFailed(
                "No valid SNP variants found in raw data".into(),
            ));
        }

        let profile_id = Uuid::new_v4().to_string();
        let genetic_markers = Self::extract_genetic_markers(&snp_data);
        let health_markers = Self::extract_health_markers(&snp_data);
        let ancestry_composition = Self::estimate_ancestry(&snp_data);

        Ok(ProcessedDNAData {
            profile_id,
            genetic_markers,
            snp_data,
            ancestry_composition,
            health_markers,
            privacy_level,
        })
    }

    fn parse_text_format(content: &str) -> Result<Vec<SNPVariant>, GeneticError> {
        let mut variants = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
                continue;
            }

            let rsid = parts[0].trim().to_string();
            if !rsid.starts_with("rs") {
                continue;
            }

            let chromosome = Self::parse_chromosome(parts[1].trim())?;
            let position = parts[2].trim().parse::<u64>().map_err(|_| {
                GeneticError::InvalidInput(format!("Invalid position: {}", parts[2]))
            })?;
            let genotype = parts[3].trim().to_uppercase();

            if !Self::is_valid_genotype(&genotype) {
                continue;
            }

            variants.push(SNPVariant {
                rsid,
                chromosome,
                position,
                genotype,
                significance: VariantSignificance::Uncertain,
            });
        }

        Ok(variants)
    }

    fn parse_binary_format(raw_data: &[u8]) -> Result<Vec<SNPVariant>, GeneticError> {
        // Synthetic binary format: 15 bytes per SNP record
        // [8 bytes position][1 byte chromosome][1 byte allele1][1 byte allele2][4 bytes rsid suffix]
        if raw_data.len() < 15 {
            return Err(GeneticError::InvalidInput(
                "Binary DNA data too short (minimum 15 bytes per SNP)".into(),
            ));
        }

        let mut variants = Vec::new();
        let record_count = raw_data.len() / 15;

        for i in 0..record_count {
            let offset = i * 15;
            let position = u64::from_le_bytes(raw_data[offset..offset + 8].try_into().unwrap());
            let chromosome = raw_data[offset + 8];
            let allele1 = raw_data[offset + 9];
            let allele2 = raw_data[offset + 10];
            let rsid_suffix =
                u32::from_le_bytes(raw_data[offset + 11..offset + 15].try_into().unwrap());

            let genotype = format!(
                "{}{}",
                Self::allele_to_char(allele1),
                Self::allele_to_char(allele2)
            );

            variants.push(SNPVariant {
                rsid: format!("rs{rsid_suffix}"),
                chromosome,
                position,
                genotype,
                significance: VariantSignificance::Uncertain,
            });
        }

        Ok(variants)
    }

    fn allele_to_char(allele: u8) -> char {
        match allele {
            0 => 'A',
            1 => 'C',
            2 => 'G',
            3 => 'T',
            _ => 'N',
        }
    }

    fn parse_chromosome(raw: &str) -> Result<u8, GeneticError> {
        match raw.to_uppercase().as_str() {
            "X" => Ok(23),
            "Y" => Ok(24),
            "MT" | "M" => Ok(25),
            s => s
                .parse::<u8>()
                .map_err(|_| GeneticError::InvalidInput(format!("Invalid chromosome: {raw}"))),
        }
    }

    fn is_valid_genotype(genotype: &str) -> bool {
        genotype.len() == 2
            && genotype
                .chars()
                .all(|c| matches!(c, 'A' | 'C' | 'G' | 'T' | 'N' | '-' | '0'))
    }

    fn extract_genetic_markers(snp_data: &[SNPVariant]) -> HashMap<String, GeneticMarker> {
        let mut markers = HashMap::new();
        for snp in snp_data {
            markers.insert(
                snp.rsid.clone(),
                GeneticMarker {
                    marker_id: snp.rsid.clone(),
                    marker_type: "snp".into(),
                    value: snp.genotype.clone(),
                },
            );
        }
        markers
    }

    fn extract_health_markers(snp_data: &[SNPVariant]) -> Vec<HealthMarker> {
        let known_health_snps: HashMap<&str, (&str, &str)> = HashMap::from([
            ("rs429358", ("Alzheimer's Disease (APOE)", "C")),
            ("rs7412", ("Alzheimer's Disease (APOE)", "T")),
            ("rs1801133", ("MTHFR Deficiency", "T")),
            ("rs6025", ("Factor V Leiden", "A")),
            ("rs1799963", ("Prothrombin Thrombophilia", "G")),
            ("rs7903146", ("Type 2 Diabetes", "T")),
            ("rs1333049", ("Coronary Artery Disease", "C")),
        ]);

        snp_data
            .iter()
            .filter_map(|snp| {
                known_health_snps
                    .get(snp.rsid.as_str())
                    .map(|(condition, risk_allele)| {
                        let carrier_status = snp.genotype.contains(risk_allele);
                        HealthMarker {
                            rsid: snp.rsid.clone(),
                            condition: condition.to_string(),
                            risk_allele: risk_allele.to_string(),
                            carrier_status,
                        }
                    })
            })
            .collect()
    }

    fn estimate_ancestry(snp_data: &[SNPVariant]) -> AncestryBreakdown {
        // Simplified ancestry estimation based on SNP allele frequencies
        let total = snp_data.len().max(1) as f64;
        let g_count = snp_data.iter().filter(|s| s.genotype.contains('G')).count() as f64;
        let a_count = snp_data.iter().filter(|s| s.genotype.contains('A')).count() as f64;
        let t_count = snp_data.iter().filter(|s| s.genotype.contains('T')).count() as f64;

        let mut populations = HashMap::new();
        populations.insert("European".into(), (g_count / total).min(1.0));
        populations.insert("East Asian".into(), (a_count / total * 0.6).min(1.0));
        populations.insert("African".into(), (t_count / total * 0.4).min(1.0));
        populations.insert("Other".into(), 0.1);

        // Normalize to sum to ~1.0
        let sum: f64 = populations.values().sum();
        if sum > 0.0 {
            for v in populations.values_mut() {
                *v /= sum;
            }
        }

        AncestryBreakdown { populations }
    }

    pub fn compute_dna_hash(raw_data: &[u8], salt: &[u8]) -> String {
        let mut combined = salt.to_vec();
        combined.extend_from_slice(raw_data);
        let hash = digest(&SHA256, &combined);
        hex::encode(hash.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_format() {
        let raw = b"# comment\nrs429358\t19\t45411941\tCT\nrs7412\t19\t45412079\tCT\n";
        let processor = DNAProcessor;
        let result = processor.process(raw, PrivacyLevel::Protected).unwrap();
        assert_eq!(result.snp_data.len(), 2);
        assert_eq!(result.snp_data[0].rsid, "rs429358");
        assert_eq!(result.snp_data[0].genotype, "CT");
    }

    #[test]
    fn test_parse_binary_format() {
        let mut raw = Vec::new();
        // One SNP: position=1000, chr=1, alleles A/G, rsid_suffix=429358
        raw.extend_from_slice(&1000u64.to_le_bytes());
        raw.push(1); // chromosome
        raw.push(0); // A
        raw.push(2); // G
        raw.extend_from_slice(&429358u32.to_le_bytes());

        let processor = DNAProcessor;
        let result = processor.process(&raw, PrivacyLevel::Private).unwrap();
        assert_eq!(result.snp_data.len(), 1);
        assert_eq!(result.snp_data[0].genotype, "AG");
    }

    #[test]
    fn test_empty_data_rejected() {
        let processor = DNAProcessor;
        assert!(processor.process(&[], PrivacyLevel::Public).is_err());
    }

    #[test]
    fn test_dna_hash_deterministic() {
        let data = b"test dna";
        let salt = b"salt1234";
        let h1 = DNAProcessor::compute_dna_hash(data, salt);
        let h2 = DNAProcessor::compute_dna_hash(data, salt);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }
}
