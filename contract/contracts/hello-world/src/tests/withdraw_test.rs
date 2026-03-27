#![allow(unused_imports)]

use crate::mock_token::MockTokenClient;
use crate::test_utils::{deploy_mock_token, mint_tokens, setup_test_env};
use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, BytesN, Env, FromVal, String, Symbol,
};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Mint `amount` tokens directly into the contract's own address so that
/// `get_contract_balance` / `withdraw` have something to work with.
fn fund_contract(env: &Env, token: &Address, contract: &Address, amount: i128) {
    mint_tokens(env, token, contract, amount);
}

fn balance_of(env: &Env, token: &Address, who: &Address) -> i128 {
    MockTokenClient::new(env, token).balance(who)
}

// ── 1. Admin can withdraw a partial balance ───────────────────────────────────

#[test]
fn test_admin_withdraw_partial_balance() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 1000);

    client.withdraw(&test_env.admin, &token, &400, &recipient);

    assert_eq!(balance_of(env, &token, &recipient), 400);
    assert_eq!(client.get_contract_balance(&token), 600);
}

// ── 2. Admin can withdraw the full balance ────────────────────────────────────

#[test]
fn test_admin_withdraw_full_balance() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 500);

    client.withdraw(&test_env.admin, &token, &500, &recipient);

    assert_eq!(balance_of(env, &token, &recipient), 500);
    assert_eq!(client.get_contract_balance(&token), 0);
}

// ── 3. Non-admin caller returns Unauthorized ──────────────────────────────────

#[test]
#[should_panic]
fn test_non_admin_withdraw_is_unauthorized() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let non_admin = Address::generate(env);
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 1000);

    // Should panic with Unauthorized
    client.withdraw(&non_admin, &token, &100, &recipient);
}

// ── 4. Withdraw when contract is paused ──────────────────────────────────────
//
// The withdraw function does NOT call `require_not_paused`, so it should
// succeed even while the contract is paused (admin-only escape hatch).

#[test]
fn test_withdraw_succeeds_when_paused() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 300);
    client.pause(&test_env.admin);

    // Admin withdraw should still work while paused
    client.withdraw(&test_env.admin, &token, &300, &recipient);

    assert_eq!(balance_of(env, &token, &recipient), 300);
}

// ── 5. Withdraw amount exceeding contract balance returns error ───────────────

#[test]
#[should_panic]
fn test_withdraw_exceeds_balance_panics() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 100);

    // 101 > 100 → InsufficientContractBalance
    client.withdraw(&test_env.admin, &token, &101, &recipient);
}

// ── 6. Withdraw amount of 0 returns InvalidAmount ────────────────────────────

#[test]
#[should_panic]
fn test_withdraw_zero_amount_panics() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 100);

    // amount = 0 → InvalidAmount
    client.withdraw(&test_env.admin, &token, &0, &recipient);
}

// ── 7. Tokens are transferred to the specified recipient ─────────────────────

#[test]
fn test_withdraw_transfers_to_correct_recipient() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient_a = Address::generate(env);
    let recipient_b = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 200);

    client.withdraw(&test_env.admin, &token, &150, &recipient_a);

    // Only recipient_a should have received tokens
    assert_eq!(balance_of(env, &token, &recipient_a), 150);
    assert_eq!(balance_of(env, &token, &recipient_b), 0);
}

// ── 8. Withdraw emits the Withdrawal event with correct fields ────────────────

#[test]
fn test_withdraw_emits_withdrawal_event() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 500);

    client.withdraw(&test_env.admin, &token, &250, &recipient);

    let events = env.events().all();
    let withdrawal_event = events
        .iter()
        .find(|e| {
            e.1.get(0)
                .map(|v| Symbol::from_val(env, &v) == Symbol::new(env, "withdrawal"))
                .unwrap_or(false)
        })
        .expect("withdrawal event not found");

    // topics[1] = token, topics[2] = recipient
    assert_eq!(
        Address::from_val(env, &withdrawal_event.1.get(1).unwrap()),
        token
    );
    assert_eq!(
        Address::from_val(env, &withdrawal_event.1.get(2).unwrap()),
        recipient
    );

    // data = amount (single-value format)
    let amount = i128::from_val(env, &withdrawal_event.2);
    assert_eq!(amount, 250);
}

// ── 9. Withdrawing one token does not affect another token's balance ──────────

#[test]
fn test_withdraw_one_token_does_not_affect_another() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token_a = test_env.mock_tokens.get(0).unwrap();

    // Deploy a second token
    let token_b = deploy_mock_token(
        env,
        &String::from_str(env, "Token B"),
        &String::from_str(env, "TKNB"),
    );
    client.add_supported_token(&token_b, &test_env.admin);

    let recipient = Address::generate(env);

    fund_contract(env, &token_a, &test_env.autoshare_contract, 1000);
    fund_contract(env, &token_b, &test_env.autoshare_contract, 800);

    // Withdraw only from token_a
    client.withdraw(&test_env.admin, &token_a, &600, &recipient);

    assert_eq!(balance_of(env, &token_a, &recipient), 600);
    // token_b balance on contract must be untouched
    assert_eq!(client.get_contract_balance(&token_b), 800);
}

// ── 10. Multiple sequential withdrawals work correctly ───────────────────────

#[test]
fn test_multiple_sequential_withdrawals() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let token = test_env.mock_tokens.get(0).unwrap();
    let recipient = Address::generate(env);

    fund_contract(env, &token, &test_env.autoshare_contract, 900);

    client.withdraw(&test_env.admin, &token, &300, &recipient);
    assert_eq!(client.get_contract_balance(&token), 600);
    assert_eq!(balance_of(env, &token, &recipient), 300);

    client.withdraw(&test_env.admin, &token, &200, &recipient);
    assert_eq!(client.get_contract_balance(&token), 400);
    assert_eq!(balance_of(env, &token, &recipient), 500);

    client.withdraw(&test_env.admin, &token, &400, &recipient);
    assert_eq!(client.get_contract_balance(&token), 0);
    assert_eq!(balance_of(env, &token, &recipient), 900);
}
