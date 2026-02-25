use inheritx_backend::{DefaultPriceFeedService, PriceFeedService, PriceFeedSource};
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[tokio::test]
async fn test_price_feed_initialization() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));

    // Initialize defaults
    let result = service.initialize_defaults().await;
    assert!(result.is_ok(), "Failed to initialize defaults");
}

#[tokio::test]
async fn test_register_price_feed() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Register a new feed
    let result = service
        .register_feed("ETH", PriceFeedSource::Pyth, "eth-usd-feed")
        .await;

    assert!(result.is_ok(), "Failed to register price feed");
    let config = result.unwrap();
    assert_eq!(config.asset_code, "ETH");
    assert_eq!(config.source, "pyth");
    assert_eq!(config.feed_id, "eth-usd-feed");
    assert!(config.is_active);
}

#[tokio::test]
async fn test_update_and_get_price() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Update price
    let price = Decimal::new(1_00000000, 8); // 1.00000000
    let result = service.update_price("USDC", price).await;

    assert!(result.is_ok(), "Failed to update price");
    let asset_price = result.unwrap();
    assert_eq!(asset_price.asset_code, "USDC");
    assert_eq!(asset_price.price, price);

    // Get price
    let result = service.get_price("USDC").await;
    assert!(result.is_ok(), "Failed to get price");
    let fetched_price = result.unwrap();
    assert_eq!(fetched_price.price, price);
}

#[tokio::test]
async fn test_price_history() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Add multiple prices
    let price1 = Decimal::new(1_00000000, 8);
    let price2 = Decimal::new(1_01000000, 8);
    let price3 = Decimal::new(1_02000000, 8);

    service.update_price("USDC", price1).await.ok();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    service.update_price("USDC", price2).await.ok();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    service.update_price("USDC", price3).await.ok();

    // Get history
    let result = service.get_price_history("USDC", 10).await;
    assert!(result.is_ok(), "Failed to get price history");
    let history = result.unwrap();
    assert!(history.len() >= 3, "Expected at least 3 prices in history");
}

#[tokio::test]
async fn test_calculate_valuation() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Set price
    let price = Decimal::new(1_00000000, 8); // 1.00
    service.update_price("USDC", price).await.ok();

    // Calculate valuation
    let amount = Decimal::new(1000, 0); // 1000 USDC
    let result = service.calculate_valuation("USDC", amount).await;

    assert!(result.is_ok(), "Failed to calculate valuation");
    let valuation = result.unwrap();
    assert_eq!(valuation.asset_code, "USDC");
    assert_eq!(valuation.amount, amount);
    assert_eq!(valuation.current_price, price);
    assert_eq!(valuation.valuation_usd, Decimal::new(1000, 0)); // 1000 * 1.00
    assert_eq!(valuation.collateral_ratio, Decimal::from(100));
}

#[tokio::test]
async fn test_get_active_feeds() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Register additional feeds
    service
        .register_feed("ETH", PriceFeedSource::Chainlink, "eth-feed")
        .await
        .ok();
    service
        .register_feed("BTC", PriceFeedSource::Pyth, "btc-feed")
        .await
        .ok();

    // Get active feeds
    let result = service.get_active_feeds().await;
    assert!(result.is_ok(), "Failed to get active feeds");
    let feeds = result.unwrap();
    assert!(feeds.len() >= 3, "Expected at least 3 active feeds");

    // Verify USDC is in the list
    let usdc_feed = feeds.iter().find(|f| f.asset_code == "USDC");
    assert!(usdc_feed.is_some(), "USDC feed not found");
}

#[tokio::test]
async fn test_price_cache_validity() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create service with 1 second cache TTL
    let service = Arc::new(DefaultPriceFeedService::new(pool, 1));
    service.initialize_defaults().await.ok();

    // Update price
    let price = Decimal::new(1_00000000, 8);
    service.update_price("USDC", price).await.ok();

    // Get price (should be cached)
    let result1 = service.get_price("USDC").await;
    assert!(result1.is_ok());

    // Wait for cache to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Get price again (should fetch from DB)
    let result2 = service.get_price("USDC").await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_collateral_ratio_calculation() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Set price
    let price = Decimal::new(2_00000000, 8); // 2.00
    service.update_price("USDC", price).await.ok();

    // Calculate valuation with different amounts
    let amount1 = Decimal::new(500, 0);
    let valuation1 = service.calculate_valuation("USDC", amount1).await.unwrap();

    let amount2 = Decimal::new(1000, 0);
    let valuation2 = service.calculate_valuation("USDC", amount2).await.unwrap();

    // Both should have 100% collateral ratio
    assert_eq!(valuation1.collateral_ratio, Decimal::from(100));
    assert_eq!(valuation2.collateral_ratio, Decimal::from(100));

    // Valuations should be proportional
    assert_eq!(valuation1.valuation_usd, Decimal::new(1000, 0)); // 500 * 2.00
    assert_eq!(valuation2.valuation_usd, Decimal::new(2000, 0)); // 1000 * 2.00
}

#[tokio::test]
async fn test_price_feed_not_found() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Try to get price for non-existent asset
    let result = service.get_price("NONEXISTENT").await;
    assert!(result.is_err(), "Expected error for non-existent asset");
}

#[tokio::test]
async fn test_invalid_price_update() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/inheritx_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let service = Arc::new(DefaultPriceFeedService::new(pool, 300));
    service.initialize_defaults().await.ok();

    // Try to update price for non-existent feed
    let price = Decimal::new(1_00000000, 8);
    let result = service.update_price("NONEXISTENT", price).await;
    assert!(result.is_err(), "Expected error for non-existent feed");
}
