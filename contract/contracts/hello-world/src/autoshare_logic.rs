use crate::base::errors::Error;
use crate::base::events::{
    emit_contribution, emit_creator_is_member, emit_distribution, emit_fundraising_cancelled,
    emit_fundraising_target_updated, emit_funds_deposited, emit_group_members_queried,
    emit_group_protocol_fee_updated, emit_max_members_updated, emit_member_added,
    emit_member_removed, emit_payment_group_deactivated, emit_usage_fee_updated, AdminTransferred,
    AutoshareCreated, AutoshareUpdated, ContractPaused, ContractUnpaused, FundraisingStarted,
    GroupActivated, GroupDeactivated, GroupDeleted, GroupNameUpdated, GroupOwnershipTransferred,
    Withdrawal,
};

use crate::base::types::{
    ActiveFundraising, AutoShareDetails, DepositRecord, DistributionHistory, DistributionRecord,
    FundraisingConfig, FundraisingContribution, GroupMember, GroupPage, GroupStats, GroupSummary,
    MemberAmount, MemberDistributionRecord, MemberPage, PaymentHistory,
};
use soroban_sdk::{contracttype, token, Address, BytesN, Env, String, Vec};

#[contracttype]
pub enum DataKey {
    AutoShare(BytesN<32>),
    AllGroups,
    Admin,
    SupportedTokens,
    UsageFee,
    UserPaymentHistory(Address),
    GroupPaymentHistory(BytesN<32>),
    GroupDistributionHistory(BytesN<32>),
    MemberDistributions(Address),
    MemberGroupEarnings(Address, BytesN<32>),
    GroupFundraising(BytesN<32>),
    GroupContributions(BytesN<32>),
    UserContributions(Address),
    GroupStats(BytesN<32>),
    IsPaused,
    MemberGroups(Address),
    GroupDistributions(BytesN<32>),
    MaxMembers,
    MinContribution,
    // Diagnostic: per-group invocation counter for get_group_members
    GroupMembersQueryCount(BytesN<32>),
    ProtocolFee,
    ProtocolFeeRecipient,
    GroupProtocolFee(BytesN<32>),
    // Issue #299: Deposit/Treasury tracking
    GroupTreasuryBalance(BytesN<32>, Address),
    GroupDepositHistory(BytesN<32>),
    DepositorHistory(Address),
}

const DAY_IN_LEDGERS: u32 = 17280;
const PERSISTENT_BUMP_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS; // 1 week
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS; // 30 days
const MAX_MEMBERS: u32 = 50; // Maximum number of members per group to prevent DoS
const CONTRACT_VERSION: u32 = 1;

fn bump_persistent<K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, key: &K) {
    if env.storage().persistent().has(key) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_BUMP_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
}

fn is_valid_name(name: &String) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.len() > 60 {
        return false;
    }

    let name_len = name.len() as usize;
    let mut buf = [0u8; 60];
    name.copy_into_slice(&mut buf[..name_len]);

    let mut only_whitespace = true;
    for b in buf[..name_len].iter() {
        if *b != b' ' && *b != b'\t' && *b != b'\n' && *b != b'\r' {
            only_whitespace = false;
            break;
        }
    }

    if only_whitespace {
        return false;
    }

    true
}

/// Creates a new payment group with a designated admin (creator), member limit, and initial
/// subscription configuration.
///
/// The creator pays an upfront fee of `usage_count × usage_fee` tokens to fund the group's
/// distribution quota. The group is stored in persistent ledger storage and starts active with
/// an empty member list. Members can be added afterwards via `add_group_member` or
/// `batch_add_members`.
///
/// # Arguments
///
/// * `env` - The Soroban environment.
/// * `id` - A unique 32-byte identifier for the group. Must not already exist in storage.
/// * `name` - A human-readable group name (1–60 non-whitespace characters).
/// * `creator` - The address that will own and administer the group. Must authorize this call.
/// * `usage_count` - Number of payment distributions to pre-purchase. Must be ≥ 1.
/// * `payment_token` - The token used to pay the creation fee. Must be on the supported-token list.
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Events
///
/// Emits [`AutoshareCreated`] with `creator` and `id` as fields.
///
/// # Errors
///
/// | Error | Condition |
/// |---|---|
/// | `ContractPaused` | The contract is currently paused. |
/// | `EmptyName` | `name` is empty, whitespace-only, or exceeds 60 characters. |
/// | `AlreadyExists` | A group with the given `id` already exists. |
/// | `InvalidUsageCount` | `usage_count` is 0. |
/// | `UnsupportedToken` | `payment_token` is not on the supported-token list. |
///
/// # Panics
///
/// Panics if the token transfer fails (e.g. insufficient creator balance or allowance).
pub fn create_autoshare(
    env: Env,
    id: BytesN<32>,
    name: String,
    creator: Address,
    usage_count: u32,
    payment_token: Address,
) -> Result<(), Error> {
    creator.require_auth();

    // Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    if !is_valid_name(&name) {
        return Err(Error::EmptyName);
    }

    let key = DataKey::AutoShare(id.clone());

    // Check if it already exists to prevent overwriting
    if env.storage().persistent().has(&key) {
        bump_persistent(&env, &key);
        return Err(Error::AlreadyExists);
    }

    // Validate usage count
    if usage_count == 0 {
        return Err(Error::InvalidUsageCount);
    }

    // Verify token is supported
    if !is_token_supported(env.clone(), payment_token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    // Calculate total cost
    let usage_fee = get_usage_fee(env.clone());
    let total_cost = (usage_count as i128) * (usage_fee as i128);

    // Transfer tokens from creator to contract
    let token_client = token::Client::new(&env, &payment_token);
    token_client.transfer(&creator, env.current_contract_address(), &total_cost);

    let details = AutoShareDetails {
        id: id.clone(),
        name,
        metadata: String::from_str(&env, ""),
        creator: creator.clone(),
        usage_count,
        total_usages_paid: usage_count,
        members: Vec::new(&env),
        is_active: true,
    };

    // Store the details in persistent storage
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // Add to all groups list
    let all_groups_key = DataKey::AllGroups;
    let mut all_groups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));
    all_groups.push_back(id.clone());
    env.storage().persistent().set(&all_groups_key, &all_groups);
    bump_persistent(&env, &all_groups_key);

    // Record payment history
    record_payment(
        env.clone(),
        creator.clone(),
        id.clone(),
        usage_count,
        total_cost,
    );

    AutoshareCreated {
        creator: creator.clone(),
        id: id.clone(),
    }
    .publish(&env);
    Ok(())
}

pub fn get_autoshare(env: Env, id: BytesN<32>) -> Result<AutoShareDetails, Error> {
    let key = DataKey::AutoShare(id);
    let result: Option<AutoShareDetails> = env.storage().persistent().get(&key);
    if result.is_some() {
        bump_persistent(&env, &key);
    }
    result.ok_or(Error::NotFound)
}

/// Retrieves a lightweight summary of payment group metadata, status, and statistics.
///
/// This function provides efficient access to commonly requested group information
/// without loading the full group details and member list. It's designed to reduce
/// RPC calls for frontend components that need group cards or summary displays.
/// The returned `GroupSummary` contains essential metadata for group listings,
/// status indicators, and basic statistics.
///
/// # Arguments
///
/// * `env` - The Soroban environment providing access to persistent storage
/// * `id` - The unique 32-byte identifier of the AutoShare payment group
///
/// # Returns
///
/// Returns `Result<GroupSummary, Error>` containing a lightweight group summary struct
/// with the following fields:
/// - `id`: The group identifier
/// - `name`: Human-readable group name
/// - `creator`: Address of the group creator
/// - `member_count`: Number of active members in the group
/// - `is_active`: Whether the group is active and accepting operations
/// - `remaining_usages`: Number of remaining payment distributions allowed
/// - `has_active_fundraising`: Whether the group has an active fundraising campaign
/// - `total_distributions`: Total number of payment distributions processed
///
/// # Events Emitted
///
/// This function does not emit any events as it is a read-only operation.
///
/// # Authorization
///
/// No authorization is required - this is a public read operation accessible to all addresses.
///
/// # Storage Access
///
/// * **Reads**: `AutoShare(id)` - Core group details (required)
/// * **Reads**: `GroupFundraising(id)` - Fundraising status (optional)
/// * **Reads**: `GroupDistributionHistory(id)` - Distribution count (optional)
/// * **TTL Extension**: Extends TTL for all accessed persistent storage entries
///
/// # Performance Characteristics
///
/// * **Time Complexity**: O(1) - Constant time lookups with bounded data sizes
/// * **Storage Operations**: 1-3 persistent reads depending on optional data
/// * **Memory Usage**: Minimal - only loads summary data, not full member lists
/// * **Network Efficiency**: Single RPC call returns all summary information
///
/// # Error Conditions
///
/// * `NotFound` - The specified group ID does not exist in storage
///
/// # Use Cases
///
/// This function is optimized for:
/// - Group listing displays in frontend applications
/// - Status indicators and badges
/// - Quick group information lookups
/// - Reducing RPC call frequency for UI components
/// - Group search and filtering operations
///
/// # Related Functions
///
/// * `get()` - Returns full group details including complete member list
/// * `get_group_members()` - Returns only the member list
/// * `is_group_active()` - Returns only the active status
///
/// # Example
///
/// ```ignore
/// // Get summary for efficient group display
/// let summary = get_group_summary(env, group_id)?;
///
/// // Use summary data for UI rendering
/// display_group_card(
///     summary.name,
///     summary.member_count,
///     summary.is_active,
///     summary.has_active_fundraising
/// );
/// ```
pub fn get_group_summary(env: Env, id: BytesN<32>) -> Result<GroupSummary, Error> {
    use crate::base::types::GroupSummary;

    // Get basic group info
    let key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    // Get fundraising status
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    let has_active_fundraising = env
        .storage()
        .persistent()
        .get::<_, FundraisingConfig>(&fundraising_key)
        .map(|f| f.is_active)
        .unwrap_or(false);
    if env.storage().persistent().has(&fundraising_key) {
        bump_persistent(&env, &fundraising_key);
    }

    // Get distribution count
    let dist_key = DataKey::GroupDistributionHistory(id.clone());
    let total_distributions = env
        .storage()
        .persistent()
        .get::<_, Vec<DistributionHistory>>(&dist_key)
        .map(|d| d.len())
        .unwrap_or(0);
    if env.storage().persistent().has(&dist_key) {
        bump_persistent(&env, &dist_key);
    }

    Ok(GroupSummary {
        id: details.id,
        name: details.name,
        metadata: details.metadata,
        creator: details.creator,
        member_count: details.members.len(),
        is_active: details.is_active,
        remaining_usages: details.usage_count,
        has_active_fundraising,
        total_distributions,
    })
}

fn get_all_group_ids(env: &Env) -> Vec<BytesN<32>> {
    let all_groups_key = DataKey::AllGroups;
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(env));
    if !group_ids.is_empty() {
        bump_persistent(env, &all_groups_key);
    }
    group_ids
}

pub fn get_group_count(env: Env) -> u32 {
    let group_ids = get_all_group_ids(&env);
    group_ids.len()
}

