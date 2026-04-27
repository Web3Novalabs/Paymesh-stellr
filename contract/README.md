# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:
```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

## `create` — Create a Payment Group

Creates a new payment group with a designated admin (creator), a pre-purchased distribution
quota, and an initial empty member list.

### Entry point

```rust
pub fn create(
    env: Env,
    id: BytesN<32>,
    name: String,
    creator: Address,
    usage_count: u32,
    payment_token: Address,
)
```

### Arguments

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `id` | `BytesN<32>` | Unique group identifier — must not already exist |
| `name` | `String` | Human-readable name (1–60 non-whitespace characters) |
| `creator` | `Address` | Group owner; must authorize the call |
| `usage_count` | `u32` | Number of distributions to pre-purchase (≥ 1) |
| `payment_token` | `Address` | Token used to pay the creation fee; must be supported |

### How it works

1. `creator.require_auth()` — enforces Soroban authorization.
2. Validates the contract is not paused, the name is non-empty, the `id` is unused, `usage_count ≥ 1`, and `payment_token` is on the supported-token list.
3. Calculates `total_cost = usage_count × usage_fee` and transfers that amount from `creator` to the contract.
4. Stores an `AutoShareDetails` record (active, empty member list) in persistent ledger storage and appends the `id` to the global group index.
5. Records a `PaymentHistory` entry for the creator.
6. Emits `AutoshareCreated { creator, id }`.

### Return value

`()` — no return value. Panics (via `.unwrap()`) on any validation error or failed token transfer.

### Emitted events

| Event | Fields | When |
|---|---|---|
| `AutoshareCreated` | `creator`, `id` | Always on success |

### Error conditions

| Error | Condition |
|---|---|
| `ContractPaused` | Contract is paused |
| `EmptyName` | Name is empty, whitespace-only, or > 60 characters |
| `AlreadyExists` | A group with the given `id` already exists |
| `InvalidUsageCount` | `usage_count` is 0 |
| `UnsupportedToken` | `payment_token` is not on the supported-token list |

> **Panics** if the token transfer fails (e.g. insufficient creator balance or allowance).

### Post-creation workflow

After creation the group has no members. Use `add_group_member` or `batch_add_members` to add
members with percentage splits that sum to 100 before calling `distribute`.

---

---

## `get_group_members` — Fetch All Group Members

Retrieves the complete list of all current members in a specific payment group along with their
percentage allocations. This function also tracks read frequency for analytics purposes.

### Entry point

```rust
pub fn get_group_members(
    env: Env,
    id: BytesN<32>,
) -> Vec<GroupMember>
```

### Arguments

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `id` | `BytesN<32>` | Unique group identifier |

### How it works

1. Retrieves the `AutoShareDetails` for the specified group from persistent storage.
2. Extracts the complete member list from the group details.
3. Increments a per-group query counter stored at `DataKey::GroupMembersQueryCount(id)`.
4. Bumps the TTL of the query counter to prevent expiration.
5. Emits `GroupMembersQueried` event with the group ID, member count, and cumulative query count.
6. Returns the complete vector of `GroupMember` structs.

### Return value

`Vec<GroupMember>` — A vector containing all members with their addresses and percentage allocations.
Members are returned in the order they were added to the group.

### Emitted events

| Event | Fields | When |
|---|---|---|
| `GroupMembersQueried` | `group_id` (topic), `member_count`, `query_count` | Always on success |

### Error conditions

| Error | Condition |
|---|---|
| `NotFound` | No group exists with the given `id` |

> **Panics** if the group does not exist (via `.unwrap()` in the contract entry point).

### Performance considerations

- For groups with more than 20 members, consider using `get_group_members_paginated` to reduce
  data transfer and improve response times.
- Each call increments a persistent counter, incurring a small storage write cost.
- The emitted event enables off-chain indexers to track read patterns without additional RPC calls.

### Off-chain analytics

The `GroupMembersQueried` event allows off-chain systems to:
- Monitor API usage patterns and identify frequently accessed groups
- Track read frequency for performance optimization
- Build analytics dashboards showing group access metrics
- Detect unusual access patterns for security monitoring

Use `get_group_members_query_count` to retrieve the current query count without incrementing it.

---

## `get_group_members_paginated` — Fetch Group Members with Pagination

Retrieves a paginated subset of all current members in a specific payment group. This function
provides efficient access to large member lists with automatic page size capping and diagnostic
event emission for off-chain analytics.

### Entry point

```rust
pub fn get_group_members_paginated(
    env: Env,
    id: BytesN<32>,
    offset: u32,
    limit: u32,
) -> MemberPage
```

### Arguments

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `id` | `BytesN<32>` | Unique group identifier |
| `offset` | `u32` | Zero-based starting index for pagination (0 = first member) |
| `limit` | `u32` | Maximum number of members to return (automatically capped at 20) |

### Input constraints

- `offset` — Can be any `u32` value; if it exceeds the total member count, an empty page is returned
- `limit` — Automatically capped at `MAX_PAGE_SIZE` (20) to prevent excessive data transfer
- `id` — Must reference an existing group

### How it works

1. Retrieves the `AutoShareDetails` for the specified group from persistent storage.
2. Extracts the complete member list and calculates the total count.
3. Caps the `limit` parameter at `MAX_PAGE_SIZE` (20) to ensure consistent performance.
4. Calculates the start and end indices for the requested page, handling edge cases:
   - If `offset >= total`, returns an empty page
   - If `offset + limit > total`, returns only the remaining members
5. Extracts the requested page of members from the full list.
6. Emits `GroupMembersPaginatedQueried` event with pagination details.
7. Returns a `MemberPage` struct containing the page data and metadata.

### Return value

`MemberPage` — A struct containing:
- `members` — `Vec<GroupMember>` with the requested page of members (may be empty)
- `total` — Total number of members in the group (always reflects current count)
- `offset` — The offset value used for this query (echoed from input)
- `limit` — The effective limit applied (may be less than requested if capped)

### Pagination behavior

- **Page Size Capping**: The `limit` parameter is automatically capped at 20 members per page
  to ensure consistent performance and prevent excessive data transfer.
- **Offset Handling**: If `offset >= total`, an empty `members` vector is returned with `total`
  still reflecting the actual member count.
- **Partial Pages**: The last page may contain fewer members than `limit` if fewer remain.
- **Zero Limit**: If `limit` is 0, an empty page is returned (useful for checking total count only).

### Ordering guarantees

Members are returned in the order they were added to the group. Pagination preserves this ordering
across all pages. The ordering remains stable unless the member list is explicitly updated via
`update_members` or `batch_add_members`.

### Emitted events

| Event | Fields | When |
|---|---|---|
| `GroupMembersPaginatedQueried` | `group_id` (topic), `offset`, `limit`, `total_members`, `returned_count` | Always on success |

### Error conditions

| Error | Condition |
|---|---|
| `NotFound` | No group exists with the given `id` |

> **Panics** if the group does not exist (via `.unwrap()` in the contract entry point).

### Storage effects

- Reads the `AutoShareDetails` from persistent storage using `DataKey::AutoShare(id)`
- Does NOT increment any query counter (unlike `get_group_members`)
- No TTL bumps or additional storage writes beyond the read operation

### Performance considerations

- **Recommended for Large Groups**: Use this function instead of `get_group_members` for groups
  with more than 20 members to reduce data transfer and improve response times.
- **Consistent Performance**: The 20-member page size cap ensures predictable gas costs and
  response times regardless of the requested `limit`.
- **Read-Only Operation**: This function performs only storage reads; no writes or counter
  increments occur, making it suitable for high-frequency polling.
- **Event Overhead**: Each call emits a diagnostic event, which incurs minimal gas cost but
  provides valuable off-chain analytics data.

### Off-chain analytics

The `GroupMembersPaginatedQueried` event enables off-chain systems to:
- Monitor pagination patterns and identify optimal page sizes for different use cases
- Track API usage and detect performance bottlenecks
- Build analytics dashboards showing access patterns and popular groups
- Optimize frontend pagination strategies based on actual usage data
- Detect unusual access patterns for security monitoring

### Consistency considerations

- **Concurrent Modifications**: If the member list is updated between paginated queries,
  subsequent pages may reflect the new state, potentially causing inconsistencies.
- **Stable Reads**: For consistent multi-page reads, ensure no concurrent `update_members`
  calls occur during the pagination sequence.
- **Total Count**: The `total` field always reflects the current member count at query time,
  which may differ across pages if concurrent updates occur.

### Example usage

```rust
// Fetch the first page of members (up to 20)
let page1 = client.get_group_members_paginated(&group_id, &0, &20);
println!("Total members: {}", page1.total);
println!("Returned: {}", page1.members.len());

