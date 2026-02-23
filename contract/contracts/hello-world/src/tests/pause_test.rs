#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::base::types::GroupMember;
use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env, String};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &contract_address.address()),
        token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_admin_can_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    assert!(!client.get_paused_status());
    client.pause(&admin);
    assert!(client.get_paused_status());
}

#[test]
fn test_admin_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.pause(&admin);
    assert!(client.get_paused_status());

    client.unpause(&admin);
    assert!(!client.get_paused_status());
}

#[test]
fn test_paused_status_returned_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Initially not paused
    assert!(!client.get_paused_status());

    // After pause
    client.pause(&admin);
    assert!(client.get_paused_status());

    // After unpause
    client.unpause(&admin);
    assert!(!client.get_paused_status());
}

#[test]
#[should_panic]
fn test_non_admin_cannot_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.pause(&non_admin);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.pause(&admin);
    client.unpause(&non_admin);
}

#[test]
#[should_panic]
fn test_cannot_pause_already_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.pause(&admin);
    client.pause(&admin);
}

#[test]
#[should_panic]
fn test_cannot_unpause_not_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.unpause(&admin);
}

#[test]
#[should_panic]
fn test_create_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Setup token
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    let token_address = token_client.address.clone();
    client.add_supported_token(&token_address, &admin);

    client.pause(&admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");
    token_admin_client.mint(&creator, &10000000);
    client.create(&id, &name, &creator, &100u32, &token_address);
}

#[test]
#[should_panic]
fn test_add_member_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Setup token
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    let token_address = token_client.address.clone();
    client.add_supported_token(&token_address, &admin);

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    token_admin_client.mint(&creator, &10000000);
    client.create(&id, &name, &creator, &100u32, &token_address);
    client.pause(&admin);
    client.add_group_member(&id, &creator, &member, &50u32);
}

#[test]
#[should_panic]
fn test_topup_subscription_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Setup token
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    let token_address = token_client.address.clone();
    client.add_supported_token(&token_address, &admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    token_admin_client.mint(&creator, &10000000);
    client.create(&id, &name, &creator, &100u32, &token_address);

    // Pause the contract
    client.pause(&admin);

    // Attempt to top up while paused - should fail with ContractPaused
    let payer = Address::generate(&env);
    token_admin_client.mint(&payer, &10000000);
    client.topup_subscription(&id, &10u32, &token_address, &payer);
}

#[test]
fn test_read_functions_work_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Setup token
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    let token_address = token_client.address.clone();
    client.add_supported_token(&token_address, &admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    token_admin_client.mint(&creator, &10000000);
    client.create(&id, &name, &creator, &100u32, &token_address);
    client.pause(&admin);

    // These should all work while paused
    let _ = client.get(&id);
    let _ = client.get_all_groups();
    let _ = client.get_groups_by_creator(&creator);
    let _ = client.get_group_members(&id);
    let _ = client.is_group_member(&id, &creator);
    let _ = client.get_paused_status();
}

#[test]
fn test_operations_work_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Setup token
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client) = create_token_contract(&env, &token_admin);
    let token_address = token_client.address.clone();
    client.add_supported_token(&token_address, &admin);

    client.pause(&admin);
    client.unpause(&admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    token_admin_client.mint(&creator, &10000000);
    // Should work after unpause
    client.create(&id, &name, &creator, &100u32, &token_address);
    let result = client.get(&id);
    assert_eq!(result.name, name);
}
