//! Centralized per-asset metadata for the lending and risk subsystems.
//!
//! Token amounts are persisted at a flat `NUMERIC(_, 8)` scale, but each asset
//! has its own native on-chain precision and its own risk profile. Scattering
//! those facts across `match` arms in the risk engine, liquidation bot, and
//! collateral valuation invites drift: one site learns about a new token while
//! another silently treats it as unknown.
//!
//! This module is the single source of truth. Look an asset up once and read
//! its decimals, liquidation threshold, and classification from one place.
//!
//! Lookups are case-insensitive and alias-aware: wrapped representations such as
//! `WETH` / `WBTC` and chain-qualified symbols such as `STELLAR_XLM` resolve to
//! their canonical asset (`ETH`, `BTC`, `XLM`) and share its metadata.

use rust_decimal::Decimal;

/// Decimal precision assumed for assets not present in the [`REGISTRY`]. This
/// mirrors the `NUMERIC(_, 8)` storage scale used for token amounts, so an
/// unrecognized asset is valued at its stored precision rather than being
/// rejected.
pub const DEFAULT_DECIMALS: u32 = 8;

/// Scale used to express a liquidation threshold as an integral basis-point
/// count (e.g. `9500` => `0.9500`). Keeping the table integral lets the registry
/// stay a `const` without const-`Decimal` construction.
const THRESHOLD_SCALE: u32 = 4;

/// Public, read-only view of an asset's metadata, resolved through aliases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMetadata {
    /// Canonical uppercase symbol (e.g. `ETH`, even when looked up as `weth`).
    pub symbol: &'static str,
    /// Native on-chain decimal precision.
    pub decimals: u32,
    /// Liquidation threshold as a fraction in `(0, 1]`, or `None` when the asset
    /// defers to the engine-wide fallback threshold.
    pub liquidation_threshold: Option<Decimal>,
    /// Whether the asset is a fiat-pegged stablecoin.
    pub is_stablecoin: bool,
}

/// Internal registry row. `aliases` are alternate codes that resolve to
/// `symbol`; matching is case-insensitive.
struct AssetEntry {
    symbol: &'static str,
    aliases: &'static [&'static str],
    decimals: u32,
    liquidation_threshold_bps: Option<u32>,
    is_stablecoin: bool,
}

/// The supported-asset table. Add new assets here and every consumer of this
/// module picks them up automatically.
const REGISTRY: &[AssetEntry] = &[
    AssetEntry {
        symbol: "USDC",
        aliases: &[],
        decimals: 6,
        liquidation_threshold_bps: Some(9500), // 0.95
        is_stablecoin: true,
    },
    AssetEntry {
        // Supported for valuation precision; liquidation threshold defers to the
        // engine-wide fallback (preserving prior risk-engine behavior).
        symbol: "USDT",
        aliases: &[],
        decimals: 6,
        liquidation_threshold_bps: None,
        is_stablecoin: true,
    },
    AssetEntry {
        symbol: "XLM",
        aliases: &["STELLAR_XLM"],
        decimals: 7,
        liquidation_threshold_bps: Some(8000), // 0.80
        is_stablecoin: false,
    },
    AssetEntry {
        symbol: "BTC",
        aliases: &["WBTC"],
        decimals: 8,
        liquidation_threshold_bps: Some(8500), // 0.85
        is_stablecoin: false,
    },
    AssetEntry {
        symbol: "ETH",
        aliases: &["WETH"],
        decimals: 18,
        liquidation_threshold_bps: Some(8500), // 0.85
        is_stablecoin: false,
    },
];

/// Convert an integral basis-point threshold into a fractional [`Decimal`].
fn bps_to_decimal(bps: u32) -> Decimal {
    Decimal::new(bps as i64, THRESHOLD_SCALE)
}

