#![allow(unused_imports)]

use crate::test_utils::{
    create_test_group, create_test_members, deploy_autoshare_contract, deploy_mock_token,
    mint_tokens, setup_test_env,
};
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
    client.create(&id, &String::from_str(env, "Initial Name"), creator, &1, token);
    id
}

// ─── Name boundary ──────────────────────────────────────────────────────────

/// Exactly 60 characters — the maximum valid name length.
#[test]
fn test_name_at_exact_max_length_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 1);

    let name_60 = String::from_str(&env, &"X".repeat(60));
    client.update_payment_group(&id, &creator, &Some(name_60.clone()), &None, &None);
    assert_eq!(client.get(&id).name, name_60);
}

/// 61 characters — one over the limit — must be rejected.
#[test]
#[should_panic(expected = "EmptyName")]
fn test_name_one_over_max_length_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 2);

    let name_61 = String::from_str(&env, &"X".repeat(61));
    client.update_payment_group(&id, &creator, &Some(name_61), &None, &None);
}

/// Extremely long name (1 KB) must be rejected.
#[test]
#[should_panic(expected = "EmptyName")]
fn test_name_extreme_overflow_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 3);

    let huge_name = String::from_str(&env, &"A".repeat(1024));
    client.update_payment_group(&id, &creator, &Some(huge_name), &None, &None);
}

/// Single character name — minimum valid non-whitespace name.
#[test]
fn test_name_single_char_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 4);

    let name = String::from_str(&env, "Z");
    client.update_payment_group(&id, &creator, &Some(name.clone()), &None, &None);
    assert_eq!(client.get(&id).name, name);
}

/// Empty string name must be rejected.
#[test]
#[should_panic(expected = "EmptyName")]
fn test_name_empty_string_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 5);

    client.update_payment_group(&id, &creator, &Some(String::from_str(&env, "")), &None, &None);
}

/// Whitespace-only name (spaces) must be rejected.
#[test]
#[should_panic(expected = "EmptyName")]
fn test_name_spaces_only_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 6);

    client.update_payment_group(&id, &creator, &Some(String::from_str(&env, "     ")), &None, &None);
}

/// Tab-only name must be rejected.
#[test]
#[should_panic(expected = "EmptyName")]
fn test_name_tabs_only_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 7);

    client.update_payment_group(&id, &creator, &Some(String::from_str(&env, "\t\t\t")), &None, &None);
}

/// Name with leading/trailing whitespace but a valid inner character is accepted.
#[test]
fn test_name_with_surrounding_whitespace_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 8);

    let name = String::from_str(&env, "  valid  ");
    client.update_payment_group(&id, &creator, &Some(name.clone()), &None, &None);
    assert_eq!(client.get(&id).name, name);
}

// ─── State revert on invalid input ──────────────────────────────────────────

/// When name update fails, the original name must be preserved (no partial write).
#[test]
fn test_failed_name_update_does_not_mutate_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 9);

    let original_name = client.get(&id).name.clone();

    // Attempt invalid update — should panic
    let result = client.try_update_payment_group(
        &id,
        &creator,
        &Some(String::from_str(&env, "")), // invalid
        &None,
        &None,
    );
    assert!(result.is_err());

    // State must be unchanged
    assert_eq!(client.get(&id).name, original_name);
}

/// Unauthorized caller must not mutate any field.
#[test]
fn test_unauthorized_update_does_not_mutate_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let attacker = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 10);

    let before = client.get(&id);

    let result = client.try_update_payment_group(
        &id,
        &attacker,
        &Some(String::from_str(&env, "Hijacked")),
        &Some(String::from_str(&env, "evil metadata")),
        &Some(attacker.clone()),
    );
    assert!(result.is_err());

    let after = client.get(&id);
    assert_eq!(before.name, after.name);
    assert_eq!(before.metadata, after.metadata);
    assert_eq!(before.creator, after.creator);
}

/// Update on a non-existent group ID must not create a new group.
#[test]
fn test_update_nonexistent_group_does_not_create_entry() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _, client) = setup(&env);
    let caller = Address::generate(&env);
    let ghost_id = BytesN::from_array(&env, &[0xFFu8; 32]);

    let result = client.try_update_payment_group(
        &ghost_id,
        &caller,
        &Some(String::from_str(&env, "Ghost")),
        &None,
        &None,
    );
    assert!(result.is_err());

    // Confirm the group was not created
    let fetch = client.try_get(&ghost_id);
    assert!(fetch.is_err());
}

