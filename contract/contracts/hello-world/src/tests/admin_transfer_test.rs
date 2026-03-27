use crate::test_utils::{deploy_autoshare_contract, deploy_mock_token, mint_tokens};
use crate::AutoShareContractClient;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, FromVal, String, Symbol,
};

fn setup(env: &Env) -> (Address, Address, AutoShareContractClient<'_>) {
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    (admin, contract_id, client)
}

fn new_token(env: &Env) -> Address {
    deploy_mock_token(
        env,
        &String::from_str(env, "Test"),
        &String::from_str(env, "TST"),
    )
}

// 1. Transfer succeeds, get_admin reflects new admin, old admin loses privileges
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_transfer_admin_old_admin_loses_privileges() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
    client.pause(&admin); // should panic — old admin no longer authorized
}

// 2. Transfer to self — admin remains the same, can still act
#[test]
fn test_transfer_admin_to_self() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    client.transfer_admin(&admin, &admin);

    assert_eq!(client.get_admin(), admin);
    client.pause(&admin);
    assert!(client.get_paused_status());
}

// 3. Non-admin attempting transfer returns Unauthorized
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_transfer_admin_non_admin_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    let non_admin = Address::generate(&env);
    client.transfer_admin(&non_admin, &Address::generate(&env));
}

// 4. New admin can immediately perform admin actions (pause, add_token, withdraw)
#[test]
fn test_new_admin_can_perform_admin_actions() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, contract_id, client) = setup(&env);

    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);

    client.pause(&new_admin);
    assert!(client.get_paused_status());
    client.unpause(&new_admin);

    let token = new_token(&env);
    client.add_supported_token(&token, &new_admin);
    assert!(client.is_token_supported(&token));

    mint_tokens(&env, &token, &contract_id, 500);
    client.withdraw(&new_admin, &token, &500, &Address::generate(&env));
}

// 5. Transfer emits AdminTransferred event with correct old and new admin
#[test]
fn test_transfer_admin_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);

    let events = env.events().all();
    let evt = events
        .iter()
        .find(|e| {
            Symbol::from_val(&env, &e.1.get(0).unwrap()) == Symbol::new(&env, "admin_transferred")
        })
        .expect("admin_transferred event not found");

    assert_eq!(Address::from_val(&env, &evt.1.get(1).unwrap()), admin);
    assert_eq!(Address::from_val(&env, &evt.2), new_admin);
}

// 6. Double transfer A->B->C: A loses privileges after first transfer, B after second
#[test]
#[should_panic(expected = "Unauthorized")]
fn test_double_transfer_original_loses_privileges() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin_a, _, client) = setup(&env);

    let admin_b = Address::generate(&env);
    let admin_c = Address::generate(&env);

    client.transfer_admin(&admin_a, &admin_b);
    client.transfer_admin(&admin_b, &admin_c);

    assert_eq!(client.get_admin(), admin_c);
    client.pause(&admin_a); // should panic
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_double_transfer_intermediate_loses_privileges() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin_a, _, client) = setup(&env);

    let admin_b = Address::generate(&env);
    let admin_c = Address::generate(&env);

    client.transfer_admin(&admin_a, &admin_b);
    client.transfer_admin(&admin_b, &admin_c);

    client.pause(&admin_b); // should panic
}

// 7. Transfer while contract is paused still succeeds
#[test]
fn test_transfer_admin_while_paused_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    client.pause(&admin);
    let new_admin = Address::generate(&env);
    client.transfer_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
    client.unpause(&new_admin);
    assert!(!client.get_paused_status());
}