// Fetch the second page
let page2 = client.get_group_members_paginated(&group_id, &20, &20);

// Fetch all members across multiple pages
let mut all_members = Vec::new(&env);
let mut offset = 0u32;
loop {
    let page = client.get_group_members_paginated(&group_id, &offset, &20);
    if page.members.is_empty() {
        break;
    }
    for member in page.members.iter() {
        all_members.push_back(member);
    }
    offset += page.members.len();
}

// Check total count without fetching members
let page = client.get_group_members_paginated(&group_id, &0, &0);
let total_count = page.total;
```

---

## `get_group_members_query_count` — Retrieve Query Frequency Metrics

Retrieves the cumulative number of times `get_group_members` has been called for a specific group.
This function provides read-frequency metrics for off-chain analytics without incrementing the counter.

### Entry point

```rust
pub fn get_group_members_query_count(
    env: Env,
    id: BytesN<32>,
) -> u64
```

### Arguments

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `id` | `BytesN<32>` | Unique group identifier |

### How it works

1. Reads the query counter from persistent storage at `DataKey::GroupMembersQueryCount(id)`.
2. If the counter is greater than 0, bumps its TTL to prevent expiration.
3. Returns the counter value (or 0 if the key does not exist).

### Return value

`u64` — The cumulative query count. Returns `0` if:
- The group exists but `get_group_members` has never been called for it
- The counter storage key does not exist

### Storage effects

- Reads the query counter from persistent storage
- If the counter is greater than 0, bumps its TTL to prevent expiration
- Does NOT increment the counter (read-only operation)

### Emitted events

This function does not emit any events.

### Performance considerations

- This is a lightweight read-only operation with minimal gas cost
- Safe to call frequently for monitoring purposes
- Does not validate that the group exists; returns 0 for non-existent groups

### Example usage

```rust
// Check how many times the group members have been queried
let query_count = client.get_group_members_query_count(&group_id);
println!("Group has been queried {} times", query_count);