// ─── Admin rotation edge cases ───────────────────────────────────────────────

/// Rotating ownership to self is a no-op — creator stays the same.
#[test]
fn test_admin_rotation_to_self_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 11);

    client.update_payment_group(&id, &creator, &None, &None, &Some(creator.clone()));
    assert_eq!(client.get(&id).creator, creator);
}

/// After rotation, the old creator must be fully locked out.
#[test]
fn test_old_creator_locked_out_after_rotation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 12);

    client.update_payment_group(&id, &creator, &None, &None, &Some(new_owner.clone()));

    // Old creator can no longer update
    let result = client.try_update_payment_group(
        &id,
        &creator,
        &Some(String::from_str(&env, "Reclaim")),
        &None,
        &None,
    );
    assert!(result.is_err());
}

/// New creator can immediately update after rotation.
#[test]
fn test_new_creator_can_update_immediately_after_rotation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 13);

    client.update_payment_group(&id, &creator, &None, &None, &Some(new_owner.clone()));

    let updated_name = String::from_str(&env, "New Owner Update");
    client.update_payment_group(&id, &new_owner, &Some(updated_name.clone()), &None, &None);
    assert_eq!(client.get(&id).name, updated_name);
}

/// Chained rotation: A → B → C, only C should be the final owner.
#[test]
fn test_chained_admin_rotation() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let id = make_group(&env, &client, &token, &a, 14);

    client.update_payment_group(&id, &a, &None, &None, &Some(b.clone()));
    assert_eq!(client.get(&id).creator, b);

    client.update_payment_group(&id, &b, &None, &None, &Some(c.clone()));
    assert_eq!(client.get(&id).creator, c);

    // A and B are both locked out
    assert!(client.try_update_payment_group(&id, &a, &None, &None, &None).is_err());
    assert!(client.try_update_payment_group(&id, &b, &None, &None, &None).is_err());
}

// ─── Simultaneous field updates ──────────────────────────────────────────────

/// All three fields updated in one call — all must be persisted atomically.
#[test]
fn test_all_fields_updated_atomically() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 15);

    let new_name = String::from_str(&env, "Atomic Name");
    let new_meta = String::from_str(&env, "Atomic Meta");

    client.update_payment_group(
        &id,
        &creator,
        &Some(new_name.clone()),
        &Some(new_meta.clone()),
        &Some(new_owner.clone()),
    );

    let details = client.get(&id);
    assert_eq!(details.name, new_name);
    assert_eq!(details.metadata, new_meta);
    assert_eq!(details.creator, new_owner);
}

/// Passing all None fields must leave every field unchanged.
#[test]
fn test_all_none_fields_is_true_noop() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 16);

    // Set known state first
    let known_name = String::from_str(&env, "Known Name");
    let known_meta = String::from_str(&env, "Known Meta");
    client.update_payment_group(&id, &creator, &Some(known_name.clone()), &Some(known_meta.clone()), &None);

    // Now call with all None
    client.update_payment_group(&id, &creator, &None, &None, &None);

    let details = client.get(&id);
    assert_eq!(details.name, known_name);
    assert_eq!(details.metadata, known_meta);
    assert_eq!(details.creator, creator);
}

// ─── Rapid successive updates ────────────────────────────────────────────────

/// Ten sequential name updates — only the last value must be stored.
#[test]
fn test_rapid_successive_name_updates_last_wins() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 17);

    let names = [
        "Name 0", "Name 1", "Name 2", "Name 3", "Name 4",
        "Name 5", "Name 6", "Name 7", "Name 8", "Name 9",
    ];
    for name_str in names.iter() {
        let name = String::from_str(&env, name_str);
        client.update_payment_group(&id, &creator, &Some(name), &None, &None);
    }

    assert_eq!(client.get(&id).name, String::from_str(&env, "Name 9"));
}

// ─── Metadata edge cases ─────────────────────────────────────────────────────

/// Empty metadata string is accepted (no validation constraint on metadata).
#[test]
fn test_metadata_empty_string_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 18);

    let empty_meta = String::from_str(&env, "");
    client.update_payment_group(&id, &creator, &None, &Some(empty_meta.clone()), &None);
    assert_eq!(client.get(&id).metadata, empty_meta);
}

