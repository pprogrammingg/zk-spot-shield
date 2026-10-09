# Size & CU budgets (Days 18 / 29–32)

Targets for Month 2 settle. Account **data** sizes match `docs/solana-program.md` / `state.rs` asserts; this note adds **rent space**, **ix payload**, **measured CU**, and **ALT** sizing.

## Chain limits (context)

| Limit | Value | Why it matters |
| --- | --- | --- |
| Packet / tx MTU | **1232 bytes** (before ALTs) | Settle ix data is already fat; static account keys compete for the same budget → Day 32 ALTs |
| Compute units / tx | **~1.4M CU** | Groth16 verify is the hog; other settle work must fit in the remainder |
| Verifier CU **target** | **~280k CU** | Roadmap / SP1 Groth16 class |

## On-chain account rent space

Anchor always stores an **8-byte discriminator** before account data. `init` / create `space` = `8 + data`.

| Account | Data | Rent `space` | Source |
| --- | --- | --- | --- |
| `VaultState` | 120 | **128** | `state.rs` `size_of` assert; `initialize_vault` |
| `NullifierAccount` | 32 | **40** | `state.rs` assert |
| `CleanFundsRoot` | 32 | **40** | `state.rs` assert |
| `GlobalConfig` | 65 (`INIT_SPACE`: 32 + 32 + 1) | **73** | `initialize_global_config` (`8 + GlobalConfig::INIT_SPACE`) |

Narrative layout (fields, padding, seeds): [docs/solana-program.md](../docs/solana-program.md).

## Proof / journal / settle ix data

Happy-path artifacts in `zk-circuit/fixtures/happy/` (Day 12 remote wrap; see [proving.md](./proving.md)):

| Blob | Bytes | Notes |
| --- | --- | --- |
| `groth16.bin` (`proof.bytes()`) | **356** | On-chain SP1 v6 Groth16 verify |
| `journal.bin` | **104** | `PublicOutputs::JOURNAL_BYTE_LEN` = 8 + 3×32 |
| Host `pack_settle_payload` | **468** | `u32` + proof + `u32` + journal (Day 14 helper) |
| Anchor `settle_shielded_spot` ix data | **476** | 8-byte disc + Borsh `Vec` proof + `Vec` journal |

Ix data alone (~476 B) is a large fraction of 1232 B before signatures, message headers, and account keys.

## Settle account keys

`SettleShieldedSpot` metas (**11**): payer, `global_config`, `vault`, `clean_funds_root`, `nullifier_account`, `vault_token_a/b`, `user_token_a/b`, `token_program`, `system_program`. Compute-budget ix is separate (Day 30: `client::settle_compute_budget_ixs`).

Rough static-key cost if all appear as full pubkeys: **11 × 32 ≈ 352 B** — on top of ~476 B ix data → packet pressure → **ALT** (Day 32).

## Day 29 — measured settle CU (LiteSVM)

Fixture: happy Groth16 + registered root + funded vault/SPL + nullifier create.

| Metric | Value | Notes |
| --- | --- | --- |
| **`compute_units_consumed`** | **~293k CU** | LiteSVM test `settle_cu_profile_within_budget` (2026-10-09) |
| Verifier target | ~280k | Measured full settle is **within ~5%** of target → Groth16 dominates |
| Tx CU limit set by client | **1_400_000** | `client::SETTLE_CU_LIMIT` / Day 30 helpers |
| Hotspots (ordered) | 1) SP1 Groth16 verify 2) SPL transfer CPIs 3) nullifier `create_account` 4) vault/root checks | Non-verify work is small vs pairing |

Re-measure after verifier or account-layout changes:

```bash
unset CARGO_TARGET_DIR
anchor build --ignore-keys
cargo test -p zk_spot_shield --test test_settle_scaffold settle_cu_profile -- --nocapture
```

## Day 30 — compute budget helpers

`client::compute_budget`:

- `SETTLE_CU_LIMIT = 1_400_000`
- `settle_cu_limit_ix()` / `settle_cu_price_ix()` / `with_settle_compute_budget(settle_ix)`
- Price defaults to **0** (no priority fee ix) until a cluster policy needs it

## Day 31 — zero-copy load path

- `VaultState` + `CleanFundsRoot` stay `AccountLoader` / `#[account(zero_copy)]`
- Account order: loaders before token `Account`s
- Mint checks run **once** after a single `vault.load_mut()` (constraints only pin owners + vault bump) to avoid repeated `vault.load()` during validation

## Day 32 — Address Lookup Tables

Proof bytes stay in **ix data**; ALT only indexes **pubkeys**.

Static ALT candidates (`client::SETTLE_ALT_STATIC_SLOT_LABELS`): `token_program`, `system_program`, `global_config`, `vault`, `vault_token_a`, `vault_token_b`.

Per-settle keys (nullifier PDA, user ATAs, `clean_funds_root`) stay as message keys or a short-lived ALT extend.

Sizing helpers + tests: `client::alt` (`estimate_legacy_settle_message_bytes` vs `estimate_v0_settle_message_bytes_with_alt`). ALT path estimate fits under **1232**; legacy is deliberately tight.

On-chain ALT create/extend RPC wiring lands with the TS client (Day 35 packages ALT + CU + settle).

## What this is not

- Not a substitute for Day 48 hardware CU breakdown.
- Not a second source of truth for account field layouts — change `state.rs` asserts first, then update this table.
