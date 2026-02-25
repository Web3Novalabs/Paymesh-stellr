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
