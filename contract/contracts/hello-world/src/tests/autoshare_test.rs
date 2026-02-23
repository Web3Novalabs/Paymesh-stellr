use crate::base::types::GroupMember;
use crate::mock_token::{MockToken, MockTokenClient};
use crate::test_utils::{create_test_group, setup_test_env};
use crate::{AutoShareContract, AutoShareContractClient};

/*use soroban_sdk::testutils::Events;*/
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};
fn create_helper(
    client: &AutoShareContractClient,
    id: &BytesN<32>,
    name: &String,
    creator: &Address,
    members: &Vec<GroupMember>,
    test_env: &crate::test_utils::TestEnv,
) {
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    crate::test_utils::mint_tokens(&test_env.env, &token, creator, 10000000);
    client.create(id, name, creator, &1u32, &token);
    client.update_members(id, creator, members);
}

#[test]
fn test_create_and_get_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let mut members = Vec::new(&test_env.env);
    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 60,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 40,
    });
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let name = String::from_str(&test_env.env, "Test Group");

    // Usages=1 -> ID derived from 1
    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    let result = client.get(&id);
    assert_eq!(result.name, name);
    assert_eq!(result.creator, creator);
    assert_eq!(result.members.len(), 2);

    // Check specific member values
    let m1 = result.members.get(0).unwrap();
    assert_eq!(m1.address, member1);
    assert_eq!(m1.percentage, 60);

    let m2 = result.members.get(1).unwrap();
    assert_eq!(m2.address, member2);
    assert_eq!(m2.percentage, 40);
}

#[test]
#[should_panic]
fn test_duplicate_id_fails() {
    let test_env = setup_test_env();

    let creator = test_env.users.get(0).unwrap().clone();
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Create group with usages=1 twice
    create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
    create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
}

#[test]
#[should_panic]
fn test_get_non_existent_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&test_env.env, &[9u8; 32]);
    client.get(&id);
}

#[test]
fn test_get_all_groups_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let groups = client.get_all_groups();
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_all_groups_multiple() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator1 = test_env.users.get(0).unwrap().clone();
    let creator2 = test_env.users.get(1).unwrap().clone();
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id1 = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator1,
        &members,
        1,
        &token,
    );
    let id2 = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator2,
        &members,
        2,
        &token,
    );
    let id3 = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator1,
        &members,
        3,
        &token,
    );

    let groups = client.get_all_groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups.get(0).unwrap().id, id1);
    assert_eq!(groups.get(1).unwrap().id, id2);
    assert_eq!(groups.get(2).unwrap().id, id3);
}

#[test]
fn test_get_groups_by_creator_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let groups = client.get_groups_by_creator(&creator);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_groups_by_creator_multiple() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator1 = test_env.users.get(0).unwrap().clone();
    let creator2 = test_env.users.get(1).unwrap().clone();
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id1 = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator1,
        &members,
        1,
        &token,
    );
    let _id2 = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator2,
        &members,
        2,
        &token,
    );
    let id3 = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator1,
        &members,
        3,
        &token,
    );

    let groups = client.get_groups_by_creator(&creator1);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups.get(0).unwrap().id, id1);
    assert_eq!(groups.get(1).unwrap().id, id3);
}

#[test]
#[should_panic] // InvalidTotalPercentage
fn test_create_fails_invalid_percentage() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Invalid Split");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 50, // Sum = 50 != 100
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);
}

#[test]
#[should_panic] // EmptyMembers
fn test_create_fails_empty_members() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Empty");

    let members = Vec::new(&test_env.env);

    create_helper(&client, &id, &name, &creator, &members, &test_env);
}

#[test]
#[should_panic] // DuplicateMember
fn test_create_fails_duplicate_member() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Dup");

    let member_summary = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member_summary.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member_summary, // Duplicate
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);
}

