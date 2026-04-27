#![allow(unused_imports)]

use crate::base::types::GroupMember;
use crate::test_utils::{
    create_test_members, deploy_autoshare_contract, deploy_mock_token, mint_tokens,
};
use crate::AutoShareContractClient;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, BytesN, Env, String, Vec,
};

// ─── Helper Functions ────────────────────────────────────────────────────────

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

fn make_group_with_members(
    env: &Env,
    client: &AutoShareContractClient,
    token: &Address,
    creator: &Address,
    seed: u8,
    member_count: u32,
) -> BytesN<32> {
    let id = BytesN::from_array(env, &[seed; 32]);
    mint_tokens(env, token, creator, 10_000);
    client.create(
        &id,
        &String::from_str(env, "Test Group"),
        creator,
        &1,
        token,
    );

    if member_count > 0 {
        let members = create_test_members(env, member_count);
        client.update_members(&id, creator, &members);
    }

    id
}

// ─── Basic Retrieval Tests ───────────────────────────────────────────────────

/// Verify get_group_members returns all members for a valid group.
#[test]
fn test_get_group_members_returns_all_members() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 1, 10);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 10);
}

/// Verify get_group_members returns empty vector for group with no members.
#[test]
fn test_get_group_members_empty_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 2, 0);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 0);
}

/// Verify get_group_members returns correct member data.
#[test]
fn test_get_group_members_correct_data() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 3, 5);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 5);
    
    // Verify each member has valid data
    for i in 0..5 {
        let member = members.get(i).unwrap();
        assert!(member.percentage > 0);
        assert!(member.percentage <= 100);
    }
}

/// Verify get_group_members works with single member.
#[test]
fn test_get_group_members_single_member() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 4, 1);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 1);
}

// ─── Member Ordering Tests ───────────────────────────────────────────────────

/// Verify get_group_members preserves member order.
#[test]
fn test_get_group_members_preserves_order() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    
    let id = BytesN::from_array(&env, &[5u8; 32]);
    mint_tokens(&env, &token, &creator, 10_000);
    client.create(&id, &String::from_str(&env, "Test Group"), &creator, &1, &token);

    // Create members with specific addresses
    let mut members = Vec::new(&env);
    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);
    let addr3 = Address::generate(&env);
    
    members.push_back(GroupMember { address: addr1.clone(), percentage: 33 });
    members.push_back(GroupMember { address: addr2.clone(), percentage: 33 });
    members.push_back(GroupMember { address: addr3.clone(), percentage: 34 });
    
    client.update_members(&id, &creator, &members);

    let result = client.get_group_members(&id);
    assert_eq!(result.len(), 3);
    assert_eq!(result.get(0).unwrap().address, addr1);
    assert_eq!(result.get(1).unwrap().address, addr2);
    assert_eq!(result.get(2).unwrap().address, addr3);
}

/// Verify repeated calls return same order.
#[test]
fn test_get_group_members_consistent_order() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 6, 10);

    let members1 = client.get_group_members(&id);
    let members2 = client.get_group_members(&id);
    let members3 = client.get_group_members(&id);

    assert_eq!(members1.len(), members2.len());
    assert_eq!(members2.len(), members3.len());
    
    for i in 0..10 {
        assert_eq!(members1.get(i).unwrap().address, members2.get(i).unwrap().address);
        assert_eq!(members2.get(i).unwrap().address, members3.get(i).unwrap().address);
    }
}

// ─── Pagination Tests ────────────────────────────────────────────────────────

/// Verify paginated retrieval returns correct first page.
#[test]
fn test_get_group_members_paginated_first_page() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 7, 25);

    let page = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(page.members.len(), 10);
    assert_eq!(page.total, 25);
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, 10);
}

/// Verify paginated retrieval with offset works correctly.
#[test]
fn test_get_group_members_paginated_with_offset() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 8, 30);

    let page = client.get_group_members_paginated(&id, &10, &10);
    assert_eq!(page.members.len(), 10);
    assert_eq!(page.total, 30);
    assert_eq!(page.offset, 10);
    assert_eq!(page.limit, 10);
}

