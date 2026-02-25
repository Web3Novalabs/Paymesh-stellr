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

#[cfg(test)]
pub mod mock_token;

#[contract]
pub struct AutoShareContract;

#[contractimpl]
impl AutoShareContract {
    // ============================================================================
    // Admin Management
    // ============================================================================

    /// Initializes the contract admin. Can only be called once.
    pub fn initialize_admin(env: Env, admin: Address) {
        autoshare_logic::initialize_admin(env, admin);
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

    // ============================================================================
    // AutoShare Group Management
    // ============================================================================

    /// Creates a new AutoShare plan with payment.
    /// Requirement: create_autoshare should store data, accept payment, and emit an event.
    pub fn create(
        env: Env,
        id: BytesN<32>,
        name: String,
        creator: Address,
        usage_count: u32,
        payment_token: Address,
    ) {
        autoshare_logic::create_autoshare(env, id, name, creator, usage_count, payment_token)
            .unwrap();
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

    /// Retrieves all AutoShare groups an address is a member of.
    pub fn get_groups_by_member(env: Env, member: Address) -> Vec<base::types::AutoShareDetails> {
        autoshare_logic::get_groups_by_member(env, member)
    }

    /// Returns a paginated list of groups.
    pub fn get_groups_paginated(env: Env, start_index: u32, limit: u32) -> base::types::GroupPage {
        autoshare_logic::get_groups_paginated(env, start_index, limit)
    }

    /// Returns a paginated list of groups created by a specific address.
    pub fn get_groups_by_creator_paginated(
        env: Env,
        creator: Address,
        offset: u32,
        limit: u32,
    ) -> base::types::GroupPage {
        autoshare_logic::get_groups_by_creator_paginated(env, creator, offset, limit)
    }

    /// Returns the total number of groups.
    pub fn get_group_count(env: Env) -> u32 {
        autoshare_logic::get_group_count(env)
    }

    /// Checks if an address is a member of a specific group.
    pub fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> bool {
        autoshare_logic::is_group_member(env, id, address).unwrap()
    }

    pub fn get_group_members(env: Env, id: BytesN<32>) -> Vec<base::types::GroupMember> {
        autoshare_logic::get_group_members(env, id).unwrap()
    }

    /// Adds a member to a group with specified percentage.
    /// Only the group creator (caller) may add members.
    pub fn add_group_member(
        env: Env,
        id: BytesN<32>,
        caller: Address,
        address: Address,
        percentage: u32,
    ) {
        autoshare_logic::add_group_member(env, id, caller, address, percentage).unwrap();
    }

    /// Removes a single member from a group. Only the creator can call; group must be active.
    /// After removal, remaining percentages may not sum to 100; call update_members to set a valid split.
    pub fn remove_group_member(env: Env, id: BytesN<32>, caller: Address, member_address: Address) {
        autoshare_logic::remove_group_member(env, id, caller, member_address).unwrap();
    }

    /// Deactivates a group. Only the creator can deactivate.
    pub fn deactivate_group(env: Env, id: BytesN<32>, caller: Address) {
        autoshare_logic::deactivate_group(env, id, caller).unwrap();
    }

    /// Activates a group. Only the creator can activate.
    pub fn activate_group(env: Env, id: BytesN<32>, caller: Address) {
        autoshare_logic::activate_group(env, id, caller).unwrap();
    }

    /// Updates the name of a group. Only the creator can update.
    pub fn update_group_name(env: Env, id: BytesN<32>, caller: Address, new_name: String) {
        autoshare_logic::update_group_name(env, id, caller, new_name).unwrap();
    }

    /// Returns whether a group is active.
    pub fn is_group_active(env: Env, id: BytesN<32>) -> bool {
        autoshare_logic::is_group_active(env, id).unwrap()
    }

    /// Permanently deletes a group. Only creator or admin can delete.
    /// Group must be deactivated first and have 0 remaining usages.
    pub fn delete_group(env: Env, id: BytesN<32>, caller: Address) {
        autoshare_logic::delete_group(env, id, caller).unwrap();
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Address {
        autoshare_logic::get_admin(env).unwrap()
    }

    /// Transfers admin rights to a new address. Only current admin can call.
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        autoshare_logic::transfer_admin(env, current_admin, new_admin).unwrap();
    }

    /// Withdraws tokens from the contract. Only admin can call.
    pub fn withdraw(env: Env, admin: Address, token: Address, amount: i128, recipient: Address) {
        autoshare_logic::withdraw(env, admin, token, amount, recipient).unwrap();
    }

    /// Returns the contract's balance for a specified token.
    pub fn get_contract_balance(env: Env, token: Address) -> i128 {
        autoshare_logic::get_contract_balance(env, token)
    }

    // ============================================================================
    // Token Management
    // ============================================================================

    /// Adds a supported payment token (admin only).
    pub fn add_supported_token(env: Env, token: Address, admin: Address) {
        autoshare_logic::add_supported_token(env, token, admin).unwrap();
    }

    /// Removes a supported payment token (admin only).
    pub fn remove_supported_token(env: Env, token: Address, admin: Address) {
        autoshare_logic::remove_supported_token(env, token, admin).unwrap();
    }

    /// Returns all supported payment tokens.
    pub fn get_supported_tokens(env: Env) -> Vec<Address> {
        autoshare_logic::get_supported_tokens(env)
    }

    /// Checks if a token is supported.
    pub fn is_token_supported(env: Env, token: Address) -> bool {
        autoshare_logic::is_token_supported(env, token)
    }

    /// Distributes a payment among group members based on their percentages.
    pub fn distribute(env: Env, id: BytesN<32>, token: Address, amount: i128, sender: Address) {
        autoshare_logic::distribute(env, id, token, amount, sender).unwrap();
    }

    // ============================================================================
    // Payment Configuration
    // ============================================================================

    /// Sets the usage fee (admin only).
    pub fn set_usage_fee(env: Env, fee: u32, admin: Address) {
        autoshare_logic::set_usage_fee(env, fee, admin).unwrap();
    }

    /// Returns the current usage fee.
    pub fn get_usage_fee(env: Env) -> u32 {
        autoshare_logic::get_usage_fee(env)
    }

    // ============================================================================
    // Subscription Management
    // ============================================================================

    /// Tops up a group's subscription with additional usages.
    pub fn topup_subscription(
        env: Env,
        id: BytesN<32>,
        additional_usages: u32,
        payment_token: Address,
        payer: Address,
    ) {
        autoshare_logic::topup_subscription(env, id, additional_usages, payment_token, payer)
            .unwrap();
    }

    // ============================================================================
    // Payment History
    // ============================================================================

    /// Returns all payment history for a user.
    pub fn get_user_payment_history(env: Env, user: Address) -> Vec<base::types::PaymentHistory> {
        autoshare_logic::get_user_payment_history(env, user)
    }

    /// Returns all payment history for a group.
    pub fn get_group_payment_history(env: Env, id: BytesN<32>) -> Vec<base::types::PaymentHistory> {
        autoshare_logic::get_group_payment_history(env, id)
    }

    // ============================================================================
    // Distribution History
    // ============================================================================

    /// Returns all distribution history for a group.
    pub fn get_group_distributions(
        env: Env,
        id: BytesN<32>,
    ) -> Vec<base::types::DistributionHistory> {
        autoshare_logic::get_group_distributions(env, id)
    }

    /// Returns all distribution history for a member.
    pub fn get_member_distributions(
        env: Env,
        member: Address,
    ) -> Vec<base::types::DistributionRecord> {
        autoshare_logic::get_member_distributions(env, member)
    }

    // ============================================================================
    // Usage Tracking
    // ============================================================================

    /// Returns the remaining usages for a group.
    pub fn get_remaining_usages(env: Env, id: BytesN<32>) -> u32 {
        autoshare_logic::get_remaining_usages(env, id).unwrap()
    }

    /// Returns the total usages paid for a group.
    pub fn get_total_usages_paid(env: Env, id: BytesN<32>) -> u32 {
        autoshare_logic::get_total_usages_paid(env, id).unwrap()
    }

    /// Returns the total earnings for a member from a specific group.
    pub fn get_member_earnings(env: Env, member: Address, group_id: BytesN<32>) -> i128 {
        autoshare_logic::get_member_earnings(env, member, group_id)
    }

    /// Returns the fundraising status for a group.
    pub fn get_fundraising_status(env: Env, id: BytesN<32>) -> base::types::FundraisingConfig {
        autoshare_logic::get_fundraising_status(env, id)
    }

    /// Returns all contributions for a specific group.
    pub fn get_group_contributions(
        env: Env,
        id: BytesN<32>,
    ) -> Vec<base::types::FundraisingContribution> {
        autoshare_logic::get_group_contributions(env, id)
    }

    /// Returns all contributions made by a specific user.
    pub fn get_user_contributions(
        env: Env,
        user: Address,
    ) -> Vec<base::types::FundraisingContribution> {
        autoshare_logic::get_user_contributions(env, user)
    }

    /// Starts a fundraising campaign for a group.
    pub fn start_fundraising(env: Env, id: BytesN<32>, caller: Address, target_amount: i128) {
        autoshare_logic::start_fundraising(env, id, caller, target_amount).unwrap();
    }

    /// Contributes funds to a fundraising campaign.
    pub fn contribute(
        env: Env,
        id: BytesN<32>,
        token: Address,
        amount: i128,
        contributor: Address,
    ) {
        autoshare_logic::contribute(env, id, token, amount, contributor).unwrap();
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

#[cfg(test)]
#[path = "tests/test_utils.rs"]
pub mod test_utils;

#[cfg(test)]
#[path = "tests/get_groups_by_member_test.rs"]
mod get_groups_by_member_test;

#[cfg(test)]
#[path = "tests/test_utils_test.rs"]
mod test_utils_test;

#[cfg(test)]
#[path = "tests/distribute_test.rs"]
mod distribute_test;

#[cfg(test)]
#[path = "tests/earnings_test.rs"]
mod earnings_test;

#[cfg(test)]
#[path = "tests/pagination_test.rs"]
mod pagination_test;

#[cfg(test)]
#[path = "tests/fundraising_test.rs"]
mod fundraising_test;

#[cfg(test)]
#[path = "tests/fundraising_start_test.rs"]
mod fundraising_start_test;

#[cfg(test)]
#[path = "tests/fundraising_contribute_test.rs"]
mod fundraising_contribute_test;

#[cfg(test)]
#[path = "tests/max_members_test.rs"]
mod max_members_test;
