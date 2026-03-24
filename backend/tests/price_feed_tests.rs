use chrono::Utc;
use inheritx_backend::price_feed::{AssetPrice, PriceFeedSource};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn test_price_feed_source_as_str() {
    assert_eq!(PriceFeedSource::Pyth.as_str(), "pyth");
    assert_eq!(PriceFeedSource::Chainlink.as_str(), "chainlink");
    assert_eq!(PriceFeedSource::Custom.as_str(), "custom");
}

#[test]
fn test_asset_price_creation() {
    let price = Decimal::from_str("1000.50").unwrap();
    let asset = AssetPrice {
        asset_code: "BTC".to_string(),
        price,
        timestamp: Utc::now(),
        source: "custom".to_string(),
    };

    assert_eq!(asset.asset_code, "BTC");
    assert_eq!(asset.price, Decimal::from_str("1000.50").unwrap());
    assert_eq!(asset.source, "custom");
}

#[test]
fn test_decimal_price_parsing() {
    let price_str = "1234.56";
    let price = Decimal::from_str(price_str).unwrap();
    assert_eq!(price, Decimal::from_str("1234.56").unwrap());
}

#[test]
fn test_price_calculation() {
    let price = Decimal::from_str("100.00").unwrap();
    let amount = Decimal::from_str("2.5").unwrap();
    let valuation = price * amount;
    assert_eq!(valuation, Decimal::from_str("250.00").unwrap());
}

#[test]
fn test_collateral_ratio_calculation() {
    let collateral_value = Decimal::from_str("1000.00").unwrap();
    let loan_amount = Decimal::from_str("500.00").unwrap();
    let ratio = collateral_value / loan_amount;
    assert_eq!(ratio, Decimal::from_str("2.00").unwrap());
}

#[test]
fn test_price_feed_source_equality() {
    assert_eq!(PriceFeedSource::Pyth, PriceFeedSource::Pyth);
    assert_ne!(PriceFeedSource::Pyth, PriceFeedSource::Chainlink);
}

#[test]
fn test_asset_price_clone() {
    let price = Decimal::from_str("500.00").unwrap();
    let asset1 = AssetPrice {
        asset_code: "ETH".to_string(),
        price,
        timestamp: Utc::now(),
        source: "pyth".to_string(),
    };

    let asset2 = asset1.clone();
    assert_eq!(asset1.asset_code, asset2.asset_code);
    assert_eq!(asset1.price, asset2.price);
}

#[test]
fn test_multiple_asset_prices() {
    let btc_price = Decimal::from_str("45000.00").unwrap();
    let eth_price = Decimal::from_str("2500.00").unwrap();

    let btc = AssetPrice {
        asset_code: "BTC".to_string(),
        price: btc_price,
        timestamp: Utc::now(),
        source: "custom".to_string(),
    };

    let eth = AssetPrice {
        asset_code: "ETH".to_string(),
        price: eth_price,
        timestamp: Utc::now(),
        source: "custom".to_string(),
    };

    assert_eq!(btc.asset_code, "BTC");
    assert_eq!(eth.asset_code, "ETH");
    assert!(btc.price > eth.price);
}

#[test]
fn test_price_feed_initialization() {
    let source = PriceFeedSource::Custom;
    assert_eq!(source.as_str(), "custom");
}

#[test]
fn test_invalid_price_handling() {
    // Test that we can handle price parsing correctly
    let valid_price = Decimal::from_str("1000.00");
    assert!(valid_price.is_ok());

    let invalid_price = Decimal::from_str("invalid");
    assert!(invalid_price.is_err());
}