/// Find the registry row whose canonical symbol or aliases match `asset_code`,
/// case-insensitively.
fn find_entry(asset_code: &str) -> Option<&'static AssetEntry> {
    let upper = asset_code.to_uppercase();
    REGISTRY.iter().find(|entry| {
        entry.symbol == upper || entry.aliases.iter().any(|a| a.to_uppercase() == upper)
    })
}

/// Resolve full [`TokenMetadata`] for an asset code, or `None` if unsupported.
pub fn lookup(asset_code: &str) -> Option<TokenMetadata> {
    find_entry(asset_code).map(|entry| TokenMetadata {
        symbol: entry.symbol,
        decimals: entry.decimals,
        liquidation_threshold: entry.liquidation_threshold_bps.map(bps_to_decimal),
        is_stablecoin: entry.is_stablecoin,
    })
}

/// Native decimal precision for an asset, falling back to [`DEFAULT_DECIMALS`]
/// for unknown assets.
pub fn decimals_for(asset_code: &str) -> u32 {
    find_entry(asset_code)
        .map(|entry| entry.decimals)
        .unwrap_or(DEFAULT_DECIMALS)
}

/// Liquidation threshold for an asset, using `fallback` when the asset is
/// unknown or defers its threshold to the engine-wide value.
pub fn liquidation_threshold_for(asset_code: &str, fallback: Decimal) -> Decimal {
    find_entry(asset_code)
        .and_then(|entry| entry.liquidation_threshold_bps)
        .map(bps_to_decimal)
        .unwrap_or(fallback)
}

/// Canonical uppercase symbol for an asset code (resolving aliases), or `None`.
pub fn canonical_symbol(asset_code: &str) -> Option<&'static str> {
    find_entry(asset_code).map(|entry| entry.symbol)
}

/// Whether the asset is a known fiat-pegged stablecoin.
pub fn is_stablecoin(asset_code: &str) -> bool {
    find_entry(asset_code).is_some_and(|entry| entry.is_stablecoin)
}

/// Whether the asset appears in the registry (directly or via an alias).
pub fn is_supported(asset_code: &str) -> bool {
    find_entry(asset_code).is_some()
}

