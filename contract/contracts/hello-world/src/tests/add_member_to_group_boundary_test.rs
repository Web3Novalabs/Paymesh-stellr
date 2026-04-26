use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, IntoVal, Vec};

// ─── Test 1: Maximum Allowed Capacity ───────────────────────────────────────
// Create a group and add members until exactly reaching the maximum capacity.
// Then verify adding one more member fails.
#[test]
fn test_add_member_to_group_max_capacity_boundary() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let admin = client.get_admin();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Set a moderate max capacity for the test
    let max_capacity = 10u32;
    client.set_max_members(&admin, &max_capacity);

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &Vec::new(env),
        5,
        &token,
    );

    // Add (max_capacity - 1) members, leaving room for 1 more
    // Since creator is not a member initially, we add up to max_capacity
    // We'll give each a 1% share
    for _i in 0..max_capacity {
        let member = Address::generate(env);
        client.add_member_to_group(&id, &creator, &member, &1);
    }

    assert_eq!(client.get_group_member_count(&id), max_capacity);

    // Adding one more should exceed max capacity
    let extra_member = Address::generate(env);
    let result = env.try_invoke_contract::<(), crate::base::errors::Error>(
        &test_env.autoshare_contract,
        &soroban_sdk::Symbol::new(env, "add_member_to_group"),
        (id.clone(), creator.clone(), extra_member.clone(), 1u32).into_val(env),
    );

    assert!(result.is_err(), "Expected MaxMembersExceeded error");
}

// ─── Test 2: Percentage Overflow Simulation ──────────────────────────────────
// Test passing very large values like u32::MAX to ensure overflow handling
#[test]
#[should_panic(expected = "InvalidInput")]
fn test_add_member_to_group_percentage_overflow() {
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
        3,
        &token,
    );
    let member = Address::generate(env);

    // Pass u32::MAX. Should panic with InvalidInput because > 100.
    client.add_member_to_group(&id, &creator, &member, &u32::MAX);
}

// ─── Test 3: Total Percentage Overflow ──────────────────────────────────────
// Total percentages exceed 100
#[test]
#[should_panic(expected = "InvalidTotalPercentage")]
fn test_add_member_to_group_total_percentage_overflow() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let mut initial_members = Vec::new(env);
    initial_members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 99,
    });

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &initial_members,
        3,
        &token,
    );
    let m2 = Address::generate(env);

    client.add_member_to_group(&id, &creator, &m2, &2); // 99 + 2 = 101
}

// ─── Test 4: Revert State on Failure ─────────────────────────────────────────
// Verify that if add_member_to_group fails due to bad percentage, no state changes
#[test]
fn test_add_member_to_group_state_revert_on_failure() {
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
        3,
        &token,
    );
    let member = Address::generate(env);

    // Attempt to add a member with invalid percentage (0)
    let result = env.try_invoke_contract::<(), crate::base::errors::Error>(
        &test_env.autoshare_contract,
        &soroban_sdk::Symbol::new(env, "add_member_to_group"),
        (id.clone(), creator.clone(), member.clone(), 0u32).into_val(env),
    );
    assert!(result.is_err(), "Expected transaction to fail");

    // Verify member count remains 0
    assert_eq!(client.get_group_member_count(&id), 0);
    // Verify is_group_member is false
    assert!(!client.is_group_member(&id, &member));
}
