//! Advanced boundary and edge case tests for `remove_member_from_group`.

use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, create_test_members, setup_test_env, mint_tokens};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::{Address as _, Events}, Address, BytesN, String, Vec, vec, IntoVal};

#[test]
fn test_remove_member_from_max_capacity_group() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Max capacity is 50
    let max_members = 50;
    let members = create_test_members(env, max_members);
    
    let id = create_test_group(
        env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        10,
        &token,
    );

    assert_eq!(client.get_group_members(&id).len(), max_members);

    // Remove a member from the middle (index 25)
    let member_to_remove = members.get(25).unwrap().address;
    client.remove_member_from_group(&id, &creator, &member_to_remove);

    let after = client.get_group_members(&id);
    assert_eq!(after.len(), max_members - 1);
    assert!(!client.is_group_member(&id, &member_to_remove));
    
    // Verify member groups index for the removed member
    let member_groups = client.get_groups_by_member(&member_to_remove);
    assert_eq!(member_groups.len(), 0);
}

#[test]
fn test_remove_last_member_leaves_empty_list() {
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

    client.remove_member_from_group(&id, &creator, &m1);

    let after = client.get_group_members(&id);
    assert_eq!(after.len(), 0);
    
    // Distribution should now fail with EmptyMembers (Error code 12)
    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(env, &token, &sender, 1000);
    
    let result = env.as_contract(&test_env.autoshare_contract, || {
        client.try_distribute(&id, &token, &100, &sender)
    });
    
    assert!(result.is_err());
}

#[test]
fn test_remove_member_with_large_pending_earnings_event() {
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
        5,
        &token,
    );

    let sender = test_env.users.get(1).unwrap().clone();
    let dist_amount = 1_000_000_000i128;
    mint_tokens(env, &token, &sender, dist_amount * 2);
    
    client.distribute(&id, &token, &dist_amount, &sender);
    
    // Member should have 1,000,000,000 pending/earned
    let earnings = client.get_member_earnings(&m1, &id);
    assert_eq!(earnings, dist_amount);

    client.remove_member_from_group(&id, &creator, &m1);

    // Verify event emission (simplified check for presence of MemberRemoved)
    let last_event = env.events().all().last().unwrap();
    // In soroban tests, event topics are usually [Symbol, ID, MemberAddress]
    // The MemberRemoved event topic pattern: [Symbol("MemberRemoved"), id, member_address]
    assert_eq!(last_event.2, dist_amount.into_val(env));
}

#[test]
fn test_remove_and_re_add_member() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let m2 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember { address: m1.clone(), percentage: 50 });
    members.push_back(GroupMember { address: m2.clone(), percentage: 50 });

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &members, 2, &token);

    client.remove_member_from_group(&id, &creator, &m1);
    assert!(!client.is_group_member(&id, &m1));

    // Re-adding m1 with 50%. Since only m2 (50%) remains, adding m1 (50%) will make it 100% total.
    client.add_member_to_group(&id, &creator, &m1, &50);
    assert!(client.is_group_member(&id, &m1));
    assert_eq!(client.get_group_members(&id).len(), 2);
}

#[test]
fn test_remove_member_from_many_groups() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let shared_member = Address::generate(env);
    let mut group_ids = Vec::new(env);

    for i in 0..10 {
        let mut members = Vec::new(env);
        members.push_back(GroupMember {
            address: shared_member.clone(),
            percentage: 100,
        });
        
        let mut id_bytes = [0u8; 32];
        id_bytes[31] = i as u8;
        let id = BytesN::from_array(env, &id_bytes);
        
        mint_tokens(env, &token, &creator, 1000);
        client.create(&id, &String::from_str(env, "Group"), &creator, &1, &token);
        client.update_members(&id, &creator, &members);
        group_ids.push_back(id);
    }

    assert_eq!(client.get_groups_by_member(&shared_member).len(), 10);

    // Remove from the 5th group
    let id_to_remove = group_ids.get(4).unwrap();
    client.remove_member_from_group(&id_to_remove, &creator, &shared_member);

    let remaining_groups = client.get_groups_by_member(&shared_member);
    assert_eq!(remaining_groups.len(), 9);
    
    for g in remaining_groups.iter() {
        assert!(g.id != id_to_remove);
    }
}

#[test]
fn test_remove_member_max_id_boundary() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let id = BytesN::from_array(env, &[0xFF; 32]);
    let m1 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember { address: m1.clone(), percentage: 100 });

    mint_tokens(env, &token, &creator, 1000);
    client.create(&id, &String::from_str(env, "Max ID Group"), &creator, &1, &token);
    client.update_members(&id, &creator, &members);

    client.remove_member_from_group(&id, &creator, &m1);
    assert!(!client.is_group_member(&id, &m1));
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_remove_member_paused_reverts() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);
    let admin = test_env.admin.clone();
    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let m1 = Address::generate(env);
    let mut members = Vec::new(env);
    members.push_back(GroupMember { address: m1.clone(), percentage: 100 });

    let id = create_test_group(env, &test_env.autoshare_contract, &creator, &members, 1, &token);

    client.pause(&admin);
    client.remove_member_from_group(&id, &creator, &m1);
}