/// All canonical symbols known to the registry.
pub fn supported_symbols() -> Vec<&'static str> {
    REGISTRY.iter().map(|entry| entry.symbol).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // --- lookup ---

    #[test]
    fn lookup_known_asset_returns_metadata() {
        let usdc = lookup("USDC").expect("USDC is a supported asset");
        assert_eq!(usdc.symbol, "USDC");
        assert_eq!(usdc.decimals, 6);
        assert_eq!(usdc.liquidation_threshold, Some(dec!(0.95)));
        assert!(usdc.is_stablecoin);
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert_eq!(lookup("usdc").unwrap().symbol, "USDC");
        assert_eq!(lookup("UsDc").unwrap().symbol, "USDC");
        assert_eq!(lookup("eth").unwrap().symbol, "ETH");
    }

    #[test]
    fn lookup_resolves_wrapped_aliases_to_canonical_symbol() {
        // Wrapped variants share the canonical asset's metadata.
        let weth = lookup("WETH").unwrap();
        assert_eq!(weth.symbol, "ETH");
        assert_eq!(weth.decimals, 18);

        let wbtc = lookup("WBTC").unwrap();
        assert_eq!(wbtc.symbol, "BTC");
        assert_eq!(wbtc.decimals, 8);

        let xlm = lookup("STELLAR_XLM").unwrap();
        assert_eq!(xlm.symbol, "XLM");
        assert_eq!(xlm.decimals, 7);
    }

    #[test]
    fn lookup_unknown_asset_returns_none() {
        assert!(lookup("DOGE").is_none());
        assert!(lookup("").is_none());
    }

    // --- decimals_for ---

    #[test]
    fn decimals_for_known_assets() {
        assert_eq!(decimals_for("USDC"), 6);
        assert_eq!(decimals_for("USDT"), 6);
        assert_eq!(decimals_for("XLM"), 7);
        assert_eq!(decimals_for("BTC"), 8);
        assert_eq!(decimals_for("ETH"), 18);
        assert_eq!(decimals_for("WETH"), 18);
    }

    #[test]
    fn decimals_for_unknown_returns_storage_default() {
        assert_eq!(decimals_for("DOGE"), DEFAULT_DECIMALS);
        assert_eq!(DEFAULT_DECIMALS, 8);
    }

    // --- liquidation_threshold_for ---

    #[test]
    fn liquidation_threshold_for_known_assets() {
        let fallback = dec!(0.90);
        assert_eq!(liquidation_threshold_for("USDC", fallback), dec!(0.95));
        assert_eq!(liquidation_threshold_for("ETH", fallback), dec!(0.85));
        assert_eq!(liquidation_threshold_for("WBTC", fallback), dec!(0.85));
        assert_eq!(liquidation_threshold_for("XLM", fallback), dec!(0.80));
    }

    #[test]
    fn liquidation_threshold_for_asset_without_one_uses_fallback() {
        // USDT is supported for decimals but defers its liquidation threshold to
        // the engine-wide fallback.
        let fallback = dec!(0.88);
        assert_eq!(liquidation_threshold_for("USDT", fallback), fallback);
    }

    #[test]
    fn liquidation_threshold_for_unknown_uses_fallback() {
        let fallback = dec!(0.77);
        assert_eq!(liquidation_threshold_for("DOGE", fallback), fallback);
    }

    // --- canonical_symbol ---

    #[test]
    fn canonical_symbol_resolves_aliases() {
        assert_eq!(canonical_symbol("weth"), Some("ETH"));
        assert_eq!(canonical_symbol("WBTC"), Some("BTC"));
        assert_eq!(canonical_symbol("usdc"), Some("USDC"));
    }

    #[test]
    fn canonical_symbol_unknown_is_none() {
        assert_eq!(canonical_symbol("DOGE"), None);
    }

    // --- is_stablecoin / is_supported ---

    #[test]
    fn is_stablecoin_classifies_correctly() {
        assert!(is_stablecoin("USDC"));
        assert!(is_stablecoin("usdt"));
        assert!(!is_stablecoin("ETH"));
        assert!(!is_stablecoin("DOGE"));
    }

    #[test]
    fn is_supported_reflects_registry() {
        assert!(is_supported("USDC"));
        assert!(is_supported("STELLAR_XLM"));
        assert!(!is_supported("DOGE"));
    }

    #[test]
    fn supported_symbols_lists_unique_canonical_symbols() {
        let symbols = supported_symbols();
        assert!(symbols.contains(&"USDC"));
        assert!(symbols.contains(&"ETH"));
        assert!(symbols.contains(&"BTC"));
        assert!(symbols.contains(&"XLM"));
        // No canonical symbol appears twice.
        let mut deduped = symbols.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), symbols.len());
    }

    // --- registry invariants ---

    #[test]
    fn all_thresholds_are_within_the_unit_interval() {
        for entry in REGISTRY {
            if let Some(bps) = entry.liquidation_threshold_bps {
                let t = bps_to_decimal(bps);
                assert!(
                    t > dec!(0) && t <= dec!(1),
                    "threshold for {} out of (0, 1]: {t}",
                    entry.symbol
                );
            }
        }
    }

    #[test]
    fn all_decimals_are_realistic() {
        for entry in REGISTRY {
            assert!(
                entry.decimals <= 18,
                "decimals for {} unrealistically high: {}",
                entry.symbol,
                entry.decimals
            );
        }
    }

    #[test]
    fn aliases_never_collide_with_a_canonical_symbol() {
        for entry in REGISTRY {
            for alias in entry.aliases {
                assert_ne!(
                    alias.to_uppercase(),
                    entry.symbol,
                    "alias duplicates its own symbol: {}",
                    entry.symbol
                );
                assert!(
                    !REGISTRY.iter().any(|e| e.symbol == alias.to_uppercase()),
                    "alias {alias} collides with a canonical symbol"
                );
            }
        }
    }
}
