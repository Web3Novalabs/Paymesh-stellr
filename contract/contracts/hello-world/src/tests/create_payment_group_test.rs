//! Unit tests for `create_payment_group` (payment group creation).

use crate::base::types::GroupMember;
use crate::test_utils::{deploy_mock_token, fund_user_with_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, BytesN, FromVal, String, Symbol, Vec,
};

#[test]
fn test_create_payment_group_success_stores_creator_usage_and_token_config() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[201u8; 32]);
    let usage = 7u32;

    fund_user_with_tokens(env, &token, &creator, 50_000);

    client.create_payment_group(
        &id,
        &String::from_str(env, "Team payouts"),
        &creator,
        &usage,
        &token,
    );

    let group = client.get(&id);
    assert_eq!(group.creator, creator);
    assert_eq!(group.usage_count, usage);
    assert_eq!(group.total_usages_paid, usage);
    assert!(group.is_active);
    assert_eq!(group.members.len(), 0);
}

#[test]
fn test_create_payment_group_emits_payment_group_created_event_with_indexed_and_payload_fields() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[207u8; 32]);
    let usage_count = 4u32;

    // Ensure event payload values are deterministic.
    client.set_max_members(&client.get_admin(), &12);
    client.set_usage_fee(&25, &client.get_admin());
    fund_user_with_tokens(env, &token, &creator, 50_000);

    client.create_payment_group(
        &id,
        &String::from_str(env, "Eventful group"),
        &creator,
        &usage_count,
        &token,
    );

    let events = env.events().all();
    let event = events
        .iter()
        .find(|e| {
            Symbol::from_val(env, &e.1.get(0).unwrap()) == Symbol::new(env, "payment_group_created")
        })
        .expect("payment_group_created event not found");

    // topics: [SYMBOL(payment_group_created), id, creator, payment_token]
    assert_eq!(BytesN::<32>::from_val(env, &event.1.get(1).unwrap()), id);
    assert_eq!(Address::from_val(env, &event.1.get(2).unwrap()), creator);
    assert_eq!(Address::from_val(env, &event.1.get(3).unwrap()), token);

    // data = map { usage_count, usage_fee, total_cost, member_limit, timestamp }
    let data = soroban_sdk::Map::<soroban_sdk::Symbol, soroban_sdk::Val>::from_val(env, &event.2);
    let event_usage_count = u32::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "usage_count"))
            .unwrap(),
    );
    let usage_fee = u32::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "usage_fee"))
            .unwrap(),
    );
    let total_cost = i128::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "total_cost"))
            .unwrap(),
    );
    let member_limit = u32::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "member_limit"))
            .unwrap(),
    );
    let timestamp = u64::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "timestamp"))
            .unwrap(),
    );

    assert_eq!(event_usage_count, usage_count);
    assert_eq!(usage_fee, 25);
    assert_eq!(total_cost, 100);
    assert_eq!(member_limit, 12);
    assert_eq!(timestamp, env.ledger().timestamp());
}

#[test]
fn test_create_payment_group_then_update_members_sets_initial_split() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[202u8; 32]);

    fund_user_with_tokens(env, &token, &creator, 20_000);
    client.create_payment_group(
        &id,
        &String::from_str(env, "Ops split"),
        &creator,
        &3,
        &token,
    );

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 60,
    });
    members.push_back(GroupMember {
        address: m2.clone(),
        percentage: 40,
    });
    client.update_members(&id, &creator, &members);

    let stored = client.get_group_members(&id);
    assert_eq!(stored.len(), 2);
    assert_eq!(stored.get(0).unwrap().percentage, 60);
    assert_eq!(stored.get(1).unwrap().percentage, 40);
}

#[test]
#[should_panic(expected = "AlreadyExists")]
fn test_create_payment_group_duplicate_id_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[203u8; 32]);

    fund_user_with_tokens(env, &token, &creator, 30_000);
    client.create_payment_group(&id, &String::from_str(env, "First"), &creator, &2, &token);
    client.create_payment_group(&id, &String::from_str(env, "Second"), &creator, &2, &token);
}

#[test]
#[should_panic(expected = "InvalidUsageCount")]
fn test_create_payment_group_zero_usage_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[204u8; 32]);

    fund_user_with_tokens(env, &token, &creator, 10_000);
    client.create_payment_group(
        &id,
        &String::from_str(env, "Bad usage"),
        &creator,
        &0,
        &token,
    );
}

#[test]
#[should_panic(expected = "UnsupportedToken")]
fn test_create_payment_group_unsupported_token_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let supported = test_env.mock_tokens.get(0).unwrap().clone();
    let unsupported = deploy_mock_token(
        env,
        &String::from_str(env, "Other"),
        &String::from_str(env, "OTH"),
    );

    fund_user_with_tokens(env, &supported, &creator, 10_000);
    fund_user_with_tokens(env, &unsupported, &creator, 10_000);

    let id = BytesN::from_array(env, &[205u8; 32]);
    client.create_payment_group(
        &id,
        &String::from_str(env, "Rogue token"),
        &creator,
        &1,
        &unsupported,
    );
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_create_payment_group_when_paused_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let admin = client.get_admin();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let id = BytesN::from_array(env, &[206u8; 32]);

    fund_user_with_tokens(env, &token, &creator, 10_000);
    client.pause(&admin);

    client.create_payment_group(
        &id,
        &String::from_str(env, "While paused"),
        &creator,
        &1,
        &token,
    );
}
