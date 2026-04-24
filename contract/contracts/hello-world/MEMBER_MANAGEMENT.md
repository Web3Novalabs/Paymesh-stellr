# Member Management

This document describes how group members are managed and retrieved in the Paymesh AutoShare contract.

## Data Structures

### GroupMember
Represents a single member of a group with their distribution share.
```rust
pub struct GroupMember {
    pub address: Address,
    pub percentage: u32,
}
```

### MemberPage
A paginated container for group members.
```rust
pub struct MemberPage {
    pub members: Vec<GroupMember>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}
```

## Functions

### `get_group_members`
Retrieves the full list of members for a specific group.

- **Arguments**:
    - `id: BytesN<32>`: The unique identifier of the group.
- **Returns**: `Vec<GroupMember>`: A list of all members and their percentages.
- **Panics**: If the group ID does not exist (`Error::NotFound`).

### `get_group_members_paginated`
Retrieves a paginated list of members for a specific group. This is recommended for groups with a large number of members to avoid hitting resource limits.

- **Arguments**:
    - `id: BytesN<32>`: The unique identifier of the group.
    - `offset: u32`: The starting index (0-based).
    - `limit: u32`: The maximum number of members to return.
- **Returns**: `MemberPage`: A page of members with pagination metadata.
- **Behavior**:
    - The `limit` is automatically capped at **20** to ensure optimal performance.
    - If `offset` is greater than or equal to the total number of members, an empty list is returned.
- **Panics**: If the group ID does not exist (`Error::NotFound`).

### `get_group_member_count`
Returns the total number of members in a group without loading the entire member list into memory.

- **Arguments**:
    - `id: BytesN<32>`: The unique identifier of the group.
- **Returns**: `u32`: The total number of members.

## Usage in Frontend
When displaying a list of members in the UI, it is recommended to use `get_group_members_paginated` to handle groups efficiently, especially when the member count grows.

```typescript
// Example frontend call (pseudocode)
const page = await client.get_group_members_paginated({
  id: group_id,
  offset: 0,
  limit: 10
});

console.log(`Showing ${page.members.length} of ${page.total} members`);
```

## Events
The following events are related to member management (emitted by other functions):

- `MemberAdded`: Emitted when a new member is added to a group.
- `AutoshareUpdated`: Emitted when the member list is updated via `update_members`.
