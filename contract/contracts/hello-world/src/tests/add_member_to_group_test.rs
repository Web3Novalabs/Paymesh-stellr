
use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

// ─── Test 1 ──────────────────────────────────────────────────────────────────
// Happy path: add a single member to a group with no existing members.

#[test]
fn test_add_member_to_group_success() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 5, &token);

    client.add_member_to_group(&id, &creator, &new_member, &100);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 1);
    assert_eq!(members.get(0).unwrap().address, new_member);
    assert_eq!(members.get(0).unwrap().percentage, 100);
}

// ─── Test 2 ──────────────────────────────────────────────────────────────────
// Add multiple members sequentially; running total must not exceed 100.

#[test]
fn test_add_member_to_group_sequential_adds_succeed() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 5, &token);

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let m3 = Address::generate(env);

    client.add_member_to_group(&id, &creator, &m1, &40);
    client.add_member_to_group(&id, &creator, &m2, &35);
    client.add_member_to_group(&id, &creator, &m3, &25);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 3);
    assert_eq!(client.get_member_percentage(&id, &m1), 40);
    assert_eq!(client.get_member_percentage(&id, &m2), 35);
    assert_eq!(client.get_member_percentage(&id, &m3), 25);
}

// ─── Test 3 ──────────────────────────────────────────────────────────────────
// MemberGroups index is updated: new member appears in get_groups_by_member.

#[test]
fn test_add_member_to_group_updates_member_groups_index() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 3, &token);

    client.add_member_to_group(&id, &creator, &new_member, &100);

    let groups = client.get_groups_by_member(&new_member);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups.get(0).unwrap().id, id);
}

// ─── Test 4 ──────────────────────────────────────────────────────────────────
// Unauthorized caller (non-creator) must fail with Unauthorized.

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_add_member_to_group_unauthorized_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let attacker = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 3, &token);

    client.add_member_to_group(&id, &attacker, &new_member, &100);
}

// ─── Test 5 ──────────────────────────────────────────────────────────────────
// Non-existent group ID must fail with NotFound.

#[test]
#[should_panic(expected = "NotFound")]
fn test_add_member_to_group_nonexistent_group_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let caller = test_env.users.get(0).unwrap().clone();
    let new_member = Address::generate(env);
    let ghost_id = soroban_sdk::BytesN::from_array(env, &[0xDEu8; 32]);

    client.add_member_to_group(&ghost_id, &caller, &new_member, &100);
}

// ─── Test 6 ──────────────────────────────────────────────────────────────────
// Inactive group must fail with GroupInactive.

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_add_member_to_group_inactive_group_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 2, &token);
    client.deactivate_payment_group(&id, &creator);

    client.add_member_to_group(&id, &creator, &new_member, &100);
}

// ─── Test 7 ──────────────────────────────────────────────────────────────────
// Percentage = 0 must fail with InvalidInput.

#[test]
#[should_panic(expected = "InvalidInput")]
fn test_add_member_to_group_zero_percentage_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 2, &token);

    client.add_member_to_group(&id, &creator, &new_member, &0);
}

// ─── Test 8 ──────────────────────────────────────────────────────────────────
// Percentage > 100 must fail with InvalidInput.

#[test]
#[should_panic(expected = "InvalidInput")]
fn test_add_member_to_group_percentage_over_100_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 2, &token);

    client.add_member_to_group(&id, &creator, &new_member, &101);
}

// ─── Test 9 ──────────────────────────────────────────────────────────────────
// Adding a member that would push the running total over 100 must fail with
// InvalidTotalPercentage.

#[test]
#[should_panic(expected = "InvalidTotalPercentage")]
fn test_add_member_to_group_exceeds_total_percentage_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 3, &token);

    client.add_member_to_group(&id, &creator, &Address::generate(env), &60);
    // 60 + 50 = 110 > 100 → InvalidTotalPercentage
    client.add_member_to_group(&id, &creator, &Address::generate(env), &50);
}

// ─── Test 10 ─────────────────────────────────────────────────────────────────
// Adding a duplicate member address must fail with AlreadyExists.

#[test]
#[should_panic(expected = "AlreadyExists")]
fn test_add_member_to_group_duplicate_member_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 3, &token);

    client.add_member_to_group(&id, &creator, &member, &50);
    // Same address again → AlreadyExists
    client.add_member_to_group(&id, &creator, &member, &30);
}

// ─── Test 11 ─────────────────────────────────────────────────────────────────
// Capacity limit: adding a member when the group is already at max capacity
// must fail with MaxMembersExceeded.

