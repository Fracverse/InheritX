use super::errors::{EHRError, HealthError};
use super::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait EHRClient: Send + Sync {
    async fn fetch_patient_records(&self, patient_id: &str)
        -> Result<Vec<MedicalRecord>, EHRError>;
    async fn get_lab_results(
        &self,
        patient_id: &str,
        date_range: DateRange,
    ) -> Result<Vec<LabResult>, EHRError>;
    async fn retrieve_imaging_studies(
        &self,
        patient_id: &str,
    ) -> Result<Vec<ImagingStudy>, EHRError>;
}

pub struct LabResultParser;

impl LabResultParser {
    pub fn parse_result(&self, raw_value: &str, test_code: &str) -> Result<f64, HealthError> {
        raw_value.trim().parse::<f64>().map_err(|e| {
            HealthError::InvalidData(format!("Failed to parse lab result for {test_code}: {e}"))
        })
    }

    pub fn is_result_abnormal(&self, value: f64, reference_range: &str) -> bool {
        let range = reference_range.replace(' ', "");
        let parts: Vec<&str> = range.split(|c| c == '-' || c == '–' || c == '~').collect();

        if parts.len() != 2 {
            return false;
        }

        let low = parts[0].trim().parse::<f64>().ok();
        let high = parts[1].trim().parse::<f64>().ok();

        match (low, high) {
            (Some(l), Some(h)) => value < l || value > h,
            _ => false,
        }
    }

    pub fn categorize_result(&self, value: f64, test_name: &str) -> String {
        match test_name {
            t if t.contains("glucose") || t.contains("Glucose") => {
                if value < 70.0 {
                    "Low (Hypoglycemia)".into()
                } else if value > 140.0 {
                    "High (Hyperglycemia)".into()
                } else {
                    "Normal".into()
                }
            }
            t if t.contains("cholesterol") || t.contains("Cholesterol") => {
                if value > 200.0 {
                    "High".into()
                } else {
                    "Normal".into()
                }
            }
            _ => {
                if self.is_result_abnormal(value, "0-100") {
                    "Abnormal".into()
                } else {
                    "Normal".into()
                }
            }
        }
    }
}

pub struct MedicalImagingAnalyzer;

impl MedicalImagingAnalyzer {
    pub fn extract_key_findings(&self, study: &ImagingStudy) -> Vec<String> {
        let mut findings = Vec::new();

        let keywords = [
            "abnormal",
            "lesion",
            "tumor",
            "mass",
            "fracture",
            "stenosis",
            "dilation",
            "calcification",
            "atrophy",
            "infarction",
            "hemorrhage",
            "edema",
            "nodule",
            "cyst",
        ];

        let lower_findings = study.findings.to_lowercase();
        for keyword in &keywords {
            if lower_findings.contains(keyword) {
                findings.push(keyword.to_string());
            }
        }

        findings
    }

    pub fn assess_urgency(&self, study: &ImagingStudy) -> UrgencyLevel {
        let critical_keywords = [
            "hemorrhage",
            "infarction",
            "fracture",
            "malignant",
            "emergency",
        ];
        let high_keywords = ["tumor", "mass", "stenosis", "lesion", "nodule"];

        let lower_impression = study.impression.to_lowercase();

        if critical_keywords
            .iter()
            .any(|k| lower_impression.contains(k))
        {
            UrgencyLevel::Critical
        } else if high_keywords.iter().any(|k| lower_impression.contains(k)) {
            UrgencyLevel::High
        } else {
            UrgencyLevel::Low
        }
    }
}

pub struct PrescriptionTracker;

impl PrescriptionTracker {
    pub fn check_drug_interactions(&self, medications: &[Medication]) -> Vec<String> {
        let mut warnings = Vec::new();
        for i in 0..medications.len() {
            for j in (i + 1)..medications.len() {
                let interaction =
                    self.lookup_interaction(&medications[i].name, &medications[j].name);
                if let Some(warning) = interaction {
                    warnings.push(warning);
                }
            }
        }
        warnings
    }

    pub fn check_dosage(&self, medication: &Medication, max_daily_dose: f64) -> bool {
        let dosage_value = medication
            .dosage
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        dosage_value <= max_daily_dose
    }

