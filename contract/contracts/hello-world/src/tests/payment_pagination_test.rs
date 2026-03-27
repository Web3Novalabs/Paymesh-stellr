#![cfg(test)]
use crate::mock_token::MockTokenClient;
use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

#[test]
fn test_get_user_pay_history_paginated() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let payer = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Mint tokens for the payer
    let token_client = MockTokenClient::new(&test_env.env, &token);
    token_client.mint(&payer, &10000);

    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    // Create a group
    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    // Make 25 topups for the same user
    for _ in 0..25 {
        client.topup_subscription(&id, &1, &token, &payer);
    }

    // Test first page
    let (page1, total) = client.get_user_pay_history_paginated(&payer, &0, &10);
    assert_eq!(page1.len(), 10);
    assert_eq!(total, 25);

    // Test second page
    let (page2, _) = client.get_user_pay_history_paginated(&payer, &10, &10);
    assert_eq!(page2.len(), 10);

    // Test third page (remaining 5)
    let (page3, _) = client.get_user_pay_history_paginated(&payer, &20, &10);
    assert_eq!(page3.len(), 5);

    // Test limit cap (should cap at 20)
    let (page_capped, _) = client.get_user_pay_history_paginated(&payer, &0, &50);
    assert_eq!(page_capped.len(), 20);

    // Test offset out of bounds
    let (page_empty, _) = client.get_user_pay_history_paginated(&payer, &30, &10);
    assert_eq!(page_empty.len(), 0);

    // Test zero limit
    let (page_zero_limit, _) = client.get_user_pay_history_paginated(&payer, &0, &0);
    assert_eq!(page_zero_limit.len(), 0);
}

#[test]
fn test_get_group_pay_history_paginated() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let payer = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Mint tokens for the payer
    let token_client = MockTokenClient::new(&test_env.env, &token);
    token_client.mint(&payer, &10000);

    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    // Create a group
    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    // Make 25 topups for the same group
    for _ in 0..25 {
        client.topup_subscription(&id, &1, &token, &payer);
    }

    // Test first page
    let (page1, total) = client.get_group_pay_history_paginated(&id, &0, &10);
    assert_eq!(page1.len(), 10);
    assert_eq!(total, 25 + 1); // +1 because create_test_group also records a payment

    // Test second page
    let (page2, _) = client.get_group_pay_history_paginated(&id, &10, &10);
    assert_eq!(page2.len(), 10);

    // Test third page
    let (page3, _) = client.get_group_pay_history_paginated(&id, &20, &10);
    assert_eq!(page3.len(), 6); // 26 total - 20 = 6 remaining

    // Test limit cap (should cap at 20)
    let (page_capped, _) = client.get_group_pay_history_paginated(&id, &0, &50);
    assert_eq!(page_capped.len(), 20);

    // Test offset out of bounds
    let (page_empty, _) = client.get_group_pay_history_paginated(&id, &30, &10);
    assert_eq!(page_empty.len(), 0);
}
