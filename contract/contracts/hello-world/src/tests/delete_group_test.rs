#![cfg(test)]

use crate::test_utils::{deploy_autoshare_contract, deploy_mock_token, mint_tokens};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

/// Helper function to create a group for testing
fn create_test_group(
    env: &Env,
    contract_id: &Address,
    token_id: &Address,
    creator: &Address,
    group_id: BytesN<32>,
) {
    let client = crate::AutoShareContractClient::new(env, contract_id);
    let name = String::from_str(env, "Test Group");
    
    // Fund the creator with tokens
    let fee = 10; // Default usage fee
    let amount = (10 as i128) * (fee as i128) + 10000;
    mint_tokens(env, token_id, creator, amount);
    
    client.create(&group_id, &name, creator, &10, token_id);
}

#[test]
fn test_delete_group_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[1u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Verify group exists
    let group = client.get(&group_id);
    assert_eq!(group.id, group_id);
    assert_eq!(group.creator, creator);

    // Deactivate the group first
    client.deactivate_group(&group_id, &creator);
    assert_eq!(client.is_group_active(&group_id), false);

    // Reduce all usages to 0
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }
    assert_eq!(client.get_remaining_usages(&group_id), 0);

    // Delete the group
    client.delete_group(&group_id, &creator);

    // Verify group is not in all_groups list
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 0);
}

#[test]
fn test_delete_group_by_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[2u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Deactivate the group
    client.deactivate_group(&group_id, &creator);

    // Reduce all usages to 0
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }

    // Admin deletes the group (not creator)
    client.delete_group(&group_id, &admin);

    // Verify group is not in all_groups list
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 0);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_delete_group_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[3u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Deactivate the group
    client.deactivate_group(&group_id, &creator);

    // Reduce all usages to 0
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }

    // Try to delete with unauthorized user - should fail
    client.delete_group(&group_id, &unauthorized);
}

#[test]
#[should_panic(expected = "GroupNotDeactivated")]
fn test_delete_group_not_deactivated() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[4u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Try to delete without deactivating first - should fail
    client.delete_group(&group_id, &creator);
}

#[test]
#[should_panic(expected = "NotFound")]
fn test_delete_nonexistent_group() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin
    client.initialize_admin(&admin);

    // Try to delete a group that doesn't exist
    let group_id = BytesN::from_array(&env, &[5u8; 32]);
    client.delete_group(&group_id, &creator);
}

#[test]
fn test_delete_group_with_remaining_usages() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[6u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Deactivate the group
    client.deactivate_group(&group_id, &creator);

    // Don't reduce usages - group still has 10 usages
    assert_eq!(client.get_remaining_usages(&group_id), 10);

    // Delete the group (should succeed with forfeiture)
    client.delete_group(&group_id, &creator);

    // Verify group is not in all_groups list
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 0);
}

#[test]
fn test_delete_group_preserves_payment_history() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[7u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Verify payment history exists
    let history_before = client.get_group_payment_history(&group_id);
    assert_eq!(history_before.len(), 1);

    let user_history_before = client.get_user_payment_history(&creator);
    assert_eq!(user_history_before.len(), 1);

    // Deactivate and delete the group
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }
    client.delete_group(&group_id, &creator);

    // Verify payment history is preserved
    let history_after = client.get_group_payment_history(&group_id);
    assert_eq!(history_after.len(), 1);

    let user_history_after = client.get_user_payment_history(&creator);
    assert_eq!(user_history_after.len(), 1);
}

#[test]
fn test_delete_multiple_groups() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create multiple groups
    let group_id_1 = BytesN::from_array(&env, &[8u8; 32]);
    let group_id_2 = BytesN::from_array(&env, &[9u8; 32]);
    let group_id_3 = BytesN::from_array(&env, &[10u8; 32]);

    create_test_group(&env, &contract_id, &token_id, &creator, group_id_1.clone());
    create_test_group(&env, &contract_id, &token_id, &creator, group_id_2.clone());
    create_test_group(&env, &contract_id, &token_id, &creator, group_id_3.clone());

    // Verify all groups exist
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 3);

    // Deactivate and delete first group
    client.deactivate_group(&group_id_1, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id_1);
    }
    client.delete_group(&group_id_1, &creator);

    // Verify only 2 groups remain
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 2);

    // Deactivate and delete second group
    client.deactivate_group(&group_id_2, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id_2);
    }
    client.delete_group(&group_id_2, &creator);

    // Verify only 1 group remains
    let all_groups = client.get_all_groups();
    assert_eq!(all_groups.len(), 1);
    assert_eq!(all_groups.get(0).unwrap().id, group_id_3);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_delete_group_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[11u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Deactivate the group
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }

    // Pause the contract
    client.pause(&admin);

    // Try to delete - should fail
    client.delete_group(&group_id, &creator);
}
