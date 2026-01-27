#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

// 1. Declare the foundational modules (Requirement: Modular Structure)
pub mod base {
    pub mod errors;
    pub mod events;
    pub mod types;
}

pub mod interfaces {
    pub mod autoshare;
}

// 2. Declare the main logic file where the functions are implemented
mod autoshare_logic;

pub mod mock_token;

#[contract]
pub struct AutoShareContract;

#[contractimpl]
impl AutoShareContract {
    /// Initializes the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        autoshare_logic::set_admin(env, admin).unwrap();
    }

    /// Pauses the contract. Only admin can call.
    pub fn pause(env: Env, admin: Address) {
        autoshare_logic::pause(env, admin).unwrap();
    }

    /// Unpauses the contract. Only admin can call.
    pub fn unpause(env: Env, admin: Address) {
        autoshare_logic::unpause(env, admin).unwrap();
    }

    /// Returns the current pause status.
    pub fn get_paused_status(env: Env) -> bool {
        autoshare_logic::get_paused_status(&env)
    }

    /// Creates a new AutoShare plan.
    /// Requirement: create_autoshare should store data and emit an event.
    pub fn create(
        env: Env,
        id: BytesN<32>,
        name: String,
        creator: Address,
        members: Vec<base::types::GroupMember>,
    ) {
        autoshare_logic::create_autoshare(env, id, name, creator, members).unwrap();
    }

    /// Update members of an existing AutoShare plan.
    /// Requirement: Only creator can update. Validates percentages.
    pub fn update_members(
        env: Env,
        id: BytesN<32>,
        caller: Address,
        new_members: Vec<base::types::GroupMember>,
    ) {
        autoshare_logic::update_members(env, id, caller, new_members).unwrap();
    }

    /// Retrieves an existing AutoShare plan.
    /// Requirement: get_autoshare should return the plan details.
    pub fn get(env: Env, id: BytesN<32>) -> base::types::AutoShareDetails {
        autoshare_logic::get_autoshare(env, id).unwrap()
    }

    /// Retrieves all AutoShare groups.
    pub fn get_all_groups(env: Env) -> Vec<base::types::AutoShareDetails> {
        autoshare_logic::get_all_groups(env)
    }

    /// Retrieves all AutoShare groups created by a specific address.
    pub fn get_groups_by_creator(env: Env, creator: Address) -> Vec<base::types::AutoShareDetails> {
        autoshare_logic::get_groups_by_creator(env, creator)
    }

    /// Checks if an address is a member of a specific group.
    pub fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> bool {
        autoshare_logic::is_group_member(env, id, address).unwrap()
    }

    pub fn get_group_members(env: Env, id: BytesN<32>) -> Vec<base::types::GroupMember> {
        autoshare_logic::get_group_members(env, id).unwrap()
    }

    /// Adds a member to a group with specified percentage.
    pub fn add_group_member(env: Env, id: BytesN<32>, address: Address, percentage: u32) {
        autoshare_logic::add_group_member(env, id, address, percentage).unwrap();
    }
}

// 3. Link the tests (Requirement: Unit Tests)
#[cfg(test)]
#[path = "tests/autoshare_test.rs"]
mod autoshare_test; // Links the internal tests/autoshare_test.rs inside src

#[cfg(test)]
#[path = "tests/pause_test.rs"]
mod pause_test;

#[cfg(test)]
#[path = "tests/mock_token_test.rs"]
mod mock_token_test;
