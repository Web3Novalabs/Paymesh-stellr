use super::test_utils::{create_test_group, mint_tokens, setup_test_env};
use crate::base::types::GroupMember;
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

// ============================================================================
// Helper: build a two-member Vec summing to 100%
// ============================================================================

fn two_members(env: &soroban_sdk::Env, a: &Address, pct_a: u32, b: &Address, pct_b: u32) -> Vec<GroupMember> {
    let mut members = Vec::new(env);
    members.push_back(GroupMember { address: a.clone(), percentage: pct_a });
    members.push_back(GroupMember { address: b.clone(), percentage: pct_b });
    members
}

// ============================================================================
// Test 1: member with earnings across multiple groups
// ============================================================================

#[test]
fn test_breakdown_returns_earnings_for_all_groups() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member = Address::generate(&env);
    let other = Address::generate(&env);

    let creator = test_env.users.get(0).unwrap().clone();
    let sender = test_env.users.get(1).unwrap().clone();

    // Create two groups where `member` has different percentage splits.
    let members_a = two_members(&env, &member, 60, &other, 40);
    let members_b = two_members(&env, &member, 25, &other, 75);

    let group_a = create_test_group(&env, &contract, &creator, &members_a, 5, &token);
    let group_b = create_test_group(&env, &contract, &creator, &members_b, 6, &token);

    // Distribute into both groups.
    mint_tokens(&env, &token, &sender, 1000);
    client.distribute(&group_a, &token, &1000, &sender); // member gets 600

    mint_tokens(&env, &token, &sender, 400);
    client.distribute(&group_b, &token, &400, &sender);  // member gets 100

    let breakdown = client.get_member_earnings_breakdown(&member);

    // Both groups should appear (both have earnings > 0).
    assert_eq!(breakdown.len(), 2);

    // Verify each (group_id, earnings) pair regardless of order.
    let mut found_a = false;
    let mut found_b = false;
    for i in 0..breakdown.len() {
        let (gid, amt) = breakdown.get(i).unwrap();
        if gid == group_a {
            assert_eq!(amt, 600);
            found_a = true;
        } else if gid == group_b {
            assert_eq!(amt, 100);
            found_b = true;
        }
    }
    assert!(found_a, "group_a not in breakdown");
    assert!(found_b, "group_b not in breakdown");
}

// ============================================================================
// Test 2: groups with zero earnings are filtered out
// ============================================================================

#[test]
fn test_breakdown_filters_out_zero_earnings_groups() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member = Address::generate(&env);
    let other = Address::generate(&env);

    let creator = test_env.users.get(0).unwrap().clone();
    let sender = test_env.users.get(1).unwrap().clone();

    let members_a = two_members(&env, &member, 50, &other, 50);
    let members_b = two_members(&env, &member, 50, &other, 50);

    // group_a will receive a distribution; group_b will not.
    let group_a = create_test_group(&env, &contract, &creator, &members_a, 5, &token);
    let _group_b = create_test_group(&env, &contract, &creator, &members_b, 6, &token);

    mint_tokens(&env, &token, &sender, 200);
    client.distribute(&group_a, &token, &200, &sender); // member gets 100 from group_a

    let breakdown = client.get_member_earnings_breakdown(&member);

    // Only group_a should appear; group_b has zero earnings and must be filtered.
    assert_eq!(breakdown.len(), 1);
    let (gid, amt) = breakdown.get(0).unwrap();
    assert_eq!(gid, group_a);
    assert_eq!(amt, 100);
}

// ============================================================================
// Test 3: member with no groups at all returns empty Vec
// ============================================================================

#[test]
fn test_breakdown_empty_for_member_with_no_groups() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let client = AutoShareContractClient::new(&env, &contract);

    // An address that has never been added to any group.
    let stranger = Address::generate(&env);

    let breakdown = client.get_member_earnings_breakdown(&stranger);
    assert_eq!(breakdown.len(), 0);
}

// ============================================================================
// Test 4: member in groups but zero earnings everywhere returns empty Vec
// ============================================================================

#[test]
fn test_breakdown_empty_when_member_has_zero_earnings_everywhere() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member = Address::generate(&env);
    let other = Address::generate(&env);
    let creator = test_env.users.get(0).unwrap().clone();

    let members = two_members(&env, &member, 70, &other, 30);

    // Add member to a group but never distribute — earnings remain 0.
    let _group = create_test_group(&env, &contract, &creator, &members, 5, &token);

    let breakdown = client.get_member_earnings_breakdown(&member);
    assert_eq!(breakdown.len(), 0);
}

// ============================================================================
// Test 5: earnings accumulate correctly across multiple distributions
// ============================================================================

#[test]
fn test_breakdown_reflects_cumulative_earnings() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member = Address::generate(&env);
    let other = Address::generate(&env);
    let creator = test_env.users.get(0).unwrap().clone();
    let sender = test_env.users.get(1).unwrap().clone();

    let members = two_members(&env, &member, 50, &other, 50);
    let group = create_test_group(&env, &contract, &creator, &members, 10, &token);

    // Three distributions — cumulative earnings should be 500 + 250 + 750 = 1500
    mint_tokens(&env, &token, &sender, 1000);
    client.distribute(&group, &token, &1000, &sender);

    mint_tokens(&env, &token, &sender, 500);
    client.distribute(&group, &token, &500, &sender);

    mint_tokens(&env, &token, &sender, 1500);
    client.distribute(&group, &token, &1500, &sender);

    let breakdown = client.get_member_earnings_breakdown(&member);

    assert_eq!(breakdown.len(), 1);
    let (gid, amt) = breakdown.get(0).unwrap();
    assert_eq!(gid, group);
    assert_eq!(amt, 1500); // 500 + 250 + 750
}

// ============================================================================
// Test 6: function is read-only — calling it does not alter earnings
// ============================================================================

#[test]
fn test_breakdown_does_not_modify_state() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member = Address::generate(&env);
    let other = Address::generate(&env);
    let creator = test_env.users.get(0).unwrap().clone();
    let sender = test_env.users.get(1).unwrap().clone();

    let members = two_members(&env, &member, 80, &other, 20);
    let group = create_test_group(&env, &contract, &creator, &members, 5, &token);

    mint_tokens(&env, &token, &sender, 100);
    client.distribute(&group, &token, &100, &sender); // member earns 80

    // Call breakdown twice — earnings must remain 80 both times.
    let first = client.get_member_earnings_breakdown(&member);
    let second = client.get_member_earnings_breakdown(&member);

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);

    let (_, amt1) = first.get(0).unwrap();
    let (_, amt2) = second.get(0).unwrap();
    assert_eq!(amt1, 80);
    assert_eq!(amt2, 80);

    // Also verify via get_member_earnings that underlying storage is unchanged.
    assert_eq!(client.get_member_earnings(&member, &group), 80);
}