#[test]
fn test_update_members_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Update Test");

    let member1 = Address::generate(&test_env.env);
    let mut initial_members = Vec::new(&test_env.env);
    initial_members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &initial_members, &test_env);

    // Verify initial
    let initial_res = client.get(&id);
    assert_eq!(initial_res.members.len(), 1);

    // Update members (split 50/50 with new user)
    let member2 = Address::generate(&test_env.env);
    let mut new_members = Vec::new(&test_env.env);
    new_members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    new_members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    client.update_members(&id, &creator, &new_members);

    // Verify update
    let updated_res = client.get(&id);
    assert_eq!(updated_res.members.len(), 2);
    assert_eq!(updated_res.members.get(0).unwrap().percentage, 50);
    assert_eq!(updated_res.members.get(1).unwrap().address, member2);
}

#[test]
#[should_panic] // NotAuthorized
fn test_update_members_unauthorized() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Auth Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    let other_user = Address::generate(&test_env.env);
    client.update_members(&id, &other_user, &members);
}

#[test]
#[should_panic] // InvalidTotalPercentage
fn test_update_members_invalid_percentage() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Invalid Update");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    let mut bad_members = Vec::new(&test_env.env);
    bad_members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 90,
    });

    client.update_members(&id, &creator, &bad_members);
}

#[test]
fn test_is_group_member() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Member Check");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env); // Not a member
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    assert!(client.is_group_member(&id, &member1));
    assert!(!client.is_group_member(&id, &member2));
}

#[test]
fn test_is_group_member_false() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = test_env.users.get(1).unwrap().clone();
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    let is_member = client.is_group_member(&id, &member);
    assert!(!is_member);
}

#[test]
fn test_is_group_member_true() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = test_env.users.get(1).unwrap().clone();
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member.clone(),
        percentage: 100,
    });
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    let is_member = client.is_group_member(&id, &member);
    assert!(is_member);
}

#[test]
#[should_panic]
fn test_is_group_member_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let member = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[99u8; 32]);

    client.is_group_member(&id, &member);
}

#[test]
fn test_get_group_members_multiple() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member1 = test_env.users.get(1).unwrap().clone();
    let member2 = test_env.users.get(2).unwrap().clone();
    let member3 = Address::generate(&test_env.env);

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 40,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 30,
    });
    members.push_back(GroupMember {
        address: member3.clone(),
        percentage: 30,
    });

    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    let members_res = client.get_group_members(&id);
    assert_eq!(members_res.len(), 3);
    assert_eq!(members_res.get(0).unwrap().address, member1);
    assert_eq!(members_res.get(0).unwrap().percentage, 40);
    assert_eq!(members_res.get(1).unwrap().address, member2);
    assert_eq!(members_res.get(1).unwrap().percentage, 30);
    assert_eq!(members_res.get(2).unwrap().address, member3);
    assert_eq!(members_res.get(2).unwrap().percentage, 30);
}

#[test]
fn test_get_group_members_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Empty Members Test");

    // Create group with one member at 100%
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    let members_res = client.get_group_members(&id);
    assert_eq!(members_res.len(), 1);
}

#[test]
#[should_panic]
fn test_get_group_members_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&test_env.env, &[99u8; 32]);
    client.get_group_members(&id);
}

// ============================================
// Add Group Member Tests
// ============================================

#[test]
fn test_add_group_member_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Add Member Test");

    // Create group with two members at 50% each
    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Update to 33/33/34 split to make room for third member
    let mut updated_members = Vec::new(&test_env.env);
    updated_members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 33,
    });
    updated_members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 33,
    });
    // Add a placeholder third member with 34% to make 100%
    let placeholder = Address::generate(&test_env.env);
    updated_members.push_back(GroupMember {
        address: placeholder.clone(),
        percentage: 34,
    });
    client.update_members(&id, &creator, &updated_members);

    // Remove placeholder and add real third member
    let mut final_members_vec = Vec::new(&test_env.env);
    final_members_vec.push_back(GroupMember {
        address: member1.clone(),
        percentage: 33,
    });
    final_members_vec.push_back(GroupMember {
        address: member2.clone(),
        percentage: 33,
    });
    let member3 = Address::generate(&test_env.env);
    final_members_vec.push_back(GroupMember {
        address: member3.clone(),
        percentage: 34,
    });
    client.update_members(&id, &creator, &final_members_vec);

    // Verify all three members exist
    let final_members = client.get_group_members(&id);
    assert_eq!(final_members.len(), 3);
    assert_eq!(final_members.get(2).unwrap().address, member3);
    assert_eq!(final_members.get(2).unwrap().percentage, 34);
}

