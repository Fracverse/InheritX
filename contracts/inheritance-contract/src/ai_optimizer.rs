//! AI Estate Optimizer data structures (issue #738).
//!
//! Core data structures for AI-powered estate optimization including risk
//! profiles, market data, beneficiary profiles, optimization recommendations,
//! and optimizer configuration. These are pure data definitions with
//! self-contained validation — they do not touch contract storage so they can
//! be reused by higher-level AI optimization logic and unit tested in isolation.

use soroban_sdk::{contracttype, String, Vec};

use crate::cross_chain::SupportedChain;

// ── Risk Profile ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskProfile {
    pub level: RiskLevel,
    pub risk_tolerance: u32,
    pub time_horizon: u64,
    pub volatility_preference: u32,
    pub liquidity_needs: u32,
}

// ── Market Data ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketData {
    pub asset_symbol: String,
    pub current_price: u64,
    pub price_change_24h: i32,
    pub volatility_30d: u32,
    pub market_cap: u64,
    pub liquidity_score: u32,
    pub last_updated: u64,
}

// ── Beneficiary Profile ─────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinancialSituation {
    Student,
    EarlyCareer,
    Established,
    Retired,
    Dependent,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeedType {
    Education,
    Healthcare,
    Housing,
    Emergency,
    Investment,
    Business,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Requirement {
    Accessibility,
    LegalGuardianship,
    TrustStructure,
    TaxExemption,
    RegularDistributions,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeneficiaryProfile {
    pub beneficiary_id: u32,
    pub age: u32,
    pub financial_situation: FinancialSituation,
    pub needs_priority: Vec<NeedType>,
    pub risk_capacity: RiskLevel,
    pub geographic_location: String,
    pub special_requirements: Vec<Requirement>,
}

// ── Optimization Configuration ──────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptimizationGoal {
    MaximizeValue,
    MinimizeRisk,
    BalanceGrowthStability,
    TaxOptimization,
    LiquidityPreservation,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebalancingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AIEstateOptimizer {
    pub optimization_goals: Vec<OptimizationGoal>,
    pub rebalancing_frequency: RebalancingFrequency,
    pub tax_optimization_enabled: bool,
    pub market_conditions_weight: u32,
    pub beneficiary_needs_weight: u32,
    pub risk_adjustment_weight: u32,
}

// ── Optimization Recommendations ────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetAllocation {
    pub asset_symbol: String,
    pub chain: SupportedChain,
    pub recommended_percentage: u32,
    pub current_percentage: u32,
    pub adjustment_reason: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationRecommendation {
    pub plan_id: u64,
    pub recommended_allocations: Vec<AssetAllocation>,
    pub confidence_score: u32,
    pub expected_return: u32,
    pub risk_score: u32,
    pub reasoning: String,
    pub generated_at: u64,
}

// ── Validation ──────────────────────────────────────────────────────────────

const MAX_SCALE: u32 = 100;
const MAX_BASIS_POINTS: u32 = 10000;
const MAX_WEIGHT: u32 = 100;

impl RiskProfile {
    pub fn validate(&self) -> bool {
        self.risk_tolerance <= MAX_SCALE
            && self.volatility_preference <= MAX_SCALE
            && self.liquidity_needs <= MAX_SCALE
            && self.time_horizon > 0
    }
}

impl MarketData {
    pub fn validate(&self) -> bool {
        self.liquidity_score <= MAX_SCALE && self.asset_symbol.len() > 0
    }
}

impl BeneficiaryProfile {
    pub fn validate(&self) -> bool {
        self.geographic_location.len() > 0
    }
}

impl AIEstateOptimizer {
    pub fn validate(&self) -> bool {
        let total_weight = (self.market_conditions_weight as u64)
            + (self.beneficiary_needs_weight as u64)
            + (self.risk_adjustment_weight as u64);
        !self.optimization_goals.is_empty()
            && self.market_conditions_weight <= MAX_WEIGHT
            && self.beneficiary_needs_weight <= MAX_WEIGHT
            && self.risk_adjustment_weight <= MAX_WEIGHT
            && total_weight <= MAX_WEIGHT as u64
    }
}

impl AssetAllocation {
    pub fn validate(&self) -> bool {
        self.recommended_percentage <= MAX_BASIS_POINTS
            && self.current_percentage <= MAX_BASIS_POINTS
            && self.asset_symbol.len() > 0
    }
}

impl OptimizationRecommendation {
    pub fn validate(&self) -> bool {
        if self.confidence_score > MAX_SCALE || self.risk_score > MAX_SCALE {
            return false;
        }
        if self.recommended_allocations.is_empty() {
            return false;
        }
        let mut total_bp: u32 = 0;
        for alloc in self.recommended_allocations.iter() {
            if !alloc.validate() {
                return false;
            }
            total_bp = match total_bp.checked_add(alloc.recommended_percentage) {
                Some(v) => v,
                None => return false,
            };
        }
        total_bp <= MAX_BASIS_POINTS
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, String, Vec};

    // ── RiskProfile ─────────────────────────────────────────────────────

    #[test]
    fn risk_profile_valid() {
        let profile = RiskProfile {
            level: RiskLevel::Moderate,
            risk_tolerance: 50,
            time_horizon: 10,
            volatility_preference: 30,
            liquidity_needs: 40,
        };
        assert!(profile.validate());
    }

    #[test]
    fn risk_profile_tolerance_exceeds_scale() {
        let profile = RiskProfile {
            level: RiskLevel::Aggressive,
            risk_tolerance: 101,
            time_horizon: 5,
            volatility_preference: 50,
            liquidity_needs: 50,
        };
        assert!(!profile.validate());
    }

    #[test]
    fn risk_profile_zero_time_horizon() {
        let profile = RiskProfile {
            level: RiskLevel::Conservative,
            risk_tolerance: 20,
            time_horizon: 0,
            volatility_preference: 10,
            liquidity_needs: 10,
        };
        assert!(!profile.validate());
    }

    #[test]
    fn risk_profile_volatility_exceeds_scale() {
        let profile = RiskProfile {
            level: RiskLevel::Custom,
            risk_tolerance: 50,
            time_horizon: 5,
            volatility_preference: 101,
            liquidity_needs: 50,
        };
        assert!(!profile.validate());
    }

    #[test]
    fn risk_profile_liquidity_exceeds_scale() {
        let profile = RiskProfile {
            level: RiskLevel::Moderate,
            risk_tolerance: 50,
            time_horizon: 5,
            volatility_preference: 50,
            liquidity_needs: 101,
        };
        assert!(!profile.validate());
    }

    #[test]
    fn risk_profile_boundary_values() {
        let profile = RiskProfile {
            level: RiskLevel::Moderate,
            risk_tolerance: 100,
            time_horizon: 1,
            volatility_preference: 100,
            liquidity_needs: 100,
        };
        assert!(profile.validate());
    }

    #[test]
    fn risk_level_equality() {
        assert_eq!(RiskLevel::Conservative, RiskLevel::Conservative);
        assert_ne!(RiskLevel::Conservative, RiskLevel::Aggressive);
    }

    // ── MarketData ──────────────────────────────────────────────────────

    #[test]
    fn market_data_valid() {
        let env = Env::default();
        let data = MarketData {
            asset_symbol: String::from_str(&env, "USDC"),
            current_price: 1_000_000,
            price_change_24h: -50,
            volatility_30d: 200,
            market_cap: 30_000_000_000,
            liquidity_score: 95,
            last_updated: 1_700_000_000,
        };
        assert!(data.validate());
    }

    #[test]
    fn market_data_liquidity_exceeds_scale() {
        let env = Env::default();
        let data = MarketData {
            asset_symbol: String::from_str(&env, "BTC"),
            current_price: 50_000_000,
            price_change_24h: 100,
            volatility_30d: 500,
            market_cap: 1_000_000_000_000,
            liquidity_score: 101,
            last_updated: 1_700_000_000,
        };
        assert!(!data.validate());
    }

    #[test]
    fn market_data_empty_symbol() {
        let env = Env::default();
        let data = MarketData {
            asset_symbol: String::from_str(&env, ""),
            current_price: 1_000,
            price_change_24h: 0,
            volatility_30d: 100,
            market_cap: 1_000_000,
            liquidity_score: 50,
            last_updated: 1_700_000_000,
        };
        assert!(!data.validate());
    }

    #[test]
    fn market_data_negative_price_change() {
        let env = Env::default();
        let data = MarketData {
            asset_symbol: String::from_str(&env, "ETH"),
            current_price: 2_000_000,
            price_change_24h: -500,
            volatility_30d: 300,
            market_cap: 250_000_000_000,
            liquidity_score: 90,
            last_updated: 1_700_000_000,
        };
        assert!(data.validate());
    }

    // ── BeneficiaryProfile ──────────────────────────────────────────────

    #[test]
    fn beneficiary_profile_valid() {
        let env = Env::default();
        let mut needs = Vec::new(&env);
        needs.push_back(NeedType::Education);
        needs.push_back(NeedType::Healthcare);

        let mut reqs = Vec::new(&env);
        reqs.push_back(Requirement::TrustStructure);

        let profile = BeneficiaryProfile {
            beneficiary_id: 1,
            age: 25,
            financial_situation: FinancialSituation::EarlyCareer,
            needs_priority: needs,
            risk_capacity: RiskLevel::Moderate,
            geographic_location: String::from_str(&env, "US"),
            special_requirements: reqs,
        };
        assert!(profile.validate());
    }

    #[test]
    fn beneficiary_profile_empty_location() {
        let env = Env::default();
        let needs = Vec::new(&env);
        let reqs = Vec::new(&env);

        let profile = BeneficiaryProfile {
            beneficiary_id: 1,
            age: 30,
            financial_situation: FinancialSituation::Established,
            needs_priority: needs,
            risk_capacity: RiskLevel::Conservative,
            geographic_location: String::from_str(&env, ""),
            special_requirements: reqs,
        };
        assert!(!profile.validate());
    }

    #[test]
    fn financial_situation_equality() {
        assert_eq!(FinancialSituation::Student, FinancialSituation::Student);
        assert_ne!(FinancialSituation::Student, FinancialSituation::Retired);
    }

    #[test]
    fn need_type_equality() {
        assert_eq!(NeedType::Education, NeedType::Education);
        assert_ne!(NeedType::Education, NeedType::Housing);
    }

    // ── AIEstateOptimizer ───────────────────────────────────────────────

    #[test]
    fn optimizer_config_valid() {
        let env = Env::default();
        let mut goals = Vec::new(&env);
        goals.push_back(OptimizationGoal::MaximizeValue);
        goals.push_back(OptimizationGoal::TaxOptimization);

        let config = AIEstateOptimizer {
            optimization_goals: goals,
            rebalancing_frequency: RebalancingFrequency::Monthly,
            tax_optimization_enabled: true,
            market_conditions_weight: 40,
            beneficiary_needs_weight: 30,
            risk_adjustment_weight: 30,
        };
        assert!(config.validate());
    }

    #[test]
    fn optimizer_config_empty_goals() {
        let env = Env::default();
        let goals = Vec::new(&env);

        let config = AIEstateOptimizer {
            optimization_goals: goals,
            rebalancing_frequency: RebalancingFrequency::Weekly,
            tax_optimization_enabled: false,
            market_conditions_weight: 33,
            beneficiary_needs_weight: 33,
            risk_adjustment_weight: 34,
        };
        assert!(!config.validate());
    }

    #[test]
    fn optimizer_config_weights_exceed_limit() {
        let env = Env::default();
        let mut goals = Vec::new(&env);
        goals.push_back(OptimizationGoal::MinimizeRisk);

        let config = AIEstateOptimizer {
            optimization_goals: goals,
            rebalancing_frequency: RebalancingFrequency::Quarterly,
            tax_optimization_enabled: true,
            market_conditions_weight: 50,
            beneficiary_needs_weight: 30,
            risk_adjustment_weight: 30,
        };
        assert!(!config.validate());
    }

    #[test]
    fn optimizer_config_individual_weight_exceeds_max() {
        let env = Env::default();
        let mut goals = Vec::new(&env);
        goals.push_back(OptimizationGoal::BalanceGrowthStability);

        let config = AIEstateOptimizer {
            optimization_goals: goals,
            rebalancing_frequency: RebalancingFrequency::Annually,
            tax_optimization_enabled: false,
            market_conditions_weight: 101,
            beneficiary_needs_weight: 0,
            risk_adjustment_weight: 0,
        };
        assert!(!config.validate());
    }

    #[test]
    fn optimization_goal_equality() {
        assert_eq!(
            OptimizationGoal::MaximizeValue,
            OptimizationGoal::MaximizeValue
        );
        assert_ne!(
            OptimizationGoal::MaximizeValue,
            OptimizationGoal::MinimizeRisk
        );
    }

    #[test]
    fn rebalancing_frequency_equality() {
        assert_eq!(RebalancingFrequency::Daily, RebalancingFrequency::Daily);
        assert_ne!(RebalancingFrequency::Daily, RebalancingFrequency::Monthly);
    }

    // ── AssetAllocation ─────────────────────────────────────────────────

    #[test]
    fn asset_allocation_valid() {
        let env = Env::default();
        let alloc = AssetAllocation {
            asset_symbol: String::from_str(&env, "USDC"),
            chain: SupportedChain::Stellar,
            recommended_percentage: 5000,
            current_percentage: 4000,
            adjustment_reason: String::from_str(&env, "Increase stablecoin exposure"),
        };
        assert!(alloc.validate());
    }

    #[test]
    fn asset_allocation_percentage_exceeds_basis_points() {
        let env = Env::default();
        let alloc = AssetAllocation {
            asset_symbol: String::from_str(&env, "ETH"),
            chain: SupportedChain::Ethereum,
            recommended_percentage: 10001,
            current_percentage: 3000,
            adjustment_reason: String::from_str(&env, "Overweight"),
        };
        assert!(!alloc.validate());
    }

    #[test]
    fn asset_allocation_empty_symbol() {
        let env = Env::default();
        let alloc = AssetAllocation {
            asset_symbol: String::from_str(&env, ""),
            chain: SupportedChain::Polygon,
            recommended_percentage: 2000,
            current_percentage: 2000,
            adjustment_reason: String::from_str(&env, "No change"),
        };
        assert!(!alloc.validate());
    }

    // ── OptimizationRecommendation ──────────────────────────────────────

    fn make_recommendation(
        env: &Env,
        allocations: Vec<AssetAllocation>,
    ) -> OptimizationRecommendation {
        OptimizationRecommendation {
            plan_id: 1,
            recommended_allocations: allocations,
            confidence_score: 85,
            expected_return: 750,
            risk_score: 40,
            reasoning: String::from_str(env, "Diversified portfolio"),
            generated_at: 1_700_000_000,
        }
    }

    #[test]
    fn recommendation_valid() {
        let env = Env::default();
        let mut allocations = Vec::new(&env);
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "USDC"),
            chain: SupportedChain::Stellar,
            recommended_percentage: 6000,
            current_percentage: 5000,
            adjustment_reason: String::from_str(&env, "Increase stability"),
        });
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "ETH"),
            chain: SupportedChain::Ethereum,
            recommended_percentage: 4000,
            current_percentage: 5000,
            adjustment_reason: String::from_str(&env, "Reduce volatility"),
        });

        let rec = make_recommendation(&env, allocations);
        assert!(rec.validate());
    }

    #[test]
    fn recommendation_empty_allocations() {
        let env = Env::default();
        let allocations = Vec::new(&env);
        let rec = make_recommendation(&env, allocations);
        assert!(!rec.validate());
    }

    #[test]
    fn recommendation_confidence_exceeds_scale() {
        let env = Env::default();
        let mut allocations = Vec::new(&env);
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "USDC"),
            chain: SupportedChain::Stellar,
            recommended_percentage: 10000,
            current_percentage: 10000,
            adjustment_reason: String::from_str(&env, "Hold"),
        });

        let rec = OptimizationRecommendation {
            plan_id: 1,
            recommended_allocations: allocations,
            confidence_score: 101,
            expected_return: 500,
            risk_score: 30,
            reasoning: String::from_str(&env, "Test"),
            generated_at: 1_700_000_000,
        };
        assert!(!rec.validate());
    }

    #[test]
    fn recommendation_risk_score_exceeds_scale() {
        let env = Env::default();
        let mut allocations = Vec::new(&env);
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "BTC"),
            chain: SupportedChain::Bitcoin,
            recommended_percentage: 10000,
            current_percentage: 10000,
            adjustment_reason: String::from_str(&env, "Hold"),
        });

        let rec = OptimizationRecommendation {
            plan_id: 1,
            recommended_allocations: allocations,
            confidence_score: 80,
            expected_return: 500,
            risk_score: 101,
            reasoning: String::from_str(&env, "Test"),
            generated_at: 1_700_000_000,
        };
        assert!(!rec.validate());
    }

    #[test]
    fn recommendation_allocations_exceed_total() {
        let env = Env::default();
        let mut allocations = Vec::new(&env);
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "USDC"),
            chain: SupportedChain::Stellar,
            recommended_percentage: 6000,
            current_percentage: 5000,
            adjustment_reason: String::from_str(&env, "Increase"),
        });
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "ETH"),
            chain: SupportedChain::Ethereum,
            recommended_percentage: 5000,
            current_percentage: 5000,
            adjustment_reason: String::from_str(&env, "Hold"),
        });

        let rec = make_recommendation(&env, allocations);
        assert!(!rec.validate());
    }

    #[test]
    fn recommendation_with_invalid_allocation() {
        let env = Env::default();
        let mut allocations = Vec::new(&env);
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, ""),
            chain: SupportedChain::Stellar,
            recommended_percentage: 10000,
            current_percentage: 10000,
            adjustment_reason: String::from_str(&env, "Invalid"),
        });

        let rec = make_recommendation(&env, allocations);
        assert!(!rec.validate());
    }

    // ── Clone / Equality ────────────────────────────────────────────────

    #[test]
    fn risk_profile_clone_equals_original() {
        let profile = RiskProfile {
            level: RiskLevel::Moderate,
            risk_tolerance: 50,
            time_horizon: 10,
            volatility_preference: 30,
            liquidity_needs: 40,
        };
        assert_eq!(profile.clone(), profile);
    }

    #[test]
    fn market_data_clone_equals_original() {
        let env = Env::default();
        let data = MarketData {
            asset_symbol: String::from_str(&env, "USDC"),
            current_price: 1_000_000,
            price_change_24h: -50,
            volatility_30d: 200,
            market_cap: 30_000_000_000,
            liquidity_score: 95,
            last_updated: 1_700_000_000,
        };
        assert_eq!(data.clone(), data);
    }

    #[test]
    fn optimizer_config_clone_equals_original() {
        let env = Env::default();
        let mut goals = Vec::new(&env);
        goals.push_back(OptimizationGoal::MaximizeValue);

        let config = AIEstateOptimizer {
            optimization_goals: goals,
            rebalancing_frequency: RebalancingFrequency::Monthly,
            tax_optimization_enabled: true,
            market_conditions_weight: 40,
            beneficiary_needs_weight: 30,
            risk_adjustment_weight: 30,
        };
        assert_eq!(config.clone(), config);
    }

    #[test]
    fn recommendation_clone_equals_original() {
        let env = Env::default();
        let mut allocations = Vec::new(&env);
        allocations.push_back(AssetAllocation {
            asset_symbol: String::from_str(&env, "USDC"),
            chain: SupportedChain::Stellar,
            recommended_percentage: 10000,
            current_percentage: 10000,
            adjustment_reason: String::from_str(&env, "Hold"),
        });
        let rec = make_recommendation(&env, allocations);
        assert_eq!(rec.clone(), rec);
    }
}
