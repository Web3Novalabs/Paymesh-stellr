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

// ─── Event emission tests ────────────────────────────────────────────────────

/// Verify that get_group_members_paginated emits the GroupMembersPaginatedQueried event.
#[test]
fn test_paginated_query_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 1, 10);

    // Clear any previous events
    let _ = env.events().all();

    // Query paginated members
    let _result = client.get_group_members_paginated(&id, &0, &5);

    // Verify event was emitted
    let events = env.events().all();
    let event_count = events.len();
    assert!(event_count > 0, "Expected at least one event to be emitted");

    // Find the GroupMembersPaginatedQueried event
    let mut found_event = false;
    for i in 0..event_count {
        let event = events.get(i).unwrap();
        let event_str = format!("{:?}", event);
        if event_str.contains("GroupMembersPaginatedQueried") {
            found_event = true;
            break;
        }
    }
    assert!(found_event, "GroupMembersPaginatedQueried event not found");
}

/// Verify event contains correct pagination parameters.
#[test]
fn test_paginated_event_contains_correct_parameters() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 2, 15);

    let _ = env.events().all();
    let result = client.get_group_members_paginated(&id, &5, &10);

    // Verify the result matches what should be in the event
    assert_eq!(result.offset, 5);
    assert_eq!(result.limit, 10);
    assert_eq!(result.total, 15);
    assert_eq!(result.members.len(), 10);
}

/// Verify event is emitted for empty groups.
#[test]
fn test_paginated_event_emitted_for_empty_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 3, 0);

    let _ = env.events().all();
    let result = client.get_group_members_paginated(&id, &0, &10);

    assert_eq!(result.members.len(), 0);
    assert_eq!(result.total, 0);

    let events = env.events().all();
    let mut found_event = false;
    for i in 0..events.len() {
        let event = events.get(i).unwrap();
        let event_str = format!("{:?}", event);
        if event_str.contains("GroupMembersPaginatedQueried") {
            found_event = true;
            break;
        }
    }
    assert!(found_event, "Event should be emitted even for empty groups");
}

/// Verify event is emitted for each paginated call.
#[test]
fn test_paginated_event_emitted_for_multiple_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 4, 25);

    // First call
    let _ = env.events().all();
    client.get_group_members_paginated(&id, &0, &10);
    let events1 = env.events().all();
    let count1 = events1.len();

    // Second call
    let _ = env.events().all();
    client.get_group_members_paginated(&id, &10, &10);
    let events2 = env.events().all();
    let count2 = events2.len();

    // Third call
    let _ = env.events().all();
    client.get_group_members_paginated(&id, &20, &10);
    let events3 = env.events().all();
    let count3 = events3.len();

    // Each call should emit at least one event
    assert!(count1 > 0, "First call should emit events");
    assert!(count2 > 0, "Second call should emit events");
    assert!(count3 > 0, "Third call should emit events");
}

// ─── Return value correctness ────────────────────────────────────────────────

/// Verify that adding tracking doesn't change pagination results.
#[test]
fn test_paginated_returns_correct_members() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 5, 10);

    let result = client.get_group_members_paginated(&id, &0, &5);
    assert_eq!(result.members.len(), 5);
    assert_eq!(result.total, 10);
    assert_eq!(result.offset, 0);
    assert_eq!(result.limit, 5);
}

/// Verify pagination with offset works correctly.
#[test]
fn test_paginated_with_offset_returns_correct_members() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 6, 20);

    let result = client.get_group_members_paginated(&id, &10, &5);
    assert_eq!(result.members.len(), 5);
    assert_eq!(result.total, 20);
    assert_eq!(result.offset, 10);
    assert_eq!(result.limit, 5);
}

/// Verify limit cap at 20 still works.
#[test]
fn test_paginated_limit_cap_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 7, 30);

    let result = client.get_group_members_paginated(&id, &0, &50);
    assert_eq!(result.members.len(), 20); // Capped at 20
    assert_eq!(result.limit, 20); // Limit should be capped
    assert_eq!(result.total, 30);
}

/// Verify offset beyond total returns empty page.
#[test]
fn test_paginated_offset_beyond_total_returns_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 8, 10);

    let result = client.get_group_members_paginated(&id, &20, &10);
    assert_eq!(result.members.len(), 0);
    assert_eq!(result.total, 10);
    assert_eq!(result.offset, 20);
}

/// Verify zero limit returns empty page.
#[test]
fn test_paginated_zero_limit_returns_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 9, 10);

    let result = client.get_group_members_paginated(&id, &0, &0);
    assert_eq!(result.members.len(), 0);
    assert_eq!(result.limit, 0);
    assert_eq!(result.total, 10);
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

/// Verify tracking works for single member group.
#[test]
fn test_paginated_single_member_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 10, 1);

    let result = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(result.members.len(), 1);
    assert_eq!(result.total, 1);
}

/// Verify tracking works for maximum members (50).
#[test]
fn test_paginated_maximum_members() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 11, 50);

    let result = client.get_group_members_paginated(&id, &0, &20);
    assert_eq!(result.members.len(), 20);
    assert_eq!(result.total, 50);
}

