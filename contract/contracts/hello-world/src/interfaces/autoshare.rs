use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::base::types::{AutoShareDetails, GroupMember, PaymentHistory};

/// AutoShareTrait defines the interface for the AutoShare contract.
/// This trait serves as a formal specification that the AutoShareContract implementation
/// must adhere to, enabling compile-time verification and serving as documentation
/// for external integrators.
pub trait AutoShareTrait {
    // ============================================================================
    // Admin Management
    // ============================================================================

    /// Initializes the contract admin. Can only be called once.
    fn initialize_admin(env: Env, admin: Address);

    /// Pauses the contract. Only admin can call.
    fn pause(env: Env, admin: Address);

    /// Unpauses the contract. Only admin can call.
    fn unpause(env: Env, admin: Address);

    /// Returns the current pause status.
    fn get_paused_status(env: Env) -> bool;

    /// Returns the current admin address.
    fn get_admin(env: Env) -> Address;

    /// Transfers admin rights to a new address. Only current admin can call.
    fn transfer_admin(env: Env, current_admin: Address, new_admin: Address);

    /// Withdraws tokens from the contract. Only admin can call.
    fn withdraw(env: Env, admin: Address, token: Address, amount: i128, recipient: Address);

    /// Returns the contract's balance for a specified token.
    fn get_contract_balance(env: Env, token: Address) -> i128;

    // ============================================================================
    // AutoShare Group Management
    // ============================================================================

    /// Creates a new AutoShare plan with payment.
    fn create(
        env: Env,
        id: BytesN<32>,
        name: String,
        creator: Address,
        usage_count: u32,
        payment_token: Address,
    );

    /// Update members of an existing AutoShare plan.
    /// Only creator can update. Validates percentages.
    fn update_members(env: Env, id: BytesN<32>, caller: Address, new_members: Vec<GroupMember>);

    /// Retrieves an existing AutoShare plan.
    fn get(env: Env, id: BytesN<32>) -> AutoShareDetails;

    /// Retrieves all AutoShare groups.
    fn get_all_groups(env: Env) -> Vec<AutoShareDetails>;

    /// Retrieves all AutoShare groups created by a specific address.
    fn get_groups_by_creator(env: Env, creator: Address) -> Vec<AutoShareDetails>;

    /// Checks if an address is a member of a specific group.
    fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> bool;

    /// Returns all members of a group.
    fn get_group_members(env: Env, id: BytesN<32>) -> Vec<GroupMember>;

    /// Adds a member to a group with specified percentage.
    /// Only the group creator (caller) may add members.
    fn add_group_member(
        env: Env,
        id: BytesN<32>,
        caller: Address,
        address: Address,
        percentage: u32,
    );

    /// Removes a single member from a group. Only the creator can call; group must be active.
    /// After removal, remaining percentages may not sum to 100; call update_members to set a valid split.
    fn remove_group_member(env: Env, id: BytesN<32>, caller: Address, member_address: Address);

    /// Deactivates a group. Only the creator can deactivate.
    fn deactivate_group(env: Env, id: BytesN<32>, caller: Address);

    /// Activates a group. Only the creator can activate.
    fn activate_group(env: Env, id: BytesN<32>, caller: Address);

    /// Returns whether a group is active.
    fn is_group_active(env: Env, id: BytesN<32>) -> bool;

    /// Permanently deletes a group. Only creator or admin can delete.
    /// Group must be deactivated first and have 0 remaining usages.
    fn delete_group(env: Env, id: BytesN<32>, caller: Address);

    // ============================================================================
    // Token Management
    // ============================================================================

    /// Adds a supported payment token (admin only).
    fn add_supported_token(env: Env, token: Address, admin: Address);

    /// Removes a supported payment token (admin only).
    fn remove_supported_token(env: Env, token: Address, admin: Address);

    /// Returns all supported payment tokens.
    fn get_supported_tokens(env: Env) -> Vec<Address>;

    /// Checks if a token is supported.
    fn is_token_supported(env: Env, token: Address) -> bool;

    /// Distributes a payment among group members based on their percentages.
    fn distribute(env: Env, id: BytesN<32>, token: Address, amount: i128, sender: Address);

    // ============================================================================
    // Payment Configuration
    // ============================================================================

    /// Sets the usage fee (admin only).
    fn set_usage_fee(env: Env, fee: u32, admin: Address);

    /// Returns the current usage fee.
    fn get_usage_fee(env: Env) -> u32;

    // ============================================================================
    // Subscription Management
    // ============================================================================

    /// Tops up a group's subscription with additional usages.
    fn topup_subscription(
        env: Env,
        id: BytesN<32>,
        additional_usages: u32,
        payment_token: Address,
        payer: Address,
    );

    // ============================================================================
    // Payment History
    // ============================================================================

    /// Returns all payment history for a user.
    fn get_user_payment_history(env: Env, user: Address) -> Vec<PaymentHistory>;

    /// Returns all payment history for a group.
    fn get_group_payment_history(env: Env, id: BytesN<32>) -> Vec<PaymentHistory>;

    // ============================================================================
    // Usage Tracking
    // ============================================================================

    /// Returns the remaining usages for a group.
    fn get_remaining_usages(env: Env, id: BytesN<32>) -> u32;

    /// Returns the total usages paid for a group.
    fn get_total_usages_paid(env: Env, id: BytesN<32>) -> u32;

    /// Reduces the usage count by 1 (dummy function for testing).
    fn reduce_usage(env: Env, id: BytesN<32>);
}
