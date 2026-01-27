use crate::mock_token::{MockToken, MockTokenClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_mock_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MockToken, ());
    let client = MockTokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Test Initialize
    client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Mock Token"),
        &String::from_str(&env, "MOCK"),
    );

    assert_eq!(client.decimals(), 7);
    assert_eq!(client.name(), String::from_str(&env, "Mock Token"));
    assert_eq!(client.symbol(), String::from_str(&env, "MOCK"));

    // Test Mint
    client.mint(&user1, &1000);
    assert_eq!(client.balance(&user1), 1000);
    assert_eq!(client.total_supply(), 1000);

    // Test Transfer
    client.transfer(&user1, &user2, &200);
    assert_eq!(client.balance(&user1), 800);
    assert_eq!(client.balance(&user2), 200);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
#[should_panic(expected = "Insufficient balance")]
fn test_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MockToken, ());
    let client = MockTokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Mock Token"),
        &String::from_str(&env, "MOCK"),
    );

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &101);
}

#[test]
#[should_panic(expected = "Invalid amount")]
fn test_invalid_mint_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MockToken, ());
    let client = MockTokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Mock Token"),
        &String::from_str(&env, "MOCK"),
    );

    client.mint(&user1, &0);
}

#[test]
#[should_panic(expected = "Invalid amount")]
fn test_invalid_transfer_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MockToken, ());
    let client = MockTokenClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Mock Token"),
        &String::from_str(&env, "MOCK"),
    );

    client.mint(&user1, &100);
    client.transfer(&user1, &user2, &-10);
}
