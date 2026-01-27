use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)] // This is required for most Soroban errors
pub enum Error {
    InvalidInput = 1,
    AlreadyExists = 2,
    NotFound = 3,
    ContractPaused = 4,
    AlreadyPaused = 5,
    NotPaused = 6,
    NotAuthorized = 7,
    InvalidTotalPercentage = 8,
    EmptyMembers = 9,
    DuplicateMember = 10,
    InsufficientBalance = 11,
    InvalidAmount = 12,
}
