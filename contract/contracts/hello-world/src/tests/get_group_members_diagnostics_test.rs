#![cfg(test)]
#![allow(unused_imports)]

use crate::test_utils::{
    create_test_members, deploy_autoshare_contract, deploy_mock_token, mint_tokens,
};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

// ─── helpers ────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (Address, Address, AutoShareContractClient<'_>) {
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

// ─── Counter: initial state ──────────────────────────────────────────────────

/// Query count is 0 before get_group_members is ever called.
#[test]
fn test_query_count_starts_at_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 1);

    assert_eq!(client.get_group_members_query_count(&id), 0);
}

// ─── Counter: increments ─────────────────────────────────────────────────────

/// Each call to get_group_members increments the counter by exactly 1.
#[test]
fn test_query_count_increments_on_each_call() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 2);

    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 1);

    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 2);

    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 3);
}

/// Ten consecutive calls produce a counter of exactly 10.
#[test]
fn test_query_count_after_ten_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 3);

    for _ in 0..10 {
        client.get_group_members(&id);
    }
    assert_eq!(client.get_group_members_query_count(&id), 10);
}

// ─── Counter: isolation ──────────────────────────────────────────────────────

/// Querying group A does not affect group B's counter.
#[test]
fn test_query_count_is_isolated_per_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id_a = make_group(&env, &client, &token, &creator, 4);
    let id_b = make_group(&env, &client, &token, &creator, 5);

    client.get_group_members(&id_a);
    client.get_group_members(&id_a);
    client.get_group_members(&id_a);

    assert_eq!(client.get_group_members_query_count(&id_a), 3);
    assert_eq!(client.get_group_members_query_count(&id_b), 0);
}

/// Querying two groups independently tracks each counter separately.
#[test]
fn test_query_count_independent_for_two_groups() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id_a = make_group(&env, &client, &token, &creator, 6);
    let id_b = make_group(&env, &client, &token, &creator, 7);

    client.get_group_members(&id_a);
    client.get_group_members(&id_b);
    client.get_group_members(&id_b);

    assert_eq!(client.get_group_members_query_count(&id_a), 1);
    assert_eq!(client.get_group_members_query_count(&id_b), 2);
}

// ─── Return value correctness ────────────────────────────────────────────────

/// get_group_members still returns the correct member list after diagnostics are added.
#[test]
fn test_get_group_members_returns_correct_members() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 8);

    let members = create_test_members(&env, 3);
    client.update_members(&id, &creator, &members);

    let result = client.get_group_members(&id);
    assert_eq!(result.len(), 3);
    for (i, m) in members.iter().enumerate() {
        assert_eq!(result.get(i as u32).unwrap().address, m.address);
        assert_eq!(result.get(i as u32).unwrap().percentage, m.percentage);
    }
}

/// get_group_members returns an empty list for a group with no members set.
#[test]
fn test_get_group_members_empty_group_returns_empty_vec() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 9);

    let result = client.get_group_members(&id);
    assert_eq!(result.len(), 0);
    // Counter still incremented even for empty result
    assert_eq!(client.get_group_members_query_count(&id), 1);
}

/// Counter increments even when the member list is empty.
#[test]
fn test_query_count_increments_for_empty_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 10);

    client.get_group_members(&id);
    client.get_group_members(&id);

    assert_eq!(client.get_group_members_query_count(&id), 2);
}

// ─── Counter persists across member list changes ─────────────────────────────

/// Counter is not reset when members are updated.
#[test]
fn test_query_count_persists_across_member_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 11);

    client.get_group_members(&id);
    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 2);

    // Update members
    let new_members = create_test_members(&env, 2);
    client.update_members(&id, &creator, &new_members);

    // Query again — counter must continue from 2, not reset
    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 3);
}

/// Counter is not reset when a member is added.
#[test]
fn test_query_count_persists_after_member_added() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 12);

    // Seed with one member at 100%
    let initial = create_test_members(&env, 1);
    client.update_members(&id, &creator, &initial);

    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 1);

    // Add a second member (split 50/50)
    let new_members = create_test_members(&env, 2);
    client.update_members(&id, &creator, &new_members);

    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 2);
}

// ─── Non-existent group ──────────────────────────────────────────────────────

/// Calling get_group_members on a non-existent group panics and does not
/// increment the counter (no partial state mutation).
#[test]
fn test_query_count_not_incremented_for_nonexistent_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);

    // Should panic — group does not exist
    let result = client.try_get_group_members(&ghost_id);
    assert!(result.is_err());

    // Counter must remain 0 — no increment on failure
    assert_eq!(client.get_group_members_query_count(&ghost_id), 0);
}

// ─── Deactivated group ───────────────────────────────────────────────────────

/// get_group_members works on an inactive group and still increments the counter.
#[test]
fn test_query_count_increments_for_inactive_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 13);

    client.deactivate_payment_group(&id, &creator);

    // get_group_members is a read — it does not check active status
    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 1);
}

// ─── get_group_members_query_count read helper ───────────────────────────────

/// get_group_members_query_count returns 0 for a group that has never been queried.
#[test]
fn test_query_count_read_returns_zero_for_unqueried_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 14);

    assert_eq!(client.get_group_members_query_count(&id), 0);
}

/// Calling get_group_members_query_count does not itself increment the counter.
#[test]
fn test_reading_query_count_does_not_increment_it() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 15);

    // Read the counter multiple times without calling get_group_members
    let _ = client.get_group_members_query_count(&id);
    let _ = client.get_group_members_query_count(&id);
    let _ = client.get_group_members_query_count(&id);

    assert_eq!(client.get_group_members_query_count(&id), 0);
}
