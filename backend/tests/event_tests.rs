use inheritx_backend::events::{
    BorrowMetadata, DepositMetadata, InterestAccrualMetadata, LiquidationMetadata, RepayMetadata,
};
use rust_decimal_macros::dec;

#[test]
fn test_deposit_metadata_serialization() {
    let metadata = DepositMetadata {
        collateral_ratio: Some(dec!(150.00)),
        total_deposited: dec!(1000.50),
    };

    let json = serde_json::to_value(&metadata).unwrap();
    assert!(json.is_object());
    assert_eq!(json["total_deposited"], "1000.50");
}

#[test]
fn test_borrow_metadata_serialization() {
    let metadata = BorrowMetadata {
        interest_rate: dec!(5.5),
        collateral_asset: "USDC".to_string(),
        collateral_amount: dec!(1500.00),
        loan_to_value: dec!(75.00),
        maturity_date: None,
    };

    let json = serde_json::to_value(&metadata).unwrap();
    assert!(json.is_object());
    assert_eq!(json["collateral_asset"], "USDC");
    assert_eq!(json["interest_rate"], "5.5");
}

#[test]
fn test_repay_metadata_serialization() {
    let metadata = RepayMetadata {
        principal_amount: dec!(500.00),
        interest_amount: dec!(25.50),
        remaining_balance: dec!(474.50),
    };

    let json = serde_json::to_value(&metadata).unwrap();
    assert!(json.is_object());
    assert_eq!(json["principal_amount"], "500.00");
    assert_eq!(json["interest_amount"], "25.50");
}

#[test]
fn test_liquidation_metadata_serialization() {
    let liquidator_id = uuid::Uuid::new_v4();
    let metadata = LiquidationMetadata {
        liquidator_id,
        collateral_asset: "USDC".to_string(),
        collateral_seized: dec!(1200.00),
        debt_covered: dec!(1000.00),
        liquidation_penalty: dec!(50.00),
    };

    let json = serde_json::to_value(&metadata).unwrap();
    assert!(json.is_object());
    assert_eq!(json["collateral_asset"], "USDC");
    assert_eq!(json["debt_covered"], "1000.00");
}

#[test]
fn test_interest_accrual_metadata_serialization() {
    let metadata = InterestAccrualMetadata {
        interest_rate: dec!(5.5),
        principal_balance: dec!(1000.00),
        accrued_interest: dec!(4.58),
        total_balance: dec!(1004.58),
    };

    let json = serde_json::to_value(&metadata).unwrap();
    assert!(json.is_object());
    assert_eq!(json["interest_rate"], "5.5");
    assert_eq!(json["accrued_interest"], "4.58");
}
