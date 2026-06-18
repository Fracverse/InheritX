//! Cross-chain asset data structures and enums (issue #732).
//!
//! Foundation types that let an inheritance plan describe assets held on, and
//! bridged across, multiple blockchains. These are pure data definitions plus
//! self-contained validation — they intentionally do not touch contract storage
//! so they can be reused by higher-level cross-chain inheritance logic and unit
//! tested in isolation.

use soroban_sdk::{contracterror, contracttype, Address, String, Vec};

/// Minimum length (in bytes) of an asset symbol such as `"USDC"`.
pub const MIN_ASSET_SYMBOL_LEN: u32 = 1;

/// Maximum length (in bytes) of an asset symbol. Twelve bytes comfortably fits
/// real-world tickers (`"USDC"`, `"WBTC"`, `"stETH"`) while bounding storage.
pub const MAX_ASSET_SYMBOL_LEN: u32 = 12;

/// Maximum number of distinct cross-chain assets allowed in a single plan.
pub const MAX_CROSS_CHAIN_ASSETS: u32 = 20;

/// Blockchains supported for cross-chain inheritance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupportedChain {
    Stellar,
    Ethereum,
    Bitcoin,
    Polygon,
    Arbitrum,
    BinanceSmartChain,
    Avalanche,
}

impl SupportedChain {
    /// Whether the chain is EVM-compatible (uses Ethereum-style addresses and
    /// chain IDs). Stellar and Bitcoin are not.
    pub fn is_evm_compatible(&self) -> bool {
        matches!(
            self,
            SupportedChain::Ethereum
                | SupportedChain::Polygon
                | SupportedChain::Arbitrum
                | SupportedChain::BinanceSmartChain
                | SupportedChain::Avalanche
        )
    }

    /// EVM chain ID for the network, or `0` for non-EVM chains (Stellar,
    /// Bitcoin) which have no EVM chain ID. Useful for bridge routing.
    pub fn evm_chain_id(&self) -> u32 {
        match self {
            SupportedChain::Ethereum => 1,
            SupportedChain::BinanceSmartChain => 56,
            SupportedChain::Polygon => 137,
            SupportedChain::Avalanche => 43114,
            SupportedChain::Arbitrum => 42161,
            SupportedChain::Stellar | SupportedChain::Bitcoin => 0,
        }
    }
}

/// Bridge protocols usable to move assets between chains.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeProtocol {
    Allbridge,
    Wormhole,
    LayerZero,
    ChainlinkCCIP,
}

/// A single asset held on a specific chain and bridgeable via a given protocol.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainAsset {
    /// Chain the asset lives on.
    pub chain: SupportedChain,
    /// Asset contract address (token contract / SAC) on its chain.
    pub contract_address: Address,
    /// Amount expressed in the asset's smallest unit.
    pub amount: u128,
    /// Human-readable ticker, e.g. `"USDC"`.
    pub asset_symbol: String,
    /// Bridge protocol used to move the asset cross-chain.
    pub bridge_protocol: BridgeProtocol,
}

impl CrossChainAsset {
    /// Validate the asset's self-contained invariants: a non-zero amount and an
    /// asset symbol within the permitted length bounds.
    pub fn validate(&self) -> Result<(), CrossChainError> {
        if self.amount == 0 {
            return Err(CrossChainError::ZeroAmount);
        }
        let symbol_len = self.asset_symbol.len();
        if symbol_len < MIN_ASSET_SYMBOL_LEN {
            return Err(CrossChainError::EmptyAssetSymbol);
        }
        if symbol_len > MAX_ASSET_SYMBOL_LEN {
            return Err(CrossChainError::AssetSymbolTooLong);
        }
        Ok(())
    }
}

/// A multi-chain inheritance plan: a set of cross-chain assets owned by a single
/// account, anchored to a primary chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainInheritancePlan {
    /// Identifier of the underlying inheritance plan.
    pub plan_id: u64,
    /// Owner of the plan.
    pub owner: Address,
    /// Chain the plan is primarily anchored to (where it is administered).
    pub primary_chain: SupportedChain,
    /// Assets across all supported chains.
    pub assets: Vec<CrossChainAsset>,
    /// Creation ledger timestamp.
    pub created_at: u64,
    /// Whether the plan is active.
    pub is_active: bool,
}

impl CrossChainInheritancePlan {
    /// Validate the plan: it must hold at least one asset, no more than
    /// [`MAX_CROSS_CHAIN_ASSETS`], and every asset must itself be valid.
    pub fn validate(&self) -> Result<(), CrossChainError> {
        if self.assets.is_empty() {
            return Err(CrossChainError::NoAssets);
        }
        if self.assets.len() > MAX_CROSS_CHAIN_ASSETS {
            return Err(CrossChainError::TooManyAssets);
        }
        for asset in self.assets.iter() {
            asset.validate()?;
        }
        Ok(())
    }
}

