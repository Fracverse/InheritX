use super::errors::*;
use super::types::*;
use chrono::Utc;

#[derive(Debug, Clone, Default)]
pub struct QualityOfLifeAssessor;

impl QualityOfLifeAssessor {
    pub async fn assess_activities_of_daily_living(
        &self,
        user_data: &UserActivityData,
    ) -> Result<ADLAssessment, AssessmentError> {
        if user_data.daily_activities.is_empty() {
            return Err(AssessmentError::InsufficientHistory);
        }

        let latest = user_data.daily_activities.last().unwrap();
        let bathing = if latest.bathing { 100.0 } else { 0.0 };
        let dressing = if latest.dressing { 100.0 } else { 0.0 };
        let toileting = if latest.toileting { 100.0 } else { 0.0 };
        let transferring = if latest.transferring { 100.0 } else { 0.0 };
        let feeding = if latest.feeding { 100.0 } else { 0.0 };

        let overall_adl_score = (bathing + dressing + toileting + transferring + feeding) / 5.0;

        let mut assistance_required = Vec::new();
        if bathing < 100.0 {
            assistance_required.push("Bathing".to_string());
        }
        if dressing < 100.0 {
            assistance_required.push("Dressing".to_string());
        }
        if toileting < 100.0 {
            assistance_required.push("Toileting".to_string());
        }
        if transferring < 100.0 {
            assistance_required.push("Transferring".to_string());
        }
        if feeding < 100.0 {
            assistance_required.push("Feeding".to_string());
        }

        Ok(ADLAssessment {
            user_id: user_data.user_id.clone(),
            assessed_at: user_data.recorded_at,
            bathing_independence: bathing,
            dressing_independence: dressing,
            toileting_independence: toileting,
            transferring_independence: transferring,
            feeding_independence: feeding,
            overall_adl_score,
            assistance_required,
        })
    }

    pub async fn evaluate_instrumental_activities(
        &self,
        user_data: &UserActivityData,
    ) -> Result<IADLAssessment, EvaluationError> {
        if user_data.daily_activities.is_empty() {
            return Err(EvaluationError::InsufficientData);
        }

        let latest = user_data.daily_activities.last().unwrap();
        let cooking = if latest.cooking { 100.0 } else { 0.0 };
        let cleaning = if latest.cleaning { 100.0 } else { 0.0 };
        let shopping = if latest.shopping { 100.0 } else { 0.0 };
        let medication = if latest.medication_management { 100.0 } else { 0.0 };
        let transportation = if latest.transportation { 100.0 } else { 0.0 };

        let overall_iadl_score = (cooking + cleaning + shopping + medication + transportation) / 5.0;

        let mut assistance_required = Vec::new();
        if cooking < 100.0 {
            assistance_required.push("Cooking".to_string());
        }
        if cleaning < 100.0 {
            assistance_required.push("Cleaning".to_string());
        }
        if shopping < 100.0 {
            assistance_required.push("Shopping".to_string());
        }
        if medication < 100.0 {
            assistance_required.push("Medication management".to_string());
        }
        if transportation < 100.0 {
            assistance_required.push("Transportation".to_string());
        }

        Ok(IADLAssessment {
            user_id: user_data.user_id.clone(),
            assessed_at: user_data.recorded_at,
            cooking_ability: cooking,
            cleaning_ability: cleaning,
            shopping_ability: shopping,
            medication_management_ability: medication,
            transportation_ability: transportation,
            overall_iadl_score,
            assistance_required,
        })
    }

    pub async fn calculate_quality_of_life_score(
        &self,
        adl: &ADLAssessment,
        iadl: &IADLAssessment,
        _health_data: &HealthSnapshot,
    ) -> Result<QualityOfLifeScore, CalculationError> {
        let adl_score = adl.overall_adl_score;
        let iadl_score = iadl.overall_iadl_score;
        let overall_quality_of_life = (adl_score * 0.6 + iadl_score * 0.4).clamp(0.0, 100.0);

        let domain_scores = vec![
            DomainScore {
                domain: "Activities of Daily Living".to_string(),
                score: adl_score,
                weight: 0.6,
            },
            DomainScore {
                domain: "Instrumental Activities of Daily Living".to_string(),
                score: iadl_score,
                weight: 0.4,
            },
        ];

        let mut improvement_areas = Vec::new();
        if adl_score < 80.0 {
            improvement_areas.push("ADL support".to_string());
        }
        if iadl_score < 80.0 {
            improvement_areas.push("IADL support".to_string());
        }

        Ok(QualityOfLifeScore {
            user_id: adl.user_id.clone(),
            calculated_at: adl.assessed_at,
            adl_score,
            iadl_score,
            overall_quality_of_life,
            domain_scores,
            improvement_areas,
        })
    }
}
