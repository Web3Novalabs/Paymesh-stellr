//! Unit tests for `create_payment_group` (payment group creation).

use crate::base::types::GroupMember;
use crate::test_utils::{deploy_mock_token, fund_user_with_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, String, Vec};

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
