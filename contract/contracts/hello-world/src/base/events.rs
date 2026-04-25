use soroban_sdk::{contractevent, Address, BytesN, Env};

/// Emitted when funds are distributed to group members.
pub fn emit_distribution(
    env: &soroban_sdk::Env,
    group_id: &BytesN<32>,
    sender: &Address,
    token: &Address,
    amount: i128,
    member_count: u32,
) {
    Distribution {
        id: group_id.clone(),
        token: token.clone(),
        sender: sender.clone(),
        amount,
        member_count,
    }
    .publish(env);
}

/// Emitted when someone contributes to a fundraiser.
pub fn emit_contribution(
    env: &soroban_sdk::Env,
    group_id: &BytesN<32>,
    contributor: &Address,
    token: &Address,
    amount: i128,
) {
    Contribution {
        group_id: group_id.clone(),
        contributor: contributor.clone(),
        token: token.clone(),
        amount,
    }
    .publish(env);
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct AutoshareCreated {
    #[topic]
    pub creator: Address,
    pub id: BytesN<32>,
}

#[contractevent]
#[derive(Clone)]
pub struct ContractPaused {}

#[contractevent]
#[derive(Clone)]
pub struct ContractUnpaused {}

#[contractevent]
#[derive(Clone)]
pub struct AutoshareUpdated {
    #[topic]
    pub id: BytesN<32>,
    #[topic]
    pub updater: Address,
    pub name_updated: bool,
    pub metadata_updated: bool,
    pub new_creator: Option<Address>,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct GroupDeactivated {
    #[topic]
    pub creator: Address,
    pub id: BytesN<32>,
}

#[contractevent]
#[derive(Clone)]
pub struct PaymentGroupDeactivated {
    #[topic]
    pub id: BytesN<32>,
    #[topic]
    pub caller: Address,
    pub member_count: u32,
    pub timestamp: u64,
}

pub fn emit_payment_group_deactivated(
    env: &Env,
    id: BytesN<32>,
    caller: Address,
    member_count: u32,
) {
    PaymentGroupDeactivated {
        id,
        caller,
        member_count,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct GroupActivated {
    #[topic]
    pub creator: Address,
    pub id: BytesN<32>,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct GroupDeleted {
    #[topic]
    pub deleter: Address,
    pub id: BytesN<32>,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct AdminTransferred {
    #[topic]
    pub old_admin: Address,
    pub new_admin: Address,
}

#[contractevent]
#[derive(Clone)]
pub struct GroupOwnershipTransferred {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub old_creator: Address,
    #[topic]
    pub new_creator: Address,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct Withdrawal {
    #[topic]
    pub token: Address,
    #[topic]
    pub recipient: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct Distribution {
    #[topic]
    pub id: BytesN<32>,
    #[topic]
    pub token: Address,
    #[topic]
    pub sender: Address,
    pub amount: i128,
    pub member_count: u32,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct GroupNameUpdated {
    #[topic]
    pub updater: Address,
    pub id: BytesN<32>,
}
#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct MemberAdded {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub member: Address,
    pub percentage: u32,
}

pub fn emit_member_added(env: &Env, group_id: BytesN<32>, member: Address, percentage: u32) {
    MemberAdded {
        group_id,
        member,
        percentage,
    }
    .publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct MemberRemoved {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub member: Address,
    /// The percentage share held by the member at the time of removal.
    pub removed_percentage: u32,
    /// Cumulative earnings accrued by the member in this group at the time of removal.
    pub pending_earnings: i128,
}

pub fn emit_member_removed(
    env: &Env,
    group_id: BytesN<32>,
    member: Address,
    removed_percentage: u32,
    pending_earnings: i128,
) {
    MemberRemoved {
        group_id,
        member,
        removed_percentage,
        pending_earnings,
    }
    .publish(env);
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct FundraisingStarted {
    #[topic]
    pub group_id: BytesN<32>,
    pub target_amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct FundraisingTargetUpdated {
    #[topic]
    pub group_id: BytesN<32>,
    pub old_target: i128,
    pub new_target: i128,
}

pub fn emit_fundraising_target_updated(
    env: &Env,
    group_id: BytesN<32>,
    old_target: i128,
    new_target: i128,
) {
    FundraisingTargetUpdated {
        group_id,
        old_target,
        new_target,
    }
    .publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct Contribution {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub contributor: Address,
    #[topic]
    pub token: Address,
    pub amount: i128,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct CreatorIsMember {
    #[topic]
    pub id: BytesN<32>,
}

pub fn emit_creator_is_member(env: &Env, id: BytesN<32>) {
    CreatorIsMember { id }.publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct TokenAdded {
    #[topic]
    pub admin: Address,
    #[topic]
    pub token: Address,
}

pub fn emit_token_added(env: &Env, admin: Address, token: Address) {
    TokenAdded { admin, token }.publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct TokenRemoved {
    #[topic]
    pub admin: Address,
    #[topic]
    pub token: Address,
}

pub fn emit_token_removed(env: &Env, admin: Address, token: Address) {
    TokenRemoved { admin, token }.publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct FundraisingCompleted {
    #[topic]
    pub group_id: BytesN<32>,
    pub target_amount: i128,
    pub total_raised: i128,
    pub contribution_count: u32,
}

pub fn emit_fundraising_completed(
    env: &Env,
    group_id: BytesN<32>,
    target_amount: i128,
    total_raised: i128,
    contribution_count: u32,
) {
    FundraisingCompleted {
        group_id,
        target_amount,
        total_raised,
        contribution_count,
    }
    .publish(env);
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct FundraisingReset {
    #[topic]
    pub id: BytesN<32>,
}

pub fn emit_fundraising_reset(env: &Env, id: BytesN<32>) {
    FundraisingReset { id }.publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct MaxMembersUpdated {
    pub old_max: u32,
    pub new_max: u32,
}

pub fn emit_max_members_updated(env: &Env, old_max: u32, new_max: u32) {
    MaxMembersUpdated { old_max, new_max }.publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct UsageFeeUpdated {
    #[topic]
    pub admin: Address,
    pub old_fee: u32,
    pub new_fee: u32,
}

pub fn emit_usage_fee_updated(env: &Env, admin: Address, old_fee: u32, new_fee: u32) {
    UsageFeeUpdated {
        admin,
        old_fee,
        new_fee,
    }
    .publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct FundraisingCancelled {
    #[topic]
    pub group_id: BytesN<32>,
    pub total_raised: i128,
}

pub fn emit_fundraising_cancelled(env: &Env, group_id: BytesN<32>, total_raised: i128) {
    FundraisingCancelled {
        group_id,
        total_raised,
    }
    .publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct MemberAddedToGroup {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub member: Address,
    #[topic]
    pub caller: Address,
    pub percentage: u32,
    pub new_member_count: u32,
    pub timestamp: u64,
}

pub fn emit_member_added_to_group(
    env: &Env,
    group_id: BytesN<32>,
    member: Address,
    caller: Address,
    percentage: u32,
    new_member_count: u32,
) {
    MemberAddedToGroup {
        group_id,
        member,
        caller,
        percentage,
        new_member_count,
        timestamp: env.ledger().timestamp(),
    }
    .publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct FundsDeposited {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub depositor: Address,
    #[topic]
    pub token: Address,
    pub amount: i128,
    pub new_treasury_balance: i128,
}

pub fn emit_funds_deposited(
    env: &Env,
    group_id: BytesN<32>,
    depositor: Address,
    token: Address,
    amount: i128,
    new_treasury_balance: i128,
) {
    FundsDeposited {
        group_id,
        depositor,
        token,
        amount,
        new_treasury_balance,
    }
    .publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct ProtocolFeeUpdated {
    #[topic]
    pub admin: Address,
    pub old_fee: u32,
    pub new_fee: u32,
    pub old_recipient: Address,
    pub new_recipient: Address,
}

pub fn emit_protocol_fee_updated(
    env: &Env,
    admin: Address,
    old_fee: u32,
    new_fee: u32,
    old_recipient: Address,
    new_recipient: Address,
) {
    ProtocolFeeUpdated {
        admin,
        old_fee,
        new_fee,
        old_recipient,
        new_recipient,
    }
    .publish(env);
}

/// Emitted every time `get_protocol_fee` is invoked for analytics.
#[contractevent]
#[derive(Clone)]
pub struct ProtocolFeeRead {
    pub fee: u32,
    pub recipient: Address,
}

pub fn emit_protocol_fee_read(env: &Env, fee: u32, recipient: Address) {
    ProtocolFeeRead { fee, recipient }.publish(env);
}

#[contractevent]
#[derive(Clone)]
pub struct GroupProtocolFeeUpdated {
    #[topic]
    pub group_id: BytesN<32>,
    pub old_fee: u32,
    pub new_fee: u32,
}

pub fn emit_group_protocol_fee_updated(
    env: &Env,
    group_id: BytesN<32>,
    old_fee: u32,
    new_fee: u32,
) {
    GroupProtocolFeeUpdated {
        group_id,
        old_fee,
        new_fee,
    }
    .publish(env);
}

/// Emitted when get_group_members is queried for analytics tracking.
#[contractevent]
#[derive(Clone)]
pub struct GroupMembersQueried {
    #[topic]
    pub group_id: BytesN<32>,
    pub member_count: u32,
    pub query_count: u64,
}

pub fn emit_group_members_queried(
    env: &Env,
    group_id: BytesN<32>,
    member_count: u32,
    query_count: u64,
) {
    GroupMembersQueried {
        group_id,
        member_count,
        query_count,
    }
    .publish(env);
}