pub fn get_all_groups(env: Env) -> Vec<AutoShareDetails> {
    let group_ids = get_all_group_ids(&env);
    let mut result: Vec<AutoShareDetails> = Vec::new(&env);
    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id) {
            result.push_back(details);
        }
    }
    result
}

pub fn get_active_groups(env: Env) -> Vec<AutoShareDetails> {
    let group_ids = get_all_group_ids(&env);
    let mut result: Vec<AutoShareDetails> = Vec::new(&env);
    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id) {
            if details.is_active {
                result.push_back(details);
            }
        }
    }
    result
}

pub fn get_groups_by_creator(env: Env, creator: Address) -> Vec<AutoShareDetails> {
    let group_ids = get_all_group_ids(&env);
    let mut result: Vec<AutoShareDetails> = Vec::new(&env);

    for id in group_ids.iter() {
        if let Ok(group) = get_autoshare(env.clone(), id) {
            if group.creator == creator {
                result.push_back(group);
            }
        }
    }
    result
}

pub fn get_groups_paginated(
    env: Env,
    start_index: u32,
    limit: u32,
) -> crate::base::types::GroupPage {
    let group_ids = get_all_group_ids(&env);
    let total = group_ids.len();

    // Cap limit at 20 as per requirement
    let actual_limit = limit.min(20);

    let mut groups: Vec<AutoShareDetails> = Vec::new(&env);

    if actual_limit > 0 && start_index < total {
        let end = start_index.saturating_add(actual_limit).min(total);
        for i in start_index..end {
            if let Some(id) = group_ids.get(i) {
                if let Ok(details) = get_autoshare(env.clone(), id) {
                    groups.push_back(details);
                }
            }
        }
    }

    crate::base::types::GroupPage {
        groups,
        total,
        offset: start_index,
        limit: actual_limit,
    }
}

pub fn get_groups_by_creator_paginated(
    env: Env,
    creator: Address,
    offset: u32,
    limit: u32,
) -> crate::base::types::GroupPage {
    let group_ids = get_all_group_ids(&env);

    // Cap limit at 20 as per requirement
    let actual_limit = limit.min(20);
    if actual_limit == 0 {
        return crate::base::types::GroupPage {
            groups: Vec::new(&env),
            total: 0,
            offset,
            limit: actual_limit,
        };
    }

    let mut groups: Vec<AutoShareDetails> = Vec::new(&env);
    let mut total_matches = 0;
    let mut matches_returned = 0;

    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id) {
            if details.creator == creator {
                if total_matches >= offset && matches_returned < actual_limit {
                    groups.push_back(details);
                    matches_returned += 1;
                }
                total_matches += 1;
            }
        }
    }

    crate::base::types::GroupPage {
        groups,
        total: total_matches,
        offset,
        limit: actual_limit,
    }
}

pub fn get_groups_by_member(env: Env, member: Address) -> Vec<AutoShareDetails> {
    let key = DataKey::MemberGroups(member);
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(&env));

    if !group_ids.is_empty() {
        bump_persistent(&env, &key);
    }

    let mut result: Vec<AutoShareDetails> = Vec::new(&env);

    for id in group_ids.iter() {
        if let Ok(group) = get_autoshare(env.clone(), id.clone()) {
            if group.is_active {
                result.push_back(group);
            } else {
                // To match behavior where inactive groups are still returned by `get_autoshare`,
                // we include them. If they should be skipped, we add a check here.
                result.push_back(group);
            }
        }
    }

    result
}

pub fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> Result<bool, Error> {
    let details = get_autoshare(env, id)?;

    for member in details.members.iter() {
        if member.address == address {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns all members of a group.
///
/// ### Arguments
/// * `id` - The unique 32-byte identifier of the AutoShare group.
///
/// ### Returns
/// * `Result<Vec<GroupMember>, Error>` - A vector containing all group members and their percentages,
///   or an error if the group is not found.
///
/// ### Panics
/// * This function does not panic but returns `Error::NotFound` if the group ID is invalid.
pub fn get_group_members(env: Env, id: BytesN<32>) -> Result<Vec<GroupMember>, Error> {
    let details = get_autoshare(env.clone(), id.clone())?;
    let members = details.members;

    // ── Diagnostic tracking ──────────────────────────────────────────────────
    // Increment the per-group invocation counter and emit a diagnostic event so
    // off-chain indexers can track read frequency without any additional RPC calls.
    let counter_key = DataKey::GroupMembersQueryCount(id.clone());
    let prev_count: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0u64);
    let new_count = prev_count + 1;
    env.storage().persistent().set(&counter_key, &new_count);
    bump_persistent(&env, &counter_key);

    emit_group_members_queried(&env, id, members.len(), new_count);
    // ────────────────────────────────────────────────────────────────────────

    Ok(members)
}

/// Returns the cumulative number of times `get_group_members` has been called
/// for the given group. Returns 0 if the function has never been invoked.
/// Intended for off-chain analytics dashboards.
pub fn get_group_members_query_count(env: Env, id: BytesN<32>) -> u64 {
    let key = DataKey::GroupMembersQueryCount(id);
    let count: u64 = env.storage().persistent().get(&key).unwrap_or(0u64);
    if count > 0 {
        bump_persistent(&env, &key);
    }
    count
}

/// Returns a paginated list of all current members in a specific group.
///
/// This function provides efficient access to group members with pagination support,
/// optimized for storage reads and minimal data transfer.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `id` - The unique identifier of the AutoShare group
/// * `offset` - The starting index for pagination (0-based)
/// * `limit` - The maximum number of members to return
///
/// # Returns
///
/// Returns a `MemberPage` struct containing:
/// * `members` - Vector of GroupMember in the current page
/// * `total` - Total number of members in the group
/// * `offset` - The offset used for this query
/// * `limit` - The limit used for this query
///
/// # Errors
///
/// Returns `NotFound` if the group does not exist.
pub fn get_group_members_paginated(
    env: Env,
    id: BytesN<32>,
    offset: u32,
    limit: u32,
) -> Result<MemberPage, Error> {
    let details = get_autoshare(env.clone(), id.clone())?;
    let all_members = details.members;
    let total = all_members.len();

    const MAX_PAGE_SIZE: u32 = 20;
    let limit = limit.min(MAX_PAGE_SIZE);

    // Calculate start and end indices for pagination
    let start = offset.min(total);
    let end = (start + limit).min(total);

    // Extract the page of members
    let mut page_members = Vec::new(&env);
    let mut i = start;
    while i < end {
        page_members.push_back(all_members.get(i).unwrap());
        i += 1;
    }

    Ok(MemberPage {
        members: page_members,
        total,
        offset,
        limit,
    })
}

pub fn get_member_percentage(env: Env, id: BytesN<32>, member: Address) -> Result<u32, Error> {
    let details = get_autoshare(env, id)?;
    for m in details.members.iter() {
        if m.address == member {
            return Ok(m.percentage);
        }
    }
    Err(Error::MemberNotFound)
}

/// Adds a new member to an existing payment group while verifying capacity limits.
///
/// This function allows the group creator to add a single member to their AutoShare group.
/// The operation includes comprehensive validation to ensure data integrity and prevent
/// abuse. After adding the member, the total percentage allocation across all members
/// must equal exactly 100% for the group to remain valid.
///
/// # Arguments
///
/// * `env` - The Soroban environment providing access to storage, events, and authentication
/// * `id` - The unique 32-byte identifier of the AutoShare group
/// * `caller` - The address of the caller, must be the group creator for authorization
/// * `address` - The Stellar address of the new member to add to the group
/// * `percentage` - The percentage share (1-99) this member will receive in distributions
///
/// # Returns
///
/// Returns `Ok(())` on successful addition of the member, or an `Error` if validation fails.
///
/// # Events Emitted
///
/// * `AutoshareUpdated` - Published when the group membership is successfully modified
/// * `MemberAdded` - Published with the group ID, new member address, and assigned percentage
/// * `CreatorIsMember` - Published if the new member address matches the group creator
///
/// # Authorization
///
/// Only the group creator can call this function. The caller's address must match
/// the `creator` field stored in the group's `AutoShareDetails`.
///
/// # Validation Checks
///
/// 1. **Contract State**: Contract must not be paused
/// 2. **Group Existence**: Group with the given ID must exist
/// 3. **Authorization**: Caller must be the group creator
/// 4. **Group Status**: Group must be active (not deactivated)
/// 5. **Duplicate Prevention**: Address must not already be a member
/// 6. **Capacity Limits**: Group must not exceed maximum members (default 50, configurable)
/// 7. **Percentage Validation**: After addition, total member percentages must sum to 100%
/// 8. **Percentage Bounds**: Individual percentage must be greater than 0
///
/// # Storage Updates
///
/// * Updates the `AutoShare(id)` persistent storage with the new member list
/// * Updates the `MemberGroups(address)` index to include this group for the new member
/// * Extends TTL for all accessed persistent storage entries
///
/// # Potential Errors
///
/// * `ContractPaused` - Contract is currently paused by admin
/// * `NotFound` - No group exists with the specified ID
/// * `Unauthorized` - Caller is not the group creator
/// * `GroupInactive` - Group has been deactivated
/// * `AlreadyExists` - The address is already a member of this group
/// * `MaxMembersExceeded` - Adding this member would exceed the group's maximum capacity
/// * `InvalidTotalPercentage` - After addition, member percentages don't sum to 100%
/// * `InvalidInput` - The percentage value is 0 or invalid
///
/// # Security Considerations
///
/// * Prevents DoS attacks by enforcing maximum member limits per group
/// * Ensures percentage integrity to prevent distribution calculation errors
/// * Maintains audit trail through event emission
/// * Requires explicit authorization from group creator
///
/// # Performance Notes
///
/// * Iterates through existing members to check for duplicates (O(n) where n ≤ 50)
/// * Validates percentage sums across all members after addition
/// * Updates multiple storage entries with TTL extensions
///
/// # Example
///
/// ```ignore
/// // Add a member with 25% share to an existing group
/// add_group_member(env, group_id, creator_address, member_address, 25)?;
/// ```
pub fn add_group_member(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    address: Address,
    percentage: u32,
) -> Result<(), Error> {
    // Require caller auth and check pause
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    // Only the group creator can add members
    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    // Check if already a member
    for member in details.members.iter() {
        if member.address == address {
            return Err(Error::AlreadyExists);
        }
    }

    // Check if adding this member would exceed the max members limit
    if details.members.len() >= get_max_members(&env) {
        return Err(Error::MaxMembersExceeded);
    }

    if address == details.creator {
        emit_creator_is_member(&env, id.clone());
    }

    // Add new member
    details.members.push_back(GroupMember {
        address: address.clone(),
        percentage,
    });

    // Validate total percentage after adding
    validate_members(&details.members)?;

    // Save updated details
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // Update MemberGroups index
    let member_groups_key = DataKey::MemberGroups(address.clone());
    let mut member_groups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&member_groups_key)
        .unwrap_or(Vec::new(&env));
    member_groups.push_back(id.clone());
    env.storage()
        .persistent()
        .set(&member_groups_key, &member_groups);
    bump_persistent(&env, &member_groups_key);

    AutoshareUpdated {
        id: id.clone(),
        updater: caller.clone(),
        name_updated: false,
        metadata_updated: false,
        new_creator: None,
    }
    .publish(&env);

    crate::base::events::emit_member_added(&env, id.clone(), address.clone(), percentage);

    crate::base::events::emit_member_added_to_group(
        &env,
        id,
        address,
        caller,
        percentage,
        details.members.len(),
    );

    Ok(())
}

/// Adds a single member to an existing payment group.
/// Distinct from `add_group_member` in that it:
///   - validates percentage is in [1, 100]
///   - ensures the running total across all existing members + new member does not exceed 100
///   - enforces capacity limit before any other mutation
pub fn add_member_to_group(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_member: Address,
    percentage: u32,
) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // Validate percentage range before touching storage
    if percentage == 0 || percentage > 100 {
        return Err(Error::InvalidInput);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    // Capacity check before any mutation
    if details.members.len() >= get_max_members(&env) {
        return Err(Error::MaxMembersExceeded);
    }

    // Duplicate check
    for member in details.members.iter() {
        if member.address == new_member {
            return Err(Error::AlreadyExists);
        }
    }

    // Running-total check: existing sum + new percentage must not exceed 100
    let mut current_total: u32 = 0;
    for member in details.members.iter() {
        current_total += member.percentage;
    }
    if current_total + percentage > 100 {
        return Err(Error::InvalidTotalPercentage);
    }

    if new_member == details.creator {
        emit_creator_is_member(&env, id.clone());
    }

    details.members.push_back(GroupMember {
        address: new_member.clone(),
        percentage,
    });

    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // Update MemberGroups index
    let member_groups_key = DataKey::MemberGroups(new_member.clone());
    let mut member_groups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&member_groups_key)
        .unwrap_or(Vec::new(&env));
    member_groups.push_back(id.clone());
    env.storage()
        .persistent()
        .set(&member_groups_key, &member_groups);
    bump_persistent(&env, &member_groups_key);

    AutoshareUpdated {
        id: id.clone(),
        updater: caller,
        name_updated: false,
        metadata_updated: false,
        new_creator: None,
    }
    .publish(&env);

    emit_member_added(&env, id, new_member, percentage);

    Ok(())
}