// Use for analytics or rate limiting
if query_count > 1000 {
    // High-traffic group - consider caching strategies
}
```

---

## Payment Flow Events

The contract emits the following events for fund flow tracking:

- `emit_distribution(env, group_id, sender, token, total_amount, member_count)`: Emitted when funds are split and sent to group members.
- `emit_contribution(env, group_id, contributor, token, amount)`: Emitted when someone contributes to a fundraiser.

These events are essential for the frontend transaction history page and analytics dashboard to display real-time payment activity.

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

---

## Protocol Configuration

This section covers the two functions that control how protocol fees are applied
across the contract. Fees are deducted from distributions before member payouts.

---

### set_protocol_fee

Sets the **global** protocol fee percentage and the address that receives those
fees. This value is the contract-wide default; any group without its own override
inherits it.

Only the contract admin can call this function.

#### Entry point

```rust
pub fn set_protocol_fee(
    env: Env,
    fee: u32,
    recipient: Address,
    admin: Address,
)
```

#### Arguments

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment |
| `fee` | `u32` | New fee in **basis points** (0 = 0 %, 10 000 = 100 %) |
| `recipient` | `Address` | Stellar address that receives collected protocol fees |
| `admin` | `Address` | Current contract admin; must authorize this call |

> **Basis-point reference:** 1 bp = 0.01 %. Common values: `50` = 0.5 %, `100` = 1 %, `500` = 5 %.

#### How it works

1. `admin.require_auth()` — enforces Soroban authorization.
2. `require_admin(&env, &admin)` — verifies the caller is the stored contract admin.
3. Validates `fee ≤ 10 000`; rejects with `InvalidInput` otherwise.
4. Reads the current `old_fee` and `old_recipient` from persistent storage.
5. Writes `fee` to `DataKey::ProtocolFee` and `recipient` to `DataKey::ProtocolFeeRecipient`.
6. Bumps the TTL of both storage keys.
7. Emits `ProtocolFeeUpdated`.

#### Return value

`()` — no return value. The entry point calls `.unwrap()`, so any error panics the transaction.

#### Emitted events

| Event | Fields | When |
|---|---|---|
| `ProtocolFeeUpdated` | `admin` *(topic)*, `old_fee`, `new_fee`, `old_recipient`, `new_recipient` | Always on success |

#### Error conditions

| Error | Condition |
|---|---|
| `Unauthorized` | `admin` is not the contract administrator |
| `InvalidInput` | `fee` exceeds 10 000 basis points (100 %) |

> **Panics** if the caller is not the admin, if `fee > 10 000`, or if storage operations fail.

#### Storage keys affected

| Key | Type | Description |
|---|---|---|
| `DataKey::ProtocolFee` | `u32` | Global fee in basis points |
| `DataKey::ProtocolFeeRecipient` | `Address` | Fee recipient address |

---

### set_group_protocol_fee

Sets a **group-specific** protocol fee percentage, overriding the global value
for a single group. Groups without an override continue to inherit the global fee
from `set_protocol_fee`.

Only the contract admin can call this function.

#### Entry point

```rust
pub fn set_group_protocol_fee(
    env: Env,
    admin: Address,
    id: BytesN<32>,
    percentage: u32,
)
```

#### Arguments

| Parameter | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban execution environment |
| `admin` | `Address` | Contract admin address; must authorize this call |
| `id` | `BytesN<32>` | 32-byte unique identifier of the target payment group |
| `percentage` | `u32` | New fee as a **whole percentage** (0–100) |

> **Note:** Unlike the global fee (basis points), this parameter is a direct
> percentage — `5` means 5 %, not 0.05 %.

#### How it works

1. `admin.require_auth()` — enforces Soroban authorization.
2. `require_admin(&env, &admin)` — verifies the caller is the stored contract admin.
3. Checks that the group identified by `id` exists; rejects with `NotFound` otherwise.
4. Validates `percentage ≤ 100`; rejects with `InvalidAmount` otherwise.
5. Reads the current effective fee via `get_group_protocol_fee` (falls back to global if no prior override).
6. Writes `percentage` to `DataKey::GroupProtocolFee(id)` and bumps its TTL.
7. Emits `GroupProtocolFeeUpdated`.

#### Return value

`()` — no return value. The entry point calls `.unwrap()`, so any error panics the transaction.

#### Emitted events

| Event | Fields | When |
|---|---|---|
| `GroupProtocolFeeUpdated` | `group_id` *(topic)*, `old_fee`, `new_fee` | Always on success |

#### Error conditions

| Error | Condition |
|---|---|
| `Unauthorized` | `admin` is not the contract administrator |
| `NotFound` | No group exists with the given `id` |
| `InvalidAmount` | `percentage` exceeds 100 |

> **Panics** if the caller is not the admin, if the group does not exist, or if `percentage > 100`.

#### Storage keys affected

| Key | Type | Description |
|---|---|---|
| `DataKey::GroupProtocolFee(id)` | `u32` | Per-group fee percentage override |

---

### get_protocol_fee

Returns the current global protocol fee and recipient address.

```rust
pub fn get_protocol_fee(env: Env) -> (u32, Address)
```

**Returns:** `(fee: u32, recipient: Address)` — fee in basis points; recipient defaults to the contract admin if never explicitly set.

**Side effect:** Emits `ProtocolFeeRead { fee, recipient }` on every call for off-chain analytics.

---

### get_group_protocol_fee

Returns the effective protocol fee for a specific group.

```rust
pub fn get_group_protocol_fee(env: Env, id: BytesN<32>) -> u32
```

**Returns:** The group-specific override if one exists, otherwise the global fee from `get_protocol_fee`.

---

### Fee resolution order

```
distribute(group_id, ...)
    └─ get_group_protocol_fee(group_id)
           ├─ GroupProtocolFee(group_id) exists? → use group override
           └─ otherwise → use DataKey::ProtocolFee (global)
```

