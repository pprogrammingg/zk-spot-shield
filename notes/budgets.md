# Size, compute units, latency

Account **data** sizes match `state.rs` asserts and [`docs/solana-program.md`](../docs/solana-program.md). This note is the measured-cost source of truth.

## Chain limits

| Limit | Value |
| --- | --- |
| Packet / transaction size | **1232 bytes** (before lookup tables) |
| Compute units / transaction | **1,400,000** (`client::SETTLE_CU_LIMIT`) |
| Verifier class | **~280k** CU (SP1 Groth16) |

## On-chain account rent space

Anchor stores an **8-byte discriminator** before account data. `init` space = `8 + data`.

| Account | Data | Rent `space` | Layout |
| --- | --- | --- | --- |
| `VaultState` | 120 | **128** | `zero_copy` `repr(C)`; `_padding: [u8; 7]` after `bump` |
| `DepositLeafAccount` | 104 | **112** | `zero_copy` `repr(C)`; already 8-byte aligned |
| `NullifierAccount` | 32 | **40** | `zero_copy` |
| `CleanFundsRoot` | 32 | **40** | `zero_copy` |
| `GlobalConfig` | 65 | **73** | `Account` + `InitSpace` |

Change `state.rs` asserts first, then this table. Signers: [`layout-signer-audit.md`](./layout-signer-audit.md).

## Proof, journal, instruction data

Happy-path artifacts in `zk-circuit/fixtures/happy/` ([`proving.md`](./proving.md)):

| Blob | Bytes |
| --- | --- |
| `groth16.bin` | **356** |
| `journal.bin` | **104** (LE `u64` amount + mint + nullifier + root; no inner serde) |
| Host `pack_settle_payload` | **468** |
| Anchor `settle_shielded_spot` data | **476** (8-byte discriminator + Borsh vectors) |

Shrinking the journal means dropping a public field (circuit, vkey, and on-chain parse).

Eleven settle metas (payer, config, vault, root, nullifier, vault token A/B, user token A/B, token program, system program) ≈ **352 bytes** of pubkeys on top of ~476 bytes of data. Compute-budget instruction is separate (`client::settle_compute_budget_ixs`). Packet pressure → address lookup tables.

Proof bytes stay in **instruction data**. A lookup table only indexes **pubkeys**. Static ALT candidates: token program, system program, `global_config`, `vault`, vault token A/B. Per-settle keys (nullifier, user token accounts, `clean_funds_root`) stay as message keys. Helpers: `client::alt`.

## Measured settle compute units (LiteSVM, 2026-10-20)

Full vault-only settle: **285,947** CU of **1,400,000**. Groth16 runs inside `settle_shielded_spot`, so it cannot be metered alone. The ~270–280k band is “total minus the small leftover.”

| Band | Est. CU |
| --- | --- |
| SP1 v6 Groth16 verify | **~270k–280k** |
| Vault → user token transfer | **~3k–8k** |
| Nullifier account create + write | **~3k–6k** |
| Pause / journal / root / mint checks | **~1k–3k** |
| **Measured total** | **285,947** |

Non-verify work is about **6–16k** CU. Priority fee defaults to **0**.

```bash
unset CARGO_TARGET_DIR
anchor build --ignore-keys
cargo test -p zk_spot_shield --test test_settle_scaffold settle_cu_profile --locked -- --nocapture
```

`VaultState` and `CleanFundsRoot` stay `AccountLoader` / `zero_copy`. Loaders come before token accounts. Mint checks run once after a single `vault.load_mut()`.

## Latency (2026-10-20, this laptop)

Pipeline: load fixture proof → relayer sign / HTTP 402 accept → LiteSVM confirm. Network Groth16 was **not** re-run; wall time is still [`proving.md`](./proving.md).

| Stage | Wall |
| --- | --- |
| Load Groth16 + journal fixtures | **0.58 ms** (Rust) / **0.1 ms** (SDK) |
| Witness `generateProofInputs` (depth 20) | **4.2 ms** |
| Relayer `process_intent_dry_run` | **0.59 ms** |
| HTTP 402 `accept_journal` | **0.06 ms** |
| SDK `requestShieldedSwap` | **6.4 ms** |
| LiteSVM settle confirm | **84.8 ms** (**285,947** CU) |
| Network Groth16 wrap (not re-run) | **30.7 s** + ~0.42 `$PROVE` |

With fixtures, proving is disk I/O; the bottleneck is on-chain verify. In production, remote wrap still dominates unless you already have a proof. Relayer dry-run is not cluster confirm.

```bash
cargo test -p zk-spot-shield-relayer --lib latency --locked -- --nocapture
cd sdk && npx vitest run test/latency.test.ts --reporter=verbose
```

## Throughput

Relayer: 16 parallel `process_intent_dry_run` jobs (fixture-shaped settle JSON). Sign only; no cluster.

On-chain, one happy-path nullifier is single-use. A second settle of the same fixture must fail `NullifierAlreadyUsed`. Distinct notes need distinct Groth16 proofs.

```bash
cargo test -p zk-spot-shield-relayer --lib concurrent --locked -- --nocapture
```

## Program binary hash

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
unset CARGO_TARGET_DIR
anchor build --ignore-keys
./scripts/program-hash.sh
```

Hash on the machine that built. Compare Devnet dumps to the **deploy** digest (`target/deploy/zk_spot_shield.so`), not the SBPF path.

| Path | SHA-256 |
| --- | --- |
| `target/deploy/zk_spot_shield.so` (current local) | `b14e59e6cd0ecd67de4cc46c912f123749d81ee02046e7956945384be74c8c9e` |
| `target/sbpf-solana-solana/release/zk_spot_shield.so` (same build) | `7790bcd2cc8de10d6579fe4f478cd8372d0bd809742d1df01ef61e9cb5770327` — **not** the deploy path |

Cluster record: [`devnet.md`](./devnet.md). Guest ELF is gitignored; happy vkey is `constants::VKEY_HASH`.
