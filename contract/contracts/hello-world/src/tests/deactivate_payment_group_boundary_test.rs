use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Vec};

// ─── Helper ──────────────────────────────────────────────────────────────────

fn two_member_50_50(env: &soroban_sdk::Env) -> Vec<GroupMember> {
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: Address::generate(env),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: Address::generate(env),
        percentage: 50,
    });
    members
}

// ─── Test 1 ──────────────────────────────────────────────────────────────────
// Deactivate a group whose ID is all 0xFF bytes (max BytesN<32> value).

#[test]
fn test_deactivate_group_with_max_id_bytes_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    // Manually create a group with the max-value ID
    let max_id = BytesN::from_array(env, &[0xFFu8; 32]);
    let fee: i128 = 10;
    let usages: u32 = 1;
    mint_tokens(env, &token, &creator, fee * usages as i128 + 10_000);
    client.create(
        &max_id,
        &soroban_sdk::String::from_str(env, "Max ID Group"),
        &creator,
        &usages,
        &token,
    );
    client.update_members(&max_id, &creator, &members);

    assert!(client.is_group_active(&max_id));
    client.deactivate_payment_group(&max_id, &creator);
    assert!(!client.is_group_active(&max_id));
}

// ─── Test 2 ──────────────────────────────────────────────────────────────────
// Deactivate a group whose ID is all 0x00 bytes (min / zero BytesN<32> value).

#[test]
fn test_deactivate_group_with_zero_id_bytes_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let zero_id = BytesN::from_array(env, &[0x00u8; 32]);
    mint_tokens(env, &token, &creator, 10_010);
    client.create(
        &zero_id,
        &soroban_sdk::String::from_str(env, "Zero ID Group"),
        &creator,
        &1u32,
        &token,
    );
    client.update_members(&zero_id, &creator, &members);

    client.deactivate_payment_group(&zero_id, &creator);
    assert!(!client.is_group_active(&zero_id));
}

// ─── Test 3 ──────────────────────────────────────────────────────────────────
// Deactivate a group that has the maximum allowed members (50 × 2%).

#[test]
fn test_deactivate_group_with_max_members_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &Vec::new(env),
        1,
        &token,
    );

    let mut members = Vec::new(env);
    for _ in 0..50u32 {
        members.push_back(GroupMember {
            address: Address::generate(env),
            percentage: 2,
        });
    }
    client.update_members(&id, &creator, &members);

    assert_eq!(client.get_group_member_count(&id), 50);
    client.deactivate_payment_group(&id, &creator);
    assert!(!client.is_group_active(&id));
    assert!(!client.get_group_summary(&id).is_active);
}

// ─── Test 4 ──────────────────────────────────────────────────────────────────
// Deactivate a group that has 0 remaining usages (exhausted subscription).

#[test]
fn test_deactivate_group_with_zero_remaining_usages_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    // Create with 1 usage, distribute once to exhaust it
    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(env, &token, &sender, 1_000);
    client.distribute(&id, &token, &100, &sender);

    assert_eq!(client.get_remaining_usages(&id), 0);

    // Group is still active — deactivation must succeed even with 0 usages
    client.deactivate_payment_group(&id, &creator);
    assert!(!client.is_group_active(&id));
}

// ─── Test 5 ──────────────────────────────────────────────────────────────────
// Deactivate a group that has u32::MAX usages (overflow boundary check).

#[test]
fn test_deactivate_group_with_max_usages_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    // u32::MAX usages would require enormous funds; use a large but feasible value instead
    // to test the high-end boundary without overflowing i128 token arithmetic.
    let large_usages: u32 = 100_000;
    let fee: i128 = 10;
    mint_tokens(env, &token, &creator, fee * large_usages as i128 + 10_000);

    let mut id_bytes = [0u8; 32];
    id_bytes[0..4].copy_from_slice(&large_usages.to_be_bytes());
    let id = BytesN::from_array(env, &id_bytes);

    client.create(
        &id,
        &soroban_sdk::String::from_str(env, "Large Usages Group"),
        &creator,
        &large_usages,
        &token,
    );
    client.update_members(&id, &creator, &members);

    assert_eq!(client.get_remaining_usages(&id), large_usages);
    client.deactivate_payment_group(&id, &creator);
    assert!(!client.is_group_active(&id));
    // Remaining usages must be unchanged after deactivation
    assert_eq!(client.get_remaining_usages(&id), large_usages);
}

// ─── Test 6 ──────────────────────────────────────────────────────────────────
// State revert: after a successful deactivation, re-activating and then
// verifying the group is truly active again confirms state is not permanently
// corrupted (round-trip integrity check).

#[test]
fn test_deactivate_state_is_reversible_via_activate() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    let usages_before = client.get_remaining_usages(&id);
    let member_count_before = client.get_group_member_count(&id);

    client.deactivate_payment_group(&id, &creator);
    assert!(!client.is_group_active(&id));

    // Re-activate — state must be fully restored
    client.activate_group(&id, &creator);
    assert!(client.is_group_active(&id));
    assert_eq!(client.get_remaining_usages(&id), usages_before);
    assert_eq!(client.get_group_member_count(&id), member_count_before);
}