pub fn batch_add_members(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_members: Vec<GroupMember>,
) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    if new_members.is_empty() {
        return Err(Error::EmptyMembers);
    }

    // Check combined count won't exceed the configured max members
    if details.members.len() + new_members.len() > get_max_members(&env) {
        return Err(Error::MaxMembersExceeded);
    }

    // Validate no duplicates within new_members and against existing members
    let mut seen: Vec<Address> = Vec::new(&env);
    for new_member in new_members.iter() {
        // Check against existing members
        for existing in details.members.iter() {
            if existing.address == new_member.address {
                return Err(Error::DuplicateMember);
            }
        }
        // Check within new_members
        for s in seen.iter() {
            if s == new_member.address {
                return Err(Error::DuplicateMember);
            }
        }
        seen.push_back(new_member.address.clone());
    }

    // Validate that existing percentages + new member percentages sum to exactly 100
    let mut total: u32 = 0;
    for m in details.members.iter() {
        total += m.percentage;
    }
    for m in new_members.iter() {
        if m.percentage == 0 {
            return Err(Error::InvalidInput);
        }
        total += m.percentage;
    }
    if total != 100 {
        return Err(Error::InvalidTotalPercentage);
    }

    // Append all new members and update MemberGroups index
    for new_member in new_members.iter() {
        details.members.push_back(new_member.clone());

        let member_groups_key = DataKey::MemberGroups(new_member.address.clone());
        let mut member_groups: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&member_groups_key)
            .unwrap_or(Vec::new(&env));
        member_groups.push_back(id.clone());
        env.storage()
            .persistent()
            .set(&member_groups_key, &member_groups);
        bump_persistent(&env, &member_groups_key);
    }

    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    AutoshareUpdated {
        id: id.clone(),
        updater: caller,
        name_updated: false,
        metadata_updated: false,
        new_creator: None,
    }
    .publish(&env);

    Ok(())
}

pub fn remove_group_member(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    member_address: Address,
) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    let mut found = false;
    let mut removed_percentage: u32 = 0;
    let mut new_members: Vec<GroupMember> = Vec::new(&env);
    for member in details.members.iter() {
        if member.address == member_address {
            found = true;
            removed_percentage = member.percentage;
        } else {
            new_members.push_back(member.clone());
        }
    }
    if !found {
        return Err(Error::MemberNotFound);
    }

    details.members = new_members.clone();
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // Update MemberGroups index
    let member_groups_key = DataKey::MemberGroups(member_address.clone());
    let member_groups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&member_groups_key)
        .unwrap_or(Vec::new(&env));

    let mut new_member_groups: Vec<BytesN<32>> = Vec::new(&env);
    let mut group_removed = false;
    for group_id in member_groups.iter() {
        if group_id != id {
            new_member_groups.push_back(group_id);
        } else {
            group_removed = true;
        }
    }

    if group_removed {
        env.storage()
            .persistent()
            .set(&member_groups_key, &new_member_groups);
        bump_persistent(&env, &member_groups_key);
    }

    let pending_earnings = get_member_earnings(env.clone(), member_address.clone(), id.clone());

    AutoshareUpdated {
        id: id.clone(),
        updater: caller,
        name_updated: false,
        metadata_updated: false,
        new_creator: None,
    }
    .publish(&env);

    emit_member_removed(
        &env,
        id.clone(),
        member_address.clone(),
        removed_percentage,
        pending_earnings,
    );

    Ok(())
}

// ============================================================================
// Admin Management
// ============================================================================

pub fn initialize_admin(env: Env, admin: Address) {
    admin.require_auth();
    let admin_key = DataKey::Admin;

    // Only set if not already initialized
    if !env.storage().persistent().has(&admin_key) {
        env.storage().persistent().set(&admin_key, &admin);
        bump_persistent(&env, &admin_key);

        // Initialize default usage fee (10 tokens per usage)
        let usage_fee_key = DataKey::UsageFee;
        env.storage().persistent().set(&usage_fee_key, &10u32);
        bump_persistent(&env, &usage_fee_key);

        // Initialize empty supported tokens list
        let tokens_key = DataKey::SupportedTokens;
        let empty_tokens: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&tokens_key, &empty_tokens);
        bump_persistent(&env, &tokens_key);
    } else {
        bump_persistent(&env, &admin_key);
    }
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin_key = DataKey::Admin;
    let admin: Address = env
        .storage()
        .persistent()
        .get(&admin_key)
        .ok_or(Error::Unauthorized)?;
    bump_persistent(env, &admin_key);

    if admin != *caller {
        return Err(Error::Unauthorized);
    }

    Ok(())
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    let admin_key = DataKey::Admin;
    let result: Option<Address> = env.storage().persistent().get(&admin_key);
    if result.is_some() {
        bump_persistent(&env, &admin_key);
    }
    result.ok_or(Error::NotFound)
}

pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) -> Result<(), Error> {
    current_admin.require_auth();
    require_admin(&env, &current_admin)?;

    let admin_key = DataKey::Admin;
    env.storage().persistent().set(&admin_key, &new_admin);
    bump_persistent(&env, &admin_key);
    AdminTransferred {
        old_admin: current_admin,
        new_admin,
    }
    .publish(&env);
    Ok(())
}

// ============================================================================
// Pause Management
// ============================================================================

pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let pause_key = DataKey::IsPaused;
    let is_paused: bool = env.storage().persistent().get(&pause_key).unwrap_or(false);
    bump_persistent(&env, &pause_key);

    if is_paused {
        return Err(Error::AlreadyPaused);
    }

    env.storage().persistent().set(&pause_key, &true);
    bump_persistent(&env, &pause_key);
    ContractPaused {}.publish(&env);
    Ok(())
}

pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let pause_key = DataKey::IsPaused;
    let is_paused: bool = env.storage().persistent().get(&pause_key).unwrap_or(false);
    bump_persistent(&env, &pause_key);

    if !is_paused {
        return Err(Error::NotPaused);
    }

    env.storage().persistent().set(&pause_key, &false);
    bump_persistent(&env, &pause_key);
    ContractUnpaused {}.publish(&env);
    Ok(())
}

pub fn get_paused_status(env: &Env) -> bool {
    let pause_key = DataKey::IsPaused;
    let is_paused: bool = env.storage().persistent().get(&pause_key).unwrap_or(false);
    if is_paused {
        bump_persistent(env, &pause_key);
    }
    is_paused
}

pub fn get_contract_version(_env: Env) -> u32 {
    CONTRACT_VERSION
}

// ============================================================================
// Supported Tokens Management
// ============================================================================

pub fn add_supported_token(env: Env, token: Address, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let tokens_key = DataKey::SupportedTokens;
    let mut tokens: Vec<Address> = env
        .storage()
        .persistent()
        .get(&tokens_key)
        .unwrap_or(Vec::new(&env));
    if !tokens.is_empty() {
        bump_persistent(&env, &tokens_key);
    }

    // Check if token is already supported
    for existing_token in tokens.iter() {
        if existing_token == token {
            return Err(Error::AlreadyExists);
        }
    }

    tokens.push_back(token.clone());
    env.storage().persistent().set(&tokens_key, &tokens);
    bump_persistent(&env, &tokens_key);
    crate::base::events::emit_token_added(&env, admin, token);
    Ok(())
}

pub fn remove_supported_token(env: Env, token: Address, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let tokens_key = DataKey::SupportedTokens;
    let tokens: Vec<Address> = env
        .storage()
        .persistent()
        .get(&tokens_key)
        .unwrap_or(Vec::new(&env));
    if !tokens.is_empty() {
        bump_persistent(&env, &tokens_key);
    }

    let mut new_tokens: Vec<Address> = Vec::new(&env);
    let mut found = false;

    for existing_token in tokens.iter() {
        if existing_token != token {
            new_tokens.push_back(existing_token);
        } else {
            found = true;
        }
    }

    if !found {
        return Err(Error::NotFound);
    }

    env.storage().persistent().set(&tokens_key, &new_tokens);
    bump_persistent(&env, &tokens_key);
    crate::base::events::emit_token_removed(&env, admin, token);
    Ok(())
}

pub fn get_supported_tokens(env: Env) -> Vec<Address> {
    let tokens_key = DataKey::SupportedTokens;
    let result: Option<Vec<Address>> = env.storage().persistent().get(&tokens_key);
    if result.is_some() {
        bump_persistent(&env, &tokens_key);
    }
    result.unwrap_or(Vec::new(&env))
}

pub fn is_token_supported(env: Env, token: Address) -> bool {
    let tokens = get_supported_tokens(env);
    for supported_token in tokens.iter() {
        if supported_token == token {
            return true;
        }
    }
    false
}

// ============================================================================
// Payment Configuration
// ============================================================================

pub fn set_usage_fee(env: Env, fee: u32, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;
    if fee == 0 {
        return Err(Error::InvalidAmount);
    }

    let old_fee = get_usage_fee(env.clone());
    let fee_key = DataKey::UsageFee;
    env.storage().persistent().set(&fee_key, &fee);
    bump_persistent(&env, &fee_key);

    emit_usage_fee_updated(&env, admin, old_fee, fee);
    Ok(())
}

