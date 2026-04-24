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

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_distribute_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.pause(&test_env.admin);
    client.distribute(&group_id, &token, &100, &creator);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_remove_group_member_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let member = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.pause(&test_env.admin);
    client.remove_group_member(&group_id, &creator, &member);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_update_members_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let member = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    let mut members = soroban_sdk::Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member.clone(),
        percentage: 100,
    });

    client.pause(&test_env.admin);
    client.update_members(&group_id, &creator, &members);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_activate_group_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.deactivate_group(&group_id, &creator);
    client.pause(&test_env.admin);
    client.activate_group(&group_id, &creator);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_deactivate_group_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.pause(&test_env.admin);
    client.deactivate_group(&group_id, &creator);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_update_group_name_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.pause(&test_env.admin);
    client.update_group_name(
        &group_id,
        &creator,
        &String::from_str(&test_env.env, "New Name"),
    );
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_delete_group_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.deactivate_group(&group_id, &creator); // must be inactive to delete usually
                                                  // Also requires usages = 0 usually, but pause should trigger first.
    client.pause(&test_env.admin);
    client.delete_group(&group_id, &creator);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_start_fundraising_fails_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.pause(&test_env.admin);
    client.start_fundraising(&group_id, &creator, &1000);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_contribute_fail_when_paused() {
    let test_env = crate::test_utils::setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 10000);
    client.create(
        &group_id,
        &String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.start_fundraising(&group_id, &creator, &1000);
    client.pause(&test_env.admin);
    client.contribute(&group_id, &token, &100, &creator);
}