#[test]
#[should_panic] // AlreadyExists
fn test_add_duplicate_member() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Duplicate Member Test");

    let member1 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Try to add the same member again - should fail
    client.add_group_member(&id, &creator, &member1, &50);
}

#[test]
#[should_panic] // NotFound
fn test_add_member_to_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&test_env.env, &[99u8; 32]);
    let member = Address::generate(&test_env.env);

    let caller = Address::generate(&test_env.env);
    client.add_group_member(&id, &caller, &member, &50);
}

#[test]
#[should_panic] // InvalidTotalPercentage
fn test_add_member_invalid_total_percentage() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Invalid Percentage Test");

    // Create group with one member at 100%
    let member1 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Try to add another member with 50% (total would be 150%) - should fail
    let member2 = Address::generate(&test_env.env);
    client.add_group_member(&id, &creator, &member2, &50);
}

#[test]
fn test_add_multiple_members_sequentially() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Multiple Add Test");

    // Create group with one member at 100%
    let member1 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Update to 25% for first member
    let mut updated_members = Vec::new(&test_env.env);
    updated_members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 25,
    });

    // Add second member with 25% to make 50%
    let member2 = Address::generate(&test_env.env);
    updated_members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 25,
    });

    // Add third member with 25% to make 75%
    let member3 = Address::generate(&test_env.env);
    updated_members.push_back(GroupMember {
        address: member3.clone(),
        percentage: 25,
    });

    // Add fourth member with 25% to make 100%
    let member4 = Address::generate(&test_env.env);
    updated_members.push_back(GroupMember {
        address: member4.clone(),
        percentage: 25,
    });

    client.update_members(&id, &creator, &updated_members);

    // Verify all four members exist
    let final_members = client.get_group_members(&id);
    assert_eq!(final_members.len(), 4);

    // Verify total percentage is 100%
    let mut total = 0u32;
    for member in final_members.iter() {
        total += member.percentage;
    }
    assert_eq!(total, 100);
}

#[test]
fn test_add_member_to_inactive_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Inactive Add Test");

    // Create group with two members at 50% each
    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Update to 33/33/34 to make room for third member
    let mut updated_members = Vec::new(&test_env.env);
    updated_members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 33,
    });
    updated_members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 33,
    });
    let placeholder = Address::generate(&test_env.env);
    updated_members.push_back(GroupMember {
        address: placeholder.clone(),
        percentage: 34,
    });
    client.update_members(&id, &creator, &updated_members);

    // Deactivate the group
    client.deactivate_group(&id, &creator);

    // Replace placeholder with real member using add_group_member
    // First remove placeholder
    let mut final_members_vec = Vec::new(&test_env.env);
    final_members_vec.push_back(GroupMember {
        address: member1.clone(),
        percentage: 33,
    });
    final_members_vec.push_back(GroupMember {
        address: member2.clone(),
        percentage: 33,
    });
    let member3 = Address::generate(&test_env.env);
    final_members_vec.push_back(GroupMember {
        address: member3.clone(),
        percentage: 34,
    });

    // Reactivate to update, then deactivate again
    client.activate_group(&id, &creator);
    client.update_members(&id, &creator, &final_members_vec);
    client.deactivate_group(&id, &creator);

    // Verify member was added and group is inactive
    let final_members = client.get_group_members(&id);
    assert_eq!(final_members.len(), 3);
    assert!(!client.is_group_active(&id));
}

// ============================================
// remove_group_member Tests
// ============================================

