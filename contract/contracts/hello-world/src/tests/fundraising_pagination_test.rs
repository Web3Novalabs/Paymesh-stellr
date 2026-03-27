use crate::autoshare_logic::DataKey;
use crate::base::types::FundraisingContribution;
use crate::test_utils::setup_test_env;
use crate::AutoShareContractClient;
use soroban_sdk::{BytesN, Vec};

#[test]
fn test_get_contributions_paginated_empty() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let group_id = BytesN::from_array(&test_env.env, &[1u8; 32]);
    let user = test_env.users.get(0).unwrap();

    let (g_contributions, g_total) = client.get_group_contribs_paginated(&group_id, &0, &10);
    let (u_contributions, u_total) = client.get_user_contribs_paginated(&user, &0, &10);

    assert_eq!(g_contributions.len(), 0);
    assert_eq!(g_total, 0);
    assert_eq!(u_contributions.len(), 0);
    assert_eq!(u_total, 0);
}

#[test]
fn test_get_group_contributions_pagination() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let group_id = BytesN::from_array(&test_env.env, &[2u8; 32]);
    let contributor = test_env.users.get(0).unwrap();
    let token = test_env.mock_tokens.get(0).unwrap();

    // Create 25 contributions
    test_env.env.as_contract(&test_env.autoshare_contract, || {
        let mut contributions = Vec::new(&test_env.env);
        for i in 0..25 {
            contributions.push_back(FundraisingContribution {
                group_id: group_id.clone(),
                contributor: contributor.clone(),
                token: token.clone(),
                amount: (i + 1) as i128,
                timestamp: i as u64,
            });
        }
        let key = DataKey::GroupContributions(group_id.clone());
        test_env
            .env
            .storage()
            .persistent()
            .set(&key, &contributions);
    });

    // Test page 1: offset 0, limit 10
    let (page1, total) = client.get_group_contribs_paginated(&group_id, &0, &10);
    assert_eq!(page1.len(), 10);
    assert_eq!(total, 25);
    assert_eq!(page1.get(0).unwrap().amount, 1);
    assert_eq!(page1.get(9).unwrap().amount, 10);

    // Test page 2: offset 10, limit 10
    let (page2, total) = client.get_group_contribs_paginated(&group_id, &10, &10);
    assert_eq!(page2.len(), 10);
    assert_eq!(total, 25);
    assert_eq!(page2.get(0).unwrap().amount, 11);
    assert_eq!(page2.get(9).unwrap().amount, 20);

    // Test page 3: offset 20, limit 10
    let (page3, total) = client.get_group_contribs_paginated(&group_id, &20, &10);
    assert_eq!(page3.len(), 5);
    assert_eq!(total, 25);
    assert_eq!(page3.get(0).unwrap().amount, 21);
    assert_eq!(page3.get(4).unwrap().amount, 25);

    // Test limit capping: offset 0, limit 50 (should cap at 20)
    let (capped_page, total) = client.get_group_contribs_paginated(&group_id, &0, &50);
    assert_eq!(capped_page.len(), 20);
    assert_eq!(total, 25);

    // Test offset out of bounds: offset 30, limit 10
    let (empty_page, total) = client.get_group_contribs_paginated(&group_id, &30, &10);
    assert_eq!(empty_page.len(), 0);
    assert_eq!(total, 25);
}

#[test]
fn test_get_user_contributions_pagination() {
    let test_env = setup_test_env();
    let client = AutoShareContractClient::new(&test_env.env, &test_env.autoshare_contract);

    let user = test_env.users.get(1).unwrap();
    let group_id = BytesN::from_array(&test_env.env, &[3u8; 32]);
    let token = test_env.mock_tokens.get(0).unwrap();

    // Create 15 contributions for this user
    test_env.env.as_contract(&test_env.autoshare_contract, || {
        let mut contributions = Vec::new(&test_env.env);
        for i in 0..15 {
            contributions.push_back(FundraisingContribution {
                group_id: group_id.clone(),
                contributor: user.clone(),
                token: token.clone(),
                amount: (i + 1) as i128,
                timestamp: i as u64,
            });
        }
        let key = DataKey::UserContributions(user.clone());
        test_env
            .env
            .storage()
            .persistent()
            .set(&key, &contributions);
    });

    // Test page 1: offset 0, limit 10
    let (page1, total) = client.get_user_contribs_paginated(&user, &0, &10);
    assert_eq!(page1.len(), 10);
    assert_eq!(total, 15);

    // Test page 2: offset 10, limit 10
    let (page2, total) = client.get_user_contribs_paginated(&user, &10, &10);
    assert_eq!(page2.len(), 5);
    assert_eq!(total, 15);
}
