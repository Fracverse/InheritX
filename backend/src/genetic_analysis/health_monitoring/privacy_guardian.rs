use super::super::types::PrivacyLevel;
use super::errors::{AuditError, PrivacyError};
use super::types::*;
use ring::digest::{digest, SHA256};
use std::collections::HashMap;

pub struct ConsentManager;

impl ConsentManager {
    pub fn verify_consent(
        &self,
        user_id: u64,
        data_type: MedicalDataType,
        granted_consents: &HashMap<(u64, MedicalDataType), bool>,
    ) -> bool {
        *granted_consents
            .get(&(user_id, data_type))
            .unwrap_or(&false)
    }

    pub fn revoke_consent(
        &self,
        user_id: u64,
        data_type: MedicalDataType,
        granted_consents: &mut HashMap<(u64, MedicalDataType), bool>,
    ) {
        granted_consents.insert((user_id, data_type), false);
    }

    pub fn grant_consent(
        &self,
        user_id: u64,
        data_type: MedicalDataType,
        granted_consents: &mut HashMap<(u64, MedicalDataType), bool>,
    ) {
        granted_consents.insert((user_id, data_type), true);
    }

    pub fn get_consented_data_types(
        &self,
        user_id: u64,
        granted_consents: &HashMap<(u64, MedicalDataType), bool>,
    ) -> Vec<MedicalDataType> {
        granted_consents
            .iter()
            .filter(|(&(uid, _), &granted)| uid == user_id && granted)
            .map(|(&(_, dt), _)| dt)
            .collect()
    }
}

pub struct HealthDataAnonymizer;

impl HealthDataAnonymizer {
    pub fn anonymize(&self, records: &[MedicalRecord]) -> Vec<AnonymizedRecord> {
        records
            .iter()
            .map(|r| {
                let age_range = self.compute_age_range(r.timestamp);
                AnonymizedRecord {
                    anonymized_id: self.hash_id(&r.record_id),
                    record_type: r.record_type.clone(),
                    diagnosis_codes: r.diagnosis_codes.clone(),
                    vital_signs: r.vital_signs.clone(),
                    timestamp: r.timestamp,
                    age_range,
                    gender: None,
                }
            })
            .collect()
    }

    pub fn anonymize_with_privacy_level(
        &self,
        records: &[MedicalRecord],
        level: PrivacyLevel,
    ) -> Vec<AnonymizedRecord> {
        records
            .iter()
            .map(|r| {
                let age_range = self.compute_age_range(r.timestamp);
                let codes = match level {
                    PrivacyLevel::Medical => r.diagnosis_codes.clone(),
                    PrivacyLevel::Private => r
                        .diagnosis_codes
                        .iter()
                        .map(|_| "REDACTED".to_string())
                        .collect(),
                    PrivacyLevel::Protected | PrivacyLevel::Public => r
                        .diagnosis_codes
                        .iter()
                        .take(3)
                        .map(|c| format!("{}-XXX", &c[..c.len().min(1)]))
                        .collect(),
                };

                let vitals = match level {
                    PrivacyLevel::Medical => r.vital_signs.clone(),
                    PrivacyLevel::Private | PrivacyLevel::Protected => VitalSigns {
                        temperature: None,
                        heart_rate: None,
                        blood_pressure_systolic: None,
                        blood_pressure_diastolic: None,
                        respiratory_rate: None,
                        oxygen_saturation: None,
                        pain_level: None,
                    },
                    PrivacyLevel::Public => VitalSigns {
                        temperature: None,
                        heart_rate: None,
                        blood_pressure_systolic: None,
                        blood_pressure_diastolic: None,
                        respiratory_rate: None,
                        oxygen_saturation: None,
                        pain_level: None,
                    },
                };

                AnonymizedRecord {
                    anonymized_id: self.hash_id(&r.record_id),
                    record_type: r.record_type.clone(),
                    diagnosis_codes: codes,
                    vital_signs: vitals,
                    timestamp: r.timestamp,
                    age_range,
                    gender: None,
                }
            })
            .collect()
    }

