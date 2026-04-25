use crate::base::types::GroupMember;
use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use soroban_sdk::{testutils::Address as _, Address, Vec};

#[test]
fn test_get_group_members_paginated() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let token = test_env.mock_tokens.get(0).unwrap().clone();

    // Create a group with 25 members
    let mut members = Vec::new(&test_env.env);
    for _i in 0..25 {
        members.push_back(GroupMember {
            address: Address::generate(&test_env.env),
            percentage: 4, // 25 * 4 = 100
        });
    }

    let id = create_test_group(
        &test_env.env,
        &test_env.autoshare_contract,
        &creator,
        &members,
        1,
        &token,
    );

    // Test full list
    let all_members = client.get_group_members(&id);
    assert_eq!(all_members.len(), 25);

    // Test first page
    let page1 = client.get_group_members_paginated(&id, &0, &10);
    assert_eq!(page1.members.len(), 10);
    assert_eq!(page1.total, 25);
    assert_eq!(page1.offset, 0);
    assert_eq!(page1.limit, 10);

    // Test second page
    let page2 = client.get_group_members_paginated(&id, &10, &10);
    assert_eq!(page2.members.len(), 10);
    assert_eq!(page2.offset, 10);

    // Test third page (remaining 5)
    let page3 = client.get_group_members_paginated(&id, &20, &10);
    assert_eq!(page3.members.len(), 5);
    assert_eq!(page3.offset, 20);

    // Test limit cap (should cap at 20)
    let page_capped = client.get_group_members_paginated(&id, &0, &50);
    assert_eq!(page_capped.members.len(), 20);
    assert_eq!(page_capped.limit, 20);

    // Test offset out of bounds
    let page_empty = client.get_group_members_paginated(&id, &30, &10);
    assert_eq!(page_empty.members.len(), 0);
    assert_eq!(page_empty.total, 25);

    // Test zero limit
    let page_zero_limit = client.get_group_members_paginated(&id, &0, &0);
    assert_eq!(page_zero_limit.members.len(), 0);
    assert_eq!(page_zero_limit.total, 25);
    assert_eq!(page_zero_limit.limit, 0);
}

#[test]
#[should_panic]
fn test_get_group_members_paginated_not_found() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let id = soroban_sdk::BytesN::from_array(&test_env.env, &[0u8; 32]);
    client.get_group_members_paginated(&id, &0, &10);
}
