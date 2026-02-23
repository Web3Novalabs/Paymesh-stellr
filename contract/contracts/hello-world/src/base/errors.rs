use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)] // This is required for most Soroban errors
pub enum Error {
    InvalidInput = 1,
    AlreadyExists = 2,
    NotFound = 3,
    UnsupportedToken = 4,
    InsufficientPayment = 5,
    NoUsagesRemaining = 6,
    InvalidUsageCount = 7,
    Unauthorized = 8,
    InsufficientBalance = 9,
    InvalidAmount = 10,
    ContractPaused = 11,
    AlreadyPaused = 12,
    NotPaused = 13,
    NotAuthorized = 14,
    InvalidTotalPercentage = 15,
    EmptyMembers = 16,
    DuplicateMember = 17,
    GroupInactive = 18,
    GroupAlreadyActive = 19,
    GroupAlreadyInactive = 20,
    InsufficientContractBalance = 21,
    MemberNotFound = 22,
}
