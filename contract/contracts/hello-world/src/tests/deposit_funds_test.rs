#![allow(unused_imports)]

use crate::test_utils::{deploy_autoshare_contract, deploy_mock_token, mint_tokens};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

// ─── helpers ────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (Address, Address, AutoShareContractClient<'_>) {
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);

    let token = deploy_mock_token(
        env,
        &String::from_str(env, "Test Token"),
        &String::from_str(env, "TEST"),
    );
    client.add_supported_token(&token, &admin);
    (admin, token, client)
}

fn make_group(
    env: &Env,
    client: &AutoShareContractClient,
    token: &Address,
    creator: &Address,
    seed: u8,
) -> BytesN<32> {
    let id = BytesN::from_array(env, &[seed; 32]);
    mint_tokens(env, token, creator, 10_000);
    client.create(
        &id,
        &String::from_str(env, "Test Group"),
        creator,
        &1,
        token,
    );
    id
}

// ─── Happy path ──────────────────────────────────────────────────────────────

/// Basic deposit increases the treasury balance by the deposited amount.
#[test]
fn test_deposit_increases_treasury_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 1);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);

    assert_eq!(client.get_group_treasury_balance(&id, &token), 0);
    client.deposit_funds(&id, &token, &200, &depositor);
    assert_eq!(client.get_group_treasury_balance(&id, &token), 200);
}

/// Multiple deposits from the same depositor accumulate correctly.
#[test]
fn test_multiple_deposits_accumulate() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 2);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 1_000);

    client.deposit_funds(&id, &token, &100, &depositor);
    client.deposit_funds(&id, &token, &250, &depositor);
    client.deposit_funds(&id, &token, &50, &depositor);

    assert_eq!(client.get_group_treasury_balance(&id, &token), 400);
}

/// Deposits from different depositors all credit the same group treasury.
#[test]
fn test_deposits_from_different_depositors_accumulate() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 3);

    let d1 = Address::generate(&env);
    let d2 = Address::generate(&env);
    mint_tokens(&env, &token, &d1, 500);
    mint_tokens(&env, &token, &d2, 500);

    client.deposit_funds(&id, &token, &300, &d1);
    client.deposit_funds(&id, &token, &150, &d2);

    assert_eq!(client.get_group_treasury_balance(&id, &token), 450);
}

/// Deposit of exactly 1 (minimum valid amount) is accepted.
#[test]
fn test_deposit_minimum_amount_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 4);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 100);

    client.deposit_funds(&id, &token, &1, &depositor);
    assert_eq!(client.get_group_treasury_balance(&id, &token), 1);
}

/// Deposit of a very large amount is accepted (no artificial cap).
#[test]
fn test_deposit_large_amount_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 5);

    let depositor = Address::generate(&env);
    let large: i128 = 1_000_000_000_000;
    mint_tokens(&env, &token, &depositor, large);

    client.deposit_funds(&id, &token, &large, &depositor);
    assert_eq!(client.get_group_treasury_balance(&id, &token), large);
}

// ─── History recording ───────────────────────────────────────────────────────

/// Each deposit is appended to the group's deposit history.
#[test]
fn test_deposit_appended_to_group_history() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 6);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 1_000);

    client.deposit_funds(&id, &token, &100, &depositor);
    client.deposit_funds(&id, &token, &200, &depositor);

    let history = client.get_group_deposit_history(&id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().amount, 100);
    assert_eq!(history.get(1).unwrap().amount, 200);
}

/// Each deposit is appended to the depositor's personal history.
#[test]
fn test_deposit_appended_to_depositor_history() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 7);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 1_000);

    client.deposit_funds(&id, &token, &75, &depositor);
    client.deposit_funds(&id, &token, &125, &depositor);

    let history = client.get_depositor_history(&depositor);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().amount, 75);
    assert_eq!(history.get(1).unwrap().amount, 125);
}

/// Deposit records carry the correct group_id, depositor, token, and amount.
#[test]
fn test_deposit_record_fields_are_correct() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 8);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);

    client.deposit_funds(&id, &token, &333, &depositor);

    let record = client.get_group_deposit_history(&id).get(0).unwrap();
    assert_eq!(record.group_id, id);
    assert_eq!(record.depositor, depositor);
    assert_eq!(record.token, token);
    assert_eq!(record.amount, 333);
}

/// Depositor history across two different groups is tracked independently.
#[test]
fn test_depositor_history_spans_multiple_groups() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id_a = make_group(&env, &client, &token, &creator, 9);
    let id_b = make_group(&env, &client, &token, &creator, 10);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 1_000);

    client.deposit_funds(&id_a, &token, &100, &depositor);
    client.deposit_funds(&id_b, &token, &200, &depositor);

    let history = client.get_depositor_history(&depositor);
    assert_eq!(history.len(), 2);
}

// ─── Treasury isolation ──────────────────────────────────────────────────────

