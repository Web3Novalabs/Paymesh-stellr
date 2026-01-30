use crate::test_utils::{
    assert_balance, assert_group_exists, create_test_group, create_test_users,
    deploy_autoshare_contract, deploy_mock_token, mint_tokens, setup_test_env,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_setup_test_env() {
    let test_env = setup_test_env();
    assert_eq!(test_env.users.len(), 3);
    assert!(!test_env.mock_tokens.is_empty());
    // Ensure admin is not one of the users (impl detail check)
    assert!(test_env.admin != test_env.users.get(0).unwrap());
}

#[test]
fn test_create_zero_users() {
    let env = Env::default();
    env.mock_all_auths();
    let users = create_test_users(&env, 0);
    assert_eq!(users.len(), 0);
}

#[test]
fn test_mock_token_helpers() {
    let env = Env::default();
    env.mock_all_auths();

    let token = deploy_mock_token(
        &env,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "TST"),
    );
    let user = Address::generate(&env);

    mint_tokens(&env, &token, &user, 100);
    assert_balance(&env, &token, &user, 100);
}

#[test]
fn test_autoshare_helpers() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = deploy_autoshare_contract(&env, &Address::generate(&env));
    let creator = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(crate::base::types::GroupMember {
        address: Address::generate(&env),
        percentage: 100,
    });

    let group_id = create_test_group(
        &env,
        &contract,
        &creator,
        &members,
        1,
        &Address::generate(&env), // Dummy token
    );

    assert_group_exists(&env, &contract, &group_id);
}