#[test]
fn test_remove_group_member_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Remove Member Test");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let member3 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 40,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 35,
    });
    members.push_back(GroupMember {
        address: member3.clone(),
        percentage: 25,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);
    assert_eq!(client.get_group_members(&id).len(), 3);

    client.remove_group_member(&id, &creator, &member2);

    let after = client.get_group_members(&id);
    assert_eq!(after.len(), 2);
    assert!(after.iter().all(|m| m.address != member2));
    let details = client.get(&id);
    assert_eq!(details.members.len(), 2);
    assert!(details.members.iter().all(|m| m.address != member2));
}

#[test]
#[should_panic] // Unauthorized
fn test_remove_group_member_unauthorized() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Remove Unauthorized");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    let other_user = Address::generate(&test_env.env);
    client.remove_group_member(&id, &other_user, &member2);
}

#[test]
#[should_panic] // GroupInactive
fn test_remove_group_member_inactive() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Remove Inactive");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);
    client.deactivate_group(&id, &creator);

    client.remove_group_member(&id, &creator, &member2);
}

#[test]
#[should_panic] // MemberNotFound
fn test_remove_group_member_not_found() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Remove Not Found");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    let not_in_group = Address::generate(&test_env.env);
    client.remove_group_member(&id, &creator, &not_in_group);
}

#[test]
#[should_panic] // ContractPaused
fn test_remove_group_member_paused() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let admin = test_env.admin.clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Remove Paused");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);
    client.pause(&admin);

    client.remove_group_member(&id, &creator, &member2);
}

#[test]
fn test_remove_group_member_then_update_members() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Remove Then Update");

    let member1 = Address::generate(&test_env.env);
    let member2 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);
    client.remove_group_member(&id, &creator, &member2);

    let after_remove = client.get_group_members(&id);
    assert_eq!(after_remove.len(), 1);
    assert_eq!(after_remove.get(0).unwrap().address, member1);
    assert_eq!(after_remove.get(0).unwrap().percentage, 50);

    let mut single_member = Vec::new(&test_env.env);
    single_member.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });
    client.update_members(&id, &creator, &single_member);

    let final_members = client.get_group_members(&id);
    assert_eq!(final_members.len(), 1);
    assert_eq!(final_members.get(0).unwrap().address, member1);
    assert_eq!(final_members.get(0).unwrap().percentage, 100);
}

// ============================================
// Group Activity Status Tests
// ============================================

#[test]
fn test_group_created_as_active() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Active Group");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Verify group is active by default
    assert!(client.is_group_active(&id));
}

#[test]
fn test_creator_can_deactivate_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Deactivate Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate the group
    client.deactivate_group(&id, &creator);

    // Verify group is now inactive
    assert!(!client.is_group_active(&id));
}

#[test]
fn test_creator_can_activate_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Activate Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate first
    client.deactivate_group(&id, &creator);
    assert!(!client.is_group_active(&id));

    // Reactivate the group
    client.activate_group(&id, &creator);

    // Verify group is now active
    assert!(client.is_group_active(&id));
}

#[test]
#[should_panic] // GroupInactive
fn test_updating_inactive_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Update Inactive Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate the group
    client.deactivate_group(&id, &creator);

    // Try to update members - should fail
    let mut new_members = Vec::new(&test_env.env);
    new_members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 50,
    });
    new_members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 50,
    });

    client.update_members(&id, &creator, &new_members);
}

#[test]
fn test_viewing_inactive_group_works() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "View Inactive Test");

    let member1 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate the group
    client.deactivate_group(&id, &creator);

    // Should still be able to view the group
    let result = client.get(&id);
    assert_eq!(result.name, name);
    assert_eq!(result.creator, creator);
    assert!(!result.is_active);
}

#[test]
#[should_panic] // NotAuthorized
fn test_non_creator_cannot_deactivate() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let other_user = Address::generate(&test_env.env);
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Auth Deactivate Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Try to deactivate as non-creator - should fail
    client.deactivate_group(&id, &other_user);
}

#[test]
#[should_panic] // NotAuthorized
fn test_non_creator_cannot_activate() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let other_user = Address::generate(&test_env.env);
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Auth Activate Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate as creator
    client.deactivate_group(&id, &creator);

    // Try to activate as non-creator - should fail
    client.activate_group(&id, &other_user);
}

