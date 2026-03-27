use crate::test_utils::{mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, String};

#[test]
fn test_fundraising_distribution_interaction() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let member1 = test_env.users.get(1).unwrap();
    let contributor = test_env.users.get(2).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    // Fund accounts
    mint_tokens(env, &token, &creator, 1000000);
    mint_tokens(env, &token, &contributor, 1000000);

    // Create group with 10 usages paid
    let group_id = BytesN::from_array(env, &[1u8; 32]);
    let name = String::from_str(env, "Interaction Group");
    client.create(&group_id, &name, &creator, &10, &token);

    // Add member
    client.add_group_member(&group_id, &creator, &member1, &100);

    // Start fundraising
    client.start_fundraising(&group_id, &creator, &5000);

    // (1) Distribute works while fundraising is active
    client.distribute(&group_id, &token, &1000, &creator);
    assert_eq!(client.get_remaining_usages(&group_id), 9);

    // (2) Contribute works while group has remaining usages
    client.contribute(&group_id, &token, &2000, &contributor);
    assert_eq!(client.get_remaining_usages(&group_id), 9); // Should NOT affect usage count

    // (3) Distribute does NOT decrement fundraising totals
    let status = client.get_fundraising_status(&group_id);
    assert_eq!(status.total_raised, 2000); // Only contribution counts towards total_raised

    // (4) Contribute correctly updates fundraising totals but does NOT affect usage count
    assert_eq!(client.get_remaining_usages(&group_id), 9);

    // (5) Earnings from both distribute and contribute accumulate correctly in MemberGroupEarnings
    let earnings = client.get_member_earnings(&member1, &group_id);
    assert_eq!(earnings, 3000); // 1000 from distribute + 2000 from contribute (100% share)

    // (6) get_member_distributions correctly records both distribution sources
    let member_distributions = client.get_member_distributions(&member1);
    // If it records both, it should have 2 entries.
    // NOTE: If this fails, contribute might not be calling record_distribution.
    assert_eq!(member_distributions.len(), 2);
}

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_contribute_blocked_on_inactive_group() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    mint_tokens(env, &token, &creator, 1000000);
    mint_tokens(env, &token, &contributor, 1000);

    let group_id = BytesN::from_array(env, &[2u8; 32]);
    let name = String::from_str(env, "Inactive Group");
    client.create(&group_id, &name, &creator, &10, &token);
    client.add_group_member(&group_id, &creator, &creator, &100);
    client.start_fundraising(&group_id, &creator, &5000);

    // (7) Deactivating a group while fundraising is active - verify contribute is blocked
    client.deactivate_group(&group_id, &creator);

    client.contribute(&group_id, &token, &1000, &contributor);
}

#[test]
fn test_fundraising_completion_doesnt_affect_distribute() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap();
    let contributor = test_env.users.get(1).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    mint_tokens(env, &token, &creator, 1000000);
    mint_tokens(env, &token, &contributor, 1000000);

    let group_id = BytesN::from_array(env, &[3u8; 32]);
    let name = String::from_str(env, "Completion Group");
    client.create(&group_id, &name, &creator, &10, &token);
    client.add_group_member(&group_id, &creator, &creator, &100);
    client.start_fundraising(&group_id, &creator, &1000);

    // (8) fundraising completion doesn't affect distribute functionality.
    // Complete fundraising
    client.contribute(&group_id, &token, &1000, &contributor);
    let status = client.get_fundraising_status(&group_id);
    assert!(!status.is_active);

    // verify distribute still works
    client.distribute(&group_id, &token, &500, &creator);
    assert_eq!(client.get_remaining_usages(&group_id), 9);
}