    fn hash_id(&self, id: &str) -> String {
        let hash = digest(&SHA256, id.as_bytes());
        hex::encode(hash.as_ref())
    }

    fn compute_age_range(&self, _timestamp: u64) -> String {
        let ranges = [
            (0, 18, "0-17"),
            (18, 30, "18-29"),
            (30, 40, "30-39"),
            (40, 50, "40-49"),
            (50, 60, "50-59"),
            (60, 70, "60-69"),
            (70, 80, "70-79"),
            (80, u64::MAX, "80+"),
        ];

        for (low, high, label) in &ranges {
            if _timestamp > 0 && *_timestamp > 0 {
                let age_estimate = 40;
                if age_estimate >= *low && age_estimate < *high {
                    return label.to_string();
                }
            }
        }
        "Unknown".into()
    }
}

pub struct MedicalAccessController {
    access_log: Vec<AccessLogEntry>,
}

struct AccessLogEntry {
    accessor: String,
    accessed_data: MedicalDataType,
    action: String,
    timestamp: u64,
    allowed: bool,
}

impl MedicalAccessController {
    pub fn new() -> Self {
        Self {
            access_log: Vec::new(),
        }
    }

    pub fn check_access(
        &mut self,
        accessor: &str,
        data_type: MedicalDataType,
        user_consented: bool,
    ) -> bool {
        let allowed = user_consented;

        self.access_log.push(AccessLogEntry {
            accessor: accessor.to_string(),
            accessed_data: data_type,
            action: if allowed {
                "ACCESS_GRANTED".into()
            } else {
                "ACCESS_DENIED".into()
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            allowed,
        });

        allowed
    }

    pub fn audit_access(
        &self,
        accessor: &str,
        _accessed_data: &MedicalDataType,
    ) -> Result<(), AuditError> {
        let recent_access: Vec<&AccessLogEntry> = self
            .access_log
            .iter()
            .filter(|e| e.accessor == accessor)
            .collect();

        if recent_access.is_empty() {
            return Err(AuditError::UnauthorizedAccess(format!(
                "No access records found for {accessor}"
            )));
        }

        let denied_count = recent_access.iter().filter(|e| !e.allowed).count();
        if denied_count > 3 {
            return Err(AuditError::UnauthorizedAccess(format!(
                "Multiple denied access attempts ({denied_count}) by {accessor}"
            )));
        }

        Ok(())
    }

    pub fn get_access_history(&self, accessor: &str) -> Vec<(MedicalDataType, bool, u64)> {
        self.access_log
            .iter()
            .filter(|e| e.accessor == accessor)
            .map(|e| (e.accessed_data, e.allowed, e.timestamp))
            .collect()
    }
}

pub struct HealthPrivacyGuardian {
    pub consent_manager: ConsentManager,
    pub data_anonymizer: HealthDataAnonymizer,
    pub access_controller: MedicalAccessController,
    granted_consents: HashMap<(u64, MedicalDataType), bool>,
}

impl HealthPrivacyGuardian {
    pub fn new() -> Self {
        Self {
            consent_manager: ConsentManager,
            data_anonymizer: HealthDataAnonymizer,
            access_controller: MedicalAccessController::new(),
            granted_consents: HashMap::new(),
        }
    }

    pub fn verify_medical_consent(
        &self,
        user_id: u64,
        data_type: MedicalDataType,
    ) -> Result<bool, PrivacyError> {
        Ok(self
            .consent_manager
            .verify_consent(user_id, data_type, &self.granted_consents))
    }

    pub fn anonymize_health_data(
        &self,
        records: &[MedicalRecord],
    ) -> Result<Vec<AnonymizedRecord>, PrivacyError> {
        if records.is_empty() {
            return Err(PrivacyError::AnonymizationFailed(
                "No records to anonymize".into(),
            ));
        }
        Ok(self.data_anonymizer.anonymize(records))
    }

