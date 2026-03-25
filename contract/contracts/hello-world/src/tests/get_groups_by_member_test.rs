use crate::{base::types::GroupMember, test_utils::setup_test_env, AutoShareContractClient};
use soroban_sdk::{BytesN, String, Vec};

#[test]
fn test_get_groups_by_member() {
    let test_env = setup_test_env();
    let env = &test_env.env;

    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    // Get characters for testing
    let creator1 = test_env.users.get(0).unwrap();
    let creator2 = test_env.users.get(1).unwrap();
    let member1 = test_env.users.get(2).unwrap();
    let token_id = test_env.mock_tokens.get(0).unwrap();

    let id1 = BytesN::from_array(env, &[1; 32]);
    let name1 = String::from_str(env, "Group 1");
    let usage_count1 = 10u32;

    // create_test_group automatically funds the creator and creates the group, but wait
    // create_test_group uses hardcoded id based on usage count, so let's just fund and create directly
    crate::test_utils::fund_user_with_tokens(env, &token_id, &creator1, 10000);
    crate::test_utils::fund_user_with_tokens(env, &token_id, &creator2, 10000);

    let id2 = BytesN::from_array(env, &[2; 32]);
    let name2 = String::from_str(env, "Group 2");
    let usage_count2 = 10u32;

    client.create(&id1, &name1, &creator1, &usage_count1, &token_id);

    client.create(&id2, &name2, &creator2, &usage_count2, &token_id);

    // Initial check: member1 is not in any group
    let groups = client.get_groups_by_member(&member1);
    assert_eq!(groups.len(), 0);

    // Add member1 to group 1
    client.add_group_member(&id1, &creator1, &member1, &100);
    let groups = client.get_groups_by_member(&member1);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups.get(0).unwrap().id, id1);

    // Add member1 to group 2
    client.add_group_member(&id2, &creator2, &member1, &100);
    let groups = client.get_groups_by_member(&member1);
    assert_eq!(groups.len(), 2);

    // Use update_members to remove member1 from group1 and add someone else
    let admin = test_env.admin.clone(); // Just another user
    let mut new_members = Vec::new(env);
    new_members.push_back(GroupMember {
        address: admin.clone(),
        percentage: 100,
    });
    client.update_members(&id1, &creator1, &new_members);

    // admin should now see group 1
    let admin_groups = client.get_groups_by_member(&admin);
    assert_eq!(admin_groups.len(), 1);
    assert_eq!(admin_groups.get(0).unwrap().id, id1);

    // member1 should now only see group 2
    let m1_groups = client.get_groups_by_member(&member1);
    assert_eq!(m1_groups.len(), 1);
    assert_eq!(m1_groups.get(0).unwrap().id, id2);

    // Remove member1 from group 2 via remove_group_member
    client.remove_group_member(&id2, &creator2, &member1);
    let m1_groups_final = client.get_groups_by_member(&member1);
    assert_eq!(m1_groups_final.len(), 0);

    // Delete group 1 to see if admin still has it indexed
    client.deactivate_group(&id1, &creator1);
    client.delete_group(&id1, &creator1);

    // admin was in group 1, should no longer see it after the group is deleted
    let admin_groups_after_delete = client.get_groups_by_member(&admin);
    assert_eq!(admin_groups_after_delete.len(), 0);
}

#[test]
fn test_get_groups_by_member_paginated() {
    let test_env = setup_test_env();
    let env = &test_env.env;
    let client = AutoShareContractClient::new(env, &test_env.autoshare_contract);

    let creator = test_env.users.get(0).unwrap().clone();
    let member = test_env.users.get(1).unwrap().clone();
    let token_id = test_env.mock_tokens.get(0).unwrap().clone();

    crate::test_utils::fund_user_with_tokens(env, &token_id, &creator, 100000);

    // Create 5 groups and add member to all of them
    for i in 0u8..5 {
        let id = BytesN::from_array(env, &[i + 10; 32]);
        let name = String::from_str(env, "Group");
        client.create(&id, &name, &creator, &10u32, &token_id);
        client.add_group_member(&id, &creator, &member, &100);
    }

    // Page 1: offset 0, limit 2
    let page1 = client.get_groups_by_member_paginated(&member, &0, &2);
    assert_eq!(page1.groups.len(), 2);
    assert_eq!(page1.total, 5);
    assert_eq!(page1.offset, 0);
    assert_eq!(page1.limit, 2);

    // Page 2: offset 2, limit 2
    let page2 = client.get_groups_by_member_paginated(&member, &2, &2);
    assert_eq!(page2.groups.len(), 2);
    assert_eq!(page2.total, 5);

    // Page 3: offset 4, limit 2
    let page3 = client.get_groups_by_member_paginated(&member, &4, &2);
    assert_eq!(page3.groups.len(), 1);
    assert_eq!(page3.total, 5);

    // Limit exceeding 20
    let page_max = client.get_groups_by_member_paginated(&member, &0, &50);
    assert_eq!(page_max.limit, 20);
}
