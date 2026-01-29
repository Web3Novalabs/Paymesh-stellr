use crate::base::errors::Error;
use crate::base::events::{
    emit_admin_transferred, emit_autoshare_created, emit_autoshare_updated, emit_contract_paused,
    emit_contract_unpaused, emit_group_activated, emit_group_deactivated, emit_withdrawal,
};
use crate::base::types::{AutoShareDetails, GroupMember};
use soroban_sdk::{contracttype, token, Address, BytesN, Env, String, Vec};

#[contracttype]
pub enum DataKey {
    AutoShare(BytesN<32>),
    GroupMembers(BytesN<32>),
    AllGroups,
    Admin,
    IsPaused,
}

pub fn set_admin(env: Env, admin: Address) -> Result<(), Error> {
    if env.storage().persistent().has(&DataKey::Admin) {
        return Err(Error::AlreadyExists);
    }
    env.storage().persistent().set(&DataKey::Admin, &admin);
    Ok(())
}

pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();

    let stored_admin: Address = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .ok_or(Error::NotAuthorized)?;

    if admin != stored_admin {
        return Err(Error::NotAuthorized);
    }

    let is_paused: bool = env
        .storage()
        .persistent()
        .get(&DataKey::IsPaused)
        .unwrap_or(false);

    if is_paused {
        return Err(Error::AlreadyPaused);
    }

    env.storage().persistent().set(&DataKey::IsPaused, &true);
    emit_contract_paused(&env);
    Ok(())
}

pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();

    let stored_admin: Address = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .ok_or(Error::NotAuthorized)?;

    if admin != stored_admin {
        return Err(Error::NotAuthorized);
    }

    let is_paused: bool = env
        .storage()
        .persistent()
        .get(&DataKey::IsPaused)
        .unwrap_or(false);

    if !is_paused {
        return Err(Error::NotPaused);
    }

    env.storage().persistent().set(&DataKey::IsPaused, &false);
    emit_contract_unpaused(&env);
    Ok(())
}

pub fn get_paused_status(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

fn require_not_paused(env: &Env) -> Result<(), Error> {
    if get_paused_status(env) {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let stored_admin: Address = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .ok_or(Error::NotAuthorized)?;

    if caller != &stored_admin {
        return Err(Error::NotAuthorized);
    }
    Ok(())
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Admin)
        .ok_or(Error::NotFound)
}

pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) -> Result<(), Error> {
    current_admin.require_auth();
    require_admin(&env, &current_admin)?;

    env.storage().persistent().set(&DataKey::Admin, &new_admin);
    emit_admin_transferred(&env, current_admin, new_admin);
    Ok(())
}

// Helper to validate members
fn validate_members(members: &Vec<GroupMember>) -> Result<(), Error> {
    if members.is_empty() {
        return Err(Error::EmptyMembers);
    }

    let mut total_percentage: u32 = 0;
    for member in members.iter() {
        total_percentage += member.percentage;
    }

    if total_percentage != 100 {
        return Err(Error::InvalidTotalPercentage);
    }

    // Check for duplicates
    // O(N^2) is acceptable here as member lists are expected to be small
    for (i, member1) in members.iter().enumerate() {
        for (j, member2) in members.iter().enumerate() {
            if i != j && member1.address == member2.address {
                return Err(Error::DuplicateMember);
            }
        }
    }

    Ok(())
}

pub fn create_autoshare(
    env: Env,
    id: BytesN<32>,
    name: String,
    creator: Address,
    members: Vec<GroupMember>,
) -> Result<(), Error> {
    require_not_paused(&env)?;

    let key = DataKey::AutoShare(id.clone());

    // Check if it already exists to prevent overwriting
    if env.storage().persistent().has(&key) {
        return Err(Error::AlreadyExists);
    }

    // Validate members
    validate_members(&members)?;

    let details = AutoShareDetails {
        id: id.clone(),
        name,
        creator: creator.clone(),
        members,
        is_active: true,
    };

    // Store the details in persistent storage
    env.storage().persistent().set(&key, &details);

    // Add to all groups list
    let all_groups_key = DataKey::AllGroups;
    let mut all_groups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));
    all_groups.push_back(id.clone());
    env.storage().persistent().set(&all_groups_key, &all_groups);

    // Emit event
    emit_autoshare_created(&env, id, creator);
    Ok(())
}