pub fn get_usage_fee(env: Env) -> u32 {
    let fee_key = DataKey::UsageFee;
    let result: Option<u32> = env.storage().persistent().get(&fee_key);
    if result.is_some() {
        bump_persistent(&env, &fee_key);
    }
    result.unwrap_or(10u32)
}

// ============================================================================
// Subscription Management
// ============================================================================

pub fn set_max_members(env: Env, admin: Address, max: u32) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    if max == 0 {
        return Err(Error::InvalidInput);
    }

    let old_max = get_max_members(&env);
    let key = DataKey::MaxMembers;
    env.storage().persistent().set(&key, &max);
    bump_persistent(&env, &key);

    emit_max_members_updated(&env, old_max, max);
    Ok(())
}

pub fn get_max_members(env: &Env) -> u32 {
    let key = DataKey::MaxMembers;
    let max: u32 = env.storage().persistent().get(&key).unwrap_or(MAX_MEMBERS);
    if env.storage().persistent().has(&key) {
        bump_persistent(env, &key);
    }
    max
}

pub fn set_min_contribution(env: Env, admin: Address, min_amount: i128) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;
    if min_amount < 0 {
        return Err(Error::InvalidAmount);
    }
    let key = DataKey::MinContribution;
    env.storage().persistent().set(&key, &min_amount);
    bump_persistent(&env, &key);
    Ok(())
}

pub fn get_min_contribution(env: Env) -> i128 {
    let key = DataKey::MinContribution;
    let result: Option<i128> = env.storage().persistent().get(&key);
    if result.is_some() {
        bump_persistent(&env, &key);
    }
    result.unwrap_or(0i128)
}

pub fn topup_subscription(
    env: Env,
    id: BytesN<32>,
    additional_usages: u32,
    payment_token: Address,
    payer: Address,
) -> Result<(), Error> {
    payer.require_auth();

    // Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // Validate usage count
    if additional_usages == 0 {
        return Err(Error::InvalidUsageCount);
    }

    // Verify group exists
    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    // Verify token is supported
    if !is_token_supported(env.clone(), payment_token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    // Calculate cost
    let usage_fee = get_usage_fee(env.clone());
    let total_cost = (additional_usages as i128) * (usage_fee as i128);

    // Transfer tokens from payer to contract
    let token_client = token::Client::new(&env, &payment_token);
    token_client.transfer(&payer, env.current_contract_address(), &total_cost);

    // Update usage counts
    details.usage_count += additional_usages;
    details.total_usages_paid += additional_usages;

    // Save updated details
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // Record payment history
    record_payment(env, payer, id, additional_usages, total_cost);

    Ok(())
}

// ============================================================================
// Payment History
// ============================================================================

fn record_payment(
    env: Env,
    user: Address,
    group_id: BytesN<32>,
    usages_purchased: u32,
    amount_paid: i128,
) {
    let timestamp = env.ledger().timestamp();

    let payment = PaymentHistory {
        user: user.clone(),
        group_id: group_id.clone(),
        usages_purchased,
        amount_paid,
        timestamp,
    };

    // Add to user's payment history
    let user_history_key = DataKey::UserPaymentHistory(user.clone());
    let mut user_history: Vec<PaymentHistory> = env
        .storage()
        .persistent()
        .get(&user_history_key)
        .unwrap_or(Vec::new(&env));
    if !user_history.is_empty() {
        bump_persistent(&env, &user_history_key);
    }
    user_history.push_back(payment.clone());
    env.storage()
        .persistent()
        .set(&user_history_key, &user_history);
    bump_persistent(&env, &user_history_key);

    // Add to group's payment history
    let group_history_key = DataKey::GroupPaymentHistory(group_id);
    let mut group_history: Vec<PaymentHistory> = env
        .storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env));
    if !group_history.is_empty() {
        bump_persistent(&env, &group_history_key);
    }
    group_history.push_back(payment);
    env.storage()
        .persistent()
        .set(&group_history_key, &group_history);
    bump_persistent(&env, &group_history_key);
}

pub fn get_user_payment_history(env: Env, user: Address) -> Vec<PaymentHistory> {
    let user_history_key = DataKey::UserPaymentHistory(user);
    let result: Option<Vec<PaymentHistory>> = env.storage().persistent().get(&user_history_key);
    if result.is_some() {
        bump_persistent(&env, &user_history_key);
    }
    result.unwrap_or(Vec::new(&env))
}

pub fn get_group_payment_history(env: Env, id: BytesN<32>) -> Vec<PaymentHistory> {
    let group_history_key = DataKey::GroupPaymentHistory(id);
    let result: Option<Vec<PaymentHistory>> = env.storage().persistent().get(&group_history_key);
    if result.is_some() {
        bump_persistent(&env, &group_history_key);
    }
    result.unwrap_or(Vec::new(&env))
}

pub fn get_user_pay_history_paginated(
    env: Env,
    user: Address,
    offset: u32,
    limit: u32,
) -> (Vec<PaymentHistory>, u32) {
    let user_history_key = DataKey::UserPaymentHistory(user);
    let history: Vec<PaymentHistory> = env
        .storage()
        .persistent()
        .get(&user_history_key)
        .unwrap_or(Vec::new(&env));

    let total = history.len();
    if total > 0 {
        bump_persistent(&env, &user_history_key);
    }

    let actual_limit = limit.min(20);
    let mut paginated_history = Vec::new(&env);

    if actual_limit > 0 && offset < total {
        let end = offset.saturating_add(actual_limit).min(total);
        for i in offset..end {
            if let Some(payment) = history.get(i) {
                paginated_history.push_back(payment);
            }
        }
    }

    (paginated_history, total)
}

pub fn get_group_pay_history_paginated(
    env: Env,
    id: BytesN<32>,
    offset: u32,
    limit: u32,
) -> (Vec<PaymentHistory>, u32) {
    let group_history_key = DataKey::GroupPaymentHistory(id);
    let history: Vec<PaymentHistory> = env
        .storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env));

    let total = history.len();
    if total > 0 {
        bump_persistent(&env, &group_history_key);
    }

    let actual_limit = limit.min(20);
    let mut paginated_history = Vec::new(&env);

    if actual_limit > 0 && offset < total {
        let end = offset.saturating_add(actual_limit).min(total);
        for i in offset..end {
            if let Some(payment) = history.get(i) {
                paginated_history.push_back(payment);
            }
        }
    }

    (paginated_history, total)
}

// ============================================================================
// Distribution History
// ============================================================================

fn record_distribution(
    env: Env,
    group_id: BytesN<32>,
    sender: Address,
    total_amount: i128,
    token: Address,
    member_amounts: Vec<MemberAmount>,
    distribution_number: u32,
) {
    let timestamp = env.ledger().timestamp();

    let distribution = DistributionHistory {
        group_id: group_id.clone(),
        sender: sender.clone(),
        total_amount,
        token: token.clone(),
        member_amounts: member_amounts.clone(),
        timestamp,
        distribution_number,
    };

    // Add to group's distribution history
    let group_history_key = DataKey::GroupDistributionHistory(group_id.clone());
    let mut group_history: Vec<DistributionHistory> = env
        .storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env));
    group_history.push_back(distribution.clone());
    env.storage()
        .persistent()
        .set(&group_history_key, &group_history);

    // Add to group's distribution history (Issue #107)
    let group_dist_key = DataKey::GroupDistributions(group_id.clone());
    let mut group_distributions: Vec<DistributionRecord> = env
        .storage()
        .persistent()
        .get(&group_dist_key)
        .unwrap_or(Vec::new(&env));
    group_distributions.push_back(DistributionRecord {
        group_id: group_id.clone(),
        sender: sender.clone(),
        token: token.clone(),
        total_amount,
        member_count: member_amounts.len(),
        timestamp,
    });
    env.storage()
        .persistent()
        .set(&group_dist_key, &group_distributions);
    bump_persistent(&env, &group_dist_key);

    // Add to each member's distribution history
    for member_amount in member_amounts.iter() {
        let member_history_key = DataKey::MemberDistributions(member_amount.address.clone());
        let mut member_history: Vec<MemberDistributionRecord> = env
            .storage()
            .persistent()
            .get(&member_history_key)
            .unwrap_or(Vec::new(&env));
        let record = MemberDistributionRecord {
            group_id: group_id.clone(),
            amount: member_amount.amount,
            token: token.clone(),
            timestamp,
        };
        member_history.push_back(record);
        env.storage()
            .persistent()
            .set(&member_history_key, &member_history);
    }
}

pub fn get_group_distributions(env: Env, id: BytesN<32>) -> Vec<DistributionRecord> {
    let group_dist_key = DataKey::GroupDistributions(id);
    env.storage()
        .persistent()
        .get(&group_dist_key)
        .unwrap_or(Vec::new(&env))
}

pub fn get_distribution_history_paginated(
    env: Env,
    id: BytesN<32>,
    offset: u32,
    limit: u32,
) -> (Vec<DistributionRecord>, u32) {
    let group_dist_key = DataKey::GroupDistributions(id);
    let distributions: Vec<DistributionRecord> = env
        .storage()
        .persistent()
        .get(&group_dist_key)
        .unwrap_or(Vec::new(&env));

    let total = distributions.len();
    if total == 0 {
        return (Vec::new(&env), 0);
    }

    bump_persistent(&env, &group_dist_key);

    // Cap limit at 20
    let actual_limit = limit.min(20);
    let mut paginated = Vec::new(&env);

    if actual_limit > 0 && offset < total {
        let end = offset.saturating_add(actual_limit).min(total);
        for i in offset..end {
            if let Some(record) = distributions.get(i) {
                paginated.push_back(record);
            }
        }
    }

    (paginated, total)
}

pub fn get_group_total_distributed(env: Env, id: BytesN<32>) -> i128 {
    let distributions = get_group_distributions(env, id);
    let mut total: i128 = 0;
    for dist in distributions.iter() {
        total += dist.total_amount;
    }
    total
}

pub fn get_member_distributions(env: Env, member: Address) -> Vec<MemberDistributionRecord> {
    let member_dist_key = DataKey::MemberDistributions(member);
    env.storage()
        .persistent()
        .get(&member_dist_key)
        .unwrap_or(Vec::new(&env))
}

pub fn get_member_distrib_paginated(
    env: Env,
    member: Address,
    offset: u32,
    limit: u32,
) -> (Vec<MemberDistributionRecord>, u32) {
    let member_dist_key = DataKey::MemberDistributions(member);
    let distributions: Vec<MemberDistributionRecord> = env
        .storage()
        .persistent()
        .get(&member_dist_key)
        .unwrap_or(Vec::new(&env));

    let total = distributions.len();
    if total == 0 {
        return (Vec::new(&env), 0);
    }

    bump_persistent(&env, &member_dist_key);

    // Cap limit at 20
    let actual_limit = limit.min(20);
    let mut paginated_distributions = Vec::new(&env);

    if actual_limit > 0 && offset < total {
        let end = offset.saturating_add(actual_limit).min(total);
        for i in offset..end {
            if let Some(distribution) = distributions.get(i) {
                paginated_distributions.push_back(distribution);
            }
        }
    }

    (paginated_distributions, total)
}