    pub fn anonymize_health_data_with_level(
        &self,
        records: &[MedicalRecord],
        level: PrivacyLevel,
    ) -> Result<Vec<AnonymizedRecord>, PrivacyError> {
        if records.is_empty() {
            return Err(PrivacyError::AnonymizationFailed(
                "No records to anonymize".into(),
            ));
        }
        Ok(self
            .data_anonymizer
            .anonymize_with_privacy_level(records, level))
    }

    pub fn audit_medical_access(
        &self,
        accessor: &str,
        accessed_data: &MedicalDataType,
    ) -> Result<(), AuditError> {
        self.access_controller.audit_access(accessor, accessed_data)
    }

    pub fn grant_consent(&mut self, user_id: u64, data_type: MedicalDataType) {
        self.consent_manager
            .grant_consent(user_id, data_type, &mut self.granted_consents);
    }

    pub fn revoke_consent(&mut self, user_id: u64, data_type: MedicalDataType) {
        self.consent_manager
            .revoke_consent(user_id, data_type, &mut self.granted_consents);
    }
}

impl Default for HealthPrivacyGuardian {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_manager_grant_and_verify() {
        let consent = ConsentManager;
        let mut consents = HashMap::new();

        consent.grant_consent(1, MedicalDataType::LabResults, &mut consents);
        assert!(consent.verify_consent(1, MedicalDataType::LabResults, &consents));
        assert!(!consent.verify_consent(1, MedicalDataType::GeneticData, &consents));
    }

    #[test]
    fn test_consent_manager_revoke() {
        let consent = ConsentManager;
        let mut consents = HashMap::new();

        consent.grant_consent(1, MedicalDataType::LabResults, &mut consents);
        consent.revoke_consent(1, MedicalDataType::LabResults, &mut consents);
        assert!(!consent.verify_consent(1, MedicalDataType::LabResults, &consents));
    }

    #[test]
    fn test_consent_manager_get_consented_types() {
        let consent = ConsentManager;
        let mut consents = HashMap::new();

        consent.grant_consent(1, MedicalDataType::LabResults, &mut consents);
        consent.grant_consent(1, MedicalDataType::GeneticData, &mut consents);
        consent.grant_consent(2, MedicalDataType::VitalSigns, &mut consents);

        let types = consent.get_consented_data_types(1, &consents);
        assert_eq!(types.len(), 2);
        assert!(types.contains(&MedicalDataType::LabResults));
        assert!(types.contains(&MedicalDataType::GeneticData));
    }

    #[test]
    fn test_anonymizer_anonymize_records() {
        let anonymizer = HealthDataAnonymizer;
        let records = vec![MedicalRecord {
            record_id: "rec-001".into(),
            patient_id: "pat-001".into(),
            record_type: RecordType::LabReport,
            diagnosis_codes: vec!["E11".into()],
            procedures: vec![],
            medications: vec![],
            vital_signs: VitalSigns {
                temperature: Some(37.0),
                heart_rate: Some(72),
                blood_pressure_systolic: None,
                blood_pressure_diastolic: None,
                respiratory_rate: None,
                oxygen_saturation: None,
                pain_level: None,
            },
            timestamp: 1700000000,
            provider_info: ProviderInfo {
                provider_id: "prov-001".into(),
                provider_name: "Dr. Smith".into(),
                facility: "General Hospital".into(),
                specialty: None,
            },
        }];

        let anonymized = anonymizer.anonymize(&records);
        assert_eq!(anonymized.len(), 1);
        assert_ne!(anonymized[0].anonymized_id, "rec-001");
    }