/// Very large metadata (4 KB) is accepted — no size cap on metadata.
#[test]
fn test_metadata_4kb_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 19);

    let large_meta = String::from_str(&env, &"M".repeat(4096));
    client.update_payment_group(&id, &creator, &None, &Some(large_meta.clone()), &None);
    assert_eq!(client.get(&id).metadata, large_meta);
}

/// Metadata with special characters (JSON-like) is stored verbatim.
#[test]
fn test_metadata_special_characters_stored_verbatim() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 20);

    let special = String::from_str(&env, r#"{"key":"value","n":42,"arr":[1,2,3]}"#);
    client.update_payment_group(&id, &creator, &None, &Some(special.clone()), &None);
    assert_eq!(client.get(&id).metadata, special);
}

/// Overwriting metadata with a shorter string does not leave stale data.
#[test]
fn test_metadata_overwrite_shorter_value_no_stale_data() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 21);

    let long_meta = String::from_str(&env, &"L".repeat(500));
    client.update_payment_group(&id, &creator, &None, &Some(long_meta), &None);

    let short_meta = String::from_str(&env, "short");
    client.update_payment_group(&id, &creator, &None, &Some(short_meta.clone()), &None);

    assert_eq!(client.get(&id).metadata, short_meta);
}

// ─── Paused / inactive state reverts ────────────────────────────────────────

/// Update while paused must not persist any field change.
#[test]
fn test_update_while_paused_does_not_mutate_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 22);

    let before = client.get(&id);
    client.pause(&admin);

    let result = client.try_update_payment_group(
        &id,
        &creator,
        &Some(String::from_str(&env, "Paused Update")),
        &None,
        &None,
    );
    assert!(result.is_err());

    client.unpause(&admin);
    let after = client.get(&id);
    assert_eq!(before.name, after.name);
}

/// Update on an inactive group must not persist any field change.
#[test]
fn test_update_inactive_group_does_not_mutate_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 23);

    let before = client.get(&id);
    client.deactivate_payment_group(&id, &creator);

    let result = client.try_update_payment_group(
        &id,
        &creator,
        &Some(String::from_str(&env, "Inactive Update")),
        &None,
        &None,
    );
    assert!(result.is_err());

    // Reactivate to read state
    client.activate_group(&id, &creator);
    let after = client.get(&id);
    assert_eq!(before.name, after.name);
}

// ─── Cross-group isolation ───────────────────────────────────────────────────

/// Updating one group must not affect a sibling group's fields.
#[test]
fn test_update_does_not_bleed_into_sibling_group() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);

    let id_a = make_group(&env, &client, &token, &creator, 24);
    let id_b = make_group(&env, &client, &token, &creator, 25);

    let before_b = client.get(&id_b);

    client.update_payment_group(
        &id_a,
        &creator,
        &Some(String::from_str(&env, "Group A Updated")),
        &Some(String::from_str(&env, "Meta A")),
        &None,
    );

    let after_b = client.get(&id_b);
    assert_eq!(before_b.name, after_b.name);
    assert_eq!(before_b.metadata, after_b.metadata);
    assert_eq!(before_b.creator, after_b.creator);
}

// ─── Reactivation after deactivation ────────────────────────────────────────

/// Group can be updated after being deactivated then reactivated.
#[test]
fn test_update_after_reactivation_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 26);

    client.deactivate_payment_group(&id, &creator);
    client.activate_group(&id, &creator);

    let new_name = String::from_str(&env, "Post-Reactivation");
    client.update_payment_group(&id, &creator, &Some(new_name.clone()), &None, &None);
    assert_eq!(client.get(&id).name, new_name);
}

// ─── Unpause then update ─────────────────────────────────────────────────────

/// Update must succeed after the contract is unpaused.
#[test]
fn test_update_after_unpause_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, token, client) = setup(&env);
    let creator = Address::generate(&env);
    let id = make_group(&env, &client, &token, &creator, 27);

    client.pause(&admin);
    client.unpause(&admin);

    let new_name = String::from_str(&env, "Post-Unpause");
    client.update_payment_group(&id, &creator, &Some(new_name.clone()), &None, &None);
    assert_eq!(client.get(&id).name, new_name);
}