    fn lookup_interaction(&self, drug_a: &str, drug_b: &str) -> Option<String> {
        let known_interactions: Vec<(&str, &str, &str)> = vec![
            ("Warfarin", "Aspirin", "Increased bleeding risk"),
            ("Metformin", "Contrast dye", "Risk of lactic acidosis"),
            (
                "ACE Inhibitors",
                "Potassium supplements",
                "Risk of hyperkalemia",
            ),
            ("SSRI", "MAOI", "Risk of serotonin syndrome"),
        ];

        for (a, b, risk) in &known_interactions {
            if (drug_a.contains(a) && drug_b.contains(b))
                || (drug_a.contains(b) && drug_b.contains(a))
            {
                return Some(format!(
                    "Interaction between {} and {}: {}",
                    drug_a, drug_b, risk
                ));
            }
        }
        None
    }
}

pub struct MedicalRecordIntegrator {
    pub ehr_clients: Vec<Box<dyn EHRClient>>,
    pub lab_result_parser: LabResultParser,
    pub imaging_analyzer: MedicalImagingAnalyzer,
    pub prescription_tracker: PrescriptionTracker,
}

impl MedicalRecordIntegrator {
    pub fn new() -> Self {
        Self {
            ehr_clients: Vec::new(),
            lab_result_parser: LabResultParser,
            imaging_analyzer: MedicalImagingAnalyzer,
            prescription_tracker: PrescriptionTracker,
        }
    }

    pub fn add_ehr_client(&mut self, client: Box<dyn EHRClient>) {
        self.ehr_clients.push(client);
    }

    pub async fn fetch_all_records(
        &self,
        patient_id: &str,
    ) -> Result<Vec<MedicalRecord>, HealthError> {
        let mut all_records = Vec::new();

        for client in &self.ehr_clients {
            match client.fetch_patient_records(patient_id).await {
                Ok(records) => all_records.extend(records),
                Err(e) => {
                    tracing::warn!("EHR client failed for patient {patient_id}: {e}");
                }
            }
        }

        Ok(all_records)
    }

    pub async fn fetch_lab_results(
        &self,
        patient_id: &str,
        date_range: DateRange,
    ) -> Result<Vec<LabResult>, HealthError> {
        let mut all_results = Vec::new();

        for client in &self.ehr_clients {
            match client.get_lab_results(patient_id, date_range.clone()).await {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    tracing::warn!("Failed to fetch lab results for {patient_id}: {e}");
                }
            }
        }

        Ok(all_results)
    }

    pub async fn fetch_imaging_studies(
        &self,
        patient_id: &str,
    ) -> Result<Vec<ImagingStudy>, HealthError> {
        let mut all_studies = Vec::new();

        for client in &self.ehr_clients {
            match client.retrieve_imaging_studies(patient_id).await {
                Ok(studies) => all_studies.extend(studies),
                Err(e) => {
                    tracing::warn!("Failed to fetch imaging studies for {patient_id}: {e}");
                }
            }
        }

        Ok(all_studies)
    }

    pub fn extract_diagnosis_codes(&self, records: &[MedicalRecord]) -> Vec<String> {
        let mut codes: Vec<String> = records
            .iter()
            .flat_map(|r| r.diagnosis_codes.clone())
            .collect();
        codes.sort();
        codes.dedup();
        codes
    }

    pub fn find_relevant_records(
        &self,
        records: &[MedicalRecord],
        condition: &str,
    ) -> Vec<&MedicalRecord> {
        let condition_lower = condition.to_lowercase();
        records
            .iter()
            .filter(|r| {
                r.diagnosis_codes
                    .iter()
                    .any(|code| code.to_lowercase().contains(&condition_lower))
                    || r.record_type == RecordType::Consultation
            })
            .collect()
    }
}

