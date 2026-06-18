//! Integration tests for the genetic analysis service (Issue #14).

use inheritx_backend::genetic_analysis::{
    DNAProfile, GeneticAnalysisService, GeneticSimilarityCalculator, HealthConditionAnalyzer,
    PrivacyLevel, SNPVariant, VariantSignificance,
};
use std::collections::HashMap;

fn synthetic_parent_dna() -> Vec<u8> {
    b"rs429358\t19\t45411941\tCC\n\
      rs7412\t19\t45412079\tCT\n\
      rs1801133\t1\t11796321\tTT\n\
      rs6025\t1\t169519049\tGG\n\
      rs7903146\t10\t114758349\tTT\n\
      rs1333049\t9\t22125503\tCC\n\
      rs9923231\t16\t31107689\tAA\n\
      rs6152\tX\t67545785\tGG\n"
        .to_vec()
}

fn synthetic_child_dna() -> Vec<u8> {
    b"rs429358\t19\t45411941\tCT\n\
      rs7412\t19\t45412079\tCT\n\
      rs1801133\t1\t11796321\tCT\n\
      rs6025\t1\t169519049\tGG\n\
      rs7903146\t10\t114758349\tTC\n\
      rs1333049\t9\t22125503\tCG\n\
      rs9923231\t16\t31107689\tAG\n\
      rs6152\tX\t67545785\tGG\n"
        .to_vec()
}

fn unrelated_dna() -> Vec<u8> {
    b"rs429358\t19\t45411941\tGG\n\
      rs7412\t19\t45412079\tTT\n\
      rs1801133\t1\t11796321\tCC\n\
      rs6025\t1\t169519049\tCC\n\
      rs7903146\t10\t114758349\tCC\n\
      rs1333049\t9\t22125503\tTT\n"
        .to_vec()
}

#[tokio::test]
async fn test_full_analysis_pipeline() {
    let service = GeneticAnalysisService::new();

    let processed = service
        .process_raw_dna_data(synthetic_parent_dna(), PrivacyLevel::Protected)
        .await
        .expect("DNA processing should succeed");

    assert!(!processed.profile_id.is_empty());
    assert_eq!(processed.snp_data.len(), 8);
    assert!(!processed.health_markers.is_empty());
    assert!(!processed.ancestry_composition.populations.is_empty());

    let profile = DNAProfile::from(&processed);

    let conditions = service
        .detect_health_conditions(&profile)
        .await
        .expect("Health condition detection should succeed");
    assert!(!conditions.is_empty());

    let assessment = service
        .assess_genetic_risks(&profile, 50)
        .await
        .expect("Risk assessment should succeed");
    assert!(assessment.overall_health_score >= 0.0);
    assert!(!assessment.polygenic_scores.is_empty());
    assert!(!assessment.lifestyle_recommendations.is_empty());

    let associations = service
        .enrich_with_external_data(&processed)
        .await
        .expect("External database enrichment should succeed");
    assert!(!associations.is_empty());
}

#[tokio::test]
async fn test_parent_child_similarity_higher_than_unrelated() {
    let service = GeneticAnalysisService::new();

    let parent_data = service
        .process_raw_dna_data(synthetic_parent_dna(), PrivacyLevel::Public)
        .await
        .unwrap();
    let child_data = service
        .process_raw_dna_data(synthetic_child_dna(), PrivacyLevel::Public)
        .await
        .unwrap();
    let unrelated_data = service
        .process_raw_dna_data(unrelated_dna(), PrivacyLevel::Public)
        .await
        .unwrap();

    let parent = DNAProfile::from(&parent_data);
    let child = DNAProfile::from(&child_data);
    let unrelated = DNAProfile::from(&unrelated_data);

    let parent_child_sim = service
        .calculate_genetic_similarity(&parent, &child)
        .await
        .unwrap();
    let parent_unrelated_sim = service
        .calculate_genetic_similarity(&parent, &unrelated)
        .await
        .unwrap();

    assert!(
        parent_child_sim > parent_unrelated_sim,
        "Parent-child similarity ({parent_child_sim}) should exceed unrelated ({parent_unrelated_sim})"
    );
}

