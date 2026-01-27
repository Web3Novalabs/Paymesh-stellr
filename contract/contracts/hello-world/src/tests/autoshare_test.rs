use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn test_create_and_get_success() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Platform Split");

    client.create(&id, &name, &creator);

    let result = client.get(&id);
    assert_eq!(result.name, name);
    assert_eq!(result.creator, creator);
}

#[test]
#[should_panic]
fn test_duplicate_id_fails() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test");
    let creator = Address::generate(&env);

    client.create(&id, &name, &creator);
    client.create(&id, &name, &creator);
}

#[test]
#[should_panic]
fn test_get_non_existent_fails() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let id = BytesN::from_array(&env, &[9u8; 32]);
    client.get(&id);
}

#[test]
fn test_get_all_groups_empty() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let groups = client.get_all_groups();
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_all_groups_multiple() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let id1 = BytesN::from_array(&env, &[1u8; 32]);
    let id2 = BytesN::from_array(&env, &[2u8; 32]);
    let id3 = BytesN::from_array(&env, &[3u8; 32]);
    let name1 = String::from_str(&env, "Group 1");
    let name2 = String::from_str(&env, "Group 2");
    let name3 = String::from_str(&env, "Group 3");

    client.create(&id1, &name1, &creator1);
    client.create(&id2, &name2, &creator2);
    client.create(&id3, &name3, &creator1);

    let groups = client.get_all_groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups.get(0).unwrap().id, id1);
    assert_eq!(groups.get(1).unwrap().id, id2);
    assert_eq!(groups.get(2).unwrap().id, id3);
}

#[test]
fn test_get_groups_by_creator_empty() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let groups = client.get_groups_by_creator(&creator);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_groups_by_creator_multiple() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let id1 = BytesN::from_array(&env, &[1u8; 32]);
    let id2 = BytesN::from_array(&env, &[2u8; 32]);
    let id3 = BytesN::from_array(&env, &[3u8; 32]);
    let name1 = String::from_str(&env, "Group 1");
    let name2 = String::from_str(&env, "Group 2");
    let name3 = String::from_str(&env, "Group 3");

    client.create(&id1, &name1, &creator1);
    client.create(&id2, &name2, &creator2);
    client.create(&id3, &name3, &creator1);

    let groups = client.get_groups_by_creator(&creator1);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups.get(0).unwrap().id, id1);
    assert_eq!(groups.get(1).unwrap().id, id3);

    let groups2 = client.get_groups_by_creator(&creator2);
    assert_eq!(groups2.len(), 1);
    assert_eq!(groups2.get(0).unwrap().id, id2);
}

#[test]
fn test_is_group_member_false() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator);

    let is_member = client.is_group_member(&id, &member);
    assert!(!is_member);
}

#[test]
fn test_is_group_member_true() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator);
    client.add_group_member(&id, &member);

    let is_member = client.is_group_member(&id, &member);
    assert!(is_member);
}

#[test]
#[should_panic]
fn test_is_group_member_non_existent_group() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[99u8; 32]);

    client.is_group_member(&id, &member);
}

#[test]
fn test_get_group_members_empty() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 0);
}

#[test]
fn test_get_group_members_multiple() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator);
    client.add_group_member(&id, &member1);
    client.add_group_member(&id, &member2);
    client.add_group_member(&id, &member3);

    let members = client.get_group_members(&id);
    assert_eq!(members.len(), 3);
    assert_eq!(members.get(0).unwrap().address, member1);
    assert_eq!(members.get(1).unwrap().address, member2);
    assert_eq!(members.get(2).unwrap().address, member3);
}

#[test]
#[should_panic]
fn test_get_group_members_non_existent_group() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let id = BytesN::from_array(&env, &[99u8; 32]);
    client.get_group_members(&id);
}

#[test]
#[should_panic]
fn test_add_member_to_non_existent_group() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[99u8; 32]);
    client.add_group_member(&id, &member);
}

#[test]
#[should_panic]
fn test_add_duplicate_member() {
    let env = Env::default();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");

    client.create(&id, &name, &creator);
    client.add_group_member(&id, &member);
    client.add_group_member(&id, &member);
}
