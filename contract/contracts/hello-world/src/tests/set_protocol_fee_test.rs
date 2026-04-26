#![allow(unused_imports)]

use crate::test_utils::deploy_autoshare_contract;
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

// ─── helpers ────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (Address, AutoShareContractClient<'_>) {
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    (admin, client)
}

fn group_id(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

// ─── Global fee ──────────────────────────────────────────────────────────────

/// Default global protocol fee is 0 before any admin sets it.
#[test]
fn test_global_protocol_fee_defaults_to_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    assert_eq!(client.get_protocol_fee_v2(&None), 0);
}

/// Admin can set the global protocol fee to a valid percentage.
#[test]
fn test_admin_can_set_global_protocol_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &5, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 5);
}

/// Admin can update the global protocol fee multiple times.
#[test]
fn test_admin_can_update_global_protocol_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &10, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 10);

    client.set_protocol_fee_v2(&admin, &25, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 25);
}

/// Admin can set the global protocol fee to 0 (fee-free).
#[test]
fn test_admin_can_set_global_fee_to_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &20, &None);
    client.set_protocol_fee_v2(&admin, &0, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 0);
}

/// Admin can set the global protocol fee to the maximum (100%).
#[test]
fn test_admin_can_set_global_fee_to_100() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &100, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 100);
}

/// Setting a fee above 100 is rejected with InvalidInput.
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_global_fee_above_100_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &101, &None);
}

/// Non-admin cannot set the global protocol fee.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_set_global_protocol_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let non_admin = Address::generate(&env);
    client.set_protocol_fee_v2(&non_admin, &5, &None);
}

// ─── Group-specific fee ──────────────────────────────────────────────────────

/// Admin can set a group-specific protocol fee override.
#[test]
fn test_admin_can_set_group_specific_protocol_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = group_id(&env, 1);
    client.set_protocol_fee_v2(&admin, &15, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 15);
}

/// Group-specific fee overrides the global fee for that group.
#[test]
fn test_group_fee_overrides_global_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set global fee
    client.set_protocol_fee_v2(&admin, &10, &None);

    // Override for group 1
    let id = group_id(&env, 1);
    client.set_protocol_fee_v2(&admin, &3, &Some(id.clone()));

    // Group 1 uses its own override
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 3);
    // Global fee is unchanged
    assert_eq!(client.get_protocol_fee_v2(&None), 10);
}

/// A group without an override falls back to the global fee.
#[test]
fn test_group_without_override_falls_back_to_global() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &8, &None);

    // group 2 has no override — should return global fee
    let id = group_id(&env, 2);
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 8);
}

/// Different groups can have independent fee overrides.
#[test]
fn test_independent_group_fee_overrides() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id1 = group_id(&env, 1);
    let id2 = group_id(&env, 2);

    client.set_protocol_fee_v2(&admin, &5, &Some(id1.clone()));
    client.set_protocol_fee_v2(&admin, &20, &Some(id2.clone()));

    assert_eq!(client.get_protocol_fee_v2(&Some(id1)), 5);
    assert_eq!(client.get_protocol_fee_v2(&Some(id2)), 20);
}

/// Admin can update a group-specific fee multiple times.
#[test]
fn test_admin_can_update_group_specific_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = group_id(&env, 1);
    client.set_protocol_fee_v2(&admin, &10, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id.clone())), 10);

    client.set_protocol_fee_v2(&admin, &50, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 50);
}

/// Group-specific fee above 100 is rejected with InvalidInput.
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_group_fee_above_100_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = group_id(&env, 1);
    client.set_protocol_fee_v2(&admin, &101, &Some(id));
}

/// Non-admin cannot set a group-specific protocol fee.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_set_group_specific_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let non_admin = Address::generate(&env);
    let id = group_id(&env, 1);
    client.set_protocol_fee_v2(&non_admin, &5, &Some(id));
}

/// Setting a group fee to 0 explicitly overrides the global fee with zero.
#[test]
fn test_group_fee_zero_overrides_global() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &20, &None);

    let id = group_id(&env, 1);
    client.set_protocol_fee_v2(&admin, &0, &Some(id.clone()));

    // Group explicitly has 0% fee even though global is 20%
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 0);
}