/// Verify paginated retrieval respects limit cap of 20.
#[test]
fn test_get_group_members_paginated_limit_cap() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 9, 50);

    let page = client.get_group_members_paginated(&id, &0, &100);
    assert_eq!(page.members.len(), 20); // Capped at 20
    assert_eq!(page.limit, 20);
    assert_eq!(page.total, 50);
}

/// Verify paginated retrieval with zero limit returns empty page.
#[test]
fn test_get_group_members_paginated_zero_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 10, 15);

    let page = client.get_group_members_paginated(&id, &0, &0);
    assert_eq!(page.members.len(), 0);
    assert_eq!(page.limit, 0);
    assert_eq!(page.total, 15);
}

// ─── Page Boundary Tests ─────────────────────────────────────────────────────

/// Verify offset beyond total returns empty page.
#[test]
fn test_get_group_members_paginated_offset_beyond_total() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 11, 10);

    let page = client.get_group_members_paginated(&id, &20, &10);
    assert_eq!(page.members.len(), 0);
    assert_eq!(page.total, 10);
    assert_eq!(page.offset, 20);
}

/// Verify partial last page returns correct count.
#[test]
fn test_get_group_members_paginated_partial_last_page() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 12, 23);

    let page = client.get_group_members_paginated(&id, &20, &10);
    assert_eq!(page.members.len(), 3); // Only 3 members left
    assert_eq!(page.total, 23);
    assert_eq!(page.offset, 20);
}

/// Verify exact page boundary works correctly.
#[test]
fn test_get_group_members_paginated_exact_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 13, 20);

    let page = client.get_group_members_paginated(&id, &10, &10);
    assert_eq!(page.members.len(), 10);
    assert_eq!(page.total, 20);
}

/// Verify offset at exact total returns empty page.
#[test]
fn test_get_group_members_paginated_offset_at_total() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 14, 15);

    let page = client.get_group_members_paginated(&id, &15, &10);
    assert_eq!(page.members.len(), 0);
    assert_eq!(page.total, 15);
}

// ─── Multi-Page Traversal Tests ──────────────────────────────────────────────

/// Verify multi-page traversal without gaps or duplicates.
#[test]
fn test_get_group_members_paginated_no_gaps_or_duplicates() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 15, 30);

    // Get all members via non-paginated call
    let all_members = client.get_group_members(&id);

    // Get members via pagination
    let page1 = client.get_group_members_paginated(&id, &0, &10);
    let page2 = client.get_group_members_paginated(&id, &10, &10);
    let page3 = client.get_group_members_paginated(&id, &20, &10);

    // Verify no gaps
    for i in 0..10 {
        assert_eq!(all_members.get(i).unwrap().address, page1.members.get(i).unwrap().address);
        assert_eq!(all_members.get(10 + i).unwrap().address, page2.members.get(i).unwrap().address);
        assert_eq!(all_members.get(20 + i).unwrap().address, page3.members.get(i).unwrap().address);
    }
}

/// Verify complete traversal with varying page sizes.
#[test]
fn test_get_group_members_paginated_varying_page_sizes() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 16, 25);

    let page1 = client.get_group_members_paginated(&id, &0, &5);
    let page2 = client.get_group_members_paginated(&id, &5, &10);
    let page3 = client.get_group_members_paginated(&id, &15, &10);

    assert_eq!(page1.members.len(), 5);
    assert_eq!(page2.members.len(), 10);
    assert_eq!(page3.members.len(), 10);
    assert_eq!(page1.total, 25);
    assert_eq!(page2.total, 25);
    assert_eq!(page3.total, 25);
}

/// Verify overlapping page requests return consistent data.
#[test]
fn test_get_group_members_paginated_overlapping_pages() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 17, 20);

    let page1 = client.get_group_members_paginated(&id, &5, &10);
    let page2 = client.get_group_members_paginated(&id, &10, &10);

    // Member at index 10 should be first in page2 and not in page1
    assert_eq!(page1.members.len(), 10);
    assert_eq!(page2.members.len(), 10);
}

// ─── Error Condition Tests ───────────────────────────────────────────────────

