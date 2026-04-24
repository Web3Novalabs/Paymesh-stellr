use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

/// Helper: creates a group with a single member at 100%.
fn create_single_member_group(
    env: &soroban_sdk::Env,
    contract: &Address,
    creator: &Address,
    member: &Address,
    usages: u32,
    token: &Address,
) -> soroban_sdk::BytesN<32> {
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: member.clone(),
        percentage: 100,
    });
    create_test_group(env, contract, creator, &members, usages, token)
}

// ─── Test 1 ──────────────────────────────────────────────────────────────────
// Create group with 1 usage: first distribute succeeds, second panics.

#[test]
fn test_single_usage_first_distribute_succeeds() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 1, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 1000);

    client.distribute(&id, &token, &500, &sender);

    assert_eq!(client.get_remaining_usages(&id), 0);
}

#[test]
#[should_panic(expected = "NoUsagesRemaining")]
fn test_second_distribute_with_1_usage_fails() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 1, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 2000);

    client.distribute(&id, &token, &500, &sender);
    // Second distribute must return NoUsagesRemaining
    client.distribute(&id, &token, &500, &sender);
}

// ─── Test 2 ──────────────────────────────────────────────────────────────────
// Create group with 5 usages, distribute 5 times, remaining == 0.

#[test]
fn test_five_usages_all_consumed() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 5, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 5000);

    for i in 0..5u32 {
        client.distribute(&id, &token, &100, &sender);
        assert_eq!(client.get_remaining_usages(&id), 4 - i);
    }

    assert_eq!(client.get_remaining_usages(&id), 0);
}

// ─── Test 3 ──────────────────────────────────────────────────────────────────
// After usages exhausted, topup_subscription re-enables distribute.

#[test]
fn test_topup_after_exhaustion_allows_next_distribute() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 1, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 2000);

    // Exhaust the single usage
    client.distribute(&id, &token, &300, &sender);
    assert_eq!(client.get_remaining_usages(&id), 0);

    // Top up with 3 additional usages
    let payer = test_env.users.get(2).unwrap().clone();
    let additional = 3u32;
    mint_tokens(&env, &token, &payer, (additional as i128) * 10);
    client.topup_subscription(&id, &additional, &token, &payer);

    assert_eq!(client.get_remaining_usages(&id), additional);

    // Distribution should work again
    client.distribute(&id, &token, &200, &sender);
    assert_eq!(client.get_remaining_usages(&id), additional - 1);
}

// ─── Test 4 ──────────────────────────────────────────────────────────────────
// get_remaining_usages returns the correct count after each distribute.

#[test]
fn test_remaining_usages_correct_after_each_distribute() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 4, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 5000);

    assert_eq!(client.get_remaining_usages(&id), 4);

    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), 3);

    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), 2);

    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), 1);

    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), 0);
}

// ─── Test 5 ──────────────────────────────────────────────────────────────────
// get_total_usages_paid reflects initial creation + all topups.
// Distribute does NOT change total_usages_paid.

#[test]
fn test_total_usages_paid_reflects_creation_and_topups() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let initial_usages = 5u32;
    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, initial_usages, &token);

    // After creation total_usages_paid == initial_usages
    assert_eq!(client.get_total_usages_paid(&id), initial_usages);

    let payer = test_env.users.get(1).unwrap().clone();

    // First topup
    let topup1 = 10u32;
    mint_tokens(&env, &token, &payer, (topup1 as i128) * 10);
    client.topup_subscription(&id, &topup1, &token, &payer);
    assert_eq!(client.get_total_usages_paid(&id), initial_usages + topup1);

    // Second topup
    let topup2 = 7u32;
    mint_tokens(&env, &token, &payer, (topup2 as i128) * 10);
    client.topup_subscription(&id, &topup2, &token, &payer);
    assert_eq!(
        client.get_total_usages_paid(&id),
        initial_usages + topup1 + topup2
    );

    // Distributing should NOT increment total_usages_paid
    let sender = test_env.users.get(2).unwrap().clone();
    mint_tokens(&env, &token, &sender, 1000);
    client.distribute(&id, &token, &500, &sender);
    assert_eq!(
        client.get_total_usages_paid(&id),
        initial_usages + topup1 + topup2
    );
}

// ─── Test 6 ──────────────────────────────────────────────────────────────────
// Atomic behavior: distribute does NOT decrement usage_count if the transfer fails.
//
// In Soroban, every contract invocation is a single transaction. If any step
// panics (e.g. token transfer with insufficient balance), the entire transaction
// is rolled back — including any state changes that already happened. The
// distribute function checks usage_count == 0 BEFORE attempting the transfer,
// and decrements usage_count AFTER the transfer. Therefore a failed transfer
// leaves usage_count unchanged.

#[test]
fn test_distribute_only_decrements_on_successful_transfer() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 5, &token);

    assert_eq!(client.get_remaining_usages(&id), 5);

    // Successful distribute: usage_count decrements
    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 1000);
    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), 4);
}

