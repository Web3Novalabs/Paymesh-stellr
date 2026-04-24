use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, setup_test_env};
use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Vec};

#[test]
fn test_set_protocol_fee_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();

    // Set to 5%
    client.set_protocol_fee(&admin, &5);
    assert_eq!(client.get_protocol_fee(), 5);

    // Set to 0%
    client.set_protocol_fee(&admin, &0);
    assert_eq!(client.get_protocol_fee(), 0);

    // Set to 100%
    client.set_protocol_fee(&admin, &100);
    assert_eq!(client.get_protocol_fee(), 100);
}

#[test]
#[should_panic]
fn test_set_protocol_fee_invalid_percentage() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();

    // Set to 101% (should panic)
    client.set_protocol_fee(&admin, &101);
}

#[test]
#[should_panic]
fn test_set_protocol_fee_unauthorized() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let non_admin = test_env.users.get(0).unwrap().clone();

    // Non-admin tries to set fee
    client.set_protocol_fee(&non_admin, &5);
}

#[test]
fn test_set_group_protocol_fee_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    // Set global fee to 5%
    client.set_protocol_fee(&admin, &5);

    // Set group fee to 10%
    client.set_group_protocol_fee(&admin, &id, &10);

    assert_eq!(client.get_group_protocol_fee(&id), 10);
    assert_eq!(client.get_protocol_fee(), 5); // Global fee remains 5%
}

#[test]
fn test_get_group_protocol_fee_fallback() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    // Set global fee to 5%
    client.set_protocol_fee(&admin, &5);

    // Group fee not set, should fallback to global
    assert_eq!(client.get_group_protocol_fee(&id), 5);
}

#[test]
#[should_panic]
fn test_set_group_protocol_fee_non_existent_group() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let id = BytesN::from_array(&test_env.env, &[9u8; 32]);

    client.set_group_protocol_fee(&admin, &id, &10);
}

#[test]
#[should_panic]
fn test_set_group_protocol_fee_invalid_percentage() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let mut members = Vec::new(&test_env.env);
    members.push_back(GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    // Set group fee to 101% (should panic)
    client.set_group_protocol_fee(&admin, &id, &101);
}

#[test]
#[should_panic]
fn test_set_protocol_fee_overflow_check() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();

    // Use a very large u32 value
    client.set_protocol_fee(&admin, &u32::MAX);
}
