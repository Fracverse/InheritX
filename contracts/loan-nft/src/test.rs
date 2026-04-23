#![cfg(test)]

use crate::{LoanNFT, LoanNFTClient, LoanMetadata};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_mint_and_burn() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, LoanNFT);
    let client = LoanNFTClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin);
    
    let metadata = LoanMetadata {
        loan_id: 1,
        borrower: user.clone(),
        principal: 1000,
        collateral_amount: 500,
        collateral_token: token.clone(),
        due_date: 0,
    };
    
    client.mint(&user, &metadata);
    
    assert_eq!(client.owner_of(&1), Some(user.clone()));
    assert_eq!(client.balance_of(&user), 1);
    assert_eq!(client.total_supply(), 1);
    
    client.burn(&1);
    
    assert_eq!(client.owner_of(&1), None);
    assert_eq!(client.balance_of(&user), 0);
    assert_eq!(client.total_supply(), 0);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, LoanNFT);
    let client = LoanNFTClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin);
    
    let metadata = LoanMetadata {
        loan_id: 2,
        borrower: user1.clone(),
        principal: 1000,
        collateral_amount: 500,
        collateral_token: token.clone(),
        due_date: 0,
    };
    
    client.mint(&user1, &metadata);
    client.transfer(&user1, &user2, &2);
    
    assert_eq!(client.owner_of(&2), Some(user2.clone()));
    assert_eq!(client.balance_of(&user1), 0);
    assert_eq!(client.balance_of(&user2), 1);
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, LoanNFT);
    let client = LoanNFTClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let operator = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin);
    let metadata = LoanMetadata {
        loan_id: 3,
        borrower: user1.clone(),
        principal: 1000,
        collateral_amount: 500,
        collateral_token: token.clone(),
        due_date: 0,
    };
    
    client.mint(&user1, &metadata);
    
    client.approve(&user1, &operator, &3);
    assert_eq!(client.get_approved(&3), Some(operator.clone()));
    
    client.transfer_from(&operator, &user1, &user2, &3);
    
    assert_eq!(client.owner_of(&3), Some(user2.clone()));
    // Approval should be cleared after transfer
    assert_eq!(client.get_approved(&3), None);
}

#[test]
fn test_approval_for_all() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, LoanNFT);
    let client = LoanNFTClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let operator = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin);
    let metadata = LoanMetadata { loan_id: 4, borrower: user1.clone(), principal: 100, collateral_amount: 50, collateral_token: token.clone(), due_date: 0 };
    client.mint(&user1, &metadata);
    
    client.set_approval_for_all(&user1, &operator, &true);
    assert!(client.is_approved_for_all(&user1, &operator));
    
    client.transfer_from(&operator, &user1, &user2, &4);
    assert_eq!(client.owner_of(&4), Some(user2));
}

#[test]
#[should_panic(expected = "NFT transfer is restricted for an active loan")]
fn test_transfer_restriction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, LoanNFT);
    let client = LoanNFTClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin);
    let metadata = LoanMetadata { loan_id: 5, borrower: user1.clone(), principal: 100, collateral_amount: 50, collateral_token: token.clone(), due_date: 0 };
    client.mint(&user1, &metadata);
    
    client.set_transferable(&5, &false);
    client.transfer(&user1, &user2, &5);
}

#[test]
fn test_token_uri() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, LoanNFT);
    let client = LoanNFTClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin);
    let metadata = LoanMetadata { loan_id: 6, borrower: user1.clone(), principal: 100, collateral_amount: 50, collateral_token: token.clone(), due_date: 0 };
    client.mint(&user1, &metadata);
    
    let uri = String::from_str(&env, "https://example.com/nft/6");
    client.set_token_uri(&6, &uri);
    
    assert_eq!(client.token_uri(&6), uri);
}
