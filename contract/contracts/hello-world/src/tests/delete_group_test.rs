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
    let amount = 10_i128 * (fee as i128) + 10000;
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
    assert!(!client.is_group_active(&group_id));

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
#[should_panic(expected = "GroupHasRemainingUsages")]
fn test_delete_group_with_remaining_usages_fails() {
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

    // Delete the group - should fail now
    client.delete_group(&group_id, &creator);
}

#[test]
fn test_delete_group_cleanup_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[12u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Add member
    client.add_group_member(&group_id, &creator, &member, &100);

    // Verify member has the group in index
    let member_groups = client.get_groups_by_member(&member);
    assert!(member_groups.iter().any(|g| g.id == group_id));

    // Distribute some funds to have history
    let distribute_amount = 1000i128;
    mint_tokens(&env, &token_id, &creator, distribute_amount);
    client.distribute(&group_id, &token_id, &distribute_amount, &creator);

    // Verify history exists
    assert_eq!(client.get_group_distributions(&group_id).len(), 1);
    assert!(client.get_member_earnings(&member, &group_id) > 0);

    let count_before = client.get_group_count();

    // Deactivate and delete
    client.deactivate_group(&group_id, &creator);
    let remaining = client.get_remaining_usages(&group_id);
    for _ in 0..remaining {
        client.reduce_usage(&group_id);
    }
    client.delete_group(&group_id, &creator);

    // (2) Removed from AllGroups
    let all_groups = client.get_all_groups();
    assert!(!all_groups.iter().any(|g| g.id == group_id));

    // (3) Removed from MemberGroups
    let member_groups_after = client.get_groups_by_member(&member);
    assert!(!member_groups_after.iter().any(|g| g.id == group_id));

    // (4) Group count decrements
    assert_eq!(client.get_group_count(), count_before - 1);

    // (6) Distributions preserved
    assert_eq!(client.get_group_distributions(&group_id).len(), 1);

    // (7) Earnings preserved
    assert!(client.get_member_earnings(&member, &group_id) > 0);

    // (8) Re-creating with same ID works
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());
    let new_group = client.get(&group_id);
    assert_eq!(new_group.id, group_id);
}

#[test]
#[should_panic(expected = "GroupHasActiveFundraising")]
fn test_delete_group_with_active_fundraising_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    let group_id = BytesN::from_array(&env, &[14u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Start fundraising
    client.start_fundraising(&group_id, &creator, &1000);
    assert!(client.get_fundraising_status(&group_id).is_active);

    // Deactivate group
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }

    // Try to delete - should fail due to active fundraising
    client.delete_group(&group_id, &creator);
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

#[test]
fn test_admin_delete_group_force_delete() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    // Initialize admin and add supported token
    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[12u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Add a member
    client.add_group_member(&group_id, &creator, &member1, &100);

    // Verify member has the group in their list
    let member_groups = client.get_groups_by_member(&member1);
    assert!(member_groups.iter().any(|g| g.id == group_id));

    // Start a fundraiser
    client.start_fundraising(&group_id, &creator, &1000);
    assert!(client.get_fundraising_status(&group_id).is_active);

    // Verify group is active and has usages
    assert!(client.is_group_active(&group_id));
    assert_eq!(client.get_remaining_usages(&group_id), 10);

    // Admin force-deletes the group
    client.admin_delete_group(&admin, &group_id);

    // Verify group is gone from storage
    let all_groups = client.get_all_groups();
    assert!(!all_groups.iter().any(|g| g.id == group_id));

    // Verify fundraiser is inactive or gone (in our impl we just remove AutoShare(id), fundraiser record remains but is orphaned)
    // Actually our impl sets it to inactive first.
    let fundraiser = client.get_fundraising_status(&group_id);
    assert!(!fundraiser.is_active);

    // Verify group is gone from member's list
    let member_groups_after = client.get_groups_by_member(&member1);
    assert!(!member_groups_after.iter().any(|g| g.id == group_id));

    // Verify attempting to get it fails
    // (get returns Error::NotFound if missing)
}

#[test]
fn test_delete_group_with_multiple_members_cleanup() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[20u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Add multiple members
    client.add_group_member(&group_id, &creator, &member1, &100);
    client.add_group_member(&group_id, &creator, &member2, &200);
    client.add_group_member(&group_id, &creator, &member3, &300);

    // Verify all members have the group in their index
    assert!(client.get_groups_by_member(&member1).iter().any(|g| g.id == group_id));
    assert!(client.get_groups_by_member(&member2).iter().any(|g| g.id == group_id));
    assert!(client.get_groups_by_member(&member3).iter().any(|g| g.id == group_id));

    // Deactivate and delete
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }
    client.delete_group(&group_id, &creator);

    // Verify group is removed from all members' indexes
    assert!(!client.get_groups_by_member(&member1).iter().any(|g| g.id == group_id));
    assert!(!client.get_groups_by_member(&member2).iter().any(|g| g.id == group_id));
    assert!(!client.get_groups_by_member(&member3).iter().any(|g| g.id == group_id));
}

