//! Boundary and edge-case tests for payment group creation (`create_payment_group`).

use crate::test_utils::{fund_user_with_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, String};

#[test]
fn test_create_payment_group_large_usage_count_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[210u8; 32]);
    let usage = 500u32;
    let fee: i128 = client.get_usage_fee() as i128;
    let needed = usage as i128 * fee + 10_000;

    fund_user_with_tokens(env, &token, &creator, needed);

    client.create_payment_group(
        &id,
        &String::from_str(env, "High volume group"),
        &creator,
        &usage,
        &token,
    );

    let g = client.get(&id);
    assert_eq!(g.usage_count, usage);
}

#[test]
#[should_panic(expected = "EmptyName")]
fn test_create_payment_group_whitespace_only_name_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[211u8; 32]);

    fund_user_with_tokens(env, &token, &creator, 5_000);
    client.create_payment_group(&id, &String::from_str(env, "   \t\n"), &creator, &1, &token);
}

#[test]
#[should_panic(expected = "EmptyName")]
fn test_create_payment_group_name_over_max_length_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[212u8; 32]);

    fund_user_with_tokens(env, &token, &creator, 5_000);
    let long = String::from_str(
        env,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    assert_eq!(long.len(), 61);
    client.create_payment_group(&id, &long, &creator, &1, &token);
}
