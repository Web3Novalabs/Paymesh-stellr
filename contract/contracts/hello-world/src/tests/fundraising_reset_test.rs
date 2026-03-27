#![cfg(test)]

use crate::test_utils::{setup_test_env, mint_tokens};
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, String};

#[test]
fn test_reset_fundraising_success() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    // Fund creator
    mint_tokens(env, &token, &creator, 1000000);
    
    // Create group
    let group_id = BytesN::from_array(env, &[1u8; 32]);
    let name = String::from_str(env, "Test Group");
    client.create(&group_id, &name, &creator, &100, &token);
    
    // Add member (required for distribution during contribution)
    let member = test_env.users.get(2).unwrap();
    client.add_group_member(&group_id, &creator, &member, &100);

    // Start fundraising
    client.start_fundraising(&group_id, &creator, &1000);
    
    // Fund and contribute
    let contributor = test_env.users.get(1).unwrap();
    mint_tokens(env, &token, &contributor, 1000);
    client.contribute(&group_id, &token, &1000, &contributor);
    
    // Verify it's inactive (completed)
    let status = client.get_fundraising_status(&group_id);
    assert!(!status.is_active);
    assert_eq!(status.total_raised, 1000);
    
    // Reset fundraising
    client.reset_fundraising(&group_id, &creator);
    
    // Verify it's cleared
    let status_after = client.get_fundraising_status(&group_id);
    assert_eq!(status_after.target_amount, 0);
    assert_eq!(status_after.total_raised, 0);
    
    // Verify contributions are cleared
    let contributions = client.get_group_contributions(&group_id);
    assert_eq!(contributions.len(), 0);
    
    // Can start a new one
    client.start_fundraising(&group_id, &creator, &2000);
    let status_new = client.get_fundraising_status(&group_id);
    assert!(status_new.is_active);
    assert_eq!(status_new.target_amount, 2000);
}

#[test]
#[should_panic(expected = "FundraisingAlreadyActive")]
fn test_reset_fundraising_active_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    mint_tokens(env, &token, &creator, 1000000);
    
    let group_id = BytesN::from_array(env, &[2u8; 32]);
    let name = String::from_str(env, "Test Group");
    client.create(&group_id, &name, &creator, &100, &token);
    
    // Add member
    let member = test_env.users.get(2).unwrap();
    client.add_group_member(&group_id, &creator, &member, &100);

    client.start_fundraising(&group_id, &creator, &1000);
    
    // Try to reset active campaign - should fail
    client.reset_fundraising(&group_id, &creator);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_reset_fundraising_non_creator_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let non_creator = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    
    mint_tokens(env, &token, &creator, 1000000);
    
    let group_id = BytesN::from_array(env, &[3u8; 32]);
    let name = String::from_str(env, "Test Group");
    client.create(&group_id, &name, &creator, &100, &token);
    
    // Add member
    let member = test_env.users.get(2).unwrap();
    client.add_group_member(&group_id, &creator, &member, &100);

    client.start_fundraising(&group_id, &creator, &1000);
    
    // Try to reset by non-creator - should fail
    client.reset_fundraising(&group_id, &non_creator);
}
