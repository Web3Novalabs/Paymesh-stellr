# Group Name Validation Tests - Implementation Summary

## Overview
This document describes the comprehensive test suite for group name validation in the Paymesh Stellar smart contract, addressing issue #168.

## Test File Location
`contract/contracts/hello-world/src/tests/group_name_validation_test.rs`

## Tests Implemented

### 1. **test_name_at_minimum_length**
- Tests that a 1-character name is accepted
- Validates minimum boundary condition
- Expected: Success

### 2. **test_name_at_maximum_length**
- Tests that a 60-character name is accepted
- Validates maximum boundary condition
- Expected: Success

### 3. **test_name_exceeding_max_length**
- Tests that a 61-character name is rejected
- Validates upper boundary enforcement
- Expected: Error (EmptyName)

### 4. **test_empty_name_returns_error**
- Tests that an empty string is rejected
- Expected: Error (EmptyName)

### 5. **test_whitespace_only_name_returns_error**
- Tests that whitespace-only strings are rejected
- Validates trimming behavior
- Expected: Error (EmptyName)

### 6. **test_name_with_leading_trailing_spaces**
- Tests behavior with leading/trailing spaces
- Documents whether spaces are preserved or trimmed
- Expected: Success (spaces are preserved)

### 7. **test_name_with_special_characters**
- Tests names with special characters (@, #, !)
- Validates character set acceptance
- Expected: Success

### 8. **test_name_with_only_numbers**
- Tests numeric-only names
- Expected: Success

### 9. **test_update_group_name_with_same_name**
- Tests updating a group with the same name
- Validates idempotent behavior
- Expected: Success

### 10. **test_update_group_name_on_deactivated_group**
- Tests updating name on deactivated group
- Expected: Error (GroupInactive)

### 11. **test_name_with_newlines**
- Tests names containing newline characters
- Expected: Success (newlines are allowed in current implementation)

### 12. **test_name_with_tabs**
- Tests names containing tab characters
- Expected: Success (tabs are allowed in current implementation)

## Additional Update Tests

### 13. **test_update_name_at_minimum_length**
- Tests updating to 1-character name
- Expected: Success

### 14. **test_update_name_at_maximum_length**
- Tests updating to 60-character name
- Expected: Success

### 15. **test_update_name_exceeding_max_length**
- Tests updating to 61-character name
- Expected: Error (EmptyName)

### 16. **test_update_name_empty**
- Tests updating to empty string
- Expected: Error (EmptyName)

### 17. **test_update_name_whitespace_only**
- Tests updating to whitespace-only string
- Expected: Error (EmptyName)

## Validation Logic

The validation is performed by the `is_valid_name` function in `autoshare_logic.rs`:

```rust
fn is_valid_name(name: &String) -> bool {
    let alloc_str: AllocString = name.to_string();
    let trimmed = alloc_str.trim();
    if trimmed.is_empty() {
        return false;
    }
    if alloc_str.len() > 60 {
        return false;
    }
    true
}
```

### Key Behaviors:
1. **Length Check**: Names must be 1-60 characters
2. **Trimming**: Names are trimmed for emptiness check but original is stored
3. **Empty Check**: After trimming, name must not be empty
4. **Character Set**: All characters are allowed (including special chars, numbers, spaces)

## Functions Tested

### create_autoshare
- Validates name during group creation
- Returns `Error::EmptyName` for invalid names

### update_group_name
- Validates name during group name updates
- Requires group to be active
- Returns `Error::EmptyName` for invalid names
- Returns `Error::GroupInactive` if group is deactivated

## Running the Tests

To run these tests:

```bash
cd contract/contracts/hello-world
cargo test group_name_validation_test
```

Or run all tests:

```bash
cargo test
```

## Notes

1. **Leading/Trailing Spaces**: The current implementation preserves leading and trailing spaces in the stored name, but checks if the trimmed version is empty.

2. **Special Characters**: All special characters, including newlines and tabs, are currently allowed as long as the trimmed name is not empty and length is within bounds.

3. **Deactivated Groups**: Name updates are not allowed on deactivated groups, which is the expected behavior for maintaining data integrity.

4. **Same Name Updates**: Updating a group with the same name is allowed and works correctly.

## Integration

The test module has been registered in `src/lib.rs`:

```rust
#[cfg(test)]
#[path = "tests/group_name_validation_test.rs"]
mod group_name_validation_test;
```

## Dependencies

The tests use:
- `setup_test_env()` - Sets up test environment with admin, users, and tokens
- `create_test_group()` - Helper to create test groups
- `AutoShareContractClient` - Contract client for testing
- Soroban SDK test utilities
