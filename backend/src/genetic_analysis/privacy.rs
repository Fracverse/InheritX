use super::types::*;
use ring::digest::{digest, SHA256};
use std::collections::HashMap;

/// Privacy-preserving genetic analysis engine.
pub struct GeneticPrivacyEngine;

impl GeneticPrivacyEngine {
    /// Create a privacy-protected genetic profile based on the requested privacy level.
    pub fn create_privacy_preserving_profile(
        &self,
        dna_data: &ProcessedDNAData,
        privacy_level: PrivacyLevel,
    ) -> PrivateProfile {
        let mut hashed_markers = HashMap::new();
        let mut redacted_count = 0;

        for (id, marker) in &dna_data.genetic_markers {
            match privacy_level {
                PrivacyLevel::Public => {
                    hashed_markers.insert(id.clone(), marker.value.clone());
                }
                PrivacyLevel::Protected => {
                    // Hash health-related markers, keep identity markers plain
                    if Self::is_health_marker(id) {
                        hashed_markers.insert(id.clone(), Self::hash_marker(&marker.value));
                        redacted_count += 1;
                    } else {
                        hashed_markers.insert(id.clone(), marker.value.clone());
                    }
                }
                PrivacyLevel::Private | PrivacyLevel::Medical => {
                    hashed_markers.insert(id.clone(), Self::hash_marker(&marker.value));
                    redacted_count += 1;
                }
            }
        }

        PrivateProfile {
            profile_id: dna_data.profile_id.clone(),
            hashed_markers,
            privacy_level,
            redacted_snp_count: redacted_count,
        }
    }

    /// Compare two privacy-protected profiles without revealing raw genetic data.
    pub fn perform_secure_comparison(
        &self,
        profile1: &PrivateProfile,
        profile2: &PrivateProfile,
    ) -> SecureComparisonResult {
        let shared_count = profile1
            .hashed_markers
            .keys()
            .filter(|k| profile2.hashed_markers.contains_key(*k))
            .count();

        if shared_count == 0 {
            return SecureComparisonResult {
                similarity_score: 0.0,
                shared_marker_count: 0,
                comparison_valid: false,
            };
        }

        let matching = profile1
            .hashed_markers
            .iter()
            .filter(|(k, v1)| profile2.hashed_markers.get(*k) == Some(v1))
            .count();

        let similarity = matching as f64 / shared_count as f64;

        SecureComparisonResult {
            similarity_score: similarity,
            shared_marker_count: shared_count,
            comparison_valid: true,
        }
    }

    /// Add Laplace-differential-privacy noise to numeric genetic data.
    pub fn generate_differential_privacy_noise(&self, data: &[f64], epsilon: f64) -> Vec<f64> {
        if epsilon <= 0.0 {
            return data.to_vec();
        }

        let sensitivity = 1.0;
        let scale = sensitivity / epsilon;

        data.iter()
            .map(|&value| {
                let noise = Self::sample_laplace(scale);
                value + noise
            })
            .collect()
    }

    fn hash_marker(value: &str) -> String {
        let hash = digest(&SHA256, value.as_bytes());
        hex::encode(hash.as_ref())
    }

    fn is_health_marker(rsid: &str) -> bool {
        matches!(
            rsid,
            "rs429358"
                | "rs7412"
                | "rs1801133"
                | "rs6025"
                | "rs1799963"
                | "rs7903146"
                | "rs1333049"
        )
    }

    fn sample_laplace(scale: f64) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let u: f64 = rng.gen::<f64>() - 0.5;
        -scale * u.signum() * (1.0 - 2.0 * u.abs()).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_processed_data() -> ProcessedDNAData {
        let mut markers = HashMap::new();
        markers.insert(
            "rs429358".into(),
            GeneticMarker {
                marker_id: "rs429358".into(),
                marker_type: "health".into(),
                value: "CT".into(),
            },
        );
        markers.insert(
            "rs123456".into(),
            GeneticMarker {
                marker_id: "rs123456".into(),
                marker_type: "identity".into(),
                value: "AG".into(),
            },
        );

        ProcessedDNAData {
            profile_id: "test-profile".into(),
            genetic_markers: markers,
            snp_data: vec![],
            ancestry_composition: AncestryBreakdown::default(),
            health_markers: vec![],
            privacy_level: PrivacyLevel::Protected,
        }
    }

    #[test]
    fn test_private_profile_redacts_all_markers() {
        let engine = GeneticPrivacyEngine;
        let data = sample_processed_data();
        let private = engine.create_privacy_preserving_profile(&data, PrivacyLevel::Private);
        assert_eq!(private.redacted_snp_count, 2);
        for value in private.hashed_markers.values() {
            assert_eq!(value.len(), 64); // SHA-256 hex
        }
    }

    #[test]
    fn test_protected_profile_selective_redaction() {
        let engine = GeneticPrivacyEngine;
        let data = sample_processed_data();
        let private = engine.create_privacy_preserving_profile(&data, PrivacyLevel::Protected);
        assert_eq!(private.redacted_snp_count, 1);
        assert_eq!(private.hashed_markers["rs123456"], "AG");
    }

    #[test]
    fn test_secure_comparison_identical_profiles() {
        let engine = GeneticPrivacyEngine;
        let data = sample_processed_data();
        let p1 = engine.create_privacy_preserving_profile(&data, PrivacyLevel::Private);
        let p2 = engine.create_privacy_preserving_profile(&data, PrivacyLevel::Private);
        let result = engine.perform_secure_comparison(&p1, &p2);
        assert!(result.comparison_valid);
        assert!((result.similarity_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_differential_privacy_adds_noise() {
        let engine = GeneticPrivacyEngine;
        let data = vec![1.0, 2.0, 3.0];
        let noisy = engine.generate_differential_privacy_noise(&data, 1.0);
        assert_eq!(noisy.len(), 3);
        // With noise, values should differ (extremely unlikely to be identical)
        assert!(noisy
            .iter()
            .zip(data.iter())
            .any(|(n, o)| (n - o).abs() > 1e-10));
    }

    #[test]
    fn test_zero_epsilon_returns_original() {
        let engine = GeneticPrivacyEngine;
        let data = vec![1.0, 2.0, 3.0];
        let result = engine.generate_differential_privacy_noise(&data, 0.0);
        assert_eq!(result, data);
    }
}