// ============================================================================
// Usage Tracking
// ============================================================================

pub fn get_remaining_usages(env: Env, id: BytesN<32>) -> Result<u32, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);
    Ok(details.usage_count)
}

pub fn get_total_usages_paid(env: Env, id: BytesN<32>) -> Result<u32, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);
    Ok(details.total_usages_paid)
}

pub fn reduce_usage(env: Env, id: BytesN<32>) -> Result<(), Error> {
    let key = DataKey::AutoShare(id);
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.usage_count == 0 {
        return Err(Error::NoUsagesRemaining);
    }

    details.usage_count -= 1;
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);
    Ok(())
}

// ============================================================================
// Group Activation Management
// ============================================================================

pub fn update_members(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_members: Vec<GroupMember>,
) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    // Validate new members
    if new_members.is_empty() {
        return Err(Error::EmptyMembers);
    }

    // Check if new members count exceeds MAX_MEMBERS
    if new_members.len() > get_max_members(&env) {
        return Err(Error::MaxMembersExceeded);
    }

    let mut total_percentage: u32 = 0;
    let mut seen_addresses = Vec::new(&env);

    for member in new_members.iter() {
        if member.percentage == 0 {
            return Err(Error::InvalidInput);
        }
        total_percentage += member.percentage;

        for seen in seen_addresses.iter() {
            if seen == member.address {
                return Err(Error::DuplicateMember);
            }
        }
        seen_addresses.push_back(member.address.clone());

        if member.address == details.creator {
            emit_creator_is_member(&env, id.clone());
        }
    }

    if total_percentage != 100 {
        return Err(Error::InvalidTotalPercentage);
    }

    // Determine old members for index updating
    let old_members = details.members.clone();

    // Update members in details
    details.members = new_members.clone();
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // Update MemberGroups index for removed and added members
    for old_member in old_members.iter() {
        let mut found_in_new = false;
        for new_member in new_members.iter() {
            if old_member.address == new_member.address {
                found_in_new = true;
                break;
            }
        }
        if !found_in_new {
            // Member was removed, remove group from their index
            let member_groups_key = DataKey::MemberGroups(old_member.address.clone());
            let member_groups: Vec<BytesN<32>> = env
                .storage()
                .persistent()
                .get(&member_groups_key)
                .unwrap_or(Vec::new(&env));

            let mut updated_member_groups: Vec<BytesN<32>> = Vec::new(&env);
            let mut group_removed = false;
            for group_id in member_groups.iter() {
                if group_id != id {
                    updated_member_groups.push_back(group_id);
                } else {
                    group_removed = true;
                }
            }
            if group_removed {
                env.storage()
                    .persistent()
                    .set(&member_groups_key, &updated_member_groups);
                bump_persistent(&env, &member_groups_key);
            }
        }
    }

    for new_member in new_members.iter() {
        let mut found_in_old = false;
        for old_member in old_members.iter() {
            if new_member.address == old_member.address {
                found_in_old = true;
                break;
            }
        }
        if !found_in_old {
            // Member was added, add group to their index
            let member_groups_key = DataKey::MemberGroups(new_member.address.clone());
            let mut member_groups: Vec<BytesN<32>> = env
                .storage()
                .persistent()
                .get(&member_groups_key)
                .unwrap_or(Vec::new(&env));

            member_groups.push_back(id.clone());
            env.storage()
                .persistent()
                .set(&member_groups_key, &member_groups);
            bump_persistent(&env, &member_groups_key);
        }
    }

    AutoshareUpdated {
        id: id.clone(),
        updater: caller,
        name_updated: false,
        metadata_updated: false,
        new_creator: None,
    }
    .publish(&env);
    Ok(())
}

pub fn deactivate_group(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupAlreadyInactive);
    }

    details.is_active = false;
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    GroupDeactivated {
        id: id.clone(),
        creator: caller,
    }
    .publish(&env);
    Ok(())
}

/// Deactivates a payment group so it can no longer accept new distributions or member changes.
/// Emits a dedicated `PaymentGroupDeactivated` event with indexed group id and caller for off-chain indexing.
pub fn deactivate_payment_group(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupAlreadyInactive);
    }

    let member_count = details.members.len();
    details.is_active = false;
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    emit_payment_group_deactivated(&env, id, caller, member_count);
    Ok(())
}

pub fn activate_group(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if details.is_active {
        return Err(Error::GroupAlreadyActive);
    }

    details.is_active = true;
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    GroupActivated {
        id: id.clone(),
        creator: caller,
    }
    .publish(&env);
    Ok(())
}

pub fn update_group_name(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_name: String,
) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    if !is_valid_name(&new_name) {
        return Err(Error::EmptyName);
    }

    details.name = new_name;
    env.storage().persistent().set(&key, &details);

    GroupNameUpdated {
        id: id.clone(),
        updater: caller,
    }
    .publish(&env);
    Ok(())
}

/// Updates the configurable settings of an existing payment group.
///
/// This function allows the group creator to update the group name, metadata, and
/// transfer ownership (admin rotation) in a single transaction. Only the current
/// creator is authorized to perform these updates.
///
/// # Arguments
///
/// * `env` - The Soroban environment.
/// * `id` - The unique 32-byte identifier of the payment group.
/// * `caller` - The address of the current creator. Must authorize this call.
/// * `new_name` - Optional new name for the group (1–60 non-whitespace characters).
/// * `new_metadata` - Optional new metadata string for the group.
/// * `new_creator` - Optional new address to transfer group ownership to.
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Events
///
/// Emits [`AutoshareUpdated`] with detailed change flags and new values.
///
/// # Errors
///
/// | Error | Condition |
/// |---|---|
/// | `ContractPaused` | The contract is currently paused. |
/// | `NotFound` | No group exists with the given `id`. |
/// | `Unauthorized` | The `caller` is not the current group creator. |
/// | `GroupInactive` | The group is currently deactivated. |
/// | `EmptyName` | The `new_name` is empty or whitespace-only. |
pub fn update_payment_group(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_name: Option<String>,
    new_metadata: Option<String>,
    new_creator: Option<Address>,
) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    let mut updated = false;
    let mut name_updated = false;
    let mut metadata_updated = false;
    let mut new_creator_addr: Option<Address> = None;

    if let Some(name) = new_name {
        if !is_valid_name(&name) {
            return Err(Error::EmptyName);
        }
        details.name = name;
        updated = true;
        name_updated = true;
    }

    if let Some(metadata) = new_metadata {
        details.metadata = metadata;
        updated = true;
        metadata_updated = true;
    }

    if let Some(creator) = new_creator {
        details.creator = creator.clone();
        updated = true;
        new_creator_addr = Some(creator);
    }

    if updated {
        env.storage().persistent().set(&key, &details);
        bump_persistent(&env, &key);

        AutoshareUpdated {
            id,
            updater: caller,
            name_updated,
            metadata_updated,
            new_creator: new_creator_addr,
        }
        .publish(&env);
    }

    Ok(())
}

pub fn transfer_group_ownership(
    env: Env,
    id: BytesN<32>,
    current_creator: Address,
    new_creator: Address,
) -> Result<(), Error> {
    // 1. Authorize current creator
    current_creator.require_auth();

    // 2. Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // 3. Verify group existence and creator
    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    // bump persistent on read
    bump_persistent(&env, &key);

    if details.creator != current_creator {
        return Err(Error::Unauthorized);
    }

    // 4. Update group creator
    let old_creator = details.creator.clone();
    details.creator = new_creator.clone();
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    // 5. Emit transfer event
    GroupOwnershipTransferred {
        group_id: id,
        old_creator,
        new_creator,
    }
    .publish(&env);

    Ok(())
}

pub fn is_group_active(env: Env, id: BytesN<32>) -> Result<bool, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);
    Ok(details.is_active)
}

// ============================================================================
// Group Deletion
// ============================================================================

/// Permanently deletes a group from the contract.
/// Requirements:
/// 1. Caller must be the group creator or admin
/// 2. Group must be deactivated
/// 3. Group must have 0 remaining usages (or they are forfeited)
/// 4. Removes group from AllGroups list
/// 5. Removes AutoShare(id) entry
/// 6. Archives payment history before deletion (keeps it for audit trail)
/// 7. Emits GroupDeleted event
pub fn delete_group(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    caller.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // Step 1: Verify group exists
    let key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    // We don't bump here if we are removing it, but we might return early if deactivated check fails
    // However, the requirement says bump on every read.
    bump_persistent(&env, &key);

    // Step 2: Verify caller is creator or admin
    let admin_result = get_admin(env.clone());
    let is_admin = admin_result.is_ok() && admin_result.unwrap() == caller;
    let is_creator = details.creator == caller;

    if !is_creator && !is_admin {
        return Err(Error::Unauthorized);
    }

    // Step 3: Check group is already deactivated
    if details.is_active {
        return Err(Error::GroupNotDeactivated);
    }

    // Step 4: Check if group has active fundraising
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    if let Some(fundraising) = env
        .storage()
        .persistent()
        .get::<_, FundraisingConfig>(&fundraising_key)
    {
        if fundraising.is_active {
            return Err(Error::GroupHasActiveFundraising);
        }
    }

    // Step 5: Check group has 0 remaining usages
    if details.usage_count > 0 {
        return Err(Error::GroupHasRemainingUsages);
    }

    // Step 5: Remove the group from AllGroups list
    let all_groups_key = DataKey::AllGroups;
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));
    if !group_ids.is_empty() {
        bump_persistent(&env, &all_groups_key);
    }

    let mut new_group_ids: Vec<BytesN<32>> = Vec::new(&env);
    for group_id in group_ids.iter() {
        if group_id != id {
            new_group_ids.push_back(group_id);
        }
    }
    env.storage()
        .persistent()
        .set(&all_groups_key, &new_group_ids);
    bump_persistent(&env, &all_groups_key);

    // Step 6: Remove the AutoShare(id) entry
    env.storage().persistent().remove(&key);

    // Step 7: Archive payment history (we keep it for audit trail)
    // Payment history is intentionally NOT deleted to maintain financial records
    // This is a best practice for compliance and auditing purposes
    // The entries remain in:
    // - DataKey::UserPaymentHistory(Address)
    // - DataKey::GroupPaymentHistory(BytesN<32>)

    // Step 8: Remove group from all members' MemberGroups index
    for member in details.members.iter() {
        let member_groups_key = DataKey::MemberGroups(member.address.clone());
        let member_groups: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&member_groups_key)
            .unwrap_or(Vec::new(&env));

        let mut updated_member_groups: Vec<BytesN<32>> = Vec::new(&env);
        let mut group_removed = false;
        for group_id in member_groups.iter() {
            if group_id != id {
                updated_member_groups.push_back(group_id);
            } else {
                group_removed = true;
            }
        }
        if group_removed {
            env.storage()
                .persistent()
                .set(&member_groups_key, &updated_member_groups);
            bump_persistent(&env, &member_groups_key);
        }
    }

    // Step 9: Emit deletion event
    GroupDeleted {
        deleter: caller,
        id: id.clone(),
    }
    .publish(&env);

    Ok(())
}