/// Errors raised when validating cross-chain data structures.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossChainError {
    /// Asset amount is zero.
    ZeroAmount = 1,
    /// Asset symbol is empty.
    EmptyAssetSymbol = 2,
    /// Asset symbol exceeds [`MAX_ASSET_SYMBOL_LEN`].
    AssetSymbolTooLong = 3,
    /// Plan contains no assets.
    NoAssets = 4,
    /// Plan exceeds [`MAX_CROSS_CHAIN_ASSETS`].
    TooManyAssets = 5,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String, Vec};

    fn asset(env: &Env, chain: SupportedChain, amount: u128, symbol: &str) -> CrossChainAsset {
        CrossChainAsset {
            chain,
            contract_address: Address::generate(env),
            amount,
            asset_symbol: String::from_str(env, symbol),
            bridge_protocol: BridgeProtocol::Allbridge,
        }
    }

    // --- enum comparisons & conversions ---

    #[test]
    fn supported_chain_equality_and_inequality() {
        assert_eq!(SupportedChain::Ethereum, SupportedChain::Ethereum);
        assert_ne!(SupportedChain::Ethereum, SupportedChain::Polygon);
    }

    #[test]
    fn is_evm_compatible_classifies_chains() {
        assert!(SupportedChain::Ethereum.is_evm_compatible());
        assert!(SupportedChain::Polygon.is_evm_compatible());
        assert!(SupportedChain::Arbitrum.is_evm_compatible());
        assert!(SupportedChain::BinanceSmartChain.is_evm_compatible());
        assert!(SupportedChain::Avalanche.is_evm_compatible());
        assert!(!SupportedChain::Stellar.is_evm_compatible());
        assert!(!SupportedChain::Bitcoin.is_evm_compatible());
    }

    #[test]
    fn evm_chain_ids_are_correct() {
        assert_eq!(SupportedChain::Ethereum.evm_chain_id(), 1);
        assert_eq!(SupportedChain::BinanceSmartChain.evm_chain_id(), 56);
        assert_eq!(SupportedChain::Polygon.evm_chain_id(), 137);
        assert_eq!(SupportedChain::Arbitrum.evm_chain_id(), 42161);
        assert_eq!(SupportedChain::Avalanche.evm_chain_id(), 43114);
        // Non-EVM chains report 0.
        assert_eq!(SupportedChain::Stellar.evm_chain_id(), 0);
        assert_eq!(SupportedChain::Bitcoin.evm_chain_id(), 0);
    }

    #[test]
    fn bridge_protocol_equality() {
        assert_eq!(BridgeProtocol::Wormhole, BridgeProtocol::Wormhole);
        assert_ne!(BridgeProtocol::Wormhole, BridgeProtocol::LayerZero);
    }

    // --- CrossChainAsset validation ---

    #[test]
    fn valid_asset_passes_validation() {
        let env = Env::default();
        let a = asset(&env, SupportedChain::Ethereum, 1_000, "USDC");
        assert_eq!(a.validate(), Ok(()));
    }

    #[test]
    fn zero_amount_asset_is_rejected() {
        let env = Env::default();
        let a = asset(&env, SupportedChain::Ethereum, 0, "USDC");
        assert_eq!(a.validate(), Err(CrossChainError::ZeroAmount));
    }

    #[test]
    fn empty_symbol_asset_is_rejected() {
        let env = Env::default();
        let a = asset(&env, SupportedChain::Ethereum, 1_000, "");
        assert_eq!(a.validate(), Err(CrossChainError::EmptyAssetSymbol));
    }

    #[test]
    fn overlong_symbol_asset_is_rejected() {
        let env = Env::default();
        // 13 bytes, one over MAX_ASSET_SYMBOL_LEN (12).
        let a = asset(&env, SupportedChain::Ethereum, 1_000, "ABCDEFGHIJKLM");
        assert_eq!(a.validate(), Err(CrossChainError::AssetSymbolTooLong));
    }

    #[test]
    fn symbol_at_max_length_is_accepted() {
        let env = Env::default();
        // Exactly 12 bytes.
        let a = asset(&env, SupportedChain::Ethereum, 1_000, "ABCDEFGHIJKL");
        assert_eq!(a.validate(), Ok(()));
    }

    // --- CrossChainInheritancePlan validation ---

    fn plan(env: &Env, assets: Vec<CrossChainAsset>) -> CrossChainInheritancePlan {
        CrossChainInheritancePlan {
            plan_id: 1,
            owner: Address::generate(env),
            primary_chain: SupportedChain::Stellar,
            assets,
            created_at: 0,
            is_active: true,
        }
    }

    #[test]
    fn plan_with_valid_assets_passes() {
        let env = Env::default();
        let mut assets = Vec::new(&env);
        assets.push_back(asset(&env, SupportedChain::Ethereum, 1_000, "USDC"));
        assets.push_back(asset(&env, SupportedChain::Bitcoin, 5, "BTC"));
        assert_eq!(plan(&env, assets).validate(), Ok(()));
    }

    #[test]
    fn empty_plan_is_rejected() {
        let env = Env::default();
        let assets = Vec::new(&env);
        assert_eq!(
            plan(&env, assets).validate(),
            Err(CrossChainError::NoAssets)
        );
    }

    #[test]
    fn plan_propagates_invalid_asset_error() {
        let env = Env::default();
        let mut assets = Vec::new(&env);
        assets.push_back(asset(&env, SupportedChain::Ethereum, 0, "USDC")); // zero amount
        assert_eq!(
            plan(&env, assets).validate(),
            Err(CrossChainError::ZeroAmount)
        );
    }

    #[test]
    fn plan_exceeding_asset_cap_is_rejected() {
        let env = Env::default();
        let mut assets = Vec::new(&env);
        for _ in 0..(MAX_CROSS_CHAIN_ASSETS + 1) {
            assets.push_back(asset(&env, SupportedChain::Ethereum, 1, "USDC"));
        }
        assert_eq!(
            plan(&env, assets).validate(),
            Err(CrossChainError::TooManyAssets)
        );
    }

    // --- clone / equality (serialization round-trip surface) ---

    #[test]
    fn asset_clone_equals_original() {
        let env = Env::default();
        let a = asset(&env, SupportedChain::Polygon, 42, "WETH");
        assert_eq!(a.clone(), a);
    }
}
