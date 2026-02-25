use crate::test_utils::{create_test_group, setup_test_env};
use crate::AutoShareContractClient;
use quickcheck::TestResult;
use quickcheck_macros::quickcheck;
use soroban_sdk::{testutils::Address as _, Address, Vec};

// Property-based tests for get_group_count function
// Feature: get-group-count-function

#[cfg(test)]
mod property_tests {
    use super::*;

    // Feature: get-group-count-function, Property 1: Count equals actual group count
    // Validates: Requirements 1.3, 4.3, 5.2
    #[quickcheck]
    fn prop_count_equals_storage_length(group_count: u8) -> TestResult {
        // Limit to reasonable test size to avoid timeout
        if group_count > 100 {
            return TestResult::discard();
        }

        let test_env = setup_test_env();
        let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

        let creator = test_env.users.get(0).unwrap().clone();
        let token = test_env.mock_tokens.get(0).unwrap().clone();

        let mut members = Vec::new(&test_env.env);
        members.push_back(crate::base::types::GroupMember {
            address: Address::generate(&test_env.env),
            percentage: 100,
        });

        // Create the specified number of groups
        for i in 0..group_count {
            create_test_group(
                &test_env.env,
                &test_env.autoshare_contract,
                &creator,
                &members,
                (i as u32) + 1, // unique usage count for unique ID
                &token,
            );
        }

        // Verify count matches
        let actual_count = client.get_group_count();
        TestResult::from_bool(actual_count == group_count as u32)
    }

    // Feature: get-group-count-function, Property 2: Consistency with get_groups_paginated
    // Validates: Requirements 4.1, 5.3, 5.5
    #[quickcheck]
    fn prop_consistent_with_paginated(group_count: u8, page_size: u8) -> TestResult {
        // Limit to reasonable test size
        if group_count > 100 || page_size == 0 || page_size > 20 {
            return TestResult::discard();
        }

        let test_env = setup_test_env();
        let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

        let creator = test_env.users.get(0).unwrap().clone();
        let token = test_env.mock_tokens.get(0).unwrap().clone();

        let mut members = Vec::new(&test_env.env);
        members.push_back(crate::base::types::GroupMember {
            address: Address::generate(&test_env.env),
            percentage: 100,
        });

        // Create groups
        for i in 0..group_count {
            create_test_group(
                &test_env.env,
                &test_env.autoshare_contract,
                &creator,
                &members,
                (i as u32) + 1,
                &token,
            );
        }

        // Get count from both methods
        let count_direct = client.get_group_count();
        let page = client.get_groups_paginated(&0, &(page_size as u32));

        // Verify consistency
        TestResult::from_bool(count_direct == page.total)
    }

    // Feature: get-group-count-function, Property 3: Count updates after create/delete
    // Validates: Requirements 4.2, 5.4
    #[quickcheck]
    fn prop_count_updates_on_operations(initial_count: u8, creates: u8, deletes: u8) -> TestResult {
        // Limit to reasonable test size and ensure we don't delete more than we create
        if initial_count > 50 || creates > 20 || deletes > creates + initial_count {
            return TestResult::discard();
        }

        let test_env = setup_test_env();
        let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

        let creator = test_env.users.get(0).unwrap().clone();
        let token = test_env.mock_tokens.get(0).unwrap().clone();

        let mut members = Vec::new(&test_env.env);
        members.push_back(crate::base::types::GroupMember {
            address: Address::generate(&test_env.env),
            percentage: 100,
        });

        // Create initial groups
        let mut group_ids = Vec::new(&test_env.env);
        for i in 0..initial_count {
            let id = create_test_group(
                &test_env.env,
                &test_env.autoshare_contract,
                &creator,
                &members,
                (i as u32) + 1,
                &token,
            );
            group_ids.push_back(id);
        }

        let mut expected_count = initial_count as u32;
        let count_after_initial = client.get_group_count();
        if count_after_initial != expected_count {
            return TestResult::failed();
        }

        // Create additional groups
        for i in 0..creates {
            let id = create_test_group(
                &test_env.env,
                &test_env.autoshare_contract,
                &creator,
                &members,
                (initial_count as u32) + (i as u32) + 100, // unique usage count
                &token,
            );
            group_ids.push_back(id);
            expected_count += 1;

            let current_count = client.get_group_count();
            if current_count != expected_count {
                return TestResult::failed();
            }
        }

        // Delete groups (need to deactivate first)
        let delete_count = deletes.min(group_ids.len() as u8);
        for i in 0..delete_count {
            if let Some(id) = group_ids.get(i as u32) {
                // Deactivate the group first (required for deletion)
                client.deactivate_group(&id, &creator);

                // Delete the group (allowed even with remaining usages)
                client.delete_group(&id, &creator);
                expected_count -= 1;

                let current_count = client.get_group_count();
                if current_count != expected_count {
                    return TestResult::failed();
                }
            }
        }

        TestResult::passed()
    }
}
