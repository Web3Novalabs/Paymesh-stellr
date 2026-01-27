use crate::base::types::GroupMember;
use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

#[test]
fn test_create_and_get_success() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Platform Split");

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 60,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 40,
    });

    client.create(&id, &name, &creator, &members);

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
#[should_panic] // InvalidTotalPercentage
fn test_create_fails_invalid_percentage() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Invalid Split");

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 50, // Sum = 50 != 100
    });

    client.create(&id, &name, &creator, &members);
}

#[test]
#[should_panic] // EmptyMembers
fn test_create_fails_empty_members() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Empty");

    let members = Vec::new(&env);

    client.create(&id, &name, &creator, &members);
}

#[test]
#[should_panic] // DuplicateMember
fn test_create_fails_duplicate_member() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Dup");

    let member_summary = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member_summary.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member_summary, // Duplicate
        percentage: 50,
    });

    client.create(&id, &name, &creator, &members);
}

#[test]
fn test_update_members_success() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Update Test");

    let member1 = Address::generate(&env);
    let mut initial_members = Vec::new(&env);
    initial_members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    client.create(&id, &name, &creator, &initial_members);

    // Verify initial
    let initial_res = client.get(&id);
    assert_eq!(initial_res.members.len(), 1);

    // Update members (split 50/50 with new user)
    let member2 = Address::generate(&env);
    let mut new_members = Vec::new(&env);
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
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Auth Test");

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });

    client.create(&id, &name, &creator, &members);

    let other_user = Address::generate(&env);
    client.update_members(&id, &other_user, &members);
}

#[test]
#[should_panic] // InvalidTotalPercentage
fn test_update_members_invalid_percentage() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Invalid Update");

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });

    client.create(&id, &name, &creator, &members);

    let mut bad_members = Vec::new(&env);
    bad_members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 90,
    });

    client.update_members(&id, &creator, &bad_members);
}

#[test]
fn test_is_group_member() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Member Check");

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env); // Not a member
    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 100,
    });

    client.create(&id, &name, &creator, &members);

    assert!(client.is_group_member(&id, &member1));
    assert!(!client.is_group_member(&id, &member2));
}

#[test]
fn test_get_groups_by_creator() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let id1 = BytesN::from_array(&env, &[1u8; 32]);
    let name1 = String::from_str(&env, "Group 1");

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });

    client.create(&id1, &name1, &creator1, &members);

    let groups = client.get_groups_by_creator(&creator1);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups.get(0).unwrap().id, id1);
}