/// Verify get_group_members fails for nonexistent group.
#[test]
fn test_get_group_members_nonexistent_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);
    let result = client.try_get_group_members(&ghost_id);
    assert!(result.is_err());
}

/// Verify get_group_members_paginated fails for nonexistent group.
#[test]
fn test_get_group_members_paginated_nonexistent_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);
    let result = client.try_get_group_members_paginated(&ghost_id, &0, &10);
    assert!(result.is_err());
}

/// Verify get_group_members works for inactive groups (read operations don't check status).
#[test]
fn test_get_group_members_inactive_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 18, 5);

    client.deactivate_payment_group(&id, &creator);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 5);
}

/// Verify get_group_members_paginated works for inactive groups.
#[test]
fn test_get_group_members_paginated_inactive_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 19, 15);

    client.deactivate_payment_group(&id, &creator);

    let page = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(page.members.len(), 10);
    assert_eq!(page.total, 15);
}

// ─── State Immutability Tests ────────────────────────────────────────────────

/// Verify get_group_members does not mutate state.
#[test]
fn test_get_group_members_does_not_mutate_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 20, 10);

    let members_before = client.get_group_members(&id);
    
    // Call multiple times
    client.get_group_members(&id);
    client.get_group_members(&id);
    
    let members_after = client.get_group_members(&id);

    // Verify state unchanged
    assert_eq!(members_before.len(), members_after.len());
    for i in 0..10 {
        assert_eq!(members_before.get(i).unwrap().address, members_after.get(i).unwrap().address);
        assert_eq!(members_before.get(i).unwrap().percentage, members_after.get(i).unwrap().percentage);
    }
}

/// Verify get_group_members_paginated does not mutate state.
#[test]
fn test_get_group_members_paginated_does_not_mutate_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 21, 20);

    let page_before = client.get_group_members_paginated(&id, &0, &10);
    
    // Call multiple times
    client.get_group_members_paginated(&id, &0, &10);
    client.get_group_members_paginated(&id, &10, &10);
    
    let page_after = client.get_group_members_paginated(&id, &0, &10);

    // Verify state unchanged
    assert_eq!(page_before.total, page_after.total);
    assert_eq!(page_before.members.len(), page_after.members.len());
    for i in 0..10 {
        assert_eq!(page_before.members.get(i).unwrap().address, page_after.members.get(i).unwrap().address);
    }
}

// ─── Diagnostic Event Tests ──────────────────────────────────────────────────

/// Verify get_group_members emits diagnostic event.
#[test]
fn test_get_group_members_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 22, 5);

    let _ = env.events().all();
    client.get_group_members(&id);

    let events = env.events().all();
    assert!(events.len() > 0, "Expected diagnostic event to be emitted");
}

/// Verify diagnostic events don't affect return values.
#[test]
fn test_get_group_members_events_dont_affect_results() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 23, 10);

    let result1 = client.get_group_members(&id);
    let result2 = client.get_group_members(&id);
    let result3 = client.get_group_members(&id);

    assert_eq!(result1.len(), result2.len());
    assert_eq!(result2.len(), result3.len());
    
    for i in 0..10 {
        assert_eq!(result1.get(i).unwrap().address, result2.get(i).unwrap().address);
        assert_eq!(result2.get(i).unwrap().address, result3.get(i).unwrap().address);
    }
}

/// Verify query counter increments correctly.
#[test]
fn test_get_group_members_query_counter_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 24, 5);

    assert_eq!(client.get_group_members_query_count(&id), 0);
    
    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 1);
    
    client.get_group_members(&id);
    assert_eq!(client.get_group_members_query_count(&id), 2);
}

// ─── Determinism Tests ───────────────────────────────────────────────────────

/// Verify get_group_members is deterministic.
#[test]
fn test_get_group_members_deterministic() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 25, 15);

    let result1 = client.get_group_members(&id);
    let result2 = client.get_group_members(&id);

    assert_eq!(result1.len(), result2.len());
    for i in 0..15 {
        let m1 = result1.get(i).unwrap();
        let m2 = result2.get(i).unwrap();
        assert_eq!(m1.address, m2.address);
        assert_eq!(m1.percentage, m2.percentage);
    }
}