#[test]
fn test_delete_group_after_fundraising_completed() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[21u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Start and complete fundraising
    let goal = 1000i128;
    client.start_fundraising(&group_id, &creator, &goal);
    
    // Contribute to reach goal
    mint_tokens(&env, &token_id, &contributor, goal);
    client.contribute_to_fundraising(&group_id, &contributor, &token_id, &goal);

    // Reset fundraising (makes it inactive)
    client.reset_fundraising(&group_id, &creator);
    
    // Verify fundraising is now inactive
    assert!(!client.get_fundraising_status(&group_id).is_active);

    // Deactivate and delete
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }
    
    // Should succeed since fundraising is inactive
    client.delete_group(&group_id, &creator);

    // Verify group is deleted
    let all_groups = client.get_all_groups();
    assert!(!all_groups.iter().any(|g| g.id == group_id));
}

#[test]
fn test_delete_group_with_zero_initial_usages() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group with 0 usages
    let group_id = BytesN::from_array(&env, &[22u8; 32]);
    let name = String::from_str(&env, "Zero Usage Group");
    let fee = 0;
    let amount = 10000;
    mint_tokens(&env, &token_id, &creator, amount);
    client.create(&group_id, &name, &creator, &fee, &token_id);

    // Verify group has 0 usages
    assert_eq!(client.get_remaining_usages(&group_id), 0);

    // Deactivate the group
    client.deactivate_group(&group_id, &creator);

    // Should be able to delete immediately since usages are already 0
    client.delete_group(&group_id, &creator);

    // Verify deletion
    let all_groups = client.get_all_groups();
    assert!(!all_groups.iter().any(|g| g.id == group_id));
}

#[test]
fn test_admin_delete_bypasses_deactivation_requirement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create an active group
    let group_id = BytesN::from_array(&env, &[23u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Verify group is active
    assert!(client.is_group_active(&group_id));
    assert_eq!(client.get_remaining_usages(&group_id), 10);

    // Admin can delete without deactivating or reducing usages
    client.admin_delete_group(&admin, &group_id);

    // Verify group is deleted
    let all_groups = client.get_all_groups();
    assert!(!all_groups.iter().any(|g| g.id == group_id));
}

#[test]
fn test_delete_group_event_emission() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[24u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Deactivate and prepare for deletion
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }

    // Delete the group
    client.delete_group(&group_id, &creator);

    // Verify GroupDeleted event was emitted
    let events = env.events().all();
    let delete_events: Vec<_> = events
        .iter()
        .filter(|e| {
            e.topics.len() > 0 && 
            e.topics.get(0).unwrap().to_string().contains("GroupDeleted")
        })
        .collect();
    
    assert!(delete_events.len() > 0, "GroupDeleted event should be emitted");
}

#[test]
fn test_delete_group_with_distributions_and_earnings() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[25u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Add members with different shares
    client.add_group_member(&group_id, &creator, &member1, &400);
    client.add_group_member(&group_id, &creator, &member2, &600);

    // Perform multiple distributions
    let distribute_amount1 = 2000i128;
    let distribute_amount2 = 3000i128;
    
    mint_tokens(&env, &token_id, &creator, distribute_amount1 + distribute_amount2);
    client.distribute(&group_id, &token_id, &distribute_amount1, &creator);
    client.distribute(&group_id, &token_id, &distribute_amount2, &creator);

    // Verify distributions and earnings exist
    let distributions_before = client.get_group_distributions(&group_id);
    assert_eq!(distributions_before.len(), 2);
    
    let member1_earnings_before = client.get_member_earnings(&member1, &group_id);
    let member2_earnings_before = client.get_member_earnings(&member2, &group_id);
    assert!(member1_earnings_before > 0);
    assert!(member2_earnings_before > 0);

    // Deactivate and delete
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }
    client.delete_group(&group_id, &creator);

    // Verify distributions and earnings are preserved after deletion
    let distributions_after = client.get_group_distributions(&group_id);
    assert_eq!(distributions_after.len(), 2);
    
    let member1_earnings_after = client.get_member_earnings(&member1, &group_id);
    let member2_earnings_after = client.get_member_earnings(&member2, &group_id);
    assert_eq!(member1_earnings_after, member1_earnings_before);
    assert_eq!(member2_earnings_after, member2_earnings_before);
}

#[test]
#[should_panic(expected = "NotFound")]
fn test_delete_already_deleted_group() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_autoshare_contract(&env, &admin);
    let token_name = String::from_str(&env, "Test Token");
    let token_symbol = String::from_str(&env, "TEST");
    let token_id = deploy_mock_token(&env, &token_name, &token_symbol);

    let client = crate::AutoShareContractClient::new(&env, &contract_id);

    client.initialize_admin(&admin);
    client.add_supported_token(&token_id, &admin);

    // Create a group
    let group_id = BytesN::from_array(&env, &[26u8; 32]);
    create_test_group(&env, &contract_id, &token_id, &creator, group_id.clone());

    // Deactivate and delete the group
    client.deactivate_group(&group_id, &creator);
    for _ in 0..10 {
        client.reduce_usage(&group_id);
    }
    client.delete_group(&group_id, &creator);

    // Verify group is deleted
    let all_groups = client.get_all_groups();
    assert!(!all_groups.iter().any(|g| g.id == group_id));

    // Try to delete the same group again - should fail with NotFound
    client.delete_group(&group_id, &creator);
}
