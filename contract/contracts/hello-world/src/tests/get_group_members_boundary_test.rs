#![cfg(test)]

use crate::test_utils::{
    create_test_members, deploy_autoshare_contract, deploy_mock_token, mint_tokens,
};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};
use crate::base::types::GroupMember;

fn setup(env: &Env) -> (Address, Address, AutoShareContractClient<'_>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);

    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST"),
    );
    client.add_supported_token(&token, &admin);
    (admin, token, client)
}

fn make_group(
    env: &Env,
    client: &AutoShareContractClient,
    token: &Address,
    creator: &Address,
    seed: u8,
) -> BytesN<32> {
    let id = BytesN::from_array(env, &[seed; 32]);
    mint_tokens(env, token, creator, 10_000);
    client.create(&id, &String::from_str(env, "Test Group"), creator, &1, token);
    id
}

#[test]
fn test_get_group_members_max_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 1);

    // Max capacity is usually 50 as per autoshare_logic.rs
    let max_members = 50;
    let mut members: Vec<GroupMember> = Vec::new(&env);
    for _i in 0..max_members {
        members.push_back(GroupMember {
            address: Address::generate(&env),
            percentage: 2, // 2% * 50 = 100%
        });
    }

    client.update_members(&id, &creator, &members);
    
    let result = client.get_group_members(&id);
    assert_eq!(result.len(), 50);
}

#[test]
#[should_panic] // GroupNotFound results in panic in client.get(id) which get_group_members uses
fn test_get_group_members_nonexistent_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    
    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);
    client.get_group_members(&ghost_id);
}

#[test]
fn test_get_group_members_after_deactivation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 2);

    let members = create_test_members(&env, 2);
    client.update_members(&id, &creator, &members);
    
    client.deactivate_group(&id, &creator);
    assert_eq!(client.is_group_active(&id), false);

    // Members should still be fetchable
    let result = client.get_group_members(&id);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_get_group_members_extreme_input() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    
    // Large ID (handled by type, but good to check)
    let id = BytesN::from_array(&env, &[0xEEu8; 32]);
    mint_tokens(&env, &token, &creator, 10_000);
    client.create(&id, &String::from_str(&env, "Extreme Group"), &creator, &1, &token);

    // One member with 100%
    let mut members: Vec<GroupMember> = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });
    client.update_members(&id, &creator, &members);

    let result = client.get_group_members(&id);
    assert_eq!(result.len(), 1);
    assert_eq!(result.get(0).unwrap().percentage, 100);
}