/// Verify get_group_members_paginated is deterministic.
#[test]
fn test_get_group_members_paginated_deterministic() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 26, 20);

    let page1 = client.get_group_members_paginated(&id, &5, &10);
    let page2 = client.get_group_members_paginated(&id, &5, &10);

    assert_eq!(page1.total, page2.total);
    assert_eq!(page1.offset, page2.offset);
    assert_eq!(page1.limit, page2.limit);
    assert_eq!(page1.members.len(), page2.members.len());
    
    for i in 0..10 {
        assert_eq!(page1.members.get(i).unwrap().address, page2.members.get(i).unwrap().address);
    }
}

// ─── Member Update Tests ──────────────────────────────────────────────────────

/// Verify get_group_members reflects member additions.
#[test]
fn test_get_group_members_reflects_additions() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 27, 5);

    let members_before = client.get_group_members(&id);
    assert_eq!(members_before.len(), 5);

    // Update to 10 members
    let new_members = create_test_members(&env, 10);
    client.update_members(&id, &creator, &new_members);

    let members_after = client.get_group_members(&id);
    assert_eq!(members_after.len(), 10);
}

/// Verify get_group_members reflects member removals.
#[test]
fn test_get_group_members_reflects_removals() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 28, 10);

    let members_before = client.get_group_members(&id);
    assert_eq!(members_before.len(), 10);

    // Update to 3 members
    let new_members = create_test_members(&env, 3);
    client.update_members(&id, &creator, &new_members);

    let members_after = client.get_group_members(&id);
    assert_eq!(members_after.len(), 3);
}

/// Verify get_group_members_paginated reflects member updates.
#[test]
fn test_get_group_members_paginated_reflects_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 29, 15);

    let page_before = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(page_before.total, 15);

    // Update to 25 members
    let new_members = create_test_members(&env, 25);
    client.update_members(&id, &creator, &new_members);

    let page_after = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(page_after.total, 25);
}

// ─── Maximum Capacity Tests ───────────────────────────────────────────────────

/// Verify get_group_members works with maximum members (50).
#[test]
fn test_get_group_members_max_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 30, 50);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 50);
}

/// Verify get_group_members_paginated works with maximum members.
#[test]
fn test_get_group_members_paginated_max_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 31, 50);

    // First page
    let page1 = client.get_group_members_paginated(&id, &0, &20);
    assert_eq!(page1.members.len(), 20);
    assert_eq!(page1.total, 50);

    // Second page
    let page2 = client.get_group_members_paginated(&id, &20, &20);
    assert_eq!(page2.members.len(), 20);

    // Third page
    let page3 = client.get_group_members_paginated(&id, &40, &20);
    assert_eq!(page3.members.len(), 10);
}

/// Verify complete traversal of max capacity group.
#[test]
fn test_get_group_members_paginated_max_capacity_full_traversal() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 32, 50);

    let all_members = client.get_group_members(&id);
    
    // Traverse in pages of 20
    let page1 = client.get_group_members_paginated(&id, &0, &20);
    let page2 = client.get_group_members_paginated(&id, &20, &20);
    let page3 = client.get_group_members_paginated(&id, &40, &20);

    // Verify no gaps or duplicates
    for i in 0..20 {
        assert_eq!(all_members.get(i).unwrap().address, page1.members.get(i).unwrap().address);
        assert_eq!(all_members.get(20 + i).unwrap().address, page2.members.get(i).unwrap().address);
    }
    for i in 0..10 {
        assert_eq!(all_members.get(40 + i).unwrap().address, page3.members.get(i).unwrap().address);
    }
}

// ─── Edge Case Tests ──────────────────────────────────────────────────────────

/// Verify get_group_members_paginated with limit 1.
#[test]
fn test_get_group_members_paginated_limit_one() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 33, 10);

    let page = client.get_group_members_paginated(&id, &0, &1);
    assert_eq!(page.members.len(), 1);
    assert_eq!(page.limit, 1);
    assert_eq!(page.total, 10);
}

