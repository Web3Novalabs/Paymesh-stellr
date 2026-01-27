use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)] // This is required for Soroban errors
pub enum Error {
    InvalidInput = 1,
    AlreadyExists = 2,
    NotFound = 3,
    ContractPaused = 4,
    AlreadyPaused = 5,
    NotPaused = 6,
    Unauthorized = 7,
    InsufficientBalance = 8,
    InvalidAmount = 9,
}
