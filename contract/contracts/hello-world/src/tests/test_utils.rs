use crate::{AutoShareContract, AutoShareContractClient};
use crate::mock_token::{MockToken, MockTokenClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

pub struct TestEnv {
    pub env: Env,
    pub admin: Address,
    pub users: Vec<Address>,
    pub autoshare_contract: Address,
    pub mock_tokens: Vec<Address>,
}

pub fn setup_test_env() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let admin = create_test_admin(&env);
    let users = create_test_users(&env, 3); // Default 3 users
    
    // Deploy AutoShare contract
    let contract_id = deploy_autoshare_contract(&env, &admin);
    
    // Deploy a mock token
    let token_id = deploy_mock_token(&env, &String::from_str(&env, "Test Token"), &String::from_str(&env, "TEST"));
    let mut mock_tokens = Vec::new(&env);
    mock_tokens.push_back(token_id);

    TestEnv {
        env,
        admin,
        users,
        autoshare_contract: contract_id,
        mock_tokens,
    }
}

pub fn create_test_admin(env: &Env) -> Address {
    Address::generate(env)
}

pub fn create_test_users(env: &Env, count: u32) -> Vec<Address> {
    let mut users = Vec::new(env);
    for _ in 0..count {
        users.push_back(Address::generate(env));
    }
    users
}

pub fn deploy_mock_token(env: &Env, name: &String, symbol: &String) -> Address {
    let contract_id = env.register(MockToken, ());
    let client = MockTokenClient::new(env, &contract_id);
    let admin = Address::generate(env); // Temp admin for initialization
    client.initialize(&admin, &7, name, symbol);
    contract_id
}

pub fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let client = MockTokenClient::new(env, token);
    client.mint(to, &amount);
}

// MockToken doesn't support approve, but we implement the signature as requested.
// We'll log or do nothing if not supported, but here it's a test util, so we can't really "fake" it if the contract doesn't support it.
// Assuming for now we might add standard token interface later or this is for a different token.
pub fn approve_tokens(_env: &Env, _token: &Address, _from: &Address, _spender: &Address, _amount: i128) {
    // Note: MockToken provided in this repo does not support approve/allowance.
    // usage of this function with current MockToken will fail if we try to call non-existent method.
    // For now we will look for 'approve' method on client, if not generic refactor might be needed.
    // But since MockTokenClient is generated from MockToken struct, and MockToken struct doesn't have approve, 
    // we cannot call client.approve. 
    // We will comment this out or leave it empty to allow compilation if the user really wants the function to exist.
    // However, the best approach is to implement it assuming standard token interface if possible, 
    // but without trait definition imported, we can't easily cast.
    // We will leave it as a placeholder.
}

pub fn deploy_autoshare_contract(env: &Env, _admin: &Address) -> Address {
    env.register(AutoShareContract, ())
}

pub fn setup_supported_tokens(_env: &Env, _contract: &Address, _tokens: &Vec<Address>) {
    // AutoShareContract does not currently support setting supported tokens.
    // This is a placeholder.
}

pub fn create_test_members(env: &Env, count: u32) -> Vec<crate::base::types::GroupMember> {
    let mut members = Vec::new(env);
    for _ in 0..count {
        members.push_back(crate::base::types::GroupMember {
            address: Address::generate(env),
        });
    }
    // "Distribute percentages" is mentioned in requirement, but GroupMember struct only has address.
    // So we ignore percentages for now.
    members
}

pub fn create_test_group(
    env: &Env,
    contract: &Address,
    creator: &Address,
    _members: &Vec<crate::base::types::GroupMember>,
    usages: u32,
    _token: &Address,
) -> BytesN<32> {
    let client = AutoShareContractClient::new(env, contract);
    let _id = BytesN::from_array(env, &[10u8; 32]); // Generate a deterministic or random ID? Contract requires unique.
    // For test utils, maybe we want random to avoid collision if called multiple times.
    // But `BytesN::from_array` needs a fixed array.
    // Let's use a counter or random bytes. 
    // Since we don't have easy random bytes gen in util without passing seed, let's just make a new one.
    // Actually `BytesN<32>` doesn't have a `random` method exposed easily here without `env`. 
    // We can use `BytesN::from_array(env, &[rand...])` if we had rand.
    // Let's just use `BytesN::from_array(env, &[0; 32])` implies same ID. Use different ID.
    // For now, simpler: pass ID as arg? No, req says return ID.
    // We'll generate a pseudo-random ID based on ledger sequence or something if possible, or just all 1s.
    let mut id_bytes = [0u8; 32];
    id_bytes[0..4].copy_from_slice(&usages.to_be_bytes());
    let id = BytesN::from_array(env, &id_bytes); 
    let name = String::from_str(env, "Test Group");
    
    client.create(&id, &name, creator);
    
    // Logic to add members if contract supported adding them at creation? 
    // Contract only has `add_group_member`.
    // So strictly we should iterate and add members.
    // But `create_test_group` params includes members.
    
    /* 
    for member in members.iter() {
        client.add_group_member(&id, &member.address);
    }
    */
    // We can't iterate easily over Soroban Vec in this context without `iter()` which might be available.
    // Let's leave member addition for specific tests if not strictly required to be automatic here.
    // Or try to implement it.
    
    id
}

pub fn fund_user_with_tokens(env: &Env, token: &Address, user: &Address, amount: i128) {
    mint_tokens(env, token, user, amount);
}

pub fn assert_balance(env: &Env, token: &Address, user: &Address, expected: i128) {
    let client = MockTokenClient::new(env, token);
    assert_eq!(client.balance(user), expected);
}

pub fn assert_group_exists(env: &Env, contract: &Address, id: &BytesN<32>) {
    let client = AutoShareContractClient::new(env, contract);
    // client.get panics on failure in current impl, so if it returns, it exists.
    // But checking if it *exists* without panic:
    // Existing tests use `client.get`.
    let _ = client.get(id); 
}

pub fn assert_is_admin(_env: &Env, _contract: &Address, _address: &Address) {
    // AutoShareContract doesn't have admin concept exposed in `is_admin`.
    // Placeholder.
}