    #[test]
    fn test_anonymizer_privacy_level_private() {
        let anonymizer = HealthDataAnonymizer;
        let records = vec![MedicalRecord {
            record_id: "rec-001".into(),
            patient_id: "pat-001".into(),
            record_type: RecordType::LabReport,
            diagnosis_codes: vec!["E11".into(), "I10".into()],
            procedures: vec![],
            medications: vec![],
            vital_signs: VitalSigns {
                temperature: Some(37.0),
                heart_rate: None,
                blood_pressure_systolic: None,
                blood_pressure_diastolic: None,
                respiratory_rate: None,
                oxygen_saturation: None,
                pain_level: None,
            },
            timestamp: 1700000000,
            provider_info: ProviderInfo {
                provider_id: "prov-001".into(),
                provider_name: "Dr. Smith".into(),
                facility: "General Hospital".into(),
                specialty: None,
            },
        }];

        let anonymized = anonymizer.anonymize_with_privacy_level(&records, PrivacyLevel::Private);
        assert_eq!(anonymized[0].diagnosis_codes[0], "REDACTED");
        assert!(anonymized[0].vital_signs.temperature.is_none());
    }

    #[test]
    fn test_access_controller_check_allows_consented() {
        let mut controller = MedicalAccessController::new();
        assert!(controller.check_access("dr_smith", MedicalDataType::LabResults, true));
    }

    #[test]
    fn test_access_controller_check_denies_unconsented() {
        let mut controller = MedicalAccessController::new();
        assert!(!controller.check_access("dr_smith", MedicalDataType::LabResults, false));
    }

    #[test]
    fn test_access_controller_audit_denies_no_records() {
        let controller = MedicalAccessController::new();
        assert!(controller
            .audit_access("unknown", &MedicalDataType::LabResults)
            .is_err());
    }

    #[test]
    fn test_privacy_guardian_anonymize_empty_records() {
        let guardian = HealthPrivacyGuardian::new();
        assert!(guardian.anonymize_health_data(&[]).is_err());
    }

    #[test]
    fn test_privacy_guardian_consent_lifecycle() {
        let mut guardian = HealthPrivacyGuardian::new();

        assert!(!guardian
            .verify_medical_consent(1, MedicalDataType::GeneticData)
            .unwrap());

        guardian.grant_consent(1, MedicalDataType::GeneticData);
        assert!(guardian
            .verify_medical_consent(1, MedicalDataType::GeneticData)
            .unwrap());

        guardian.revoke_consent(1, MedicalDataType::GeneticData);
        assert!(!guardian
            .verify_medical_consent(1, MedicalDataType::GeneticData)
            .unwrap());
    }

    #[test]
    fn test_access_controller_get_history() {
        let mut controller = MedicalAccessController::new();
        controller.check_access("dr_jones", MedicalDataType::ImagingStudies, true);
        controller.check_access("dr_jones", MedicalDataType::GeneticData, false);

        let history = controller.get_access_history("dr_jones");
        assert_eq!(history.len(), 2);
        assert!(history[0].1);
        assert!(!history[1].1);
    }

    #[test]
    fn test_privacy_guardian_anonymize_with_level() {
        let guardian = HealthPrivacyGuardian::new();
        let records = vec![MedicalRecord {
            record_id: "rec-001".into(),
            patient_id: "pat-001".into(),
            record_type: RecordType::Consultation,
            diagnosis_codes: vec!["E11".into()],
            procedures: vec![],
            medications: vec![],
            vital_signs: VitalSigns {
                temperature: Some(36.6),
                heart_rate: None,
                blood_pressure_systolic: None,
                blood_pressure_diastolic: None,
                respiratory_rate: None,
                oxygen_saturation: None,
                pain_level: None,
            },
            timestamp: 1700000000,
            provider_info: ProviderInfo {
                provider_id: "".into(),
                provider_name: "".into(),
                facility: "".into(),
                specialty: None,
            },
        }];

        let result = guardian
            .anonymize_health_data_with_level(&records, PrivacyLevel::Medical)
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].diagnosis_codes[0], "E11");
        assert_eq!(result[0].vital_signs.temperature, Some(36.6));
    }
}
