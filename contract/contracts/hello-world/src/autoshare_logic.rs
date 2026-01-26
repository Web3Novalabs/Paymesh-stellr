use crate::base::errors::Error;
use crate::base::events::{emit_autoshare_created, emit_contract_paused, emit_contract_unpaused};
use crate::base::types::{AutoShareDetails, GroupMember};
use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};

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
        .ok_or(Error::Unauthorized)?;

    if admin != stored_admin {
        return Err(Error::Unauthorized);
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
        .ok_or(Error::Unauthorized)?;

    if admin != stored_admin {
        return Err(Error::Unauthorized);
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

pub fn create_autoshare(
    env: Env,
    id: BytesN<32>,
    name: String,
    creator: Address,
) -> Result<(), Error> {
    require_not_paused(&env)?;

    let key = DataKey::AutoShare(id.clone());

    // Check if it already exists to prevent overwriting
    if env.storage().persistent().has(&key) {
        return Err(Error::AlreadyExists);
    }

    let details = AutoShareDetails {
        id: id.clone(),
        name,
        creator: creator.clone(),
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

    // Initialize empty members list
    let members_key = DataKey::GroupMembers(id.clone());
    let empty_members: Vec<GroupMember> = Vec::new(&env);
    env.storage().persistent().set(&members_key, &empty_members);

    // Emit the success event
    emit_autoshare_created(&env, id, creator);
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

    for member in members.iter() {
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

pub fn add_group_member(env: Env, id: BytesN<32>, address: Address) -> Result<(), Error> {
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

    members.push_back(GroupMember { address });
    env.storage().persistent().set(&members_key, &members);
    Ok(())
}
