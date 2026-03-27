use crate::test_utils::{create_test_group, create_test_members, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, FromVal, Symbol,
};

#[test]
fn test_token_added_event() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let admin = &test_env.admin;

    let token = Address::generate(env);

    env.mock_all_auths();
    client.add_supported_token(&token, admin);

    let events = env.events().all();
    let last_event = events.last().unwrap();

    // topics: [SYMBOL(token_added), admin, token]
    assert_eq!(
        Symbol::from_val(env, &last_event.1.get(0).unwrap()),
        Symbol::new(env, "token_added")
    );
    assert_eq!(
        Address::from_val(env, &last_event.1.get(1).unwrap()),
        admin.clone()
    );
    assert_eq!(
        Address::from_val(env, &last_event.1.get(2).unwrap()),
        token.clone()
    );
}

#[test]
fn test_token_removed_event() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let admin = &test_env.admin;

    let token = Address::generate(env);

    env.mock_all_auths();
    client.add_supported_token(&token, admin);
    client.remove_supported_token(&token, admin);

    let events = env.events().all();
    let last_event = events.last().unwrap();

    // topics: [SYMBOL(token_removed), admin, token]
    assert_eq!(
        Symbol::from_val(env, &last_event.1.get(0).unwrap()),
        Symbol::new(env, "token_removed")
    );
    assert_eq!(
        Address::from_val(env, &last_event.1.get(1).unwrap()),
        admin.clone()
    );
    assert_eq!(
        Address::from_val(env, &last_event.1.get(2).unwrap()),
        token.clone()
    );
}

#[test]
fn test_fundraising_completed_event() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    let members = create_test_members(env, 2);
    let group_id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        10,
        &token,
    );

    env.mock_all_auths();

    let target_amount = 100i128;
    client.start_fundraising(&group_id, &creator, &target_amount);

    mint_tokens(env, &token, &contributor, 100);
    client.contribute(&group_id, &token, &100, &contributor);

    let events = env.events().all();

    // FundraisingCompleted should be emitted after Contribution event
    // Find the FundraisingCompleted event
    let fundraiser_completed_event = events
        .iter()
        .find(|e| {
            Symbol::from_val(env, &e.1.get(0).unwrap()) == Symbol::new(env, "fundraising_completed")
        })
        .expect("fundraising_completed event not found");

    // topics: [SYMBOL(fundraising_completed), group_id]
    assert_eq!(
        crate::BytesN::<32>::from_val(env, &fundraiser_completed_event.1.get(1).unwrap()),
        group_id
    );
}