pub fn update_members(
    env: Env,
    id: BytesN<32>,
    caller: Address,
    new_members: Vec<GroupMember>,
) -> Result<(), Error> {
    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    // Authorization check
    if caller != details.creator {
        return Err(Error::NotAuthorized);
    }

    // Check if group is active
    if !details.is_active {
        return Err(Error::GroupInactive);
    }

    // Validate new members
    validate_members(&new_members)?;

    // Update members
    details.members = new_members;
    env.storage().persistent().set(&key, &details);

    // Emit event
    emit_autoshare_updated(&env, id, caller);
    Ok(())
}

pub fn get_autoshare(env: Env, id: BytesN<32>) -> Result<AutoShareDetails, Error> {
    let key = DataKey::AutoShare(id);
    env.storage().persistent().get(&key).ok_or(Error::NotFound)
}

pub fn get_all_groups(env: Env) -> Vec<AutoShareDetails> {
    let all_groups_key = DataKey::AllGroups;
    let group_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&all_groups_key)
        .unwrap_or(Vec::new(&env));

    let mut result: Vec<AutoShareDetails> = Vec::new(&env);
    for id in group_ids.iter() {
        if let Ok(details) = get_autoshare(env.clone(), id) {
            result.push_back(details);
        }
    }
    result
}

pub fn get_groups_by_creator(env: Env, creator: Address) -> Vec<AutoShareDetails> {
    let all_groups = get_all_groups(env.clone());
    let mut result: Vec<AutoShareDetails> = Vec::new(&env);

    for group in all_groups.iter() {
        if group.creator == creator {
            result.push_back(group);
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

pub fn get_group_members(env: Env, id: BytesN<32>) -> Result<Vec<GroupMember>, Error> {
    // First check if the group exists
    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    let members_key = DataKey::GroupMembers(id);
    let members: Vec<GroupMember> = env
        .storage()
        .persistent()
        .get(&members_key)
        .unwrap_or(Vec::new(&env));

    Ok(members)
}

pub fn add_group_member(
    env: Env,
    id: BytesN<32>,
    address: Address,
    percentage: u32,
) -> Result<(), Error> {
    require_not_paused(&env)?;

    // First check if the group exists
    let group_key = DataKey::AutoShare(id.clone());
    if !env.storage().persistent().has(&group_key) {
        return Err(Error::NotFound);
    }

    let members_key = DataKey::GroupMembers(id);
    let mut members: Vec<GroupMember> = env
        .storage()
        .persistent()
        .get(&members_key)
        .unwrap_or(Vec::new(&env));

    // Check if already a member
    for member in members.iter() {
        if member.address == address {
            return Err(Error::AlreadyExists);
        }
    }

    members.push_back(GroupMember {
        address,
        percentage,
    });
    env.storage().persistent().set(&members_key, &members);
    Ok(())
}

/// Deactivates a group. Only the creator can deactivate.
pub fn deactivate_group(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    caller.require_auth();

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    // Authorization check - only creator can deactivate
    if caller != details.creator {
        return Err(Error::NotAuthorized);
    }

    // Check if already inactive
    if !details.is_active {
        return Err(Error::GroupAlreadyInactive);
    }

    // Set to inactive
    details.is_active = false;
    env.storage().persistent().set(&key, &details);

    // Emit event
    emit_group_deactivated(&env, id, caller);
    Ok(())
}

/// Activates a group. Only the creator can activate.
pub fn activate_group(env: Env, id: BytesN<32>, caller: Address) -> Result<(), Error> {
    caller.require_auth();

    let key = DataKey::AutoShare(id.clone());
    let mut details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    // Authorization check - only creator can activate
    if caller != details.creator {
        return Err(Error::NotAuthorized);
    }

    // Check if already active
    if details.is_active {
        return Err(Error::GroupAlreadyActive);
    }

    // Set to active
    details.is_active = true;
    env.storage().persistent().set(&key, &details);

    // Emit event
    emit_group_activated(&env, id, caller);
    Ok(())
}

/// Returns whether a group is active.
pub fn is_group_active(env: Env, id: BytesN<32>) -> Result<bool, Error> {
    let key = DataKey::AutoShare(id);
    let details: AutoShareDetails = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)?;

    Ok(details.is_active)
}

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

    emit_withdrawal(&env, token, amount, recipient);
    Ok(())
}
