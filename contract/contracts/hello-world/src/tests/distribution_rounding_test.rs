use super::test_utils::{create_test_group, mint_tokens, setup_test_env};
use crate::base::types::GroupMember;
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

#[test]
fn test_three_members_33_33_34_split_amount_100() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 33,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 33,
    });
    members.push_back(GroupMember {
        address: member3.clone(),
        percentage: 34,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 100);
    client.distribute(&id, &token, &100, &sender);

    // Verify earnings: 33/100 * 100 = 33, remainder for last is 100 - (33+33) = 34
    assert_eq!(client.get_member_earnings(&member1, &id), 33);
    assert_eq!(client.get_member_earnings(&member2, &id), 33);
    assert_eq!(client.get_member_earnings(&member3, &id), 34);
}

#[test]
fn test_three_members_33_33_34_split_amount_1() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 33,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 33,
    });
    members.push_back(GroupMember {
        address: member3.clone(),
        percentage: 34,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 1);
    client.distribute(&id, &token, &1, &sender);

    // Verify earnings: 33/100 * 1 = 0, 33/100 * 1 = 0, remainder for last is 1 - (0+0) = 1
    assert_eq!(client.get_member_earnings(&member1, &id), 0);
    assert_eq!(client.get_member_earnings(&member2, &id), 0);
    assert_eq!(client.get_member_earnings(&member3, &id), 1);
}

#[test]
fn test_five_members_20_percent_each_amount_7() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let mut members = Vec::new(&env);
    let mut member_addrs = Vec::new(&env);
    for _ in 0..5 {
        let addr = Address::generate(&env);
        member_addrs.push_back(addr.clone());
        members.push_back(GroupMember {
            address: addr,
            percentage: 20,
        });
    }

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 7);
    client.distribute(&id, &token, &7, &sender);

    // 20/100 * 7 = 1.4 -> 1.
    // 4 members get 1 each = 4.
    // Last member gets 7 - 4 = 3.
    for i in 0..4 {
        let addr = member_addrs.get(i).unwrap();
        assert_eq!(client.get_member_earnings(&addr, &id), 1);
    }
    let last_addr = member_addrs.get(4).unwrap();
    assert_eq!(client.get_member_earnings(&last_addr, &id), 3);
}

#[test]
fn test_two_members_1_99_split_amount_1() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 1,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 99,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 1);
    client.distribute(&id, &token, &1, &sender);

    // 1/100 * 1 = 0.
    // Last member gets 1 - 0 = 1.
    assert_eq!(client.get_member_earnings(&member1, &id), 0);
    assert_eq!(client.get_member_earnings(&member2, &id), 1);
}

#[test]
fn test_large_amounts_no_overflow() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member1.clone(),
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 50,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    let large_amount = i128::MAX / 2;
    mint_tokens(&env, &token, &sender, large_amount);
    client.distribute(&id, &token, &large_amount, &sender);

    assert_eq!(client.get_member_earnings(&member1, &id), large_amount / 2);
    // Last member gets the rest
    assert_eq!(
        client.get_member_earnings(&member2, &id),
        large_amount - (large_amount / 2)
    );
}

#[test]
#[should_panic]
fn test_amount_zero_rejected() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let member = Address::generate(&env);
    let mut members = Vec::new(&env);
    members.push_back(GroupMember {
        address: member,
        percentage: 100,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    client.distribute(&id, &token, &0, &sender);
}

#[test]
fn test_50_members_2_percent_each_small_amount() {
    let test_env = setup_test_env();
    let env = test_env.env;
    let contract = test_env.autoshare_contract;
    let token = test_env.mock_tokens.get(0).unwrap().clone();
    let client = AutoShareContractClient::new(&env, &contract);

    let mut members = Vec::new(&env);
    let mut member_addrs = Vec::new(&env);
    for _ in 0..50 {
        let addr = Address::generate(&env);
        member_addrs.push_back(addr.clone());
        members.push_back(GroupMember {
            address: addr,
            percentage: 2,
        });
    }

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 10u32, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    let amount = 10;
    mint_tokens(&env, &token, &sender, amount);
    client.distribute(&id, &token, &amount, &sender);

    // 2/100 * 10 = 0.2 -> 0.
    // 49 members get 0 each = 0.
    // Last member gets 10 - 0 = 10.
    for i in 0..49 {
        let addr = member_addrs.get(i).unwrap();
        assert_eq!(client.get_member_earnings(&addr, &id), 0);
    }
    let last_addr = member_addrs.get(49).unwrap();
    assert_eq!(client.get_member_earnings(&last_addr, &id), 10);
}
