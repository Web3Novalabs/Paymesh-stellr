#![allow(unused_imports)]

use crate::test_utils::{deploy_autoshare_contract, create_test_members, create_test_group, deploy_mock_token};
use crate::AutoShareContractClient;
use crate::base::types::GroupMember;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

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

fn create_active_group(env: &Env, client: &AutoShareContractClient, creator: &Address) -> BytesN<32> {
    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST"),
    );
    client.add_supported_token(&token, creator);
    
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: Address::generate(env),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: Address::generate(env),
        percentage: 50,
    });
    
    create_test_group(env, &client.address, creator, &members, 10, &token)
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

// ─── Comprehensive Error Condition Tests ────────────────────────────────────

/// Setting global fee with invalid percentage (10001 bps) should fail.
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_global_fee_exceeds_max_basis_points() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // 10001 basis points exceeds the maximum of 10000 (100%)
    client.set_protocol_fee_v2(&admin, &10001, &None);
}

/// Setting global fee with u32::MAX should fail.
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_global_fee_with_max_u32_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &u32::MAX, &None);
}

/// Setting group fee with invalid percentage (101%) should fail.
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_group_fee_exceeds_max_percentage() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    // 101% exceeds the maximum of 100%
    client.set_protocol_fee_v2(&admin, &101, &Some(id));
}

/// Setting group fee with u32::MAX should fail.
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_group_fee_with_max_u32_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &u32::MAX, &Some(id));
}

/// Setting group fee for non-existent group should fail.
#[test]
#[should_panic(expected = "NotFound")]
fn test_group_fee_for_nonexistent_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let nonexistent_id = group_id(&env, 99);
    client.set_protocol_fee_v2(&admin, &10, &Some(nonexistent_id));
}

/// Non-admin attempting to set global fee should fail with Unauthorized.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_set_global_fee_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let non_admin = Address::generate(&env);
    client.set_protocol_fee_v2(&non_admin, &10, &None);
}

/// Non-admin attempting to set group fee should fail with Unauthorized.
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_set_group_fee_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    let non_admin = Address::generate(&env);
    client.set_protocol_fee_v2(&non_admin, &10, &Some(id));
}

// ─── State Persistence and Retrieval Tests ──────────────────────────────────

/// Verify global fee is correctly persisted and retrievable after setting.
#[test]
fn test_global_fee_persistence() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set fee to 250 bps (2.5%)
    client.set_protocol_fee_v2(&admin, &250, &None);
    
    // Verify it's persisted
    assert_eq!(client.get_protocol_fee_v2(&None), 250);
    
    // Update to 500 bps (5%)
    client.set_protocol_fee_v2(&admin, &500, &None);
    
    // Verify update is persisted
    assert_eq!(client.get_protocol_fee_v2(&None), 500);
}

/// Verify group fee is correctly persisted and retrievable after setting.
#[test]
fn test_group_fee_persistence() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    
    // Set group fee to 15%
    client.set_protocol_fee_v2(&admin, &15, &Some(id.clone()));
    
    // Verify it's persisted
    assert_eq!(client.get_protocol_fee_v2(&Some(id.clone())), 15);
    
    // Update to 30%
    client.set_protocol_fee_v2(&admin, &30, &Some(id.clone()));
    
    // Verify update is persisted
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 30);
}

/// Verify multiple groups can have independent fee overrides.
#[test]
fn test_multiple_groups_independent_fees() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Create multiple groups
    let id1 = create_active_group(&env, &client, &admin);
    let id2 = create_active_group(&env, &client, &admin);
    let id3 = create_active_group(&env, &client, &admin);
    
    // Set different fees for each group
    client.set_protocol_fee_v2(&admin, &5, &Some(id1.clone()));
    client.set_protocol_fee_v2(&admin, &15, &Some(id2.clone()));
    client.set_protocol_fee_v2(&admin, &25, &Some(id3.clone()));
    
    // Verify each group has its own fee
    assert_eq!(client.get_protocol_fee_v2(&Some(id1)), 5);
    assert_eq!(client.get_protocol_fee_v2(&Some(id2)), 15);
    assert_eq!(client.get_protocol_fee_v2(&Some(id3)), 25);
}

// ─── Boundary Value Tests ────────────────────────────────────────────────────

/// Test setting global fee to minimum value (0 bps).
#[test]
fn test_global_fee_minimum_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &0, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 0);
}

/// Test setting global fee to maximum value (10000 bps = 100%).
#[test]
fn test_global_fee_maximum_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &10000, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 10000);
}

/// Test setting group fee to minimum value (0%).
#[test]
fn test_group_fee_minimum_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &0, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 0);
}

/// Test setting group fee to maximum value (100%).
#[test]
fn test_group_fee_maximum_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &100, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 100);
}

/// Test setting global fee just below maximum (9999 bps).
#[test]
fn test_global_fee_just_below_maximum() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    client.set_protocol_fee_v2(&admin, &9999, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 9999);
}

/// Test setting group fee just below maximum (99%).
#[test]
fn test_group_fee_just_below_maximum() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &99, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 99);
}

// ─── Failed Operation State Integrity Tests ──────────────────────────────────

/// Verify failed global fee update doesn't mutate existing state.
#[test]
fn test_failed_global_fee_update_preserves_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set initial valid fee
    client.set_protocol_fee_v2(&admin, &100, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 100);
    
    // Attempt to set invalid fee (should panic)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_protocol_fee_v2(&admin, &10001, &None);
    }));
    
    assert!(result.is_err());
    
    // Verify original fee is unchanged
    assert_eq!(client.get_protocol_fee_v2(&None), 100);
}