/// Depositing into group A does not affect group B's treasury.
#[test]
fn test_deposit_does_not_bleed_into_sibling_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id_a = make_group(&env, &client, &token, &creator, 11);
    let id_b = make_group(&env, &client, &token, &creator, 12);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);

    client.deposit_funds(&id_a, &token, &400, &depositor);

    assert_eq!(client.get_group_treasury_balance(&id_a, &token), 400);
    assert_eq!(client.get_group_treasury_balance(&id_b, &token), 0);
}

/// Two different tokens deposited into the same group are tracked separately.
#[test]
fn test_two_tokens_tracked_independently_in_same_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, token_a, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token_a, &creator, 13);

    // Register a second token
    let token_b = deploy_mock_token(
        &env,
        &String::from_str(&env, "Token B"),
        &String::from_str(&env, "TKNB"),
    );
    client.add_supported_token(&token_b, &admin);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token_a, &depositor, 1_000);
    mint_tokens(&env, &token_b, &depositor, 1_000);

    client.deposit_funds(&id, &token_a, &300, &depositor);
    client.deposit_funds(&id, &token_b, &700, &depositor);

    assert_eq!(client.get_group_treasury_balance(&id, &token_a), 300);
    assert_eq!(client.get_group_treasury_balance(&id, &token_b), 700);
}

// ─── Error conditions ────────────────────────────────────────────────────────

/// Amount of zero is rejected with InvalidAmount.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_deposit_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 14);

    let depositor = Address::generate(&env);
    client.deposit_funds(&id, &token, &0, &depositor);
}

/// Negative amount is rejected with InvalidAmount.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_deposit_negative_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 15);

    let depositor = Address::generate(&env);
    client.deposit_funds(&id, &token, &-1, &depositor);
}

/// Depositing into a non-existent group is rejected with NotFound.
#[test]
#[should_panic(expected = "NotFound")]
fn test_deposit_nonexistent_group_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);

    client.deposit_funds(&ghost_id, &token, &100, &depositor);
}

/// Depositing into an inactive group is rejected with GroupInactive.
#[test]
#[should_panic(expected = "GroupInactive")]
fn test_deposit_inactive_group_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 16);

    client.deactivate_payment_group(&id, &creator);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    client.deposit_funds(&id, &token, &100, &depositor);
}

/// Depositing with an unsupported token is rejected with UnsupportedToken.
#[test]
#[should_panic(expected = "UnsupportedToken")]
fn test_deposit_unsupported_token_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 17);

    // Deploy a token but do NOT add it to the supported list
    let bad_token = deploy_mock_token(
        &env,
        &String::from_str(&env, "Bad Token"),
        &String::from_str(&env, "BAD"),
    );

    let depositor = Address::generate(&env);
    mint_tokens(&env, &bad_token, &depositor, 500);
    client.deposit_funds(&id, &bad_token, &100, &depositor);
}

/// Depositing while the contract is paused is rejected with ContractPaused.
#[test]
#[should_panic(expected = "ContractPaused")]
fn test_deposit_while_paused_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 18);

    client.pause(&admin);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    client.deposit_funds(&id, &token, &100, &depositor);
}

// ─── State revert on error ───────────────────────────────────────────────────

/// A failed deposit must not mutate the treasury balance.
#[test]
fn test_failed_deposit_does_not_mutate_treasury() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 19);

    // Successful deposit first
    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    client.deposit_funds(&id, &token, &100, &depositor);
    assert_eq!(client.get_group_treasury_balance(&id, &token), 100);

    // Attempt zero-amount deposit — must fail
    let result = client.try_deposit_funds(&id, &token, &0, &depositor);
    assert!(result.is_err());

    // Treasury must be unchanged
    assert_eq!(client.get_group_treasury_balance(&id, &token), 100);
}

/// A failed deposit must not append to the group deposit history.
#[test]
fn test_failed_deposit_does_not_append_to_history() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 20);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    client.deposit_funds(&id, &token, &50, &depositor);

    // Attempt invalid deposit
    let _ = client.try_deposit_funds(&id, &token, &-10, &depositor);

    assert_eq!(client.get_group_deposit_history(&id).len(), 1);
}

// ─── Post-reactivation ───────────────────────────────────────────────────────

/// Deposit succeeds after a group is deactivated then reactivated.
#[test]
fn test_deposit_after_reactivation_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 21);

    client.deactivate_payment_group(&id, &creator);
    client.activate_group(&id, &creator);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    client.deposit_funds(&id, &token, &200, &depositor);
    assert_eq!(client.get_group_treasury_balance(&id, &token), 200);
}

/// Deposit succeeds after the contract is unpaused.
#[test]
fn test_deposit_after_unpause_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 22);

    client.pause(&admin);
    client.unpause(&admin);

    let depositor = Address::generate(&env);
    mint_tokens(&env, &token, &depositor, 500);
    client.deposit_funds(&id, &token, &150, &depositor);
    assert_eq!(client.get_group_treasury_balance(&id, &token), 150);
}