pub fn admin_delete_group(env: Env, admin: Address, id: BytesN<32>) -> Result<(), Error> {
    // 1. Require admin auth
    admin.require_auth();

    // 2. Verify caller is the contract admin
    require_admin(&env, &admin)?;

    // 3. Read AutoShare(id), returning Error::NotFound if missing
    let key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    // 4. if a fundraising campaign is active, sets it to inactive first
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    if let Some(mut config) = env
        .storage()
        .persistent()
        .get::<_, FundraisingConfig>(&fundraising_key)
    {
        if config.is_active {
            config.is_active = false;
            env.storage().persistent().set(&fundraising_key, &config);
            bump_persistent(&env, &fundraising_key);
        }
    }

    // 5. Removes the group from AllGroups vector
    let all_groups_key = DataKey::AllGroups;
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));

    let mut new_group_ids: Vec<BytesN<32>> = Vec::new(&env);
    let mut group_found = false;
    for group_id in group_ids.iter() {
        if group_id != id {
            new_group_ids.push_back(group_id);
        } else {
            group_found = true;
        }
    }

    if group_found {
        env.storage()
            .persistent()
            .set(&all_groups_key, &new_group_ids);
        bump_persistent(&env, &all_groups_key);
    }

    // 6. removes the group from all members' MemberGroups
    for member in details.members.iter() {
        let member_groups_key = DataKey::MemberGroups(member.address.clone());
        if let Some(member_groups) = env
            .storage()
            .persistent()
            .get::<_, Vec<BytesN<32>>>(&member_groups_key)
        {
            let mut updated_member_groups: Vec<BytesN<32>> = Vec::new(&env);
            let mut found = false;
            for group_id in member_groups.iter() {
                if group_id != id {
                    updated_member_groups.push_back(group_id);
                } else {
                    found = true;
                }
            }
            if found {
                env.storage()
                    .persistent()
                    .set(&member_groups_key, &updated_member_groups);
                bump_persistent(&env, &member_groups_key);
            }
        }
    }

    // 7. deletes AutoShare(id) from storage
    env.storage().persistent().remove(&key);

    // 8. preserves all payment history and distribution records
    // (Explicitly not deleting UserPaymentHistory, GroupPaymentHistory, GroupDistributions keys)

    // 9. emits GroupDeleted event
    GroupDeleted {
        deleter: admin,
        id: id.clone(),
    }
    .publish(&env);

    Ok(())
}

// ============================================================================
// Contract Balance & Withdrawal
// ============================================================================

pub fn get_contract_balance(env: Env, token: Address) -> i128 {
    let client = token::TokenClient::new(&env, &token);
    client.balance(&env.current_contract_address())
}

pub fn withdraw(
    env: Env,
    admin: Address,
    token: Address,
    amount: i128,
    recipient: Address,
) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let contract_balance = get_contract_balance(env.clone(), token.clone());
    if contract_balance < amount {
        return Err(Error::InsufficientContractBalance);
    }

    let client = token::TokenClient::new(&env, &token);
    client.transfer(&env.current_contract_address(), &recipient, &amount);

    Withdrawal {
        token,
        amount,
        recipient,
    }
    .publish(&env);
    Ok(())
}

#[allow(clippy::needless_borrows_for_generic_args)]
pub fn distribute(
    env: Env,
    id: BytesN<32>,
    token: Address,
    amount: i128,
    sender: Address,
) -> Result<(), Error> {
    sender.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    if !is_token_supported(env.clone(), token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    if details.usage_count == 0 {
        return Err(Error::NoUsagesRemaining);
    }

    validate_members(&details.members)?;

    let client = token::TokenClient::new(&env, &token);
    client.transfer(&sender, &env.current_contract_address(), &amount);
    let member_amounts = perform_distribution(&env, &id, &token, amount, &details.members);
    let distribution_number = details.total_usages_paid - details.usage_count;
    record_distribution(
        env.clone(),
        id.clone(),
        sender.clone(),
        amount,
        token.clone(),
        member_amounts.clone(),
        distribution_number,
    );
    // Emit new distribution event for fund flow tracking
    emit_distribution(&env, &id, &sender, &token, amount, member_amounts.len());

    // Update group stats
    let stats_key = DataKey::GroupStats(id.clone());
    let mut stats: GroupStats = env
        .storage()
        .persistent()
        .get(&stats_key)
        .unwrap_or(GroupStats {
            total_distributed: 0,
            distribution_count: 0,
            total_raised: 0,
            contribution_count: 0,
        });
    stats.total_distributed += amount;
    stats.distribution_count += 1;
    env.storage().persistent().set(&stats_key, &stats);
    bump_persistent(&env, &stats_key);

    details.usage_count -= 1;
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    Ok(())
}

fn perform_distribution(
    env: &Env,
    id: &BytesN<32>,
    token: &Address,
    amount: i128,
    members: &Vec<GroupMember>,
) -> Vec<MemberAmount> {
    let client = token::TokenClient::new(env, token);
    let mut distributed: i128 = 0;
    let members_len = members.len() as usize;
    let mut member_amounts: Vec<MemberAmount> = Vec::new(env);
    for (idx, member) in members.iter().enumerate() {
        let share = if idx + 1 < members_len {
            let percentage = member.percentage as i128;
            (amount / 100) * percentage + (amount % 100) * percentage / 100
        } else {
            amount - distributed
        };
        if share > 0 {
            client.transfer(&env.current_contract_address(), &member.address, &share);
            distributed += share;
            member_amounts.push_back(MemberAmount {
                address: member.address.clone(),
                amount: share,
            });

            // Update running total for member group earnings
            let earnings_key = DataKey::MemberGroupEarnings(member.address.clone(), id.clone());
            let current_earnings: i128 = env.storage().persistent().get(&earnings_key).unwrap_or(0);
            env.storage()
                .persistent()
                .set(&earnings_key, &(current_earnings + share));
            bump_persistent(env, &earnings_key);
        }
    }
    member_amounts
}

pub fn get_member_earnings(env: Env, member: Address, group_id: BytesN<32>) -> i128 {
    let key = DataKey::MemberGroupEarnings(member, group_id);
    let earnings: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    if earnings > 0 {
        bump_persistent(&env, &key);
    }
    earnings
}

/// Returns a breakdown of a member's earnings across all groups they belong to.
/// Each entry in the returned Vec is a (group_id, earnings) tuple.
/// Only groups where the member has earned more than zero are included.
/// Returns an empty Vec if the member has no groups or no positive earnings.
pub fn get_member_earnings_breakdown(env: Env, member: Address) -> Vec<(BytesN<32>, i128)> {
    // Step 1: Read the list of group IDs this member belongs to.
    // MemberGroups is updated whenever a member is added/removed from a group.
    let member_groups_key = DataKey::MemberGroups(member.clone());
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&member_groups_key)
        .unwrap_or(Vec::new(&env));

    // If the member has no groups at all, return empty immediately.
    if group_ids.is_empty() {
        return Vec::new(&env);
    }

    // Bump TTL for the member groups index since we just read it.
    bump_persistent(&env, &member_groups_key);

    // Step 2: For each group, look up the member's earnings in that group.
    // Step 3: Filter out groups where earnings are zero — only keep positive entries.
    let mut breakdown: Vec<(BytesN<32>, i128)> = Vec::new(&env);
    for group_id in group_ids.iter() {
        let earnings_key = DataKey::MemberGroupEarnings(member.clone(), group_id.clone());
        let earnings: i128 = env.storage().persistent().get(&earnings_key).unwrap_or(0);

        // Only include this group if the member has actually earned something from it.
        if earnings > 0 {
            bump_persistent(&env, &earnings_key);
            breakdown.push_back((group_id, earnings));
        }
    }

    breakdown
}

pub fn get_fundraising_status(env: Env, id: BytesN<32>) -> FundraisingConfig {
    let key = DataKey::GroupFundraising(id);
    let result: Option<FundraisingConfig> = env.storage().persistent().get(&key);
    if let Some(config) = result {
        bump_persistent(&env, &key);
        config
    } else {
        FundraisingConfig {
            target_amount: 0,
            total_raised: 0,
            is_active: false,
        }
    }
}

pub fn get_group_contributions(env: Env, id: BytesN<32>) -> Vec<FundraisingContribution> {
    let key = DataKey::GroupContributions(id);
    let result: Option<Vec<FundraisingContribution>> = env.storage().persistent().get(&key);
    if result.is_some() {
        bump_persistent(&env, &key);
    }
    result.unwrap_or(Vec::new(&env))
}

pub fn get_user_contributions(env: Env, user: Address) -> Vec<FundraisingContribution> {
    let key = DataKey::UserContributions(user);
    let result: Option<Vec<FundraisingContribution>> = env.storage().persistent().get(&key);
    if result.is_some() {
        bump_persistent(&env, &key);
    }
    result.unwrap_or(Vec::new(&env))
}

pub fn get_group_contribs_paginated(
    env: Env,
    id: BytesN<32>,
    offset: u32,
    limit: u32,
) -> (Vec<FundraisingContribution>, u32) {
    let contributions = get_group_contributions(env.clone(), id);
    let total = contributions.len();

    // Cap limit at 20
    let actual_limit = limit.min(20);

    let mut result: Vec<FundraisingContribution> = Vec::new(&env);
    if actual_limit > 0 && offset < total {
        let end = offset.saturating_add(actual_limit).min(total);
        for i in offset..end {
            if let Some(contribution) = contributions.get(i) {
                result.push_back(contribution);
            }
        }
    }

    (result, total)
}

pub fn get_user_contribs_paginated(
    env: Env,
    user: Address,
    offset: u32,
    limit: u32,
) -> (Vec<FundraisingContribution>, u32) {
    let contributions = get_user_contributions(env.clone(), user);
    let total = contributions.len();

    // Cap limit at 20
    let actual_limit = limit.min(20);

    let mut result: Vec<FundraisingContribution> = Vec::new(&env);
    if actual_limit > 0 && offset < total {
        let end = offset.saturating_add(actual_limit).min(total);
        for i in offset..end {
            if let Some(contribution) = contributions.get(i) {
                result.push_back(contribution);
            }
        }
    }

    (result, total)
}

fn validate_members(members: &Vec<GroupMember>) -> Result<(), Error> {
    if members.is_empty() {
        return Err(Error::EmptyMembers);
    }
    let env = members.env();
    let mut total_percentage: u32 = 0;
    let mut seen_addresses = Vec::new(env);

    for member in members.iter() {
        if member.percentage == 0 {
            return Err(Error::InvalidInput);
        }
        total_percentage += member.percentage;
        for seen in seen_addresses.iter() {
            if seen == member.address {
                return Err(Error::DuplicateMember);
            }
        }
        seen_addresses.push_back(member.address.clone());
    }

    if total_percentage != 100 {
        return Err(Error::InvalidTotalPercentage);
    }
    Ok(())
}

