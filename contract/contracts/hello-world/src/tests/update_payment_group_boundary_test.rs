use crate::test_utils::{create_test_group, create_test_members, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::String;

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_update_payment_group_when_paused_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    client.pause(&test_env.admin);

    let new_name = String::from_str(&test_env.env, "New Name");
    client.update_payment_group(&group_id, &creator, &Some(new_name), &None, &None);
}

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_update_payment_group_inactive_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    client.deactivate_payment_group(&group_id, &creator);

    let new_name = String::from_str(&test_env.env, "New Name");
    client.update_payment_group(&group_id, &creator, &Some(new_name), &None, &None);
}

#[test]
fn test_update_payment_group_name_max_length_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    // 60 characters
    let max_name = String::from_str(&test_env.env, "A".repeat(60).as_str());
    client.update_payment_group(&group_id, &creator, &Some(max_name.clone()), &None, &None);

    let details = client.get(&group_id);
    assert_eq!(details.name, max_name);
}

#[test]
#[should_panic(expected = "EmptyName")]
fn test_update_payment_group_name_too_long_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    // 61 characters
    let too_long_name = String::from_str(&test_env.env, "A".repeat(61).as_str());
    client.update_payment_group(&group_id, &creator, &Some(too_long_name), &None, &None);
}

#[test]
fn test_update_payment_group_metadata_large_size_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    // Large metadata (e.g., 2KB)
    let large_metadata = String::from_str(&test_env.env, "M".repeat(2048).as_str());
    client.update_payment_group(
        &group_id,
        &creator,
        &None,
        &Some(large_metadata.clone()),
        &None,
    );

    let details = client.get(&group_id);
    assert_eq!(details.metadata, large_metadata);
}

#[test]
fn test_update_payment_group_no_changes_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    let before = client.get(&group_id);

    // Call with all None
    client.update_payment_group(&group_id, &creator, &None, &None, &None);

    let after = client.get(&group_id);

    assert_eq!(before.name, after.name);
    assert_eq!(before.metadata, after.metadata);
    assert_eq!(before.creator, after.creator);
}

#[test]
fn test_update_payment_group_rotate_to_self() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &create_test_members(&test_env.env, 1),
        1,
        &token,
    );

    client.update_payment_group(&group_id, &creator, &None, &None, &Some(creator.clone()));

    let details = client.get(&group_id);
    assert_eq!(details.creator, creator);
}
