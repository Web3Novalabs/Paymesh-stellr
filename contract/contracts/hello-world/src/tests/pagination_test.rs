use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

#[test]
fn test_get_groups_paginated() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    // Create 25 groups
    for i in 1..=25 {
        create_test_group(
            &test_env.env,
            &test_env.autoshare_contract,
            &creator,
            &members,
            i, // unique usages -> unique ID
            &token,
        );
    }

    // Test first page
    let page1 = client.get_groups_paginated(&0, &10);
    assert_eq!(page1.groups.len(), 10);
    assert_eq!(page1.total, 25);
    assert_eq!(page1.offset, 0);
    assert_eq!(page1.limit, 10);

    // Test second page
    let page2 = client.get_groups_paginated(&10, &10);
    assert_eq!(page2.groups.len(), 10);
    assert_eq!(page2.offset, 10);

    // Test third page (remaining 5)
    let page3 = client.get_groups_paginated(&20, &10);
    assert_eq!(page3.groups.len(), 5);
    assert_eq!(page3.offset, 20);

    // Test limit cap (should cap at 20)
    let page_capped = client.get_groups_paginated(&0, &50);
    assert_eq!(page_capped.groups.len(), 20);
    assert_eq!(page_capped.limit, 20);

    // Test offset out of bounds
    let page_empty = client.get_groups_paginated(&30, &10);
    assert_eq!(page_empty.groups.len(), 0);
    assert_eq!(page_empty.total, 25);

    // Test zero limit
    let page_zero_limit = client.get_groups_paginated(&0, &0);
    assert_eq!(page_zero_limit.groups.len(), 0);
    assert_eq!(page_zero_limit.total, 25);
    assert_eq!(page_zero_limit.limit, 0);
}

#[test]
fn test_get_groups_paginated_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let page = client.get_groups_paginated(&0, &10);
    assert_eq!(page.groups.len(), 0);
    assert_eq!(page.total, 0);
}

#[test]
fn test_get_group_count() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    assert_eq!(client.get_group_count(), 0);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );
    create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        2,
        &token,
    );
    create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        3,
        &token,
    );

    assert_eq!(client.get_group_count(), 3);
}

#[test]
fn test_get_groups_by_creator_paginated() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator1 = test_env.users.get(0).unwrap().clone();
    let creator2 = test_env.users.get(1).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    let mut members = Vec::new(&test_env.env);
    members.push_back(crate::base::types::GroupMember {
        address: Address::generate(&test_env.env),
        percentage: 100,
    });

    // Creator 1 creates 15 groups
    for i in 1..=15 {
        create_test_group(
            &test_env.env,
            &test_env.autoshare_contract,
            &creator1,
            &members,
            i,
            &token,
        );
    }

    // Creator 2 creates 10 groups
    for i in 16..=25 {
        create_test_group(
            &test_env.env,
            &test_env.autoshare_contract,
            &creator2,
            &members,
            i,
            &token,
        );
    }

    // Test Creator 1 - first page
    let c1_page1 = client.get_groups_by_creator_paginated(&creator1, &0, &10);
    assert_eq!(c1_page1.groups.len(), 10);
    assert_eq!(c1_page1.total, 15);
    assert_eq!(c1_page1.offset, 0);

    // Test Creator 1 - second page
    let c1_page2 = client.get_groups_by_creator_paginated(&creator1, &10, &10);
    assert_eq!(c1_page2.groups.len(), 5);
    assert_eq!(c1_page2.total, 15);
    assert_eq!(c1_page2.offset, 10);

    // Test Creator 2 - first page
    let c2_page1 = client.get_groups_by_creator_paginated(&creator2, &0, &5);
    assert_eq!(c2_page1.groups.len(), 5);
    assert_eq!(c2_page1.total, 10);

    // Test limit cap for Creator 1
    let c1_capped = client.get_groups_by_creator_paginated(&creator1, &0, &50);
    assert_eq!(c1_capped.groups.len(), 15); // only 15 exist
    assert_eq!(c1_capped.limit, 20);

    // Test Creator 3 (none)
    let creator3 = Address::generate(&test_env.env);
    let c3_page = client.get_groups_by_creator_paginated(&creator3, &0, &10);
    assert_eq!(c3_page.groups.len(), 0);
    assert_eq!(c3_page.total, 0);
}
