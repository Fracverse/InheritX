use super::types::*;

/// Genetic similarity calculation using IBD/IBS algorithms.
pub struct GeneticSimilarityCalculator;

impl GeneticSimilarityCalculator {
    /// Calculate Identity By State (IBS) similarity between two profiles.
    /// Returns a score from 0.0 to 1.0 representing the fraction of matching alleles
    /// at shared SNP positions.
    pub fn calculate_identity_by_state(&self, profile1: &DNAProfile, profile2: &DNAProfile) -> f64 {
        let snp_map1: std::collections::HashMap<&str, &SNPVariant> = profile1
            .snp_data
            .iter()
            .map(|s| (s.rsid.as_str(), s))
            .collect();
        let snp_map2: std::collections::HashMap<&str, &SNPVariant> = profile2
            .snp_data
            .iter()
            .map(|s| (s.rsid.as_str(), s))
            .collect();

        let shared_rsids: Vec<&&str> = snp_map1
            .keys()
            .filter(|rsid| snp_map2.contains_key(**rsid))
            .collect();

        if shared_rsids.is_empty() {
            return 0.0;
        }

        let total_ibs: f64 = shared_rsids
            .iter()
            .map(|rsid| {
                let s1 = snp_map1[**rsid];
                let s2 = snp_map2[**rsid];
                Self::ibs_score(&s1.genotype, &s2.genotype)
            })
            .sum();

        total_ibs / shared_rsids.len() as f64
    }

    /// Estimate Identity By Descent (IBD) segments.
    /// Uses runs of consecutive matching SNPs on the same chromosome as a proxy for IBD.
    pub fn calculate_identity_by_descent(
        &self,
        profile1: &DNAProfile,
        profile2: &DNAProfile,
    ) -> f64 {
        let ibs = self.calculate_identity_by_state(profile1, profile2);
        let cm_sharing = self.calculate_centimorgan_sharing(profile1, profile2);

        // IBD estimate: combine IBS with segment length signal
        // Full siblings share ~50% IBD; parent-child ~50%; unrelated ~0%
        let segment_factor = (cm_sharing / 3500.0).min(1.0); // 3500 cM = full genome
        (ibs * 0.4 + segment_factor * 0.6).min(1.0)
    }

    /// Calculate shared centiMorgans based on matching segment lengths.
    /// Approximation: each matching SNP on a chromosome contributes ~0.01 cM.
    pub fn calculate_centimorgan_sharing(
        &self,
        profile1: &DNAProfile,
        profile2: &DNAProfile,
    ) -> f64 {
        let snp_map1: std::collections::HashMap<(&str, u8), &SNPVariant> = profile1
            .snp_data
            .iter()
            .map(|s| ((s.rsid.as_str(), s.chromosome), s))
            .collect();
        let snp_map2: std::collections::HashMap<(&str, u8), &SNPVariant> = profile2
            .snp_data
            .iter()
            .map(|s| ((s.rsid.as_str(), s.chromosome), s))
            .collect();

        let mut total_cm = 0.0_f64;
        let mut current_run_length = 0_u64;
        let mut last_chromosome = 0_u8;

        let mut shared: Vec<(&str, u8, u64)> = snp_map1
            .keys()
            .filter(|key| snp_map2.contains_key(key))
            .map(|(rsid, chr)| {
                let pos = snp_map1[&(*rsid, *chr)].position;
                (*rsid, *chr, pos)
            })
            .collect();
        shared.sort_by_key(|(_, chr, pos)| (*chr, *pos));

        for (rsid, chr, _) in &shared {
            let s1 = snp_map1[&(*rsid, *chr)];
            let s2 = snp_map2[&(*rsid, *chr)];

            if Self::ibs_score(&s1.genotype, &s2.genotype) >= 1.0 {
                if *chr == last_chromosome || last_chromosome == 0 {
                    current_run_length += 1;
                } else {
                    total_cm += Self::run_to_centimorgans(current_run_length);
                    current_run_length = 1;
                }
                last_chromosome = *chr;
            } else {
                if current_run_length > 0 {
                    total_cm += Self::run_to_centimorgans(current_run_length);
                }
                current_run_length = 0;
            }
        }

        if current_run_length > 0 {
            total_cm += Self::run_to_centimorgans(current_run_length);
        }

        total_cm
    }