#[test]
#[should_panic(expected = "MaxMembersExceeded")]
fn test_add_member_to_group_at_capacity_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let admin = client.get_admin();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Set capacity to 2 so the test runs fast
    client.set_max_members(&admin, &2);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 3, &token);

    client.add_member_to_group(&id, &creator, &Address::generate(env), &50);
    client.add_member_to_group(&id, &creator, &Address::generate(env), &50);
    // Group is now at capacity (2/2) → MaxMembersExceeded
    client.add_member_to_group(&id, &creator, &Address::generate(env), &1);
}

// ─── Test 12 ─────────────────────────────────────────────────────────────────
// Contract-paused state blocks add_member_to_group.

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_add_member_to_group_blocked_when_paused() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 2, &token);

    client.pause(&test_env.admin);
    client.add_member_to_group(&id, &creator, &new_member, &100);
}

// ─── Test 13 ─────────────────────────────────────────────────────────────────
// Creator added as a member emits CreatorIsMember event (smoke-test: no panic).

#[test]
fn test_add_member_to_group_creator_as_member_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 2, &token);

    // Creator adding themselves is allowed; a CreatorIsMember event is emitted
    client.add_member_to_group(&id, &creator, &creator, &100);

    assert!(client.is_group_member(&id, &creator));
    assert_eq!(client.get_member_percentage(&id, &creator), 100);
}

// ─── Test 14 ─────────────────────────────────────────────────────────────────
// is_group_member returns true for the newly added member.

#[test]
fn test_add_member_to_group_is_group_member_reflects_addition() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let new_member = Address::generate(env);
    let non_member = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 2, &token);

    assert!(!client.is_group_member(&id, &new_member));
    client.add_member_to_group(&id, &creator, &new_member, &100);
    assert!(client.is_group_member(&id, &new_member));
    assert!(!client.is_group_member(&id, &non_member));
}

// ─── Test 15 ─────────────────────────────────────────────────────────────────
// get_group_member_count increments by 1 after each successful add.

#[test]
fn test_add_member_to_group_increments_member_count() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 5, &token);

    assert_eq!(client.get_group_member_count(&id), 0);

    client.add_member_to_group(&id, &creator, &Address::generate(env), &40);
    assert_eq!(client.get_group_member_count(&id), 1);

    client.add_member_to_group(&id, &creator, &Address::generate(env), &35);
    assert_eq!(client.get_group_member_count(&id), 2);

    client.add_member_to_group(&id, &creator, &Address::generate(env), &25);
    assert_eq!(client.get_group_member_count(&id), 3);
}

// ─── Test 16 ─────────────────────────────────────────────────────────────────
// Adding a member to a group that already has members via update_members works
// correctly, and the MemberGroups index is updated for the new member only.

#[test]
fn test_add_member_to_group_after_update_members_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let existing_member = Address::generate(env);
    let mut initial_members = Vec::new(env);
    initial_members.push_back(GroupMember {
        address: existing_member.clone(),
        percentage: 70,
    });

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &initial_members, 5, &token);

    let new_member = Address::generate(env);
    // 70 + 30 = 100 — valid
    client.add_member_to_group(&id, &creator, &new_member, &30);

    assert_eq!(client.get_group_member_count(&id), 2);
    assert_eq!(client.get_member_percentage(&id, &new_member), 30);

    // Existing member's index must be unchanged
    let existing_groups = client.get_groups_by_member(&existing_member);
    assert_eq!(existing_groups.len(), 1);

    // New member's index must now include this group
    let new_member_groups = client.get_groups_by_member(&new_member);
    assert_eq!(new_member_groups.len(), 1);
    assert_eq!(new_member_groups.get(0).unwrap().id, id);
}

// ─── Test 17 ─────────────────────────────────────────────────────────────────
// Exactly at the percentage boundary: adding a member that brings the total to
// exactly 100 must succeed.

#[test]
fn test_add_member_to_group_exact_100_total_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 3, &token);

    client.add_member_to_group(&id, &creator, &Address::generate(env), &33);
    client.add_member_to_group(&id, &creator, &Address::generate(env), &33);
    // 33 + 33 + 34 = 100 exactly
    client.add_member_to_group(&id, &creator, &Address::generate(env), &34);

    assert_eq!(client.get_group_member_count(&id), 3);
}

// ─── Test 18 ─────────────────────────────────────────────────────────────────
// Distribute succeeds after members are added via add_member_to_group and
// total percentage equals 100.

#[test]
fn test_add_member_to_group_then_distribute_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &Vec::new(env), 5, &token);

    client.add_member_to_group(&id, &creator, &m1, &60);
    client.add_member_to_group(&id, &creator, &m2, &40);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(env, &token, &sender, 1_000);
    client.distribute(&id, &token, &100, &sender);

    assert_eq!(client.get_remaining_usages(&id), 4);
    assert_eq!(client.get_member_earnings(&m1, &id), 60);
    assert_eq!(client.get_member_earnings(&m2, &id), 40);
}
