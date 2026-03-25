#![cfg(test)]

use crate::base::types::GroupMember;
use crate::test_utils::{deploy_autoshare_contract, deploy_mock_token, mint_tokens};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

// ─── helpers ────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (Address, Address, AutoShareContractClient) {
    let admin = Address::generate(env);
    let contract_id = deploy_autoshare_contract(env, &admin);
    let client = AutoShareContractClient::new(env, &contract_id);
    client.initialize_admin(&admin);
    (admin, contract_id, client)
}

fn new_token(env: &Env, symbol: &str) -> Address {
    deploy_mock_token(
        env,
        &String::from_str(env, symbol),
        &String::from_str(env, symbol),
    )
}

fn make_group_id(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

fn create_group(
    env: &Env,
    client: &AutoShareContractClient,
    token: &Address,
    creator: &Address,
    seed: u8,
) -> BytesN<32> {
    let id = make_group_id(env, seed);
    mint_tokens(env, token, creator, 10_000);
    client.create(&id, &String::from_str(env, "Test Group"), creator, &5, token);
    id
}

// ─── 1. Adding a token that is already supported returns AlreadyExists ───────

#[test]
#[should_panic(expected = "AlreadyExists")]
fn test_add_duplicate_token_errors() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");
    client.add_supported_token(&token, &admin);
    // Second add should panic with AlreadyExists
    client.add_supported_token(&token, &admin);
}

// ─── 2. Removing a token not in the list returns NotFound ────────────────────

#[test]
#[should_panic(expected = "NotFound")]
fn test_remove_nonexistent_token_errors() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");
    // Never added — should panic with NotFound
    client.remove_supported_token(&token, &admin);
}

// ─── 3. Removing the last token leaves an empty list ─────────────────────────

#[test]
fn test_remove_last_token_leaves_empty_list() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");
    client.add_supported_token(&token, &admin);
    assert_eq!(client.get_supported_tokens().len(), 1);

    client.remove_supported_token(&token, &admin);
    assert_eq!(client.get_supported_tokens().len(), 0);
}

// ─── 4. Creating a group after all tokens removed fails with UnsupportedToken ─

#[test]
#[should_panic(expected = "UnsupportedToken")]
fn test_create_group_with_no_supported_tokens_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");
    client.add_supported_token(&token, &admin);
    client.remove_supported_token(&token, &admin);

    let creator = Address::generate(&env);
    mint_tokens(&env, &token, &creator, 10_000);
    let id = make_group_id(&env, 1);
    // Should panic — token no longer supported
    client.create(&id, &String::from_str(&env, "Group"), &creator, &5, &token);
}

// ─── 5. is_token_supported reflects add/remove cycles correctly ───────────────

#[test]
fn test_is_token_supported_add_remove_cycle() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");

    assert!(!client.is_token_supported(&token));

    client.add_supported_token(&token, &admin);
    assert!(client.is_token_supported(&token));

    client.remove_supported_token(&token, &admin);
    assert!(!client.is_token_supported(&token));

    // Re-add after removal should work
    client.add_supported_token(&token, &admin);
    assert!(client.is_token_supported(&token));
}

// ─── 6. get_supported_tokens returns tokens in insertion order ────────────────

#[test]
fn test_get_supported_tokens_insertion_order() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let t1 = new_token(&env, "AAA");
    let t2 = new_token(&env, "BBB");
    let t3 = new_token(&env, "CCC");

    client.add_supported_token(&t1, &admin);
    client.add_supported_token(&t2, &admin);
    client.add_supported_token(&t3, &admin);

    let tokens = client.get_supported_tokens();
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens.get(0).unwrap(), t1);
    assert_eq!(tokens.get(1).unwrap(), t2);
    assert_eq!(tokens.get(2).unwrap(), t3);
}

// ─── 7. Non-admin cannot add or remove tokens ─────────────────────────────────

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_add_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);

    let non_admin = Address::generate(&env);
    let token = new_token(&env, "TKN");
    // non_admin is not the contract admin — should panic
    client.add_supported_token(&token, &non_admin);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_remove_token() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let non_admin = Address::generate(&env);
    let token = new_token(&env, "TKN");
    client.add_supported_token(&token, &admin);
    // non_admin is not the contract admin — should panic
    client.remove_supported_token(&token, &non_admin);
}

// ─── 8. add/remove when contract is paused ────────────────────────────────────
// The token management functions do not check the pause flag, so they should
// succeed even while the contract is paused.

#[test]
fn test_add_token_while_paused_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    client.pause(&admin);
    assert!(client.get_paused_status());

    let token = new_token(&env, "TKN");
    // Should not panic — token management is not gated by pause
    client.add_supported_token(&token, &admin);
    assert!(client.is_token_supported(&token));
}

#[test]
fn test_remove_token_while_paused_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");
    client.add_supported_token(&token, &admin);

    client.pause(&admin);
    assert!(client.get_paused_status());

    // Should not panic
    client.remove_supported_token(&token, &admin);
    assert!(!client.is_token_supported(&token));
}

// ─── 9. Removing a token does not affect existing groups using that token ──────

#[test]
fn test_remove_token_does_not_affect_existing_groups() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let token = new_token(&env, "TKN");
    client.add_supported_token(&token, &admin);

    let creator = Address::generate(&env);
    let group_id = create_group(&env, &client, &token, &creator, 1);

    // Verify group exists and is active
    let group = client.get(&group_id);
    assert!(group.is_active);
    assert_eq!(group.creator, creator);

    // Remove the token
    client.remove_supported_token(&token, &admin);
    assert!(!client.is_token_supported(&token));

    // Existing group should still be retrievable and intact
    let group_after = client.get(&group_id);
    assert!(group_after.is_active);
    assert_eq!(group_after.creator, creator);
    assert_eq!(group_after.usage_count, 5);
}

// ─── 10. Adding 20+ tokens and verifying all are queryable ────────────────────

#[test]
fn test_add_many_tokens_all_queryable() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _, client) = setup(&env);

    let mut added: Vec<Address> = Vec::new(&env);
    for i in 0u32..20 {
        // Use a unique symbol per token by embedding the index in the name
        let token = new_token(&env, "TK");
        // Disambiguate by minting a unique amount so addresses differ — addresses
        // are already unique per deploy_mock_token call, so just collect them.
        client.add_supported_token(&token, &admin);
        added.push_back(token);
    }

    let supported = client.get_supported_tokens();
    assert_eq!(supported.len(), 20);

    // Every added token must be individually supported
    for token in added.iter() {
        assert!(client.is_token_supported(&token));
    }
}
