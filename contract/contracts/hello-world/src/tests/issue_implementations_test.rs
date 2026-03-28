use crate::test_utils::setup_test_env;
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, Vec};

#[test]
fn test_set_fundraising_target_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[7u8; 32]);
    test_env.env.mock_all_auths();

    // Setup group
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // Start fundraising with target 1000
    client.start_fundraising(&group_id, &creator, &1000);

    // Update target to 2000
    client.set_fundraising_target(&group_id, &creator, &2000);

    let status = client.get_fundraising_status(&group_id);
    assert_eq!(status.target_amount, 2000);
}

#[test]
#[should_panic(expected = "InvalidTarget")]
fn test_set_fundraising_target_fails_below_raised() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[8u8; 32]);
    test_env.env.mock_all_auths();

    // Setup group
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // Start fundraising with target 1000
    client.start_fundraising(&group_id, &creator, &1000);

    // Contribute 500
    let contributor = test_env.users.get(1).unwrap();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor, 500);
    client.contribute(&group_id, &token, &500, &contributor);

    // Try to set target to 400 (below total_raised of 500)
    client.set_fundraising_target(&group_id, &creator, &400);
}

#[test]
fn test_set_max_members_and_enforcement() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = client.get_admin();
    test_env.env.mock_all_auths();

    // Set max members to 2
    client.set_max_members(&admin, &2);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[9u8; 32]);

    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // Add members using update_members to ensure percentage is 100
    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: test_env.users.get(1).unwrap().clone(),
        percentage: 50,
    });
    members.push_back(crate::base::types::GroupMember {
        address: test_env.users.get(2).unwrap().clone(),
        percentage: 50,
    });
    client.update_members(&group_id, &creator, &members);

    // Verify 2 members added
    let members = client.get_group_members(&group_id);
    assert_eq!(members.len(), 2);
}

#[test]
#[should_panic(expected = "MaxMembersExceeded")]
fn test_add_member_fails_when_max_exceeded() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = client.get_admin();
    test_env.env.mock_all_auths();

    // Set max members to 1
    client.set_max_members(&admin, &1);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[10u8; 32]);

    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // Add 1st member (100% to satisfy validate_members)
    client.add_group_member(&group_id, &creator, &test_env.users.get(1).unwrap(), &100);

    // Try to add 2nd member - should fail because max members is 1
    // (Regardless of percentage, it should fail the length check first)
    client.add_group_member(&group_id, &creator, &test_env.users.get(2).unwrap(), &10);
}

#[test]
#[should_panic(expected = "MaxMembersExceeded")]
fn test_update_members_fails_when_max_exceeded() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = client.get_admin();
    test_env.env.mock_all_auths();

    // Set max members to 1
    client.set_max_members(&admin, &1);

    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[11u8; 32]);

    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // Try to update with 2 members
    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: test_env.users.get(1).unwrap().clone(),
        percentage: 50,
    });
    members.push_back(crate::base::types::GroupMember {
        address: test_env.users.get(2).unwrap().clone(),
        percentage: 50,
    });

    client.update_members(&group_id, &creator, &members);
}