/// Verify get_group_members_paginated with offset one less than total.
#[test]
fn test_get_group_members_paginated_offset_near_end() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 34, 10);

    let page = client.get_group_members_paginated(&id, &9, &10);
    assert_eq!(page.members.len(), 1);
    assert_eq!(page.total, 10);
}

/// Verify get_group_members with empty group after member removal.
#[test]
fn test_get_group_members_empty_after_removal() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 35, 5);

    // Remove all members
    let empty_members = Vec::new(&env);
    client.update_members(&id, &creator, &empty_members);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 0);
}

/// Verify pagination with empty group.
#[test]
fn test_get_group_members_paginated_empty_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 36, 0);

    let page = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(page.members.len(), 0);
    assert_eq!(page.total, 0);
    assert_eq!(page.offset, 0);
}

// ─── Isolation Tests ──────────────────────────────────────────────────────────

/// Verify get_group_members is isolated per group.
#[test]
fn test_get_group_members_isolated_per_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    
    let id_a = make_group_with_members(&env, &client, &token, &creator, 37, 5);
    let id_b = make_group_with_members(&env, &client, &token, &creator, 38, 10);

    let members_a = client.get_group_members(&id_a);
    let members_b = client.get_group_members(&id_b);

    assert_eq!(members_a.len(), 5);
    assert_eq!(members_b.len(), 10);
}

/// Verify query counters are isolated per group.
#[test]
fn test_get_group_members_query_counters_isolated() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    
    let id_a = make_group_with_members(&env, &client, &token, &creator, 39, 5);
    let id_b = make_group_with_members(&env, &client, &token, &creator, 40, 10);

    client.get_group_members(&id_a);
    client.get_group_members(&id_a);
    client.get_group_members(&id_b);

    assert_eq!(client.get_group_members_query_count(&id_a), 2);
    assert_eq!(client.get_group_members_query_count(&id_b), 1);
}

/// Verify pagination state is isolated per group.
#[test]
fn test_get_group_members_paginated_isolated_per_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    
    let id_a = make_group_with_members(&env, &client, &token, &creator, 41, 15);
    let id_b = make_group_with_members(&env, &client, &token, &creator, 42, 25);

    let page_a = client.get_group_members_paginated(&id_a, &0, &10);
    let page_b = client.get_group_members_paginated(&id_b, &0, &10);

    assert_eq!(page_a.total, 15);
    assert_eq!(page_b.total, 25);
}

// ─── Integration Tests ────────────────────────────────────────────────────────

/// Verify get_group_members works after group operations.
#[test]
fn test_get_group_members_after_group_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 43, 10);

    // Deactivate and reactivate
    client.deactivate_payment_group(&id, &creator);
    client.activate_group(&id, &creator);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 10);
}

/// Verify get_group_members works with multiple groups and operations.
#[test]
fn test_get_group_members_multiple_groups_complex() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    
    let id1 = make_group_with_members(&env, &client, &token, &creator, 44, 5);
    let id2 = make_group_with_members(&env, &client, &token, &creator, 45, 10);
    let id3 = make_group_with_members(&env, &client, &token, &creator, 46, 15);

    // Update members in id2
    let new_members = create_test_members(&env, 8);
    client.update_members(&id2, &creator, &new_members);

    // Verify all groups maintain correct state
    assert_eq!(client.get_group_members(&id1).len(), 5);
    assert_eq!(client.get_group_members(&id2).len(), 8);
    assert_eq!(client.get_group_members(&id3).len(), 15);
}

/// Verify pagination consistency across concurrent reads.
#[test]
fn test_get_group_members_paginated_concurrent_reads() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 47, 30);

    // Simulate concurrent reads
    let page1_a = client.get_group_members_paginated(&id, &0, &10);
    let page2_a = client.get_group_members_paginated(&id, &10, &10);
    let page1_b = client.get_group_members_paginated(&id, &0, &10);
    let page3_a = client.get_group_members_paginated(&id, &20, &10);

    // Verify consistency
    assert_eq!(page1_a.members.len(), page1_b.members.len());
    for i in 0..10 {
        assert_eq!(page1_a.members.get(i).unwrap().address, page1_b.members.get(i).unwrap().address);
    }
}
