use crate::mock_token::MockTokenClient;
use crate::test_utils::{assert_balance, deploy_mock_token, setup_test_env};
use soroban_sdk::{testutils::Address as _, Address, String};

#[test]
fn test_mock_token() {
    let test_env = setup_test_env();
    let env = &test_env.env;

    let _admin = Address::generate(env);
    let user1 = Address::generate(env);
    let user2 = Address::generate(env);

    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Mock Token"),
        &String::from_str(env, "MOCK"),
    );
    let client = MockTokenClient::new(env, &token);

    assert_eq!(client.decimals(), 7);
    assert_eq!(client.name(), String::from_str(env, "Mock Token"));
    assert_eq!(client.symbol(), String::from_str(env, "MOCK"));

    // Test Mint
    client.mint(&user1, &1000);
    assert_balance(env, &token, &user1, 1000);
    assert_eq!(client.total_supply(), 1000);

    // Test Transfer
    client.transfer(&user1, &user2, &200);
    assert_balance(env, &token, &user1, 800);
    assert_balance(env, &token, &user2, 200);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_insufficient_balance() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let user1 = Address::generate(env);
    let user2 = Address::generate(env);

    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Mock Token"),
        &String::from_str(env, "MOCK"),
    );
    let client = MockTokenClient::new(env, &token);

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &101);
}

#[test]
#[should_panic(expected = "Invalid amount")]
fn test_invalid_mint_amount() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let user1 = Address::generate(env);

    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Mock Token"),
        &String::from_str(env, "MOCK"),
    );
    let client = MockTokenClient::new(env, &token);

    client.mint(&user1, &0);
}

#[test]
#[should_panic(expected = "Invalid amount")]
fn test_invalid_transfer_amount() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let user1 = Address::generate(env);
    let user2 = Address::generate(env);

    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Mock Token"),
        &String::from_str(env, "MOCK"),
    );
    let client = MockTokenClient::new(env, &token);

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &-10);
}