    /// Convert a similarity score to a relationship estimate using centimorgan thresholds.
    pub fn estimate_relationship_coefficient(&self, similarity_score: f64) -> RelationshipEstimate {
        let shared_cm = similarity_score * 3500.0;

        let candidates: Vec<(RelationshipType, f64, f64)> = vec![
            (RelationshipType::Parent, 3480.0, 50.0),
            (RelationshipType::Child, 3480.0, 50.0),
            (RelationshipType::Sibling, 2600.0, 400.0),
            (RelationshipType::Grandparent, 1750.0, 300.0),
            (RelationshipType::Grandchild, 1750.0, 300.0),
            (RelationshipType::Spouse, 3400.0, 100.0),
            (RelationshipType::Other, 0.0, 500.0),
        ];

        let mut best_match = RelationshipType::Other;
        let mut best_confidence = 0.0_f64;
        let mut alternatives = Vec::new();

        for (rel_type, expected_cm, tolerance) in &candidates {
            let distance = (shared_cm - expected_cm).abs();
            let confidence = (1.0 - distance / tolerance).clamp(0.0, 1.0);
            alternatives.push((*rel_type, confidence));
            if confidence > best_confidence {
                best_confidence = confidence;
                best_match = *rel_type;
            }
        }

        alternatives.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        alternatives.retain(|(t, _)| *t != best_match);

        RelationshipEstimate {
            most_likely_relationship: best_match,
            confidence: best_confidence,
            alternative_relationships: alternatives.into_iter().take(3).collect(),
            shared_centimorgans: shared_cm,
        }
    }

    /// IBS score: 1.0 = identical, 0.5 = one shared allele, 0.0 = no match.
    fn ibs_score(g1: &str, g2: &str) -> f64 {
        if g1 == g2 {
            return 1.0;
        }

        let alleles1: std::collections::HashSet<char> = g1.chars().filter(|c| *c != '-').collect();
        let alleles2: std::collections::HashSet<char> = g2.chars().filter(|c| *c != '-').collect();

        if alleles1.is_empty() || alleles2.is_empty() {
            return 0.0;
        }

        let shared = alleles1.intersection(&alleles2).count();
        if shared >= 2 || (alleles1.len() == 1 && alleles2.len() == 1 && shared == 1) {
            1.0
        } else if shared >= 1 {
            0.5
        } else {
            0.0
        }
    }

    fn run_to_centimorgans(run_length: u64) -> f64 {
        // Approximate: 1 cM ≈ 1 million base pairs ≈ ~100 SNPs at typical density
        run_length as f64 * 0.01
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_profile(id: &str, snps: Vec<SNPVariant>) -> DNAProfile {
        DNAProfile {
            profile_id: id.into(),
            snp_data: snps,
            genetic_markers: HashMap::new(),
        }
    }

    fn snp(rsid: &str, chr: u8, pos: u64, genotype: &str) -> SNPVariant {
        SNPVariant {
            rsid: rsid.into(),
            chromosome: chr,
            position: pos,
            genotype: genotype.into(),
            significance: VariantSignificance::Uncertain,
        }
    }

    #[test]
    fn test_identical_profiles_high_ibs() {
        let snps = vec![
            snp("rs1", 1, 100, "AA"),
            snp("rs2", 1, 200, "AG"),
            snp("rs3", 2, 300, "GG"),
        ];
        let p1 = make_profile("a", snps.clone());
        let p2 = make_profile("b", snps);
        let calc = GeneticSimilarityCalculator;
        let ibs = calc.calculate_identity_by_state(&p1, &p2);
        assert!((ibs - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unrelated_profiles_low_ibs() {
        let p1 = make_profile(
            "a",
            vec![snp("rs1", 1, 100, "AA"), snp("rs2", 1, 200, "AA")],
        );
        let p2 = make_profile(
            "b",
            vec![snp("rs1", 1, 100, "GG"), snp("rs2", 1, 200, "GG")],
        );
        let calc = GeneticSimilarityCalculator;
        let ibs = calc.calculate_identity_by_state(&p1, &p2);
        assert!(ibs < 0.5);
    }

    #[test]
    fn test_sibling_relationship_estimate() {
        let calc = GeneticSimilarityCalculator;
        // ~2600 cM / 3500 ≈ 0.74
        let estimate = calc.estimate_relationship_coefficient(0.74);
        assert_eq!(estimate.most_likely_relationship, RelationshipType::Sibling);
        assert!(estimate.confidence > 0.5);
    }

    #[test]
    fn test_no_shared_snps_returns_zero() {
        let p1 = make_profile("a", vec![snp("rs1", 1, 100, "AA")]);
        let p2 = make_profile("b", vec![snp("rs2", 2, 200, "GG")]);
        let calc = GeneticSimilarityCalculator;
        assert_eq!(calc.calculate_identity_by_state(&p1, &p2), 0.0);
    }

    #[test]
    fn test_centimorgan_sharing_positive_for_matches() {
        let snps = vec![
            snp("rs1", 1, 100, "AA"),
            snp("rs2", 1, 200, "AA"),
            snp("rs3", 1, 300, "AA"),
        ];
        let p1 = make_profile("a", snps.clone());
        let p2 = make_profile("b", snps);
        let calc = GeneticSimilarityCalculator;
        let cm = calc.calculate_centimorgan_sharing(&p1, &p2);
        assert!(cm > 0.0);
    }
}
