use crate::test_utils::{create_test_group, create_test_members, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::String;

#[test]
fn test_update_payment_group_name_success() {
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
        5,
        &token,
    );

    let new_name = String::from_str(&test_env.env, "Updated Group Name");
    client.update_payment_group(&group_id, &creator, &Some(new_name.clone()), &None, &None);

    let details = client.get(&group_id);
    assert_eq!(details.name, new_name);
}

#[test]
fn test_update_payment_group_metadata_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 1);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    let new_metadata = String::from_str(&test_env.env, "New Metadata Content");
    client.update_payment_group(
        &group_id,
        &creator,
        &None,
        &Some(new_metadata.clone()),
        &None,
    );

    let details = client.get(&group_id);
    assert_eq!(details.metadata, new_metadata);
}

#[test]
fn test_update_payment_group_admin_rotation_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let new_creator = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 1);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    client.update_payment_group(
        &group_id,
        &creator,
        &None,
        &None,
        &Some(new_creator.clone()),
    );

    let details = client.get(&group_id);
    assert_eq!(details.creator, new_creator);

    // Verify old creator can no longer update
    let result = client.try_update_payment_group(&group_id, &creator, &None, &None, &None);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_update_payment_group_unauthorized_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let unauthorized_user = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 1);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    let new_name = String::from_str(&test_env.env, "Sneaky Name Change");
    client.update_payment_group(&group_id, &unauthorized_user, &Some(new_name), &None, &None);
}

#[test]
#[should_panic(expected = "EmptyName")]
fn test_update_payment_group_invalid_name_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let members = create_test_members(&test_env.env, 1);

    let group_id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    let empty_name = String::from_str(&test_env.env, "   ");
    client.update_payment_group(&group_id, &creator, &Some(empty_name), &None, &None);
}

#[test]
#[should_panic(expected = "NotFound")]
fn test_update_payment_group_nonexistent_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let caller = test_env.users.get(0).unwrap().clone();
    let fake_id = soroban_sdk::BytesN::from_array(&test_env.env, &[7u8; 32]);

    client.update_payment_group(&fake_id, &caller, &None, &None, &None);
}
