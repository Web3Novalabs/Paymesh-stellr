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

### set_protocol_fee

Updates the global protocol fee percentage and the recipient address. Only the contract admin can call this function.

**Arguments:**
- `fee`: New fee in basis points (0–10000, where 10000 = 100%).
- `recipient`: The `Address` that will receive protocol-level fees.
- `admin`: The current contract admin address (must authorize).

**Events:**
- Emits `ProtocolFeeUpdated { admin, old_fee, new_fee, old_recipient, new_recipient }`.

**Panics:**
- If `admin` is not the authorized contract administrator.
- If `fee` exceeds 10000 bps.

### get_protocol_fee

Returns the current protocol fee percentage and recipient address.

**Returns:**
- `(u32, Address)`: The current fee in basis points and the recipient address.

**Diagnostics:**
- Emits `ProtocolFeeRead { fee, recipient }` on every invocation for off-chain analytics and usage tracking.
