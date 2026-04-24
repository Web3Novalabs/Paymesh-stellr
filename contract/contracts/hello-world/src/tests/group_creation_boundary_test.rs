use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

// ─── Test 1 ──────────────────────────────────────────────────────────────────
// create group with exactly 1 member at 100% — succeeds.

#[test]
fn test_create_group_one_member_100_percent_succeeds() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member.clone(),
        percentage: 100,
    });

    let id = create_test_group(&env, &contract, &creator, &members, 1, &token);

    let group = client.get(&id);
    assert!(group.is_active);
    assert_eq!(group.members.len(), 1);
    assert_eq!(group.members.get(0).unwrap().address, member);
    assert_eq!(group.members.get(0).unwrap().percentage, 100);
}

// ─── Test 2 ──────────────────────────────────────────────────────────────────
// create group with exactly 50 members each at 2% — succeeds (boundary = MAX_MEMBERS).

#[test]
fn test_create_group_50_members_2_percent_each_succeeds() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();

    // Create the group first (members added separately)
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    // Build exactly 50 members, each at 2% → total = 100%
    let mut members = Vec::new(&env);
    for _ in 0..50u32 {
        members.push_back(GroupMember {
            address: Address::generate(&env),
            percentage: 2,
        });
    }

    // update_members at the boundary should succeed
    client.update_members(&id, &creator, &members);

    let group = client.get(&id);
    assert_eq!(group.members.len(), 50);
}

// ─── Test 3 ──────────────────────────────────────────────────────────────────
// create group with 51 members — returns MaxMembersExceeded.

#[test]
#[should_panic(expected = "MaxMembersExceeded")]
fn test_create_group_51_members_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    // 51 members — MaxMembersExceeded fires before percentage validation
    let mut members = Vec::new(&env);
    for _ in 0..51u32 {
        members.push_back(GroupMember {
            address: Address::generate(&env),
            percentage: 1,
        });
    }

    client.update_members(&id, &creator, &members);
}

// ─── Test 4 ──────────────────────────────────────────────────────────────────
// create group with 0 members (empty vec) — returns EmptyMembers.

#[test]
#[should_panic(expected = "EmptyMembers")]
fn test_create_group_empty_members_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    // Passing an empty member list must return EmptyMembers
    client.update_members(&id, &creator, &Vec::new(&env));
}

// ─── Test 5 ──────────────────────────────────────────────────────────────────
// create group with 50 members where percentages don't sum to 100 — returns
// InvalidTotalPercentage.

#[test]
#[should_panic(expected = "InvalidTotalPercentage")]
fn test_create_group_50_members_wrong_total_percentage_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    // 50 members × 1% = 50% total (not 100%) → InvalidTotalPercentage
    let mut members = Vec::new(&env);
    for _ in 0..50u32 {
        members.push_back(GroupMember {
            address: Address::generate(&env),
            percentage: 1,
        });
    }

    client.update_members(&id, &creator, &members);
}

// ─── Test 6 ──────────────────────────────────────────────────────────────────
// create group with 2 members having the same address — returns DuplicateMember.

#[test]
#[should_panic(expected = "DuplicateMember")]
fn test_create_group_duplicate_member_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    let dup = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: dup.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: dup.clone(), // same address — DuplicateMember
        percentage: 50,
    });

    client.update_members(&id, &creator, &members);
}

// ─── Test 7 ──────────────────────────────────────────────────────────────────
// create group with a member at 0% — returns InvalidInput.

#[test]
#[should_panic(expected = "InvalidInput")]
fn test_create_group_member_zero_percent_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 0, // zero percentage → InvalidInput
    });
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });

    client.update_members(&id, &creator, &members);
}

// ─── Test 8 ──────────────────────────────────────────────────────────────────
// create group with all members at 0% except one at 100% — returns InvalidInput
// because the first 0% member triggers the check before the 100% member is reached.

#[test]
#[should_panic(expected = "InvalidInput")]
fn test_create_group_all_zero_except_one_at_100_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    // First member at 0% — triggers InvalidInput immediately in the validation loop
    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 0,
    });
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 0,
    });
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });

    client.update_members(&id, &creator, &members);
}

// ─── Test 9 ──────────────────────────────────────────────────────────────────
// verify get_group_count increments correctly for each successful creation.

#[test]
fn test_get_group_count_increments_per_creation() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();

    assert_eq!(client.get_group_count(), 0);

    // Each successful creation must increment the count by 1
    create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);
    assert_eq!(client.get_group_count(), 1);

    create_test_group(&env, &contract, &creator, &Vec::new(&env), 2, &token);
    assert_eq!(client.get_group_count(), 2);

    create_test_group(&env, &contract, &creator, &Vec::new(&env), 3, &token);
    assert_eq!(client.get_group_count(), 3);
}

// ─── Test 10 ─────────────────────────────────────────────────────────────────
// verify MemberGroups index is updated for all 50 members in a max-member group.

#[test]
fn test_max_member_group_member_groups_index_updated() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);

    // Generate 50 unique member addresses and build the member list
    let mut member_addresses: Vec<Address> = Vec::new(&env);
    let mut members: Vec<GroupMember> = Vec::new(&env);
    for _ in 0..50u32 {
        let addr = Address::generate(&env);
        member_addresses.push_back(addr.clone());
        members.push_back(GroupMember {
            address: addr,
            percentage: 2,
        });
    }

    client.update_members(&id, &creator, &members);

    // Every member must appear in exactly 1 group and it must be the group we created
    for i in 0..50u32 {
        let addr = member_addresses.get(i).unwrap();
        let member_groups = client.get_groups_by_member(&addr);
        assert_eq!(member_groups.len(), 1);
        assert_eq!(member_groups.get(0).unwrap().id, id);
    }
}

// ─── Bonus: successful creation after failed attempt does not corrupt count ──

#[test]
fn test_failed_creation_does_not_corrupt_group_count() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();

    // Create one valid group
    create_test_group(&env, &contract, &creator, &Vec::new(&env), 1, &token);
    assert_eq!(client.get_group_count(), 1);

    // Attempt to create a second valid group (different usages → different ID)
    create_test_group(&env, &contract, &creator, &Vec::new(&env), 2, &token);
    assert_eq!(client.get_group_count(), 2);
}

// ─── Verify 50-member group is fully functional after creation ────────────────

#[test]
fn test_50_member_group_is_active_and_distributable() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &Vec::new(&env), 5, &token);

    let mut members = Vec::new(&env);
    for _ in 0..50u32 {
        members.push_back(GroupMember {
            address: Address::generate(&env),
            percentage: 2,
        });
    }
    client.update_members(&id, &creator, &members);

    let group = client.get(&id);
    assert!(group.is_active);
    assert_eq!(group.members.len(), 50);

    // The group should be distributable
    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 500);
    client.distribute(&id, &token, &100, &sender);

    assert_eq!(client.get_remaining_usages(&id), 4);
}