impl Default for MedicalRecordIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockEHRClient;

    #[async_trait]
    impl EHRClient for MockEHRClient {
        async fn fetch_patient_records(
            &self,
            _patient_id: &str,
        ) -> Result<Vec<MedicalRecord>, EHRError> {
            Ok(vec![MedicalRecord {
                record_id: "rec-001".into(),
                patient_id: _patient_id.into(),
                record_type: RecordType::Consultation,
                diagnosis_codes: vec!["E11".into(), "I10".into()],
                procedures: vec![],
                medications: vec![],
                vital_signs: VitalSigns {
                    temperature: Some(37.0),
                    heart_rate: Some(72),
                    blood_pressure_systolic: Some(130),
                    blood_pressure_diastolic: Some(85),
                    respiratory_rate: Some(16),
                    oxygen_saturation: Some(98.0),
                    pain_level: Some(0),
                },
                timestamp: 1700000000,
                provider_info: ProviderInfo {
                    provider_id: "prov-001".into(),
                    provider_name: "Dr. Smith".into(),
                    facility: "General Hospital".into(),
                    specialty: Some("Cardiology".into()),
                },
            }])
        }

        async fn get_lab_results(
            &self,
            patient_id: &str,
            _date_range: DateRange,
        ) -> Result<Vec<LabResult>, EHRError> {
            Ok(vec![LabResult {
                result_id: "lab-001".into(),
                patient_id: patient_id.into(),
                test_name: "Fasting Glucose".into(),
                test_code: "GLU".into(),
                value: "110".into(),
                unit: "mg/dL".into(),
                reference_range: "70-100".to_string(),
                is_abnormal: true,
                performed_at: 1700000000,
                ordering_provider: "Dr. Smith".into(),
            }])
        }

        async fn retrieve_imaging_studies(
            &self,
            _patient_id: &str,
        ) -> Result<Vec<ImagingStudy>, EHRError> {
            Ok(vec![ImagingStudy {
                study_id: "img-001".into(),
                patient_id: _patient_id.into(),
                study_type: "CT".into(),
                body_region: "Chest".into(),
                findings: "No abnormal findings".into(),
                impression: "Normal study".into(),
                performed_at: 1700000000,
                radiologist: "Dr. Jones".into(),
            }])
        }
    }

    struct FailingMockEHRClient;

    #[async_trait]
    impl EHRClient for FailingMockEHRClient {
        async fn fetch_patient_records(
            &self,
            _patient_id: &str,
        ) -> Result<Vec<MedicalRecord>, EHRError> {
            Err(EHRError::ProviderUnavailable("Down for maintenance".into()))
        }

        async fn get_lab_results(
            &self,
            _patient_id: &str,
            _date_range: DateRange,
        ) -> Result<Vec<LabResult>, EHRError> {
            Err(EHRError::ProviderUnavailable("Down for maintenance".into()))
        }

        async fn retrieve_imaging_studies(
            &self,
            _patient_id: &str,
        ) -> Result<Vec<ImagingStudy>, EHRError> {
            Err(EHRError::ProviderUnavailable("Down for maintenance".into()))
        }
    }

    #[tokio::test]
    async fn test_fetch_all_records_with_mock_client() {
        let mut integrator = MedicalRecordIntegrator::new();
        integrator.add_ehr_client(Box::new(MockEHRClient));

        let records = integrator.fetch_all_records("pat-001").await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].patient_id, "pat-001");
    }

    #[tokio::test]
    async fn test_fetch_records_with_failing_client_does_not_panic() {
        let mut integrator = MedicalRecordIntegrator::new();
        integrator.add_ehr_client(Box::new(FailingMockEHRClient));

        let records = integrator.fetch_all_records("pat-001").await.unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_lab_results() {
        let mut integrator = MedicalRecordIntegrator::new();
        integrator.add_ehr_client(Box::new(MockEHRClient));

        let date_range = DateRange {
            start: 1600000000,
            end: 1800000000,
        };
        let results = integrator
            .fetch_lab_results("pat-001", date_range)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].test_name, "Fasting Glucose");
    }

    #[test]
    fn test_extract_diagnosis_codes_dedup() {
        let integrator = MedicalRecordIntegrator::new();
        let records = vec![
            MedicalRecord {
                record_id: "1".into(),
                patient_id: "p1".into(),
                record_type: RecordType::Consultation,
                diagnosis_codes: vec!["E11".into(), "I10".into()],
                procedures: vec![],
                medications: vec![],
                vital_signs: VitalSigns {
                    temperature: None,
                    heart_rate: None,
                    blood_pressure_systolic: None,
                    blood_pressure_diastolic: None,
                    respiratory_rate: None,
                    oxygen_saturation: None,
                    pain_level: None,
                },
                timestamp: 0,
                provider_info: ProviderInfo {
                    provider_id: "".into(),
                    provider_name: "".into(),
                    facility: "".into(),
                    specialty: None,
                },
            },
            MedicalRecord {
                record_id: "2".into(),
                patient_id: "p1".into(),
                record_type: RecordType::Consultation,
                diagnosis_codes: vec!["E11".into()],
                procedures: vec![],
                medications: vec![],
                vital_signs: VitalSigns {
                    temperature: None,
                    heart_rate: None,
                    blood_pressure_systolic: None,
                    blood_pressure_diastolic: None,
                    respiratory_rate: None,
                    oxygen_saturation: None,
                    pain_level: None,
                },
                timestamp: 0,
                provider_info: ProviderInfo {
                    provider_id: "".into(),
                    provider_name: "".into(),
                    facility: "".into(),
                    specialty: None,
                },
            },
        ];

        let codes = integrator.extract_diagnosis_codes(&records);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&"E11".to_string()));
        assert!(codes.contains(&"I10".to_string()));
    }

    #[test]
    fn test_lab_result_parser_parse_value() {
        let parser = LabResultParser;
        let result = parser.parse_result("110.5", "GLU").unwrap();
        assert!((result - 110.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lab_result_parser_abnormal_detection() {
        let parser = LabResultParser;
        assert!(parser.is_result_abnormal(120.0, "70-100"));
        assert!(!parser.is_result_abnormal(85.0, "70-100"));
    }

    #[test]
    fn test_imaging_analyzer_assess_urgency_critical() {
        let analyzer = MedicalImagingAnalyzer;
        let study = ImagingStudy {
            study_id: "s1".into(),
            patient_id: "p1".into(),
            study_type: "CT".into(),
            body_region: "Head".into(),
            findings: "Acute hemorrhage detected".into(),
            impression: "Intracranial hemorrhage - emergency".into(),
            performed_at: 0,
            radiologist: "Dr. A".into(),
        };
        assert_eq!(analyzer.assess_urgency(&study), UrgencyLevel::Critical);
    }

    #[test]
    fn test_imaging_analyzer_assess_urgency_low() {
        let analyzer = MedicalImagingAnalyzer;
        let study = ImagingStudy {
            study_id: "s1".into(),
            patient_id: "p1".into(),
            study_type: "X-Ray".into(),
            body_region: "Chest".into(),
            findings: "No abnormalities".into(),
            impression: "Normal chest X-ray".into(),
            performed_at: 0,
            radiologist: "Dr. A".into(),
        };
        assert_eq!(analyzer.assess_urgency(&study), UrgencyLevel::Low);
    }

    #[test]
    fn test_prescription_tracker_drug_interaction() {
        let tracker = PrescriptionTracker;
        let medications = vec![
            Medication {
                name: "Warfarin".into(),
                dosage: "5mg".into(),
                frequency: "daily".into(),
                prescribed_date: 0,
                prescribing_physician: "Dr. A".into(),
            },
            Medication {
                name: "Aspirin".into(),
                dosage: "100mg".into(),
                frequency: "daily".into(),
                prescribed_date: 0,
                prescribing_physician: "Dr. B".into(),
            },
        ];

        let warnings = tracker.check_drug_interactions(&medications);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("Warfarin"));
    }

    #[test]
    fn test_prescription_tracker_dosage_check() {
        let tracker = PrescriptionTracker;
        let med = Medication {
            name: "Metformin".into(),
            dosage: "500mg".into(),
            frequency: "twice daily".into(),
            prescribed_date: 0,
            prescribing_physician: "Dr. A".into(),
        };
        assert!(tracker.check_dosage(&med, 2000.0));
        assert!(!tracker.check_dosage(&med, 200.0));
    }

    #[test]
    fn test_lab_categorize_glucose() {
        let parser = LabResultParser;
        assert_eq!(
            parser.categorize_result(65.0, "Glucose"),
            "Low (Hypoglycemia)"
        );
        assert_eq!(parser.categorize_result(95.0, "Glucose"), "Normal");
        assert_eq!(
            parser.categorize_result(200.0, "Glucose"),
            "High (Hyperglycemia)"
        );
    }

    #[test]
    fn test_integrator_find_relevant_records_by_diagnosis() {
        let integrator = MedicalRecordIntegrator::new();
        let records = vec![MedicalRecord {
            record_id: "1".into(),
            patient_id: "p1".into(),
            record_type: RecordType::HospitalAdmission,
            diagnosis_codes: vec!["E11".into()],
            procedures: vec![],
            medications: vec![],
            vital_signs: VitalSigns {
                temperature: None,
                heart_rate: None,
                blood_pressure_systolic: None,
                blood_pressure_diastolic: None,
                respiratory_rate: None,
                oxygen_saturation: None,
                pain_level: None,
            },
            timestamp: 0,
            provider_info: ProviderInfo {
                provider_id: "".into(),
                provider_name: "".into(),
                facility: "".into(),
                specialty: None,
            },
        }];

        let relevant = integrator.find_relevant_records(&records, "E11");
        assert_eq!(relevant.len(), 1);
    }
}