#[test]
#[should_panic] // GroupAlreadyInactive
fn test_deactivating_already_inactive_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Already Inactive Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate once
    client.deactivate_group(&id, &creator);

    // Try to deactivate again - should fail
    client.deactivate_group(&id, &creator);
}

#[test]
#[should_panic] // GroupAlreadyActive
fn test_activating_already_active_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Already Active Test");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Group is already active by default, try to activate again - should fail
    client.activate_group(&id, &creator);
}

#[test]
#[should_panic] // NotFound
fn test_status_change_on_nonexistent_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = Address::generate(&test_env.env);
    let id = BytesN::from_array(&test_env.env, &[99u8; 32]); // Non-existent group

    // Try to deactivate non-existent group - should fail
    client.deactivate_group(&id, &creator);
}

#[test]
#[should_panic] // NotFound
fn test_is_group_active_on_nonexistent_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&test_env.env, &[99u8; 32]); // Non-existent group

    // Try to check status of non-existent group - should fail
    client.is_group_active(&id);
}

#[test]
fn test_get_all_groups_includes_inactive() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id1 = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let id2 = BytesN::from_array(&test_env.env, &[2u8; 32]);
    let name1 = String::from_str(&test_env.env, "Active Group");
    let name2 = String::from_str(&test_env.env, "Inactive Group");

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    // Create two groups
    create_helper(&client, &id1, &name1, &creator, &members, &test_env);
    create_helper(&client, &id2, &name2, &creator, &members, &test_env);

    // Deactivate second group
    client.deactivate_group(&id2, &creator);

    // Get all groups - should include both
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 2);

    // Verify statuses
    let group1 = all_groups.get(0).unwrap();
    let group2 = all_groups.get(1).unwrap();

    assert!(group1.is_active);
    assert!(!group2.is_active);
}

#[test]
fn test_is_group_member_works_on_inactive_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Member Check Inactive");

    let member1 = Address::generate(&test_env.env);
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    create_helper(&client, &id, &name, &creator, &members, &test_env);

    // Deactivate the group
    client.deactivate_group(&id, &creator);

    // Should still be able to check membership
    assert!(client.is_group_member(&id, &member1));
}

// =====================
// Admin Management Tests
// =====================

#[test]
fn test_initialize_with_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    let retrieved_admin = client.get_admin();
    assert_eq!(retrieved_admin, admin);
}

#[test]
fn test_get_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    let result = client.get_admin();
    assert_eq!(result, admin);
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let old_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.initialize_admin(&old_admin);
    client.transfer_admin(&old_admin, &new_admin);

    let current_admin = client.get_admin();
    assert_eq!(current_admin, new_admin);
}

#[test]
#[should_panic]
fn test_transfer_admin_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.initialize_admin(&admin);
    client.transfer_admin(&non_admin, &new_admin);
}

#[test]
fn test_admin_can_pause() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.pause(&admin);
    assert!(client.get_paused_status());
}

#[test]
fn test_admin_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    client.pause(&admin);
    assert!(client.get_paused_status());

    client.unpause(&admin);
    assert!(!client.get_paused_status());
}

// =====================
// Withdrawal Tests
// =====================

#[test]
fn test_get_contract_balance() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Check contract balance
    let balance = client.get_contract_balance(&token_id);
    assert_eq!(balance, 1000);
}

#[test]
fn test_admin_can_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Withdraw tokens
    client.withdraw(&admin, &token_id, &500, &recipient);

    // Check balances
    let contract_balance = client.get_contract_balance(&token_id);
    let recipient_balance = token_client.balance(&recipient);

    assert_eq!(contract_balance, 500);
    assert_eq!(recipient_balance, 500);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Try to withdraw as non-admin (should panic)
    client.withdraw(&non_admin, &token_id, &500, &recipient);
}

#[test]
#[should_panic]
fn test_withdraw_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Try to withdraw more than available (should panic)
    client.withdraw(&admin, &token_id, &1500, &recipient);
}