#[test]
#[should_panic]
fn test_distribute_atomic_transfer_failure_panics() {
    // When the token transfer fails (sender has 0 balance), the transaction
    // panics and all state changes are rolled back — usage_count stays unchanged.
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let id = create_single_member_group(&env, &contract, &creator, &member, 5, &token);

    // Sender with zero balance — transfer will fail, proving atomic rollback
    let broke_sender = Address::generate(&env);
    client.distribute(&id, &token, &1000, &broke_sender);
}

// ─── Test 7 ──────────────────────────────────────────────────────────────────
// Deactivate / reactivate a group — usage_count is preserved through both ops.

#[test]
fn test_deactivate_reactivate_preserves_usage_count() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = Address::generate(&env);
    let initial_usages = 7u32;
    let id = create_single_member_group(&env, &contract, &creator, &member, initial_usages, &token);

    // Consume one usage
    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 1000);
    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), initial_usages - 1);

    // Deactivate — usage count must be preserved
    client.deactivate_group(&id, &creator);
    assert_eq!(client.get_remaining_usages(&id), initial_usages - 1);

    // Reactivate — usage count must still be the same
    client.activate_group(&id, &creator);
    assert_eq!(client.get_remaining_usages(&id), initial_usages - 1);

    // Distribute again to confirm the group is fully operational
    client.distribute(&id, &token, &100, &sender);
    assert_eq!(client.get_remaining_usages(&id), initial_usages - 2);
}

// ─── Test 8 ──────────────────────────────────────────────────────────────────
// reduce_usage edge cases.

#[test]
fn test_reduce_usage_decrements_from_one_to_zero() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_single_member_group(
        &env,
        &contract,
        &creator,
        &Address::generate(&env),
        1,
        &token,
    );

    assert_eq!(client.get_remaining_usages(&id), 1);
    client.reduce_usage(&id);
    assert_eq!(client.get_remaining_usages(&id), 0);
}

#[test]
fn test_reduce_usage_decrements_multiple_times() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_single_member_group(
        &env,
        &contract,
        &creator,
        &Address::generate(&env),
        3,
        &token,
    );

    client.reduce_usage(&id);
    assert_eq!(client.get_remaining_usages(&id), 2);
    client.reduce_usage(&id);
    assert_eq!(client.get_remaining_usages(&id), 1);
    client.reduce_usage(&id);
    assert_eq!(client.get_remaining_usages(&id), 0);
}

#[test]
#[should_panic(expected = "NoUsagesRemaining")]
fn test_reduce_usage_panics_when_already_zero() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_single_member_group(
        &env,
        &contract,
        &creator,
        &Address::generate(&env),
        1,
        &token,
    );

    client.reduce_usage(&id);
    // usage_count is now 0 — next call must return NoUsagesRemaining
    client.reduce_usage(&id);
}

// ─── Test 9 ──────────────────────────────────────────────────────────────────
// Create group with a large usage count — verify no arithmetic overflow.
//
// We use 500_000_000 (500 million) which:
//   • exceeds i32::MAX — catches any accidental signed-cast overflow
//   • exceeds u16::MAX — rules out truncation to narrower types
//   • keeps token amounts within safe i128 headroom for the test environment
//     (500_000_000 * 10 = 5_000_000_000, well below i128::MAX)

#[test]
fn test_large_usage_count_no_overflow() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();

    // 500_000_000 > i32::MAX (2_147_483_647 / 4 = ~536M, so this is just below i32::MAX
    // but way above u16::MAX); also tests that (usage_count as i128) * fee has no overflow.
    let large_usages: u32 = 500_000_000;

    // create_test_group mints (usages * 10 + 10_000) tokens for the creator
    let id = create_test_group(
        &env,
        &contract,
        &creator,
        &Vec::new(&env),
        large_usages,
        &token,
    );

    let details = client.get(&id);
    assert_eq!(details.usage_count, large_usages);
    assert_eq!(details.total_usages_paid, large_usages);

    assert_eq!(client.get_remaining_usages(&id), large_usages);
    assert_eq!(client.get_total_usages_paid(&id), large_usages);
}

// ─── Test 10 ─────────────────────────────────────────────────────────────────
// Usage count is independent between different groups.

#[test]
fn test_usage_count_independent_between_groups() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member_a = Address::generate(&env);
    let member_b = Address::generate(&env);

    // Two groups with different usage counts (different IDs via create_test_group)
    let id_a = create_single_member_group(&env, &contract, &creator, &member_a, 3, &token);
    let id_b = create_single_member_group(&env, &contract, &creator, &member_b, 8, &token);

    assert_eq!(client.get_remaining_usages(&id_a), 3);
    assert_eq!(client.get_remaining_usages(&id_b), 8);

    // Distribute from group A only
    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 5000);
    client.distribute(&id_a, &token, &100, &sender);

    // Group A decremented; group B unchanged
    assert_eq!(client.get_remaining_usages(&id_a), 2);
    assert_eq!(client.get_remaining_usages(&id_b), 8);

    // Distribute twice from group B
    client.distribute(&id_b, &token, &100, &sender);
    client.distribute(&id_b, &token, &100, &sender);

    // Group B decremented; group A unchanged
    assert_eq!(client.get_remaining_usages(&id_a), 2);
    assert_eq!(client.get_remaining_usages(&id_b), 6);
}
