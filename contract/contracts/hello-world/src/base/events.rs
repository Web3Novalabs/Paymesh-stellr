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
    }.publish(env);
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
    }.publish(env);
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

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct AutoshareUpdated {
    #[topic]
    pub updater: Address,
    pub id: BytesN<32>,
}

#[contractevent(data_format = "single-value")]
#[derive(Clone)]
pub struct GroupDeactivated {
    #[topic]
    pub creator: Address,
    pub id: BytesN<32>,
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
    }.publish(env);
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
pub struct Contribution {
    #[topic]
    pub group_id: BytesN<32>,
    #[topic]
    pub contributor: Address,
    #[topic]
    pub token: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct CreatorIsMember {
    #[topic]
    pub id: BytesN<32>,
}

pub fn emit_creator_is_member(env: &Env, id: BytesN<32>) {
    env.events().publish(("CreatorIsMember",), CreatorIsMember { id });
}
