use crate::test_utils::{
    deploy_autoshare_contract,
};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Env, testutils::Events, FromVal};

fn setup(env: &Env) -> (Address, AutoShareContractClient<'_>) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    (admin, client)
}

#[test]
fn test_protocol_fee_initial_state() {
    let env = Env::default();
    let (admin, client) = setup(&env);
    
    let (fee, recipient) = client.get_protocol_fee();
    assert_eq!(fee, 0);
    assert_eq!(recipient, admin); // Default to admin
}

#[test]
fn test_set_protocol_fee_admin_only() {
    let env = Env::default();
    let (admin, client) = setup(&env);
    let non_admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();
    
    // Set as non-admin should fail
    let result = client.try_set_protocol_fee(&100, &recipient, &non_admin);
    assert!(result.is_err());

    // Set as admin should succeed
    client.set_protocol_fee(&100, &recipient, &admin);
    let (fee, new_recipient) = client.get_protocol_fee();
    assert_eq!(fee, 100);
    assert_eq!(new_recipient, recipient);
}

#[test]
fn test_protocol_fee_boundary_values() {
    let env = Env::default();
    let (admin, client) = setup(&env);
    let recipient = Address::generate(&env);
    env.mock_all_auths();

    // Max fee 100% (10000 bps)
    client.set_protocol_fee(&10000, &recipient, &admin);
    assert_eq!(client.get_protocol_fee().0, 10000);

    // Exceeding 100% should fail
    let result = client.try_set_protocol_fee(&10001, &recipient, &admin);
    assert!(result.is_err());

    // Zero fee
    client.set_protocol_fee(&0, &recipient, &admin);
    assert_eq!(client.get_protocol_fee().0, 0);
}

#[test]
fn test_protocol_fee_read_tracking_event() {
    let env = Env::default();
    let (admin, client) = setup(&env);
    env.mock_all_auths();

    client.set_protocol_fee(&150, &admin, &admin);
    
    // Clear previous events
    let _ = env.events().all();

    // Trigger read
    client.get_protocol_fee();

    let events = env.events().all();
    let found = events.iter().any(|e| {
        let symbol = soroban_sdk::Symbol::from_val(&env, &e.1.get(0).unwrap());
        symbol == soroban_sdk::Symbol::new(&env, "protocol_fee_read")
    });
    assert!(found, "ProtocolFeeRead event not found");
}

#[test]
fn test_protocol_fee_update_event() {
    let env = Env::default();
    let (admin, client) = setup(&env);
    let recipient = Address::generate(&env);
    env.mock_all_auths();

    let _ = env.events().all();
    client.set_protocol_fee(&200, &recipient, &admin);

    let events = env.events().all();
    let found = events.iter().any(|e| {
        let symbol = soroban_sdk::Symbol::from_val(&env, &e.1.get(0).unwrap());
        symbol == soroban_sdk::Symbol::new(&env, "protocol_fee_updated")
    });
    assert!(found, "ProtocolFeeUpdated event not found");
}
