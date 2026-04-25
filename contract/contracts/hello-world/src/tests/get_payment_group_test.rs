use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, create_test_members, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, String};

#[test]
fn test_get_payment_group_returns_metadata_status_and_member_count() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 3);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        7,
        &token,
    );

    let summary = client.get_group_summary(&group_id);

    assert_eq!(summary.id, group_id);
    assert_eq!(summary.name, String::from_str(&test_env.env, "Test Group"));
    assert_eq!(summary.creator, creator);
    assert_eq!(summary.member_count, 3);
    assert!(summary.is_active);
    assert_eq!(summary.remaining_usages, 7);
    assert!(!summary.has_active_fundraising);
    assert_eq!(summary.total_distributions, 0);
    assert_eq!(client.get_group_member_count(&group_id), 3);
}

#[test]
fn test_get_payment_group_reflects_inactive_status() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 2);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        4,
        &token,
    );

    client.deactivate_group(&group_id, &creator);

    let summary = client.get_group_summary(&group_id);
    assert_eq!(summary.member_count, 2);
    assert!(!summary.is_active);
    assert_eq!(summary.remaining_usages, 4);
}

#[test]
#[should_panic(expected = "NotFound")]
fn test_get_payment_group_nonexistent_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let missing_group_id = BytesN::from_array(&test_env.env, &[9u8; 32]);
    client.get_group_summary(&missing_group_id);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_get_payment_group_unauthorized_update_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let non_creator = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 1);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        5,
        &token,
    );

    let mut updated_members = soroban_sdk::Vec::new(&test_env.env);
    updated_members.push_back(GroupMember {
        address: non_creator.clone(),
        percentage: 100,
    });

    client.update_members(&group_id, &non_creator, &updated_members);
}
