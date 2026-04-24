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

#[test]
fn test_member_removed_event_payload() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    // Two members: member1 = 70%, member2 = 30%
    let member1 = Address::generate(env);
    let member2 = Address::generate(env);
    let mut members = soroban_sdk::Vec::new(env);
    members.push_back(crate::base::types::GroupMember {
        address: member1.clone(),
        percentage: 70,
    });
    members.push_back(crate::base::types::GroupMember {
        address: member2.clone(),
        percentage: 30,
    });

    let group_id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        5,
        &token,
    );

    env.mock_all_auths();

    // Distribute so member2 has non-zero earnings
    mint_tokens(env, &token, &creator, 1000);
    client.distribute(&group_id, &token, &1000, &creator);

    client.remove_group_member(&group_id, &creator, &member2);

    let events = env.events().all();
    let removed_event = events
        .iter()
        .find(|e| Symbol::from_val(env, &e.1.get(0).unwrap()) == Symbol::new(env, "member_removed"))
        .expect("member_removed event not found");

    // topics[1] = group_id, topics[2] = member
    assert_eq!(
        crate::BytesN::<32>::from_val(env, &removed_event.1.get(1).unwrap()),
        group_id
    );
    assert_eq!(
        Address::from_val(env, &removed_event.1.get(2).unwrap()),
        member2
    );

    // data = map { pending_earnings: i128, removed_percentage: u32 }
    let data =
        soroban_sdk::Map::<soroban_sdk::Symbol, soroban_sdk::Val>::from_val(env, &removed_event.2);
    let removed_pct = u32::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "removed_percentage"))
            .unwrap(),
    );
    let pending_earn = i128::from_val(
        env,
        &data
            .get(soroban_sdk::Symbol::new(env, "pending_earnings"))
            .unwrap(),
    );

    assert_eq!(removed_pct, 30u32);
    assert_eq!(pending_earn, 300i128); // 30% of 1000
}
