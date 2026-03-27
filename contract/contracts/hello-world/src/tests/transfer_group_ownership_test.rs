use crate::test_utils::{mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, String};

#[test]
fn test_transfer_group_ownership_success() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let new_creator = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    // Fund creator
    mint_tokens(env, &token, &creator, 1000000);

    // Create group
    let group_id = BytesN::from_array(env, &[1u8; 32]);
    let name = String::from_str(env, "Test Group");
    client.create(&group_id, &name, &creator, &100, &token);

    // Verify initial creator
    let details = client.get(&group_id);
    assert_eq!(details.creator, creator);

    // Transfer ownership
    client.transfer_group_ownership(&group_id, &creator, &new_creator);

    // Verify new creator
    let details_after = client.get(&group_id);
    assert_eq!(details_after.creator, new_creator);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_transfer_group_ownership_unauthorized() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let unauthorized_caller = test_env.users.get(1).unwrap();
    let new_creator = test_env.users.get(2).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    mint_tokens(env, &token, &creator, 1000000);

    let group_id = BytesN::from_array(env, &[2u8; 32]);
    let name = String::from_str(env, "Test Group");
    client.create(&group_id, &name, &creator, &100, &token);

    // Try to transfer with unauthorized caller - should fail
    client.transfer_group_ownership(&group_id, &unauthorized_caller, &new_creator);
}