#[tokio::test]
async fn test_relationship_estimation() {
    let service = GeneticAnalysisService::new();

    let data1 = service
        .process_raw_dna_data(synthetic_parent_dna(), PrivacyLevel::Public)
        .await
        .unwrap();
    let data2 = service
        .process_raw_dna_data(synthetic_child_dna(), PrivacyLevel::Public)
        .await
        .unwrap();

    let estimate = service
        .estimate_relationship(&DNAProfile::from(&data1), &DNAProfile::from(&data2))
        .await
        .unwrap();

    assert!(estimate.confidence > 0.0);
    assert!(estimate.shared_centimorgans > 0.0);
}

#[tokio::test]
async fn test_health_analyzer_drug_responses() {
    let analyzer = HealthConditionAnalyzer::new();
    let profile = DNAProfile {
        profile_id: "test".into(),
        snp_data: vec![SNPVariant {
            rsid: "rs9923231".into(),
            chromosome: 16,
            position: 31107689,
            genotype: "AA".into(),
            significance: VariantSignificance::Uncertain,
        }],
        genetic_markers: HashMap::new(),
    };

    let responses = analyzer.analyze_drug_responses(&profile).await.unwrap();
    assert!(!responses.is_empty());
    assert!(responses.iter().any(|r| r.drug_name == "Warfarin"));
}

#[tokio::test]
async fn test_similarity_calculator_ibd_ibs() {
    let calc = GeneticSimilarityCalculator;
    let snps = vec![
        SNPVariant {
            rsid: "rs1".into(),
            chromosome: 1,
            position: 100,
            genotype: "AA".into(),
            significance: VariantSignificance::Uncertain,
        },
        SNPVariant {
            rsid: "rs2".into(),
            chromosome: 1,
            position: 200,
            genotype: "AG".into(),
            significance: VariantSignificance::Uncertain,
        },
    ];

    let p1 = DNAProfile {
        profile_id: "a".into(),
        snp_data: snps.clone(),
        genetic_markers: HashMap::new(),
    };
    let p2 = DNAProfile {
        profile_id: "b".into(),
        snp_data: snps,
        genetic_markers: HashMap::new(),
    };

    let ibs = calc.calculate_identity_by_state(&p1, &p2);
    let ibd = calc.calculate_identity_by_descent(&p1, &p2);
    let cm = calc.calculate_centimorgan_sharing(&p1, &p2);

    assert!((ibs - 1.0).abs() < f64::EPSILON);
    assert!(ibd > 0.0);
    assert!(cm > 0.0);
}

#[tokio::test]
async fn test_privacy_preserving_analysis() {
    let service = GeneticAnalysisService::new();

    let processed = service
        .process_raw_dna_data(synthetic_parent_dna(), PrivacyLevel::Private)
        .await
        .unwrap();

    let private1 = service
        .privacy_engine
        .create_privacy_preserving_profile(&processed, PrivacyLevel::Private);
    let private2 = service
        .privacy_engine
        .create_privacy_preserving_profile(&processed, PrivacyLevel::Private);

    let comparison = service
        .privacy_engine
        .perform_secure_comparison(&private1, &private2);
    assert!(comparison.comparison_valid);
    assert!((comparison.similarity_score - 1.0).abs() < f64::EPSILON);

    let noisy = service
        .privacy_engine
        .generate_differential_privacy_noise(&[0.5, 0.8, 0.3], 1.0);
    assert_eq!(noisy.len(), 3);
}

#[tokio::test]
async fn test_risk_assessment_inheritance_triggers() {
    let service = GeneticAnalysisService::new();

    // High-risk profile with homozygous risk alleles
    let high_risk_dna = b"rs429358\t19\t45411941\tCC\n\
        rs6025\t1\t169519049\tAA\n\
        rs7903146\t10\t114758349\tTT\n\
        rs1333049\t9\t22125503\tCC\n";

    let processed = service
        .process_raw_dna_data(high_risk_dna.to_vec(), PrivacyLevel::Medical)
        .await
        .unwrap();
    let profile = DNAProfile::from(&processed);

    let assessment = service.assess_genetic_risks(&profile, 55).await.unwrap();
    assert!(assessment.overall_health_score < 85.0);
    assert!(!assessment.screening_recommendations.is_empty());
}

#[tokio::test]
async fn test_insufficient_data_error() {
    let service = GeneticAnalysisService::new();
    let empty_profile = DNAProfile {
        profile_id: "empty".into(),
        snp_data: vec![],
        genetic_markers: HashMap::new(),
    };

    let result = service
        .calculate_genetic_similarity(&empty_profile, &empty_profile)
        .await;
    assert!(result.is_err());
}
