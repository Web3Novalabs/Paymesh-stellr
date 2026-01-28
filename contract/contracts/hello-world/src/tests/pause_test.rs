use crate::{AutoShareContract, AutoShareContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

#[test]
fn test_admin_can_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert!(!client.get_paused_status());
    client.pause(&admin);
    assert!(client.get_paused_status());
}

#[test]
fn test_admin_can_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause(&admin);
    assert!(client.get_paused_status());

    client.unpause(&admin);
    assert!(!client.get_paused_status());
}

#[test]
fn test_paused_status_returned_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Initially not paused
    assert!(!client.get_paused_status());

    // After pause
    client.pause(&admin);
    assert!(client.get_paused_status());

    // After unpause
    client.unpause(&admin);
    assert!(!client.get_paused_status());
}

#[test]
#[should_panic]
fn test_non_admin_cannot_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause(&non_admin);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause(&admin);
    client.unpause(&non_admin);
}

#[test]
#[should_panic]
fn test_cannot_pause_already_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause(&admin);
    client.pause(&admin);
}

#[test]
#[should_panic]
fn test_cannot_unpause_not_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.unpause(&admin);
}

#[test]
#[should_panic]
fn test_create_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.pause(&admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");
    let members = Vec::new(&env);
    client.create(&id, &name, &creator, &members);
}

#[test]
#[should_panic]
fn test_add_member_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let creator = Address::generate(&env);
    let member = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");
    let members = Vec::new(&env);

    client.create(&id, &name, &creator, &members);
    client.pause(&admin);
    client.add_group_member(&id, &member, &50u32);
}

#[test]
fn test_read_functions_work_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");
    let mut members = Vec::new(&env);
    members.push_back(crate::base::types::GroupMember {
        address: creator.clone(),
        percentage: 100,
    });

    client.create(&id, &name, &creator, &members);
    client.pause(&admin);

    // These should all work while paused
    let _ = client.get(&id);
    let _ = client.get_all_groups();
    let _ = client.get_groups_by_creator(&creator);
    let _ = client.get_group_members(&id);
    let _ = client.is_group_member(&id, &creator);
    let _ = client.get_paused_status();
}

#[test]
fn test_operations_work_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AutoShareContract, ());
    let client = AutoShareContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause(&admin);
    client.unpause(&admin);

    let creator = Address::generate(&env);
    let id = BytesN::from_array(&env, &[1u8; 32]);
    let name = String::from_str(&env, "Test Group");
    let mut members = Vec::new(&env);
    members.push_back(crate::base::types::GroupMember {
        address: creator.clone(),
        percentage: 100,
    });

    // Should work after unpause
    client.create(&id, &name, &creator, &members);
    let result = client.get(&id);
    assert_eq!(result.name, name);
}
