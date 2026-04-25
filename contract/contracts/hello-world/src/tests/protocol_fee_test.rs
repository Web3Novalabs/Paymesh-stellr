use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Vec};

#[test]
fn test_set_protocol_fee_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let recipient = Address::generate(&test_env.env);

    // Set to 5 bps
    client.set_protocol_fee(&5, &recipient, &admin);
    let (fee, rec) = client.get_protocol_fee();
    assert_eq!(fee, 5);
    assert_eq!(rec, recipient);

    // Set to 0
    client.set_protocol_fee(&0, &recipient, &admin);
    assert_eq!(client.get_protocol_fee().0, 0);

    // Set to 10000 bps (100%)
    client.set_protocol_fee(&10000, &recipient, &admin);
    assert_eq!(client.get_protocol_fee().0, 10000);
}

#[test]
#[should_panic]
fn test_set_protocol_fee_invalid_percentage() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let recipient = Address::generate(&test_env.env);

    // > 10000 bps should panic
    client.set_protocol_fee(&10001, &recipient, &admin);
}

#[test]
#[should_panic]
fn test_set_protocol_fee_unauthorized() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let non_admin = test_env.users.get(0).unwrap().clone();
    let recipient = Address::generate(&test_env.env);

    client.set_protocol_fee(&5, &recipient, &non_admin);
}

#[test]
fn test_set_group_protocol_fee_success() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let recipient = Address::generate(&test_env.env);

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

    // Set global fee to 5 bps
    client.set_protocol_fee(&5, &recipient, &admin);

    // Set group-specific fee to 10 bps
    client.set_group_protocol_fee(&admin, &id, &10);

    assert_eq!(client.get_group_protocol_fee(&id), 10);
    assert_eq!(client.get_protocol_fee().0, 5); // global unchanged
}

#[test]
fn test_get_group_protocol_fee_fallback() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let recipient = Address::generate(&test_env.env);

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

    // Set global fee to 5 bps; no group override → fallback to global
    client.set_protocol_fee(&5, &recipient, &admin);
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

    client.set_group_protocol_fee(&admin, &id, &101);
}

#[test]
#[should_panic]
fn test_set_protocol_fee_overflow_check() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let recipient = Address::generate(&test_env.env);

    client.set_protocol_fee(&u32::MAX, &recipient, &admin);
}
