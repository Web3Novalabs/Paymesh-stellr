use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutoShareDetails {
    pub id: BytesN<32>,
    pub name: String,
    pub creator: Address,
    pub usage_count: u32,
    pub total_usages_paid: u32,
    pub members: Vec<GroupMember>,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupMember {
    pub address: Address,
    pub percentage: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentHistory {
    pub user: Address,
    pub group_id: BytesN<32>,
    pub usages_purchased: u32,
    pub amount_paid: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberAmount {
    pub address: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributionHistory {
    pub group_id: BytesN<32>,
    pub sender: Address,
    pub total_amount: i128,
    pub token: Address,
    pub member_amounts: Vec<MemberAmount>,
    pub timestamp: u64,
    pub distribution_number: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributionRecord {
    pub group_id: BytesN<32>,
    pub amount: i128,
    pub token: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupPage {
    pub groups: Vec<AutoShareDetails>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundraisingConfig {
    pub target_amount: i128,
    pub total_raised: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundraisingContribution {
    pub group_id: BytesN<32>,
    pub contributor: Address,
    pub token: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupStats {
    pub total_raised: i128,
    pub contribution_count: u32,
}
