use crate::test_utils::setup_test_env;
use crate::AutoShareContractClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Vec};

#[test]
fn test_contribute_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let member1 = test_env.users.get(2).unwrap();
    let member2 = Address::generate(&test_env.env);
    let token = test_env.mock_tokens.get(0).unwrap();

    // 1. Setup Group
    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    // client.add_supported_token(&token, admin); // Redundant

    test_env.env.mock_all_auths();
    // Fund creator
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // 2. Add Members
    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: member1.clone(),
        percentage: 60,
    });
    members.push_back(crate::base::types::GroupMember {
        address: member2.clone(),
        percentage: 40,
    });
    client.update_members(&group_id, &creator, &members);

    // 3. Start Fundraising
    let target_amount = 1000i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    // 4. Contribute
    let contribution_amount = 500i128;

    // Contributor needs funds
    crate::test_utils::fund_user_with_tokens(
        &test_env.env,
        &token,
        &contributor,
        contribution_amount,
    );

    client.contribute(&group_id, &token, &contribution_amount, &contributor);

    // 5. Verify State
    let status = client.get_fundraising_status(&group_id);
    assert_eq!(status.total_raised, contribution_amount);
    assert!(status.is_active);

    // Verify Distributions
    let earnings1 = client.get_member_earnings(&member1, &group_id);
    let earnings2 = client.get_member_earnings(&member2, &group_id);
    assert_eq!(earnings1, 300); // 60% of 500
    assert_eq!(earnings2, 200); // 40% of 500

    // Verify Contributions recorded
    let g_contributions = client.get_group_contributions(&group_id);
    assert_eq!(g_contributions.len(), 1);
    assert_eq!(g_contributions.get(0).unwrap().amount, contribution_amount);
    assert_eq!(g_contributions.get(0).unwrap().contributor, contributor);

    let u_contributions = client.get_user_contributions(&contributor);
    assert_eq!(u_contributions.len(), 1);
    assert_eq!(u_contributions.get(0).unwrap().group_id, group_id);

    // (New) Check that the emit_contribution event would be triggered (manual check: no direct event query API)
    // This is a placeholder to indicate the event is expected and should be visible in the event log.
    // In a real test, you would query the event log if supported by the test framework.
}

#[test]
fn test_contribute_completes_fundraising() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let member1 = test_env.users.get(2).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[2u8; 32]);
    // client.add_supported_token(&token, admin); // Redundant

    test_env.env.mock_all_auths();
    // Fund creator
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: member1.clone(),
        percentage: 100,
    });
    client.update_members(&group_id, &creator, &members);

    let target_amount = 1000i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    // Fund contributor
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor, target_amount);

    // Contribute exact amount
    client.contribute(&group_id, &token, &target_amount, &contributor);

    let status = client.get_fundraising_status(&group_id);
    assert_eq!(status.total_raised, target_amount);
    assert!(!status.is_active); // Should be inactive now
}

#[test]
#[should_panic(expected = "FundraisingNotActive")]
fn test_contribute_fundraising_not_active() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[3u8; 32]);
    // client.add_supported_token(&token, admin); // Redundant
    test_env.env.mock_all_auths();
    // Fund creator
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    // Contribute without starting fundraising
    client.contribute(&group_id, &token, &100, &contributor);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_contribute_paused() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[4u8; 32]);
    // client.add_supported_token(&token, admin); // Redundant
    test_env.env.mock_all_auths();
    // Fund creator
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    client.start_fundraising(&group_id, &creator, &1000);

    client.pause(admin);
    client.contribute(&group_id, &token, &100, &contributor);
}

#[test]
fn test_contribute_exceeds_target() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();
    let member1 = test_env.users.get(2).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[5u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );
    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: member1.clone(),
        percentage: 100,
    });
    client.update_members(&group_id, &creator, &members);

    let target_amount = 1000i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    let contribution_amount = 1500i128;
    crate::test_utils::fund_user_with_tokens(
        &test_env.env,
        &token,
        &contributor,
        contribution_amount,
    );

    // (2) Contribute exceeding target
    client.contribute(&group_id, &token, &contribution_amount, &contributor);

    let status = client.get_fundraising_status(&group_id);
    assert_eq!(status.total_raised, contribution_amount);
    // Campaign should be closed
    assert!(!status.is_active);

    // (8) single large contribution far exceeds target
    let earnings = client.get_member_earnings(&member1, &group_id);
    assert_eq!(earnings, contribution_amount);
}

#[test]
#[should_panic(expected = "FundraisingNotActive")]
fn test_contribute_after_closed() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[6u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    let target_amount = 1000i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor, 1500);
    // Reach target
    client.contribute(&group_id, &token, &1000, &contributor);

    // (3) Contribution after campaign is closed
    client.contribute(&group_id, &token, &100, &contributor);
}

#[test]
fn test_fundraising_read_functions_after_target_met() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[7u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    let target_amount = 1000i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor, 1000);
    client.contribute(&group_id, &token, &1000, &contributor);

    // (4) verify is_fundraising_target_reached
    assert!(client.is_fundraising_target_reached(&group_id));
    // (5) verify get_fundraising_remaining returns 0
    assert_eq!(client.get_fundraising_remaining(&group_id), 0);
    // (6) verify get_fundraising_progress returns 100
    assert_eq!(client.get_fundraising_progress(&group_id), 100);
}

#[test]
fn test_multiple_small_contributions_reach_target() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let _admin = &test_env.admin;
    let creator = test_env.users.get(0).unwrap();
    let contributor1 = test_env.users.get(1).unwrap();
    let contributor2 = test_env.users.get(2).unwrap();
    let contributor3 = Address::generate(&test_env.env);
    let token = test_env.mock_tokens.get(0).unwrap();

    let group_id = BytesN::from_array(&test_env.env, &[8u8; 32]);
    test_env.env.mock_all_auths();
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &creator, 1000);
    client.create(
        &group_id,
        &soroban_sdk::String::from_str(&test_env.env, "Test Group"),
        &creator,
        &10,
        &token,
    );

    let target_amount = 1000i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor1, 300);
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor2, 400);
    crate::test_utils::fund_user_with_tokens(&test_env.env, &token, &contributor3, 300);

    // (7) multiple small contributions
    client.contribute(&group_id, &token, &300, &contributor1);
    assert!(client.get_fundraising_status(&group_id).is_active);

    client.contribute(&group_id, &token, &400, &contributor2);
    assert!(client.get_fundraising_status(&group_id).is_active);

    // exact reach
    client.contribute(&group_id, &token, &300, &contributor3);

    let status = client.get_fundraising_status(&group_id);
    assert_eq!(status.total_raised, 1000);
    assert!(!status.is_active);
}
