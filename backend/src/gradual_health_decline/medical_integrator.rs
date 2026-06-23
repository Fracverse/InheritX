use super::errors::*;
use super::types::*;
use crate::genetic_analysis::health_monitoring::types::{LabResult, MedicalRecord as HealthMedicalRecord};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MedicalDataIntegrator;

impl MedicalDataIntegrator {
    pub async fn integrate_medical_records(
        &self,
        user_id: &str,
        medical_records: &[HealthMedicalRecord],
    ) -> Result<IntegratedHealthProfile, IntegrationError> {
        if medical_records.is_empty() {
            return Err(IntegrationError::RecordIntegrationFailed(
                "No medical records provided".to_string(),
            ));
        }

        let mut lab_results = Vec::new();
        let mut physician_assessments = Vec::new();
        let mut wearable_data = Vec::new();

        for record in medical_records {
            if record.record_type == RecordType::LabReport {
                lab_results.push(LabResult {
                    result_id: format!("{}_labs", record.record_id),
                    patient_id: record.patient_id.clone(),
                    test_name: record.diagnosis_codes.first().cloned().unwrap_or_default(),
                    test_code: record.diagnosis_codes.first().cloned().unwrap_or_default(),
                    value: format!("{:?}", record.vital_signs),
                    unit: "integrated".to_string(),
                    reference_range: "N/A".to_string(),
                    is_abnormal: false,
                    performed_at: record.timestamp,
                    ordering_provider: record.provider_info.provider_name.clone(),
                });
            }

            if record.record_type == RecordType::Consultation
                || record.record_type == RecordType::SpecialistReferral
            {
                physician_assessments.push(PhysicianAssessment {
                    assessment_id: record.record_id.clone(),
                    physician_id: record.provider_info.provider_id.clone(),
                    physician_name: record.provider_info.provider_name.clone(),
                    assessment_date: record.timestamp,
                    diagnoses: record.diagnosis_codes.clone(),
                    observations: record
                        .procedures
                        .iter()
                        .map(|p| p.description.clone())
                        .collect(),
                    prognosis: "See detailed assessment".to_string(),
                    recommended_actions: record.medications.iter().map(|m| m.name.clone()).collect(),
                    functional_capacity_score: None,
                });
            }
        }

        let completeness = if physician_assessments.is_empty() {
            0.5
        } else {
            0.9
        };

        Ok(IntegratedHealthProfile {
            user_id: user_id.to_string(),
            medical_records: medical_records.to_vec(),
            lab_results,
            physician_assessments,
            wearable_data,
            genetic_data: None,
            integrated_at: chrono::Utc::now().timestamp_millis() as u64,
            completeness_score: completeness,
        })
    }

    pub async fn incorporate_lab_results(
        &self,
        health_profile: &mut IntegratedHealthProfile,
        lab_results: &[LabResult],
    ) -> Result<(), IncorporationError> {
        if lab_results.is_empty() {
            return Ok(());
        }

        health_profile.lab_results.extend(lab_results.to_vec());
        health_profile.completeness_score = (health_profile.completeness_score + 0.1).min(1.0);

        Ok(())
    }

    pub async fn include_physician_assessments(
        &self,
        health_profile: &mut IntegratedHealthProfile,
        assessments: &[PhysicianAssessment],
    ) -> Result<(), InclusionError> {
        if assessments.is_empty() {
            return Ok(());
        }

        health_profile.physician_assessments.extend(assessments.to_vec());
        health_profile.completeness_score = (health_profile.completeness_score + 0.1).min(1.0);

        Ok(())
    }
}
