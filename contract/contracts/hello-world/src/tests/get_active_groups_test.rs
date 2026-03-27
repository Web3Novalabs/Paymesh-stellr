#![cfg(test)]

use crate::test_utils::{deploy_autoshare_contract, deploy_mock_token, mint_tokens};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup(env: &Env) -> (Address, Address, AutoShareContractClient<'_>) {
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    (admin, contract_id, client)
}

fn new_token(env: &Env) -> Address {
    deploy_mock_token(
        env,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST"),
    )
}

fn create_group(
    env: &Env,
    client: &AutoShareContractClient,
    token: &Address,
    creator: &Address,
    seed: u8,
) -> BytesN<32> {
    let id = BytesN::from_array(env, &[seed; 32]);
    mint_tokens(env, token, creator, 10_000);
    client.create(&id, &String::from_str(env, "Group"), creator, &5, token);
    id
}

// Returns empty vec when no groups exist
#[test]
fn test_get_active_groups_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    assert_eq!(client.get_active_groups().len(), 0);
}

// Returns only active groups, excludes deactivated ones
#[test]
fn test_get_active_groups_filters_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env);
    client.add_supported_token(&token, &admin);

    let creator = Address::generate(&env);
    let id1 = create_group(&env, &client, &token, &creator, 1);
    let id2 = create_group(&env, &client, &token, &creator, 2);
    let id3 = create_group(&env, &client, &token, &creator, 3);

    // Deactivate group 2
    client.deactivate_group(&id2, &creator);

    let active = client.get_active_groups();
    assert_eq!(active.len(), 2);

    // Verify the returned groups are id1 and id3
    let mut ids = soroban_sdk::Vec::new(&env);
    for g in active.iter() {
        ids.push_back(g.id);
    }
    assert!(ids.contains(&id1));
    assert!(!ids.contains(&id2));
    assert!(ids.contains(&id3));
}

// All groups active — returns all
#[test]
fn test_get_active_groups_all_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env);
    client.add_supported_token(&token, &admin);

    let creator = Address::generate(&env);
    create_group(&env, &client, &token, &creator, 1);
    create_group(&env, &client, &token, &creator, 2);

    assert_eq!(client.get_active_groups().len(), 2);
}

// All groups deactivated — returns empty
#[test]
fn test_get_active_groups_all_inactive_returns_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env);
    client.add_supported_token(&token, &admin);

    let creator = Address::generate(&env);
    let id1 = create_group(&env, &client, &token, &creator, 1);
    let id2 = create_group(&env, &client, &token, &creator, 2);

    client.deactivate_group(&id1, &creator);
    client.deactivate_group(&id2, &creator);

    assert_eq!(client.get_active_groups().len(), 0);
}

// Re-activating a group makes it appear again
#[test]
fn test_get_active_groups_reflects_reactivation() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env);
    client.add_supported_token(&token, &admin);

    let creator = Address::generate(&env);
    let id = create_group(&env, &client, &token, &creator, 1);

    client.deactivate_group(&id, &creator);
    assert_eq!(client.get_active_groups().len(), 0);

    client.activate_group(&id, &creator);
    assert_eq!(client.get_active_groups().len(), 1);
}
