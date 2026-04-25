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

