use crate::test_utils::{create_test_group, create_test_members, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN};

#[test]
fn test_deactivate_payment_group_success_updates_state() {
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

    assert!(client.is_group_active(&group_id));

    client.deactivate_payment_group(&group_id, &creator);

    assert!(!client.is_group_active(&group_id));
    assert!(!client.get_group_summary(&group_id).is_active);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_deactivate_payment_group_unauthorized_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let unauthorized = test_env.users.get(1).unwrap().clone();
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

    client.deactivate_payment_group(&group_id, &unauthorized);
}

#[test]
#[should_panic(expected = "GroupAlreadyInactive")]
fn test_deactivate_payment_group_already_inactive_fails() {
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
        2,
        &token,
    );

    client.deactivate_payment_group(&group_id, &creator);
    client.deactivate_payment_group(&group_id, &creator);
}

#[test]
#[should_panic(expected = "NotFound")]
fn test_deactivate_payment_group_missing_group_fails() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let caller = test_env.users.get(0).unwrap().clone();
    let missing_group_id = BytesN::from_array(&test_env.env, &[9u8; 32]);

    client.deactivate_payment_group(&missing_group_id, &caller);
}

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_deactivate_payment_group_blocks_new_distributions() {
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
        2,
        &token,
    );

    client.deactivate_payment_group(&group_id, &creator);
    client.distribute(&group_id, &token, &100, &creator);
}

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_deactivate_payment_group_blocks_member_additions() {
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
        2,
        &token,
    );

    client.deactivate_payment_group(&group_id, &creator);

    let new_member = Address::generate(&test_env.env);
    client.add_group_member(&group_id, &creator, &new_member, &10);
}