pub fn start_fundraising(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    target_amount: i128,
) -> Result<(), Error> {
    caller.require_auth();

    // Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // Verify group exists
    let key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    // Verify caller is the group creator
    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    // Verify group is active
    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    // Check no active fundraiser already exists for this group
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    let existing_fundraising: Option<FundraisingConfig> =
        env.storage().persistent().get(&fundraising_key);

    if let Some(config) = existing_fundraising {
        if config.is_active {
            return Err(Error::FundraisingAlreadyActive);
        }
        bump_persistent(&env, &fundraising_key);
    }

    // Validate target_amount > 0
    if target_amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Store a new FundraisingConfig
    let fundraising_config = FundraisingConfig {
        target_amount,
        total_raised: 0,
        is_active: true,
    };

    env.storage()
        .persistent()
        .set(&fundraising_key, &fundraising_config);
    bump_persistent(&env, &fundraising_key);

    // Emit a FundraisingStarted event
    FundraisingStarted {
        group_id: id,
        target_amount,
    }
    .publish(&env);

    Ok(())
}

pub fn contribute(
    env: Env,
    id: BytesN<32>,
    token: Address,
    amount: i128,
    contributor: Address,
) -> Result<(), Error> {
    contributor.require_auth();

    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let min_contribution = get_min_contribution(env.clone());
    if min_contribution > 0 && amount < min_contribution {
        return Err(Error::BelowMinimumContribution);
    }

    if !is_token_supported(env.clone(), token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    // Verify group exists and is active
    let group_key = DataKey::AutoShare(id.clone());
    let group_details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&group_key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &group_key);

    if !group_details.is_active {
        return Err(Error::GroupInactive);
    }

    // Verify fundraising is active
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    let mut fundraising_config: FundraisingConfig = env
        .storage()
        .persistent()
        .get(&fundraising_key)
        .ok_or(Error::FundraisingNotActive)?;

    if !fundraising_config.is_active {
        return Err(Error::FundraisingNotActive);
    }
    bump_persistent(&env, &fundraising_key);

    // Transfer amount from contributor to the contract
    let token_client = token::Client::new(&env, &token);
    token_client.transfer(&contributor, env.current_contract_address(), &amount);

    // Distribute funds to group members
    let member_amounts = perform_distribution(&env, &id, &token, amount, &group_details.members);

    // Record the distribution for transparency (Requirement 6)
    record_distribution(
        env.clone(),
        id.clone(),
        contributor.clone(),
        amount,
        token.clone(),
        member_amounts.clone(),
        0, // Fundraising distributions don't have a usage number
    );

    // Emit distribution event
    emit_distribution(
        &env,
        &id,
        &contributor,
        &token,
        amount,
        member_amounts.len(),
    );

    // Update fundraising total
    fundraising_config.total_raised += amount;
    let mut completed = false;
    if fundraising_config.total_raised >= fundraising_config.target_amount {
        fundraising_config.is_active = false;
        completed = true;
    }
    env.storage()
        .persistent()
        .set(&fundraising_key, &fundraising_config);
    bump_persistent(&env, &fundraising_key);

    // Record contribution
    let contribution = FundraisingContribution {
        group_id: id.clone(),
        contributor: contributor.clone(),
        token: token.clone(),
        amount,
        timestamp: env.ledger().timestamp(),
    };

    let group_contributions_key = DataKey::GroupContributions(id.clone());
    let mut group_contributions: Vec<FundraisingContribution> = env
        .storage()
        .persistent()
        .get(&group_contributions_key)
        .unwrap_or(Vec::new(&env));
    group_contributions.push_back(contribution.clone());
    env.storage()
        .persistent()
        .set(&group_contributions_key, &group_contributions);
    bump_persistent(&env, &group_contributions_key);

    let user_contributions_key = DataKey::UserContributions(contributor.clone());
    let mut user_contributions: Vec<FundraisingContribution> = env
        .storage()
        .persistent()
        .get(&user_contributions_key)
        .unwrap_or(Vec::new(&env));
    user_contributions.push_back(contribution);
    env.storage()
        .persistent()
        .set(&user_contributions_key, &user_contributions);
    bump_persistent(&env, &user_contributions_key);

    // Update group stats
    let stats_key = DataKey::GroupStats(id.clone());
    let mut stats: GroupStats = env
        .storage()
        .persistent()
        .get(&stats_key)
        .unwrap_or(GroupStats {
            total_distributed: 0,
            distribution_count: 0,
            total_raised: 0,
            contribution_count: 0,
        });
    stats.total_raised += amount;
    stats.contribution_count += 1;
    env.storage().persistent().set(&stats_key, &stats);
    bump_persistent(&env, &stats_key);

    // Emit new contribution event for fundraising tracking
    emit_contribution(&env, &id, &contributor, &token, amount);

    if completed {
        crate::base::events::emit_fundraising_completed(
            &env,
            id.clone(),
            fundraising_config.target_amount,
            fundraising_config.total_raised,
            stats.contribution_count,
        );
    }

    Ok(())
}

/// Returns the fundraising progress as a percentage (0-100).
/// Returns 0 if no fundraising campaign exists.
pub fn get_fundraising_progress(env: Env, id: BytesN<32>) -> u32 {
    let key = DataKey::GroupFundraising(id);
    let config: Option<FundraisingConfig> = env.storage().persistent().get(&key);

    if let Some(fundraising) = config {
        if fundraising.target_amount > 0 {
            bump_persistent(&env, &key);
            let progress = (fundraising.total_raised * 100) / fundraising.target_amount;
            // Cap at 100%
            if progress > 100 {
                100
            } else {
                progress as u32
            }
        } else {
            0
        }
    } else {
        0
    }
}

/// Checks if a fundraising campaign has reached its target.
pub fn is_fundraising_target_reached(env: Env, id: BytesN<32>) -> bool {
    let key = DataKey::GroupFundraising(id);
    let config: Option<FundraisingConfig> = env.storage().persistent().get(&key);

    if let Some(fundraising) = config {
        bump_persistent(&env, &key);
        fundraising.total_raised >= fundraising.target_amount
    } else {
        false
    }
}

/// Returns the total amount a user has contributed across all groups.
pub fn get_user_total_contributions(env: Env, user: Address) -> i128 {
    let key = DataKey::UserContributions(user);
    let contributions: Vec<FundraisingContribution> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(&env));

    if contributions.is_empty() {
        return 0;
    }

    bump_persistent(&env, &key);

    let mut total: i128 = 0;
    for contribution in contributions.iter() {
        total += contribution.amount;
    }
    total
}

/// Returns the number of unique contributors to a group's fundraising campaign.
pub fn get_contributor_count(env: Env, id: BytesN<32>) -> u32 {
    let key = DataKey::GroupContributions(id);
    let contributions: Vec<FundraisingContribution> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(&env));

    if contributions.is_empty() {
        return 0;
    }

    bump_persistent(&env, &key);

    // Count unique contributors
    let mut unique_contributors: Vec<Address> = Vec::new(&env);
    for contribution in contributions.iter() {
        let mut found = false;
        for existing in unique_contributors.iter() {
            if existing == contribution.contributor {
                found = true;
                break;
            }
        }
        if !found {
            unique_contributors.push_back(contribution.contributor.clone());
        }
    }
    unique_contributors.len()
}

/// Returns the remaining amount needed to reach the fundraising target.
/// Returns 0 if target is already reached or no fundraising exists.
pub fn get_fundraising_remaining(env: Env, id: BytesN<32>) -> i128 {
    let key = DataKey::GroupFundraising(id);
    let config: Option<FundraisingConfig> = env.storage().persistent().get(&key);

    if let Some(fundraising) = config {
        bump_persistent(&env, &key);
        let remaining = fundraising.target_amount - fundraising.total_raised;
        if remaining > 0 {
            remaining
        } else {
            0
        }
    } else {
        0
    }
}

pub fn reset_fundraising(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    // 1. Authorize caller
    caller.require_auth();

    // 2. Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // 3. Verify group existence and creator
    let key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    // 4. Check fundraising exists and is NOT active
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    let config: FundraisingConfig = env
        .storage()
        .persistent()
        .get(&fundraising_key)
        .ok_or(Error::FundraisingNotActive)?;
    bump_persistent(&env, &fundraising_key);

    if config.is_active {
        return Err(Error::FundraisingAlreadyActive);
    }

    // 5. Remove current fundraising configuration
    env.storage().persistent().remove(&fundraising_key);

    // 6. Clear contributions and stats for a fresh start
    let contributions_key = DataKey::GroupContributions(id.clone());
    if env.storage().persistent().has(&contributions_key) {
        env.storage().persistent().remove(&contributions_key);
    }

    let stats_key = DataKey::GroupStats(id.clone());
    if env.storage().persistent().has(&stats_key) {
        env.storage().persistent().remove(&stats_key);
    }

    // 7. Emit reset event
    Ok(())
}

pub fn set_fundraising_target(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_target: i128,
) -> Result<(), Error> {
    caller.require_auth();

    // Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // Verify creator
    let autoshare_key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&autoshare_key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &autoshare_key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    // Verify fundraising is active
    let config_key = DataKey::GroupFundraising(id.clone());
    let mut config: FundraisingConfig = env
        .storage()
        .persistent()
        .get(&config_key)
        .ok_or(Error::FundraisingNotActive)?;
    bump_persistent(&env, &config_key);

    if !config.is_active {
        return Err(Error::FundraisingNotActive);
    }

    // Validate new target
    if new_target <= 0 || new_target <= config.total_raised {
        return Err(Error::InvalidTarget);
    }

    let old_target = config.target_amount;
    config.target_amount = new_target;

    // Store back
    env.storage().persistent().set(&config_key, &config);
    bump_persistent(&env, &config_key);

    // Emit event
    emit_fundraising_target_updated(&env, id, old_target, new_target);

    Ok(())
}

pub fn get_groups_by_member_paginated(
    env: Env,
    member: Address,
    offset: u32,
    limit: u32,
) -> GroupPage {
    let member_groups_key = DataKey::MemberGroups(member);
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&member_groups_key)
        .unwrap_or(Vec::new(&env));

    let total = group_ids.len();
    let actual_limit = if limit > 20 { 20 } else { limit };

    let mut groups = Vec::new(&env);
    if offset < total {
        let end = if offset + actual_limit > total {
            total
        } else {
            offset + actual_limit
        };

        for i in offset..end {
            if let Some(id) = group_ids.get(i) {
                if let Ok(details) = get_autoshare(env.clone(), id.clone()) {
                    groups.push_back(details);
                }
            }
        }
    }

    GroupPage {
        groups,
        total,
        offset,
        limit: actual_limit,
    }
}

pub fn get_groups_by_status_paginated(
    env: Env,
    is_active: bool,
    offset: u32,
    limit: u32,
) -> GroupPage {
    let group_ids = get_all_group_ids(&env);

    // Cap limit at 20 as per requirement
    let actual_limit = limit.min(20);
    if actual_limit == 0 {
        return GroupPage {
            groups: Vec::new(&env),
            total: 0,
            offset,
            limit: actual_limit,
        };
    }

    let mut groups: Vec<AutoShareDetails> = Vec::new(&env);
    let mut total_matches = 0;
    let mut matches_returned = 0;

    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id) {
            if details.is_active == is_active {
                if total_matches >= offset && matches_returned < actual_limit {
                    groups.push_back(details);
                    matches_returned += 1;
                }
                total_matches += 1;
            }
        }
    }

    GroupPage {
        groups,
        total: total_matches,
        offset,
        limit: actual_limit,
    }
}