// ─── Test 7 ──────────────────────────────────────────────────────────────────
// Contract-paused state blocks deactivate_payment_group.

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_deactivate_payment_group_blocked_when_contract_paused() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );

    client.pause(&test_env.admin);
    // Must panic with ContractPaused
    client.deactivate_payment_group(&id, &creator);
}

// ─── Test 8 ──────────────────────────────────────────────────────────────────
// Deactivation is durable: re-reading the group after deactivation still shows
// is_active == false (no accidental state revert on subsequent reads).

#[test]
fn test_deactivated_state_persists_across_multiple_reads() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        5,
        &token,
    );

    client.deactivate_payment_group(&id, &creator);

    // Read via every available query path — all must agree
    assert!(!client.is_group_active(&id));
    assert!(!client.get(&id).is_active);
    assert!(!client.get_group_summary(&id).is_active);

    let inactive = client.get_inactive_groups();
    let found = inactive.iter().any(|gid| gid == id);
    assert!(
        found,
        "Deactivated group must appear in get_inactive_groups"
    );

    let active_groups = client.get_active_groups();
    let still_active = active_groups.iter().any(|g| g.id == id);
    assert!(
        !still_active,
        "Deactivated group must not appear in get_active_groups"
    );
}

// ─── Test 9 ──────────────────────────────────────────────────────────────────
// Deactivating one group must not affect the active status of sibling groups.

#[test]
fn test_deactivate_one_group_does_not_affect_sibling_groups() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id_a = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
    let id_b = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );
    let id_c = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    // Deactivate only group B
    client.deactivate_payment_group(&id_b, &creator);

    assert!(client.is_group_active(&id_a), "Group A must remain active");
    assert!(!client.is_group_active(&id_b), "Group B must be inactive");
    assert!(client.is_group_active(&id_c), "Group C must remain active");
}

// ─── Test 10 ─────────────────────────────────────────────────────────────────
// Topup subscription then deactivate: deactivation succeeds and the topped-up
// usage count is preserved unchanged in storage.

#[test]
fn test_deactivate_after_topup_preserves_usage_count() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        5,
        &token,
    );

    // Top up with 10 more usages
    let topup_amount: u32 = 10;
    mint_tokens(env, &token, &creator, 10 * topup_amount as i128);
    client.topup_subscription(&id, &topup_amount, &token, &creator);

    let usages_after_topup = client.get_remaining_usages(&id);
    assert_eq!(usages_after_topup, 15); // 5 original + 10 topped up

    client.deactivate_payment_group(&id, &creator);

    assert!(!client.is_group_active(&id));
    // Usage count must be untouched by deactivation
    assert_eq!(client.get_remaining_usages(&id), 15);
}

// ─── Test 11 ─────────────────────────────────────────────────────────────────
// Double-deactivation via deactivate_payment_group returns GroupAlreadyInactive
// (same as the base deactivate_group alias).

#[test]
#[should_panic(expected = "GroupAlreadyInactive")]
fn test_double_deactivate_payment_group_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );

    client.deactivate_payment_group(&id, &creator);
    client.deactivate_payment_group(&id, &creator); // must panic
}

// ─── Test 12 ─────────────────────────────────────────────────────────────────
// Deactivate then re-activate then deactivate again — full lifecycle round-trip.

#[test]
fn test_deactivate_reactivate_deactivate_round_trip() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    client.deactivate_payment_group(&id, &creator);
    assert!(!client.is_group_active(&id));

    client.activate_group(&id, &creator);
    assert!(client.is_group_active(&id));

    client.deactivate_payment_group(&id, &creator);
    assert!(!client.is_group_active(&id));
}

// ─── Test 13 ─────────────────────────────────────────────────────────────────
// Deactivating a group with a non-existent (random) ID returns NotFound.

#[test]
#[should_panic(expected = "NotFound")]
fn test_deactivate_nonexistent_group_id_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let caller = test_env.users.get(0).unwrap().clone();
    // Alternating byte pattern — highly unlikely to collide with any real group
    let ghost_id = BytesN::from_array(env, &[0xABu8; 32]);

    client.deactivate_payment_group(&ghost_id, &caller);
}

// ─── Test 14 ─────────────────────────────────────────────────────────────────
// Deactivation blocks update_members (GroupInactive) — state revert check.

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_deactivate_blocks_update_members() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = two_member_50_50(env);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );

    client.deactivate_payment_group(&id, &creator);

    let new_members = two_member_50_50(env);
    client.update_members(&id, &creator, &new_members); // must panic
}

// ─── Test 15 ─────────────────────────────────────────────────────────────────
// Deactivation blocks remove_group_member (GroupInactive).

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_deactivate_blocks_remove_group_member() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let member_addr = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: member_addr.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: Address::generate(env),
        percentage: 50,
    });

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );

    client.deactivate_payment_group(&id, &creator);
    client.remove_group_member(&id, &creator, &member_addr); // must panic
}