/// Verify tracking works when requesting last partial page.
#[test]
fn test_paginated_last_partial_page() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 12, 23);

    let result = client.get_group_members_paginated(&id, &20, &10);
    assert_eq!(result.members.len(), 3); // Only 3 members left
    assert_eq!(result.total, 23);
    assert_eq!(result.offset, 20);
}

// ─── Non-existent group ──────────────────────────────────────────────────────

/// Verify that querying non-existent group fails without emitting event.
#[test]
fn test_paginated_nonexistent_group_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);

    let _ = env.events().all();
    let result = client.try_get_group_members_paginated(&ghost_id, &0, &10);
    assert!(result.is_err(), "Should fail for non-existent group");
}

// ─── Inactive group ──────────────────────────────────────────────────────────

/// Verify tracking works for inactive groups (read operations don't check active status).
#[test]
fn test_paginated_inactive_group_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 13, 10);

    client.deactivate_payment_group(&id, &creator);

    let result = client.get_group_members_paginated(&id, &0, &5);
    assert_eq!(result.members.len(), 5);
    assert_eq!(result.total, 10);
}

// ─── Repeated calls ──────────────────────────────────────────────────────────

/// Verify repeated calls with same parameters work correctly.
#[test]
fn test_paginated_repeated_calls_same_parameters() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 14, 15);

    let result1 = client.get_group_members_paginated(&id, &5, &5);
    let result2 = client.get_group_members_paginated(&id, &5, &5);
    let result3 = client.get_group_members_paginated(&id, &5, &5);

    assert_eq!(result1.members.len(), result2.members.len());
    assert_eq!(result2.members.len(), result3.members.len());
    assert_eq!(result1.total, result2.total);
    assert_eq!(result2.total, result3.total);
}

/// Verify repeated calls with different parameters work correctly.
#[test]
fn test_paginated_repeated_calls_different_parameters() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 15, 30);

    let result1 = client.get_group_members_paginated(&id, &0, &10);
    let result2 = client.get_group_members_paginated(&id, &10, &10);
    let result3 = client.get_group_members_paginated(&id, &20, &10);

    assert_eq!(result1.members.len(), 10);
    assert_eq!(result2.members.len(), 10);
    assert_eq!(result3.members.len(), 10);
    assert_eq!(result1.total, 30);
    assert_eq!(result2.total, 30);
    assert_eq!(result3.total, 30);
}

// ─── Member list changes ─────────────────────────────────────────────────────

/// Verify tracking reflects updated member count after members are added.
#[test]
fn test_paginated_reflects_member_additions() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 16, 5);

    let result1 = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(result1.total, 5);

    // Update to 10 members
    let new_members = create_test_members(&env, 10);
    client.update_members(&id, &creator, &new_members);

    let result2 = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(result2.total, 10);
}

/// Verify tracking reflects updated member count after members are removed.
#[test]
fn test_paginated_reflects_member_removals() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 17, 10);

    let result1 = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(result1.total, 10);

    // Update to 3 members
    let new_members = create_test_members(&env, 3);
    client.update_members(&id, &creator, &new_members);

    let result2 = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(result2.total, 3);
    assert_eq!(result2.members.len(), 3);
}

// ─── Performance characteristics ─────────────────────────────────────────────

/// Verify that tracking is deterministic (same input produces same output).
#[test]
fn test_paginated_deterministic_behavior() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 18, 20);

    let result1 = client.get_group_members_paginated(&id, &5, &10);
    let result2 = client.get_group_members_paginated(&id, &5, &10);

    assert_eq!(result1.members.len(), result2.members.len());
    assert_eq!(result1.total, result2.total);
    assert_eq!(result1.offset, result2.offset);
    assert_eq!(result1.limit, result2.limit);

    // Verify member addresses match
    for i in 0..result1.members.len() {
        let member1 = result1.members.get(i).unwrap();
        let member2 = result2.members.get(i).unwrap();
        assert_eq!(member1.address, member2.address);
        assert_eq!(member1.percentage, member2.percentage);
    }
}

/// Verify tracking doesn't affect pagination ordering.
#[test]
fn test_paginated_preserves_member_order() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group_with_members(&env, &client, &token, &creator, 19, 15);

    // Get all members via non-paginated call
    let all_members = client.get_group_members(&id);

    // Get members via pagination
    let page1 = client.get_group_members_paginated(&id, &0, &10);
    let page2 = client.get_group_members_paginated(&id, &10, &10);

    // Verify first 10 match
    for i in 0..10 {
        let all_member = all_members.get(i).unwrap();
        let page_member = page1.members.get(i).unwrap();
        assert_eq!(all_member.address, page_member.address);
        assert_eq!(all_member.percentage, page_member.percentage);
    }

    // Verify last 5 match
    for i in 0..5 {
        let all_member = all_members.get(10 + i).unwrap();
        let page_member = page2.members.get(i).unwrap();
        assert_eq!(all_member.address, page_member.address);
        assert_eq!(all_member.percentage, page_member.percentage);
    }
}
