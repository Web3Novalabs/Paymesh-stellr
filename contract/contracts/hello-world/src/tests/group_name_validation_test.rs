use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, String, Vec};

#[test]
fn test_name_at_minimum_length() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let name = String::from_str(&test_env.env, "A");
    
    client.create(&id, &name, &creator, &1u32, &token);
    
    let details = client.get(&id);
    assert_eq!(details.name, name);
}

#[test]
fn test_name_at_maximum_length() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[2u8; 32]);
    let name = String::from_str(&test_env.env, "123456789012345678901234567890123456789012345678901234567890");
    
    client.create(&id, &name, &creator, &1u32, &token);
    
    let details = client.get(&id);
    assert_eq!(details.name, name);
}

#[test]
#[should_panic]
fn test_name_exceeding_max_length() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[3u8; 32]);
    let name = String::from_str(&test_env.env, "1234567890123456789012345678901234567890123456789012345678901");
    
    client.create(&id, &name, &creator, &1u32, &token);
}

#[test]
#[should_panic]
fn test_empty_name_returns_error() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[4u8; 32]);
    let name = String::from_str(&test_env.env, "");
    
    client.create(&id, &name, &creator, &1u32, &token);
}

#[test]
#[should_panic]
fn test_whitespace_only_name_returns_error() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[5u8; 32]);
    let name = String::from_str(&test_env.env, "     ");
    
    client.create(&id, &name, &creator, &1u32, &token);
}

#[test]
fn test_name_with_leading_trailing_spaces() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[6u8; 32]);
    let name = String::from_str(&test_env.env, "  Test Group  ");
    
    client.create(&id, &name, &creator, &1u32, &token);
    
    let details = client.get(&id);
    assert_eq!(details.name, name);
}

#[test]
fn test_name_with_special_characters() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[7u8; 32]);
    let name = String::from_str(&test_env.env, "Test@Group#2024!");
    
    client.create(&id, &name, &creator, &1u32, &token);
    
    let details = client.get(&id);
    assert_eq!(details.name, name);
}

#[test]
fn test_name_with_only_numbers() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[8u8; 32]);
    let name = String::from_str(&test_env.env, "123456789");
    
    client.create(&id, &name, &creator, &1u32, &token);
    
    let details = client.get(&id);
    assert_eq!(details.name, name);
}

#[test]
fn test_update_group_name_with_same_name() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    let original_name = String::from_str(&test_env.env, "Test Group");
    client.update_group_name(&id, &creator, &original_name);
    
    let details = client.get(&id);
    assert_eq!(details.name, original_name);
}

#[test]
#[should_panic]
fn test_update_group_name_on_deactivated_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    client.deactivate_group(&id, &creator);
    
    let new_name = String::from_str(&test_env.env, "Updated Name");
    client.update_group_name(&id, &creator, &new_name);
}

#[test]
#[should_panic]
fn test_name_with_newlines() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[9u8; 32]);
    let name = String::from_str(&test_env.env, "Test\nGroup");
    
    client.create(&id, &name, &creator, &1u32, &token);
}

#[test]
#[should_panic]
fn test_name_with_tabs() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    crate::test_utils::mint_tokens(&test_env.env, &token, &creator, 10000);
    
    let id = BytesN::from_array(&test_env.env, &[10u8; 32]);
    let name = String::from_str(&test_env.env, "Test\tGroup");
    
    client.create(&id, &name, &creator, &1u32, &token);
}

#[test]
fn test_update_name_at_minimum_length() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    let new_name = String::from_str(&test_env.env, "X");
    client.update_group_name(&id, &creator, &new_name);
    
    let details = client.get(&id);
    assert_eq!(details.name, new_name);
}

#[test]
fn test_update_name_at_maximum_length() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    let new_name = String::from_str(&test_env.env, "123456789012345678901234567890123456789012345678901234567890");
    client.update_group_name(&id, &creator, &new_name);
    
    let details = client.get(&id);
    assert_eq!(details.name, new_name);
}

#[test]
#[should_panic]
fn test_update_name_exceeding_max_length() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    let new_name = String::from_str(&test_env.env, "1234567890123456789012345678901234567890123456789012345678901");
    client.update_group_name(&id, &creator, &new_name);
}

#[test]
#[should_panic]
fn test_update_name_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    let new_name = String::from_str(&test_env.env, "");
    client.update_group_name(&id, &creator, &new_name);
}

#[test]
#[should_panic]
fn test_update_name_whitespace_only() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });
    
    let id = create_test_group(&test_env.env, &test_env.autoshare_contract, &creator, &members, 1, &token);
    
    let new_name = String::from_str(&test_env.env, "   ");
    client.update_group_name(&id, &creator, &new_name);
}
