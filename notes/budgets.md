# Size and compute-unit budgets

Account **data** sizes match `state.rs` asserts and [`docs/solana-program.md`](../docs/solana-program.md). This note adds rent space, instruction payload, measured compute units, and address-lookup-table sizing.

## Chain limits

| Limit | Value | Why it matters |
| --- | --- | --- |
| Packet / transaction size | **1232 bytes** (before lookup tables) | Settle instruction data is large; account keys compete for the same budget |
| Compute units / transaction | **~1.4M** | Groth16 verify uses most of it |
| Verifier target | **~280k** compute units | SP1 Groth16 class |

## On-chain account rent space

Anchor stores an **8-byte discriminator** before account data. `init` space = `8 + data`.

| Account | Data | Rent `space` | Source |
| --- | --- | --- | --- |
| `VaultState` | 120 | **128** | `initialize_vault` |
| `NullifierAccount` | 32 | **40** | `state.rs` |
| `CleanFundsRoot` | 32 | **40** | `state.rs` |
| `GlobalConfig` | 65 (32 + 32 + 1) | **73** | `initialize_global_config` |

Field layout and padding: [`docs/solana-program.md`](../docs/solana-program.md). Change `state.rs` asserts first, then this table.

## Proof, journal, instruction data

Happy-path artifacts in `zk-circuit/fixtures/happy/` ([`proving.md`](./proving.md)):

| Blob | Bytes | Notes |
| --- | --- | --- |
| `groth16.bin` | **356** | On-chain SP1 v6 Groth16 verify |
| `journal.bin` | **104** | amount + mint + nullifier + root |
| Host `pack_settle_payload` | **468** | length prefixes + proof + journal |
| Anchor `settle_shielded_spot` data | **476** | 8-byte discriminator + Borsh vectors |

~476 bytes of instruction data is a large fraction of 1232 before signatures, headers, and account keys.

## Settle account keys

Eleven metas: payer, `global_config`, `vault`, `clean_funds_root`, `nullifier_account`, vault token A/B, user token A/B, token program, system program. Compute-budget instruction is separate (`client::settle_compute_budget_ixs`).

Eleven full pubkeys ≈ **352 bytes** on top of ~476 bytes of data → packet pressure → address lookup tables.

## Measured settle compute units (LiteSVM)

Latest full settle: **285,947** compute units ([`cu-breakdown.md`](./cu-breakdown.md), 2026-10-20). Older band in this file was ~293k (2026-10-09). Groth16 dominates; non-verify work is small.

Client sets the transaction limit to **1,400,000** (`client::SETTLE_CU_LIMIT`).

```bash
unset CARGO_TARGET_DIR
anchor build --ignore-keys
cargo test -p zk_spot_shield --test test_settle_scaffold settle_cu_profile -- --nocapture
```

Helpers: `SETTLE_CU_LIMIT`, `settle_cu_limit_ix()`, `with_settle_compute_budget`. Priority fee defaults to **0**.

## Zero-copy load path

`VaultState` and `CleanFundsRoot` stay `AccountLoader` / `zero_copy`. Loaders come before token accounts. Mint checks run once after a single `vault.load_mut()`.

## Address lookup tables

Proof bytes stay in **instruction data**. A lookup table only indexes **pubkeys**.

Static candidates (`client::SETTLE_ALT_STATIC_SLOT_LABELS`): token program, system program, `global_config`, `vault`, vault token A/B.

Per-settle keys (nullifier, user token accounts, `clean_funds_root`) stay as message keys or a short-lived table extend.

Sizing helpers: `client::alt`. The lookup-table path estimate fits under **1232**; legacy is tight.
