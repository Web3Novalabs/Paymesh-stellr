//! Unit tests for `remove_member_from_group` (member removal from a payment group).

use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, fund_user_with_tokens, mint_tokens, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, BytesN, String, Vec};

#[test]
fn test_remove_member_from_group_success_updates_member_list() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 55,
    });
    members.push_back(GroupMember {
        address: m2.clone(),
        percentage: 45,
    });

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );
    assert_eq!(client.get_group_members(&id).len(), 2);

    client.remove_member_from_group(&id, &creator, &m2);

    let after = client.get_group_members(&id);
    assert_eq!(after.len(), 1);
    assert_eq!(after.get(0).unwrap().address, m1);
    assert!(!client.is_group_member(&id, &m2));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_remove_member_from_group_non_creator_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: m2.clone(),
        percentage: 50,
    });

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
    let attacker = Address::generate(env);
    client.remove_member_from_group(&id, &attacker, &m2);
}

#[test]
#[should_panic(expected = "MemberNotFound")]
fn test_remove_member_from_group_unknown_member_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 100,
    });

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
    let ghost = Address::generate(env);
    client.remove_member_from_group(&id, &creator, &ghost);
}

#[test]
#[should_panic(expected = "GroupInactive")]
fn test_remove_member_from_group_inactive_group_fails() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: m2.clone(),
        percentage: 50,
    });

    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
    client.deactivate_group(&id, &creator);
    client.remove_member_from_group(&id, &creator, &m2);
}

#[test]
fn test_remove_member_from_group_after_distribution_succeeds() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember {
        address: m1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: m2.clone(),
        percentage: 50,
    });

    let usages = 3u32;
    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        usages,
        &token,
    );

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(env, &token, &sender, 2_000);
    client.distribute(&id, &token, &1_000, &sender);

    assert_eq!(client.get_group_distributions(&id).len(), 1);

    client.remove_member_from_group(&id, &creator, &m2);

    let remaining = client.get_group_members(&id);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining.get(0).unwrap().address, m1);
    assert_eq!(client.get_remaining_usages(&id), usages - 1);
}

#[test]
fn test_remove_member_from_group_updates_member_groups_index() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator_a = test_env.users.get(0).unwrap().clone();
    let creator_b = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let shared = Address::generate(env);

    let mut members_a = Vec::new(env);
    members_a.push_back(GroupMember {
        address: shared.clone(),
        percentage: 100,
    });
    let id_a = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator_a,
        &members_a,
        1,
        &token,
    );

    let mut members_b = Vec::new(env);
    members_b.push_back(GroupMember {
        address: shared.clone(),
        percentage: 100,
    });
    let id_b = BytesN::from_array(env, &[220u8; 32]);
    fund_user_with_tokens(env, &token, &creator_b, 10_000);
    client.create_payment_group(
        &id_b,
        &String::from_str(env, "Group B"),
        &creator_b,
        &1,
        &token,
    );
    client.update_members(&id_b, &creator_b, &members_b);

    assert_eq!(client.get_groups_by_member(&shared).len(), 2);

    client.remove_member_from_group(&id_b, &creator_b, &shared);

    let groups_for_shared = client.get_groups_by_member(&shared);
    assert_eq!(groups_for_shared.len(), 1);
    assert_eq!(groups_for_shared.get(0).unwrap().id, id_a);
}