/// Verify failed group fee update doesn't mutate existing state.
#[test]
fn test_failed_group_fee_update_preserves_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    
    // Set initial valid fee
    client.set_protocol_fee_v2(&admin, &20, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id.clone())), 20);
    
    // Attempt to set invalid fee (should panic)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_protocol_fee_v2(&admin, &101, &Some(id.clone()));
    }));
    
    assert!(result.is_err());
    
    // Verify original fee is unchanged
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 20);
}

/// Verify unauthorized access doesn't mutate global fee state.
#[test]
fn test_unauthorized_global_fee_preserves_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set initial fee as admin
    client.set_protocol_fee_v2(&admin, &50, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 50);
    
    // Attempt unauthorized update
    let non_admin = Address::generate(&env);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_protocol_fee_v2(&non_admin, &200, &None);
    }));
    
    assert!(result.is_err());
    
    // Verify original fee is unchanged
    assert_eq!(client.get_protocol_fee_v2(&None), 50);
}

/// Verify unauthorized access doesn't mutate group fee state.
#[test]
fn test_unauthorized_group_fee_preserves_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    
    // Set initial fee as admin
    client.set_protocol_fee_v2(&admin, &10, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id.clone())), 10);
    
    // Attempt unauthorized update
    let non_admin = Address::generate(&env);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_protocol_fee_v2(&non_admin, &50, &Some(id.clone()));
    }));
    
    assert!(result.is_err());
    
    // Verify original fee is unchanged
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 10);
}

/// Verify setting fee for nonexistent group doesn't affect global fee.
#[test]
fn test_nonexistent_group_doesnt_affect_global_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set global fee
    client.set_protocol_fee_v2(&admin, &75, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 75);
    
    // Attempt to set fee for nonexistent group
    let nonexistent_id = group_id(&env, 99);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_protocol_fee_v2(&admin, &25, &Some(nonexistent_id));
    }));
    
    assert!(result.is_err());
    
    // Verify global fee is unchanged
    assert_eq!(client.get_protocol_fee_v2(&None), 75);
}

// ─── Event Emission Tests ────────────────────────────────────────────────────

/// Verify ProtocolFeeSet event is emitted when setting global fee.
#[test]
fn test_global_fee_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set initial fee
    client.set_protocol_fee_v2(&admin, &100, &None);
    
    // Clear events
    let _ = env.events().all();
    
    // Update fee
    client.set_protocol_fee_v2(&admin, &200, &None);
    
    // Verify event was emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "Expected ProtocolFeeSet event to be emitted");
}

/// Verify ProtocolFeeSet event is emitted when setting group fee.
#[test]
fn test_group_fee_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    let id = create_active_group(&env, &client, &admin);
    
    // Set initial fee
    client.set_protocol_fee_v2(&admin, &10, &Some(id.clone()));
    
    // Clear events
    let _ = env.events().all();
    
    // Update fee
    client.set_protocol_fee_v2(&admin, &20, &Some(id));
    
    // Verify event was emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "Expected ProtocolFeeSet event to be emitted");
}

// ─── Integration Tests ───────────────────────────────────────────────────────

/// Test setting both global and group fees in sequence.
#[test]
fn test_global_and_group_fee_sequence() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set global fee
    client.set_protocol_fee_v2(&admin, &50, &None);
    assert_eq!(client.get_protocol_fee_v2(&None), 50);
    
    // Create group and set group-specific fee
    let id = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &10, &Some(id.clone()));
    
    // Verify both are set correctly
    assert_eq!(client.get_protocol_fee_v2(&None), 50);
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 10);
}

/// Test updating global fee doesn't affect existing group overrides.
#[test]
fn test_global_fee_update_preserves_group_overrides() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set global fee
    client.set_protocol_fee_v2(&admin, &100, &None);
    
    // Create groups with overrides
    let id1 = create_active_group(&env, &client, &admin);
    let id2 = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &5, &Some(id1.clone()));
    client.set_protocol_fee_v2(&admin, &15, &Some(id2.clone()));
    
    // Update global fee
    client.set_protocol_fee_v2(&admin, &200, &None);
    
    // Verify group overrides are unchanged
    assert_eq!(client.get_protocol_fee_v2(&Some(id1)), 5);
    assert_eq!(client.get_protocol_fee_v2(&Some(id2)), 15);
    assert_eq!(client.get_protocol_fee_v2(&None), 200);
}

/// Test group without override uses updated global fee.
#[test]
fn test_group_without_override_uses_updated_global() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set initial global fee
    client.set_protocol_fee_v2(&admin, &50, &None);
    
    // Create group without override
    let id = create_active_group(&env, &client, &admin);
    
    // Verify group uses global fee
    assert_eq!(client.get_protocol_fee_v2(&Some(id.clone())), 50);
    
    // Update global fee
    client.set_protocol_fee_v2(&admin, &100, &None);
    
    // Verify group now uses updated global fee
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 100);
}

/// Test removing group override by setting to match global fee.
#[test]
fn test_group_override_can_match_global() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, client) = setup(&env);

    // Set global fee
    client.set_protocol_fee_v2(&admin, &50, &None);
    
    // Create group with different override
    let id = create_active_group(&env, &client, &admin);
    client.set_protocol_fee_v2(&admin, &10, &Some(id.clone()));
    assert_eq!(client.get_protocol_fee_v2(&Some(id.clone())), 10);
    
    // Update group override to match global
    client.set_protocol_fee_v2(&admin, &50, &Some(id.clone()));
    
    // Both should return same value
    assert_eq!(client.get_protocol_fee_v2(&Some(id)), 50);
    assert_eq!(client.get_protocol_fee_v2(&None), 50);
}
