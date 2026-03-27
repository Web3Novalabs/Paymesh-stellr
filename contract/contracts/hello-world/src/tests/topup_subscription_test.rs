use crate::test_utils::{assert_balance, create_test_group, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Vec};

#[test]
fn test_topup_subscription_success() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(crate::base::types::GroupMember {
        address: member.clone(),
        percentage: 100,
    });

    let initial_usages = 5u32;
    let id = create_test_group(&env, &contract, &creator, &members, initial_usages, &token);

    let payer = test_env.users.get(1).unwrap().clone();
    let additional_usages = 10u32;
    let fee = 10i128; // Default fee
    let total_cost = (additional_usages as i128) * fee;

    mint_tokens(&env, &token, &payer, total_cost);

    client.topup_subscription(&id, &additional_usages, &token, &payer);

    let remaining = client.get_remaining_usages(&id);
    assert_eq!(remaining, initial_usages + additional_usages);

    // Check fee was transferred (contract balance)
    // Initial 5 usages * 10 fee = 50. Topup 10 usages * 10 fee = 100. Total = 150.
    assert_balance(&env, &token, &contract, 150);
}

#[test]
#[should_panic(expected = "InvalidUsageCount")]
fn test_topup_subscription_zero_usages_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&env, &[1; 32]);
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let payer = test_env.users.get(0).unwrap().clone();

    client.topup_subscription(&id, &0, &token, &payer);
}

#[test]
#[should_panic(expected = "UnsupportedToken")]
fn test_topup_subscription_unsupported_token_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let id = create_test_group(
        &env,
        &test_env.autoshare_contract,
        &test_env.users.get(0).unwrap(),
        &Vec::new(&env),
        5,
        &test_env.mock_tokens.get(0).unwrap(),
    );

    let unsupported_token = Address::generate(&env);
    let payer = test_env.users.get(0).unwrap().clone();

    client.topup_subscription(&id, &5, &unsupported_token, &payer);
}

#[test]
#[should_panic(expected = "NotFound")]
fn test_topup_subscription_non_existent_group_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&env, &[99; 32]); // Non-existent
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let payer = test_env.users.get(0).unwrap().clone();

    client.topup_subscription(&id, &5, &token, &payer);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_topup_subscription_fails_when_paused() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    client.pause(&test_env.admin);

    let id = BytesN::from_array(&env, &[3; 32]);
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let payer = test_env.users.get(0).unwrap().clone();

    client.topup_subscription(&id, &5, &token, &payer);
}

#[test]
fn test_topup_subscription_updates_total_usages_paid() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let id = create_test_group(
        &env,
        &test_env.autoshare_contract,
        &test_env.users.get(0).unwrap(),
        &Vec::new(&env),
        5,
        &test_env.mock_tokens.get(0).unwrap(),
    );
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let payer = test_env.users.get(1).unwrap().clone();

    mint_tokens(&env, &token, &payer, 100);
    client.topup_subscription(&id, &10, &token, &payer);

    let details = client.get(&id);
    assert_eq!(details.total_usages_paid, 15); // 5 initial + 10 additional
}

#[test]
fn test_topup_subscription_records_payment_history() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let payer = test_env.users.get(1).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let id = create_test_group(
        &env,
        &test_env.autoshare_contract,
        &creator,
        &Vec::new(&env),
        5,
        &token,
    );

    mint_tokens(&env, &token, &payer, 200);
    client.topup_subscription(&id, &20, &token, &payer);

    let history = client.get_user_payment_history(&payer);
    assert_eq!(history.len(), 1);
    let payment = history.get(0).unwrap();
    assert_eq!(payment.user, payer);
    assert_eq!(payment.group_id, id);
    assert_eq!(payment.usages_purchased, 20);
    assert_eq!(payment.amount_paid, 200);
}

#[test]
fn test_topup_subscription_multiple_accumulates_correctly() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let payer = test_env.users.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let id = create_test_group(
        &env,
        &test_env.autoshare_contract,
        &payer,
        &Vec::new(&env),
        5,
        &token,
    );

    mint_tokens(&env, &token, &payer, 500);
    client.topup_subscription(&id, &10, &token, &payer);
    client.topup_subscription(&id, &20, &token, &payer);

    let remaining = client.get_remaining_usages(&id);
    assert_eq!(remaining, 35); // 5 + 10 + 20
}

#[test]
fn test_topup_subscription_works_for_exhausted_subscription() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let payer = test_env.users.get(1).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let id = create_test_group(
        &env,
        &test_env.autoshare_contract,
        &creator,
        &Vec::new(&env),
        1,
        &token,
    );

    // Add a member (crucial for distribute)
    let member = Address::generate(&env);
    client.add_group_member(&id, &creator, &member, &100);

    // Exhaust it
    let sender = test_env.users.get(2).unwrap().clone();
    mint_tokens(&env, &token, &sender, 100);
    client.distribute(&id, &token, &100, &sender);

    assert_eq!(client.get_remaining_usages(&id), 0);

    // Top up
    mint_tokens(&env, &token, &payer, 100);
    client.topup_subscription(&id, &10, &token, &payer);

    assert_eq!(client.get_remaining_usages(&id), 10);

    // Total distributed remains 100 (from the distribution before topup)
}

#[test]
#[should_panic]
fn test_topup_subscription_fails_with_insufficient_payer_balance() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(
        &env,
        &test_env.autoshare_contract,
        &creator,
        &Vec::new(&env),
        5,
        &token,
    );

    let broke_payer = Address::generate(&env);
    // Needs 100 for 10 usages, has 0
    client.topup_subscription(&id, &10, &token, &broke_payer);
}
