use crate::base::errors::Error;
use crate::base::events::{
    AdminTransferred, AutoshareCreated, AutoshareUpdated, ContractPaused, ContractUnpaused,
    Distribution, FundraisingStarted, GroupActivated, GroupDeactivated, GroupDeleted,
    GroupNameUpdated, Withdrawal,
};
use crate::base::types::{
    AutoShareDetails, DistributionHistory, FundraisingConfig, FundraisingContribution, GroupMember,
    GroupStats, MemberAmount, PaymentHistory,
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
    MemberDistributionHistory(Address),
    MemberGroupEarnings(Address, BytesN<32>),
    GroupFundraising(BytesN<32>),
    GroupContributions(BytesN<32>),
    UserContributions(Address),
    GroupStats(BytesN<32>),
    IsPaused,
}

const DAY_IN_LEDGERS: u32 = 17280;
const PERSISTENT_BUMP_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS; // 1 week
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS; // 30 days

fn bump_persistent<K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, key: &K) {
    if env.storage().persistent().has(key) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_BUMP_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
}
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

pub fn is_group_member(env: Env, id: BytesN<32>, address: Address) -> Result<bool, Error> {
    let details = get_autoshare(env, id)?;

    for member in details.members.iter() {
        if member.address == address {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn get_group_members(env: Env, id: BytesN<32>) -> Result<Vec<GroupMember>, Error> {
    let details = get_autoshare(env, id)?;
    Ok(details.members)
}

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
    let mut new_members: Vec<GroupMember> = Vec::new(&env);
    for member in details.members.iter() {
        if member.address == member_address {
            found = true;
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

    AutoshareUpdated {
        id: id.clone(),
        updater: caller,
    }
    .publish(&env);
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

    tokens.push_back(token);
    env.storage().persistent().set(&tokens_key, &tokens);
    bump_persistent(&env, &tokens_key);
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

    let fee_key = DataKey::UsageFee;
    env.storage().persistent().set(&fee_key, &fee);
    bump_persistent(&env, &fee_key);
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
    let group_history_key = DataKey::GroupDistributionHistory(group_id);
    let mut group_history: Vec<DistributionHistory> = env
        .storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env));
    group_history.push_back(distribution.clone());
    env.storage()
        .persistent()
        .set(&group_history_key, &group_history);

    // Add to each member's distribution history
    for member_amount in member_amounts.iter() {
        let member_history_key = DataKey::MemberDistributionHistory(member_amount.address.clone());
        let mut member_history: Vec<DistributionHistory> = env
            .storage()
            .persistent()
            .get(&member_history_key)
            .unwrap_or(Vec::new(&env));
        member_history.push_back(distribution.clone());
        env.storage()
            .persistent()
            .set(&member_history_key, &member_history);
    }
}

pub fn get_group_distributions(env: Env, id: BytesN<32>) -> Vec<DistributionHistory> {
    let group_history_key = DataKey::GroupDistributionHistory(id);
    env.storage()
        .persistent()
        .get(&group_history_key)
        .unwrap_or(Vec::new(&env))
}

pub fn get_member_distributions(env: Env, member: Address) -> Vec<DistributionHistory> {
    let member_history_key = DataKey::MemberDistributionHistory(member);
    env.storage()
        .persistent()
        .get(&member_history_key)
        .unwrap_or(Vec::new(&env))
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

#[cfg(test)]
#[allow(dead_code)]
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
    }

    if total_percentage != 100 {
        return Err(Error::InvalidTotalPercentage);
    }

    // Update members in details
    details.members = new_members.clone();
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    AutoshareUpdated {
        id: id.clone(),
        updater: caller,
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

    if new_name.is_empty() {
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

    // Step 4: Check group has 0 remaining usages (or warn about forfeiture)
    // We allow deletion even with remaining usages, but this is a design choice
    // In production, you might want to enforce zero usages or handle refunds
    if details.usage_count > 0 {
        // Option 1: Strict enforcement - uncomment to require zero usages
        // return Err(Error::GroupHasRemainingUsages);

        // Option 2: Allow deletion with forfeiture (current implementation)
        // The remaining usages are simply forfeited
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

    // Step 9: Emit deletion event
    GroupDeleted {
        deleter: caller,
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
        member_amounts,
        distribution_number,
    );

    details.usage_count -= 1;
    env.storage().persistent().set(&key, &details);
    bump_persistent(&env, &key);

    Distribution {
        id,
        token,
        sender,
        amount,
    }
    .publish(&env);

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
            (amount * (member.percentage as i128)) / 100
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
    perform_distribution(&env, &id, &token, amount, &group_details.members);

    // Update fundraising total
    fundraising_config.total_raised += amount;
    if fundraising_config.total_raised >= fundraising_config.target_amount {
        fundraising_config.is_active = false;
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
            total_raised: 0,
            contribution_count: 0,
        });
    stats.total_raised += amount;
    stats.contribution_count += 1;
    env.storage().persistent().set(&stats_key, &stats);
    bump_persistent(&env, &stats_key);

    // Emit event
    crate::base::events::Contribution {
        group_id: id,
        contributor,
        token,
        amount,
    }
    .publish(&env);

    Ok(())
}