/// Returns a list of all active fundraising campaigns with their group IDs.
/// Reads AllGroups and checks GroupFundraising for each group.
pub fn get_active_fundraisings(env: Env) -> Vec<ActiveFundraising> {
    let group_ids = get_all_group_ids(&env);
    let mut result: Vec<ActiveFundraising> = Vec::new(&env);

    for id in group_ids.iter() {
        let fundraising_key = DataKey::GroupFundraising(id.clone());
        if let Some(config) = env
            .storage()
            .persistent()
            .get::<_, FundraisingConfig>(&fundraising_key)
        {
            if config.is_active {
                result.push_back(ActiveFundraising {
                    group_id: id.clone(),
                    config,
                });
            }
        }
    }

    result
}

/// Returns a list of all inactive (deactivated) groups.
/// Filters groups where is_active == false.
pub fn get_inactive_groups(env: Env) -> Vec<BytesN<32>> {
    let group_ids = get_all_group_ids(&env);
    let mut result: Vec<BytesN<32>> = Vec::new(&env);

    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id.clone()) {
            if !details.is_active {
                result.push_back(id);
            }
        }
    }

    result
}

/// Returns pre-aggregated statistics for a group.
/// Includes total_distributed, distribution_count, total_raised, and contribution_count.
pub fn get_group_stats(env: Env, id: BytesN<32>) -> GroupStats {
    let stats_key = DataKey::GroupStats(id);
    env.storage()
        .persistent()
        .get(&stats_key)
        .unwrap_or(GroupStats {
            total_distributed: 0,
            distribution_count: 0,
            total_raised: 0,
            contribution_count: 0,
        })
}

/// Returns the member count of a group without loading the full member list.
pub fn get_group_member_count(env: Env, id: BytesN<32>) -> Result<u32, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;
    Ok(details.members.len())
}

/// Cancels an active fundraising campaign.
/// Only the group creator can cancel. Does NOT refund contributions already distributed to members.
pub fn cancel_fundraising(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    // 1. Require caller authentication
    caller.require_auth();

    // 2. Check if contract is paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // 3. Verify group exists and caller is the creator
    let autoshare_key = DataKey::AutoShare(id.clone());
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&autoshare_key)
        .ok_or(Error::NotFound)?;
    bump_persistent(&env, &autoshare_key);

    if details.creator != caller {
        return Err(Error::Unauthorized);
    }

    // 4. Read fundraising config and verify it's active
    let fundraising_key = DataKey::GroupFundraising(id.clone());
    let mut fundraising_config: FundraisingConfig = env
        .storage()
        .persistent()
        .get(&fundraising_key)
        .ok_or(Error::FundraisingNotActive)?;
    bump_persistent(&env, &fundraising_key);

    if !fundraising_config.is_active {
        return Err(Error::FundraisingNotActive);
    }

    // 5. Set is_active to false
    let total_raised_at_cancellation = fundraising_config.total_raised;
    fundraising_config.is_active = false;

    // 6. Store updated config
    env.storage()
        .persistent()
        .set(&fundraising_key, &fundraising_config);
    bump_persistent(&env, &fundraising_key);

    // 7. Emit FundraisingCancelled event
    emit_fundraising_cancelled(&env, id.clone(), total_raised_at_cancellation);

    Ok(())
}

pub fn get_protocol_fee(env: Env) -> (u32, Address) {
    let fee = get_protocol_fee_val(&env);
    let recipient = get_protocol_recipient_val(&env);
    (fee, recipient)
}

pub fn set_protocol_fee(
    env: Env,
    fee: u32,
    recipient: Address,
    admin: Address,
) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    if fee > 10000 {
        return Err(Error::InvalidInput);
    }

    let old_fee = get_protocol_fee_val(&env);
    let old_recipient = get_protocol_recipient_val(&env);

    let fee_key = DataKey::ProtocolFee;
    let recipient_key = DataKey::ProtocolFeeRecipient;

    env.storage().persistent().set(&fee_key, &fee);
    env.storage().persistent().set(&recipient_key, &recipient);

    bump_persistent(&env, &fee_key);
    bump_persistent(&env, &recipient_key);

    crate::base::events::emit_protocol_fee_updated(
        &env,
        admin,
        old_fee,
        fee,
        old_recipient,
        recipient,
    );

    Ok(())
}

fn get_protocol_fee_val(env: &Env) -> u32 {
    let key = DataKey::ProtocolFee;
    let fee = env.storage().persistent().get(&key).unwrap_or(0u32);
    if env.storage().persistent().has(&key) {
        bump_persistent(env, &key);
    }
    fee
}

fn get_protocol_recipient_val(env: &Env) -> Address {
    let key = DataKey::ProtocolFeeRecipient;
    // Default to admin if not set
    let admin_key = DataKey::Admin;
    let admin = env
        .storage()
        .persistent()
        .get(&admin_key)
        .expect("admin must be set");

    let recipient = env.storage().persistent().get(&key).unwrap_or(admin);
    if env.storage().persistent().has(&key) {
        bump_persistent(env, &key);
    }
    recipient
}

// ============================================================================
// Issue #299: Deposit Funds Implementation
// ============================================================================

/// Deposits funds into a group's treasury for future distributions.
///
/// This function allows any user to deposit supported tokens into an active group's
/// treasury. The deposited amount is transferred from the depositor to the contract
/// and tracked for analytics and history purposes.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `id` - The unique identifier of the AutoShare group
/// * `token` - The token address being deposited
/// * `amount` - The amount to deposit (must be > 0)
/// * `depositor` - The address of the depositor (must authorize)
///
/// # Events
///
/// Emits `FundsDeposited` event with group_id, depositor, token, amount, and new treasury balance.
///
/// # Errors
///
/// Returns `InvalidAmount` if amount <= 0.
/// Returns `ContractPaused` if the contract is paused.
/// Returns `NotFound` if the group does not exist.
/// Returns `GroupInactive` if the group is not active.
/// Returns `UnsupportedToken` if the token is not in the supported list.
pub fn deposit_funds(
    env: Env,
    id: BytesN<32>,
    token: Address,
    amount: i128,
    depositor: Address,
) -> Result<(), Error> {
    // Validate amount
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    // Check contract is not paused
    if get_paused_status(&env) {
        return Err(Error::ContractPaused);
    }

    // Validate group exists and is active
    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    let group: AutoShareDetails = env.storage().persistent().get(&group_key).unwrap();
    if !group.is_active {
        return Err(Error::GroupInactive);
    }

    // Validate token is supported
    if !is_token_supported(env.clone(), token.clone()) {
        return Err(Error::UnsupportedToken);
    }

    // Require depositor authorization
    depositor.require_auth();

    // Transfer tokens from depositor to contract
    let token_client = token::Client::new(&env, &token);
    token_client.transfer(&depositor, env.current_contract_address(), &amount);

    // Update treasury balance
    let balance_key = DataKey::GroupTreasuryBalance(id.clone(), token.clone());
    let current_balance: i128 = env.storage().persistent().get(&balance_key).unwrap_or(0);
    let new_balance = current_balance + amount;
    env.storage().persistent().set(&balance_key, &new_balance);
    bump_persistent(&env, &balance_key);

    // Create deposit record
    let deposit_record = DepositRecord {
        group_id: id.clone(),
        depositor: depositor.clone(),
        token: token.clone(),
        amount,
        timestamp: env.ledger().timestamp(),
    };

    // Record in group history
    let group_history_key = DataKey::GroupDepositHistory(id.clone());
    let mut group_history: Vec<DepositRecord> = env
        .storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or_else(|| Vec::new(&env));
    group_history.push_back(deposit_record.clone());
    env.storage()
        .persistent()
        .set(&group_history_key, &group_history);
    bump_persistent(&env, &group_history_key);

    // Record in depositor history
    let depositor_history_key = DataKey::DepositorHistory(depositor.clone());
    let mut depositor_history: Vec<DepositRecord> = env
        .storage()
        .persistent()
        .get(&depositor_history_key)
        .unwrap_or_else(|| Vec::new(&env));
    depositor_history.push_back(deposit_record);
    env.storage()
        .persistent()
        .set(&depositor_history_key, &depositor_history);
    bump_persistent(&env, &depositor_history_key);

    // Emit FundsDeposited event
    emit_funds_deposited(&env, id, depositor, token, amount, new_balance);

    Ok(())
}

/// Returns the treasury balance for a specific group and token.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `id` - The unique identifier of the AutoShare group
/// * `token` - The token address to check balance for
///
/// # Returns
///
/// Returns the current treasury balance (0 if no deposits have been made).
pub fn get_group_treasury_balance(env: Env, id: BytesN<32>, token: Address) -> i128 {
    let key = DataKey::GroupTreasuryBalance(id, token);
    let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
    if env.storage().persistent().has(&key) {
        bump_persistent(&env, &key);
    }
    balance
}

/// Returns all deposit history records for a specific group.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `id` - The unique identifier of the AutoShare group
///
/// # Returns
///
/// Returns a vector of all deposit records for the group (empty if no deposits).
pub fn get_group_deposit_history(env: Env, id: BytesN<32>) -> Vec<DepositRecord> {
    let key = DataKey::GroupDepositHistory(id);
    let history: Vec<DepositRecord> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(&env));
    if env.storage().persistent().has(&key) {
        bump_persistent(&env, &key);
    }
    history
}

/// Returns all deposit history records for a specific depositor across all groups.
///
/// # Arguments
///
/// * `env` - The Soroban environment
/// * `depositor` - The address of the depositor
///
/// # Returns
///
/// Returns a vector of all deposit records by the depositor (empty if no deposits).
pub fn get_depositor_history(env: Env, depositor: Address) -> Vec<DepositRecord> {
    let key = DataKey::DepositorHistory(depositor);
    let history: Vec<DepositRecord> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(&env));
    if env.storage().persistent().has(&key) {
        bump_persistent(&env, &key);
    }
    history
}

pub fn set_group_protocol_fee(
    env: Env,
    admin: Address,
    id: BytesN<32>,
    percentage: u32,
) -> Result<(), Error> {
    admin.require_auth();
    require_admin(&env, &admin)?;

    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    if percentage > 100 {
        return Err(Error::InvalidAmount);
    }

    let old_fee = get_group_protocol_fee(env.clone(), id.clone());
    let key = DataKey::GroupProtocolFee(id.clone());
    env.storage().persistent().set(&key, &percentage);
    bump_persistent(&env, &key);

    emit_group_protocol_fee_updated(&env, id, old_fee, percentage);
    Ok(())
}

pub fn get_group_protocol_fee(env: Env, id: BytesN<32>) -> u32 {
    let key = DataKey::GroupProtocolFee(id);
    let fee: Option<u32> = env.storage().persistent().get(&key);
    if let Some(f) = fee {
        bump_persistent(&env, &key);
        f
    } else {
        get_protocol_fee_val(&env)
    }
}
