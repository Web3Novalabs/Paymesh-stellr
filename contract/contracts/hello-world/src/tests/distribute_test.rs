use super::test_utils::{assert_balance, create_test_group, mint_tokens, setup_test_env};
use crate::base::types::GroupMember;
use crate::mock_token::MockTokenClient;
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

#[test]
fn test_distribute_splits_payment_and_decrements_usage() {
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
        percentage: 50,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 30,
    });
    members.push_back(GroupMember {
        address: member3.clone(),
        percentage: 20,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let usages = 2u32;
    let id = create_test_group(&env, &contract, &creator, &members, usages, &token);

    let sender = test_env.users.get(1).unwrap().clone();
    let amount: i128 = 1000;

    mint_tokens(&env, &token, &sender, amount);

    let token_client = MockTokenClient::new(&env, &token);
    let sender_start = token_client.balance(&sender);

    client.distribute(&id, &token, &amount, &sender);

    // Verify member balances
    assert_balance(&env, &token, &member1, 500);
    assert_balance(&env, &token, &member2, 300);
    assert_balance(&env, &token, &member3, 200);

    // Verify sender paid the amount
    let sender_end = token_client.balance(&sender);
    assert_eq!(sender_start - amount, sender_end);

    // Verify remaining usages decremented
    let remaining = client.get_remaining_usages(&id);
    assert_eq!(remaining, usages - 1);
}

#[test]
#[should_panic]
fn test_distribute_fails_when_group_inactive() {
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
        percentage: 60,
    });
    members.push_back(GroupMember {
        address: member2.clone(),
        percentage: 40,
    });

    let creator = test_env.users.get(0).unwrap().clone();
    let id = create_test_group(&env, &contract, &creator, &members, 1u32, &token);

    client.deactivate_group(&id, &creator);

    let sender = test_env.users.get(1).unwrap().clone();
    mint_tokens(&env, &token, &sender, 500);
    client.distribute(&id, &token, &500, &sender);
}
