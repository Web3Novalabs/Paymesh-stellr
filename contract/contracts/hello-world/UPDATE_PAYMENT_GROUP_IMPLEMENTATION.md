# Update Payment Group Feature - Implementation Details

## Overview

The `update_payment_group` function provides a consolidated entry point for managing the core settings of an AutoShare payment group. It replaces the need for separate granular calls when multiple settings need to be changed simultaneously, improving user experience and reducing transaction costs.

## Capabilities

The function allows updating the following fields:
1. **Group Name**: Updates the human-readable name of the group (subject to length and character validation).
2. **Metadata**: Updates the associated metadata string used for off-chain context or frontend display.
3. **Admin Rotation**: Transfers the "creator" role to a new address.

## Implementation Details

### Data Storage
The `AutoShareDetails` struct was enhanced with a `metadata` field. Updates are persisted in the `DataKey::AutoShare(id)` persistent storage entry.

### Security and Authorization
- **Caller Authentication**: The `caller.require_auth()` ensures that the transaction is signed by the specified address.
- **Creator Check**: Only the address currently stored as the `creator` for the group can perform updates.
- **Contract State**: Updates are blocked if the contract is in a `Paused` state.
- **Group Status**: Only `Active` groups can be updated. This prevents updates to deactivated or deleted groups.

### Validation
- **Name Validation**: The `is_valid_name` helper ensures the new name is between 1 and 60 characters and contains non-whitespace content.
- **Metadata**: No specific format validation is enforced on the metadata string, allowing for flexible off-chain usage (e.g., JSON, URI, or plain text).

## Events

The function emits a single, rich `AutoshareUpdated` event:

```rust
pub struct AutoshareUpdated {
    #[topic]
    pub id: BytesN<32>,          // Indexed: Group ID
    #[topic]
    pub updater: Address,        // Indexed: Who performed the update
    pub name_updated: bool,      // Flag: Was name changed?
    pub metadata_updated: bool,  // Flag: Was metadata changed?
    pub new_creator: Option<Address>, // Value: New owner if rotated
}
```

## Usage Example (Contract Interface)

```rust
let client = AutoShareContractClient::new(&env, &contract_id);

// Update name and metadata
client.update_payment_group(
    &group_id,
    &current_creator,
    &Some(String::from_str(&env, "New Name")),
    &Some(String::from_str(&env, "ipfs://hash")),
    &None
);

// Rotate ownership
client.update_payment_group(
    &group_id,
    &current_creator,
    &None,
    &None,
    &Some(new_creator_address)
);
```

## Error Codes

| Error | Rationale |
|---|---|
| `ContractPaused` | Operations suspended by admin. |
| `NotFound` | Specified group ID does not exist. |
| `Unauthorized` | Caller is not the current group creator. |
| `GroupInactive` | Group must be active to modify settings. |
| `EmptyName` | New name failed length or whitespace validation. |