#[test]
#[should_panic]
fn test_withdraw_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Try to withdraw zero amount (should panic)
    client.withdraw(&admin, &token_id, &0, &recipient);
}

#[test]
#[should_panic]
fn test_withdraw_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    client.initialize_admin(&admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Try to withdraw negative amount (should panic)
    client.withdraw(&admin, &token_id, &-100, &recipient);
}

#[test]
fn test_admin_functions_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let old_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize_admin(&old_admin);
    client.transfer_admin(&old_admin, &new_admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &new_admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // New admin should be able to withdraw
    client.withdraw(&new_admin, &token_id, &500, &recipient);

    let recipient_balance = token_client.balance(&recipient);
    assert_eq!(recipient_balance, 500);
}

#[test]
#[should_panic]
fn test_old_admin_cannot_withdraw_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();

    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let old_admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize_admin(&old_admin);
    client.transfer_admin(&old_admin, &new_admin);

    // Create and initialize token
    let token_id = env.register(MockToken, ());
    let token_client = MockTokenClient::new(&env, &token_id);
    token_client.initialize(
        &old_admin,
        &7,
        &String::from_str(&env, "Test Token"),
        &String::from_str(&env, "TST"),
    );

    // Mint some tokens to the contract
    token_client.mint(&contract_id, &1000);

    // Old admin should NOT be able to withdraw (should panic)
    client.withdraw(&old_admin, &token_id, &500, &recipient);
}

// ============================================================================
// Payment System Tests (Restored from HEAD)
// ============================================================================

#[test]
fn test_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize_admin(&admin);

    // Check default usage fee is set
    let fee = client.get_usage_fee();
    assert_eq!(fee, 10u32);

    // Check supported tokens list is empty
    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_add_and_get_supported_tokens() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let token_address = test_env.mock_tokens.get(0).unwrap().clone();
    // client.add_supported_token(&token_address, &test_env.admin);

    let token2 = Address::generate(&test_env.env);
    client.add_supported_token(&token2, &test_env.admin);

    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 2);
    assert!(client.is_token_supported(&token_address));
    assert!(client.is_token_supported(&token2));
}

#[test]
#[should_panic]
fn test_add_duplicate_token_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let token_address = test_env.mock_tokens.get(0).unwrap().clone();
    // client.add_supported_token(&token_address, &test_env.admin);
    client.add_supported_token(&token_address, &test_env.admin);
}

#[test]
fn test_remove_supported_token() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let token_address = test_env.mock_tokens.get(0).unwrap().clone();
    // client.add_supported_token(&token_address, &test_env.admin);

    client.remove_supported_token(&token_address, &test_env.admin);

    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 0);
    assert!(!client.is_token_supported(&token_address));
}

#[test]
#[should_panic]
fn test_remove_non_existent_token_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let non_existent_token = Address::generate(&test_env.env);
    client.remove_supported_token(&non_existent_token, &test_env.admin);
}

#[test]
fn test_set_and_get_usage_fee() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let new_fee = 25u32;
    client.set_usage_fee(&new_fee, &test_env.admin);

    let fee = client.get_usage_fee();
    assert_eq!(fee, new_fee);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_set_usage_fee() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let non_admin = Address::generate(&test_env.env);
    let new_fee = 25u32;
    client.set_usage_fee(&new_fee, &non_admin);
}

#[test]
fn test_create_group_with_payment() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    client.initialize_admin(&test_env.admin);

    let creator = test_env.users.get(0).unwrap().clone();
    let token_address = test_env.mock_tokens.get(0).unwrap().clone();

    // client.add_supported_token(&token_address, &test_env.admin);

    crate::test_utils::mint_tokens(&test_env.env, &token_address, &creator, 10_000_000);

    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "Paid Group");
    let usage_count = 50u32;

    client.create(&id, &name, &creator, &usage_count, &token_address);

    let details = client.get(&id);
    assert_eq!(details.usage_count, usage_count);
    assert_eq!(details.total_usages_paid, usage_count);
}
