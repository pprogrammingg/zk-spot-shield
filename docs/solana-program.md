# Solana Program and Anchor

The on-chain crate is `programs/zk_spot_shield/` (program name `zk_spot_shield`), written with the Anchor framework. It owns config, vault accounting, deposit leaves, approved Merkle roots, and spent nullifiers. Settlement verifies a Groth16 proof and moves tokens.

For diagrams of **type / owner / protocol admin / cardinality**, instruction data vs account fields, and scenario tables, see [Accounts deep dive](accounts-deep-dive.md). For the numbered user journey, see [Architecture](architecture.md).

## Accounts at a glance

| Piece | Kind | Seeds | Size (data) | Purpose |
| --- | --- | --- | --- | --- |
| `GlobalConfig` | Borsh `Account` | `["global-config"]` | 65 bytes | Admin key, trusted circuit fingerprint, pause switch |
| `VaultState` | zero-copy | `["spot_vault"]` | 120 bytes | Pair mints, reserve counters, PDA bump |
| Vault token ATAs | SPL token accounts | Associated-token derivation; **owner** = vault PDA | n/a | Real balances for mint A and mint B (two accounts) |
| `DepositLeafAccount` | zero-copy | `["deposit_leaf", leaf]` | 104 bytes | Public deposit commitment + amount/mint/depositor |
| `CleanFundsRoot` | zero-copy | `["clean_funds_root", root]` | 32 bytes | One account per approved Merkle root |
| `NullifierAccount` | zero-copy | `["nullifier", nullifier]` | 32 bytes | One account per spent nullifier |

Every Anchor account also carries an 8-byte type tag in front of the data, so allocations are `8 + size`. Rent space, proof/journal byte lengths, and CU targets live in [`notes/budgets.md`](https://github.com/pprogrammingg/zk-spot-shield/blob/main/notes/budgets.md).

## Instructions

Roughly three jobs: **admin / setup** (pause, circuit pin, vault init, register roots), **deposit** (create leaf + custody), and **spend** (`settle_shielded_spot`, which includes verify — change notes later extend this path). Relayer and SDK are off-chain; they call these same instructions.

Each instruction declares **only the accounts that call needs** in a `#[derive(Accounts)]` struct (`ctx.accounts`). That list is not “every account the program ever created” — Solana only passes the working set named in the transaction.

| Bucket | Instruction | Role |
| --- | --- | --- |
| Admin / setup | `initialize_global_config` | Create config PDA; write initial `vkey_hash`; set protocol admin |
| Admin / setup | `initialize_vault` | Create `VaultState` PDA (mints default, zero reserves, bump) |
| Admin / setup | `set_vault_mints` | Protocol admin writes `mint_a` / `mint_b` once |
| Admin / setup | `set_vkey_hash` | Protocol admin rotates `vkey_hash` after a guest rebuild |
| Admin / setup | `register_clean_funds_root` | Protocol admin allow-lists a Merkle root PDA |
| Admin / setup | `pause` / `unpause` | Protocol admin toggles `pause_flag` |
| Deposit | `deposit_shielded` | CPI: user ATA → vault ATA; `init` leaf PDA; bump reserves |
| Spend | `settle_shielded_spot` | Verify proof + journal; check root; vault-only payout of the other mint; create nullifier |

The TypeScript SDK (`sdk/`) and swap UI (`ui/`) call these same instructions. Default settle uses the committed happy-path fixtures.

## Instruction data vs durable state

`settle_shielded_spot(ctx, proof, journal)` takes **proof** and **journal** as **instruction arguments**. They are not fields on `VaultState`. After a successful settle the chain keeps updated reserves, SPL balances, and a new nullifier PDA — not a copy of the Groth16 bytes.

## What settle does today vs product direction

**Today (vault-only 1:1 spot):**

1. Reject if paused.
2. Verify Groth16 against `GlobalConfig.vkey_hash`.
3. Require journal `merkle_root` to match an existing `CleanFundsRoot`.
4. Require nullifier PDA empty.
5. `apply_vault_only_spend` on `VaultState` reserves (same `amount` both sides).
6. SPL: vault PDA pays the **other** mint to the recipient token account. No second user→vault of the journal mint (deposit already funded custody).
7. Create `NullifierAccount`.

The fee payer may be a relayer; it need not own the recipient token accounts.

**(Upcoming)** change notes for leftovers; full hidden recipient; encrypted note backup. See [Architecture](architecture.md) phase **E** and roadmap Days 73–76.

## Design notes

**`GlobalConfig` pins which circuit the program trusts.** Protocol admin pubkey (Rust field `authority`), `vkey_hash`, and `pause_flag`. Small and rarely written → ordinary `Account<T>`. Init writes the program's `VKEY_HASH`; `set_vkey_hash` rotates the on-chain fingerprint without re-init.

**`VaultState` is zero-copy** because settlement touches it every trade (`AccountLoader`, `#[repr(C)]`, `_padding` after `bump` at byte 113). Size **and** field-offset tests catch layout breakage.

**Bump vs protocol admin.** The **bump** lets the program `invoke_signed` as the vault PDA for token payouts. The **protocol admin** pubkey on config (Rust `authority`, also recorded on the vault at init) signs pause/register/vkey rotate — not SPL payouts. Anchor `has_one = authority` gates those admin instructions.

**Deposit leaf PDA.** Seeds `["deposit_leaf", leaf]` make each commitment unique (`init` fails on replay). Account data stores the public leaf hash plus amount, mint, and **depositor** — useful for indexers and audits; deposit privacy is therefore imperfect today.

**Nullifier PDA.** Seeds omit the wallet on purpose so the spend-tag address does not dox the user. Existence = already spent.

**`CleanFundsRoot`.** Same existence pattern for approved tree tips so in-flight proofs against older registered roots can still settle.

**Errors** map to checks: `InvalidProof`, `MerkleRootNotFound`, `NullifierAlreadyUsed`, `ProtocolPaused`, token/vault errors, and zero-copy failures.

**Tests.** Native unit tests cover sizes, padding offsets, seed uniqueness, PDA stability, and who must sign each of the nine instructions. LiteSVM tests exercise initialize, `set_vault_mints`, `set_vkey_hash`, deposit, and vault-only settle against a real `.so`.
