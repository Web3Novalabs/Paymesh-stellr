use crate::{AutoShareContract, AutoShareContractClient};
use crate::test_utils::{
    setup_test_env, create_test_group
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, String, Vec};

#[test]
fn test_create_and_get_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users[0].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    // Usages=1 -> ID derived from 1
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);

    let result = client.get(&id);
    assert_eq!(result.name, String::from_str(&test_env.env, "Test Group"));
    assert_eq!(result.creator, creator);
}

#[test]
#[should_panic]
fn test_duplicate_id_fails() {
    let test_env = setup_test_env();
    
    let creator = test_env.users[0].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    // Create group with usages=1 twice
    create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
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

    let creator1 = test_env.users[0].clone();
    let creator2 = test_env.users[1].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id1 = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator1, &members, 1, &token);
    let id2 = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator2, &members, 2, &token);
    let id3 = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator1, &members, 3, &token);

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

    let creator = test_env.users[0].clone();
    let groups = client.get_groups_by_creator(&creator);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_get_groups_by_creator_multiple() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator1 = test_env.users[0].clone();
    let creator2 = test_env.users[1].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id1 = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator1, &members, 1, &token);
    let id2 = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator2, &members, 2, &token);
    let id3 = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator1, &members, 3, &token);

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
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users[0].clone();
    let member = test_env.users[1].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);

    let is_member = client.is_group_member(&id, &member);
    assert!(!is_member);
}

#[test]
fn test_is_group_member_true() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users[0].clone();
    let member = test_env.users[1].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    client.add_group_member(&id, &member);

    let is_member = client.is_group_member(&id, &member);
    assert!(is_member);
}

#[test]
#[should_panic]
fn test_is_group_member_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let member = test_env.users[0].clone();
    let id = BytesN::from_array(&test_env.env, &[99u8; 32]);

    client.is_group_member(&id, &member);
}

#[test]
fn test_get_group_members_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users[0].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);

    let members_res = client.get_group_members(&id);
    assert_eq!(members_res.len(), 0);
}

#[test]
fn test_get_group_members_multiple() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users[0].clone();
    let member1 = test_env.users[1].clone();
    let member2 = test_env.users[2].clone();
    let member3 = Address::generate(&test_env.env); 
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    client.add_group_member(&id, &member1);
    client.add_group_member(&id, &member2);
    client.add_group_member(&id, &member3);

    let members_res = client.get_group_members(&id);
    assert_eq!(members_res.len(), 3);
    assert_eq!(members_res.get(0).unwrap().address, member1);
    assert_eq!(members_res.get(1).unwrap().address, member2);
    assert_eq!(members_res.get(2).unwrap().address, member3);
}

#[test]
#[should_panic]
fn test_get_group_members_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let id = BytesN::from_array(&test_env.env, &[99u8; 32]);
    client.get_group_members(&id);
}

#[test]
#[should_panic]
fn test_add_member_to_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let member = test_env.users[0].clone();
    let id = BytesN::from_array(&test_env.env, &[99u8; 32]);
    client.add_group_member(&id, &member);
}

#[test]
#[should_panic]
fn test_add_duplicate_member() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users[0].clone();
    let member = test_env.users[1].clone();
    let members = Vec::new(&test_env.env);
    let token = test_env.mock_tokens[0].clone();

    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    client.add_group_member(&id, &member);
    client.add_group_member(&id, &member);
}
