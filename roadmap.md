# ZK-Shielded Spot Settlement Engine — Session Roadmap

Calibrated for **~61 engineering sessions** (Days **0–60**) of **1.5 hours** each (3 × 25 min). That maps cleanly onto a ~90-day calendar with rest days. Each **Day** is one session with one shippable deliverable — not a checklist of 10-minute chores.

Mark items `[x]` when done. A Day is **done** only when its *Exit* criteria pass — not when the code “mostly exists.”

---

### Session rhythm (3 × 25)

```
[P1 25m] Context + design: reopen yesterday's artifact, name today's exit criteria.
[P2 25m] Build: implement the core of today's deliverable.
[P3 25m] Prove: compile, run the smallest meaningful test, commit.
```

Do not split a Day across sessions. If you finish early, deepen tests or docs for *that* deliverable — do not pull tomorrow's Day forward mid-session.

---

## Month 1: Foundation, Zero-Copy State & Off-Chain SP1 (Sessions 0–20)

### Phase 0: Cursor / Agent Efficiency (1 session)

- [x] **Day 0 — Make Cursor efficient for this repo**
  - [x] Audit existing `.cursor/rules/` MDCs (keep, fix empty, or replace)
  - [x] Always-apply project rule: monorepo layout, stack, do-nots
  - [x] Anchor / zero-copy rule scoped to `program/**`
  - [x] SP1 guest/host rule scoped to `zk-circuit/**`
  - [x] Session discipline rule: follow `roadmap.md`, one Day per session
  - [x] `AGENTS.md` index of rules + how to work the repo in Cursor
  - [x] `.vscode/settings.json` so `.git` stays visible for hook spot-checks
  - [x] *Exit:* agent can state layout + zero-copy + SP1 constraints from rules alone; Day 0 checklist all `[x]`

### Phase 1: Environment, Toolchain & Monorepo (1 session)

- [x] **Day 1 — Monorepo live on localnet**
  - [x] Create repo layout (`/program`, `/zk-circuit`, `/client`)
  - [x] Root workspace `Cargo.toml`
  - [x] `.gitignore` (targets, keypairs, ledgers, SP1 artifacts)
  - [x] Verify Rust / Solana / Anchor / SP1 CLIs
  - [x] Generate local keypair (`solana-keygen new`)
  - [x] Point Solana CLI at `localhost`
  - [x] `anchor init` the program
  - [x] Lock `Anchor.toml` for localnet + program IDs + tests (Rust/litesvm scaffold)
  - [x] *Exit:* `anchor build` (or equivalent scaffold compile) succeeds; `solana config get` shows localhost

### Phase 2: Anchor On-Chain State & Zero-Copy Architecture (4 sessions)

- [x] **Day 2 — Program deps + `GlobalConfig`**
  - [x] Wire `program/Cargo.toml` (`anchor-lang`, `anchor-spl`, `bytemuck`, `sp1-solana`)
  - [x] Implement `GlobalConfig` PDA (authority, vkey hash, pause flag)
  - [x] Register it in the program module tree
  - [x] *Exit:* Account compiles; init path stubs without panicking on layout size

- [x] **Day 3 — Zero-copy `VaultState`**
  - [x] Add `vault.rs` with `#[account(zero_copy)]` + `#[repr(C)]`
  - [x] Fields (authority, mints, reserves, bump) + explicit padding
  - [x] Size asserts / `INIT_SPACE`-equivalent checks
  - [x] *Exit:* `VaultState` size and alignment verified; load via `AccountLoader` pattern sketched

- [x] **Day 4 — Nullifier + clean-funds root accounts**
  - [x] Implement zero-copy `NullifierAccount` (32-byte nullifier)
  - [x] Implement `CleanFundsRoot` (historical Merkle roots)
  - [x] Seeds and PDA helpers for both
  - [x] *Exit:* Both accounts compile; PDA seeds documented in code

- [x] **Day 5 — Constants, errors, module surface**
  - [x] `constants.rs` seeds (`global_config`, `spot_vault`, `nullifier`, …) + derivation helpers
  - [x] `errors.rs` (`InvalidProof`, `NullifierAlreadyUsed`, `ZeroCopyDeserializationError`, `MerkleRootNotFound`, …)
  - [x] Export state + errors cleanly from `lib.rs` / `mod` tree
  - [x] *Exit:* Program builds with all state types reachable; error codes usable from instructions (even if stubs)

### Phase 3: Off-Chain ZK Circuit (SP1 Guest) (5 sessions)

- [x] **Day 6 — SP1 guest crate**
  - [x] `cargo new --bin zk-circuit/guest` + workspace membership
  - [x] `sp1-zkvm` deps + guest profile (`opt-level = 3`, `lto = true`)
  - [x] *Exit:* The guest crate compiles successfully under the SP1 toolchain target (riscv32im-succinct-zkvm-elf).

- [x] **Day 7 — Private + public I/O shapes**
  - [x] Stack-allocated `PrivateInputs` (fixed arrays: address, Merkle path depth 20)
  - [x] `PublicOutputs` journal (`requested_swap_amount`, `asset_id_mint`, `nullifier`, `merkle_root`)
  - [x] *Exit:* Types compile in guest; sizes fixed (no heap in guest hot path)

- [x] **Day 8 — Merkle inclusion (Poseidon)**
  - [x] Guest logic: binary Poseidon recursion over the path
  - [x] Assert leaf ∈ tree under `merkle_root`
  - [x] *Exit:* Guest executes a known-good inclusion vector without panic

- [x] **Day 9 — Solvency + nullifier**
  - [x] Constraint `balance >= requested_swap_amount`
  - [x] Nullifier = hash(secret, timestamp) (or chosen binding scheme)
  - [x] Document unlinkability assumptions in a short comment/module doc
  - [x] *Exit:* Guest rejects underfunded input; produces stable nullifier for fixed secret

- [x] **Day 10 — Journal commit**
  - [x] Commit `PublicOutputs` via `sp1_zkvm::io::commit`
  - [x] Tidy guest `main` so host can drive end-to-end
  - [x] *Exit:* Guest run produces expected public values for a fixture

### Phase 4: Prover Host & Local Proofs (5 sessions)

- [ ] **Day 11 — Host driver + fixtures**
  - [ ] `zk-circuit/host` with `sp1_sdk::ProverClient`
  - [ ] Mock balances; build test Merkle trees and valid paths matching guest
  - [ ] *Exit:* Host loads ELF / guest and runs execute (proof optional today)

- [ ] **Day 12 — Groth16 prove pipeline**
  - [ ] Compile RISC-V ELF, feed inputs
  - [ ] Generate serialized Groth16 proof + public values
  - [ ] *Exit:* One successful local proof artifact on disk

- [ ] **Day 13 — Negative inclusion test**
  - [ ] Host test: address *not* in tree → guest/execution failure
  - [ ] *Exit:* Test fails closed; CI-friendly assertion

- [ ] **Day 14 — Proof packaging**
  - [ ] Helpers to pack proof + journal into `Vec<u8>` for Anchor instruction data
  - [ ] *Exit:* Round-trip serialize/deserialize unit test in host

- [ ] **Day 15 — Vkey extract + latency notes**
  - [ ] CLI/script to print SP1 vkey hash as bytes for `GlobalConfig`
  - [ ] Record CPU (and GPU if available) prove latency + cycle count in `notes/proving.md`
  - [ ] *Exit:* Vkey bytes committed or documented; one measured prove time logged

### Month 1 buffer / integration (5 sessions)

- [ ] **Day 16 — Wire program instruction stubs**
  - [ ] Empty `initialize_vault` + `settle_shielded_spot` modules and enum entries

- [ ] **Day 17 — Host ↔ guest fixture pack**
  - [ ] Single shared fixture (tree, path, amounts) used by guest execute and host prove

- [ ] **Day 18 — Size & CU budget doc**
  - [ ] Capture expected account sizes and CU budget targets (~280k verifier)

- [ ] **Day 19 — Month 1 review pass**
  - [ ] Fix compile warnings, align naming, zero-copy padding review

- [ ] **Day 20 — Checkpoint**
  - [ ] Tag `v0.1-month1`
  - [ ] README section: how to build program + prove once locally

---

## Month 2: On-Chain Verifier, Settlement & Localnet E2E (Sessions 21–40)

### Phase 5: ZK Verification & Settlement Instructions (8 sessions)

- [ ] **Day 21 — `initialize_vault`**
  - [ ] Initialize zero-copy `VaultState` + `GlobalConfig` (authority, vkey, pause=false)
  - [ ] *Exit:* Localnet (or mollusk) test creates accounts with correct sizes

- [ ] **Day 22 — `settle_shielded_spot` scaffold**
  - [ ] Accounts + args: proof bytes, journal bytes, vault, nullifier PDA, clean-root, token accounts
  - [ ] *Exit:* Instruction deserializes; fails loudly if accounts missing

- [ ] **Day 23 — SP1 verifier CPI**
  - [ ] Integrate `sp1-solana` verify against stored `vkey_hash`
  - [ ] *Exit:* Valid fixture proof verifies; garbage proof fails

- [ ] **Day 24 — Journal parse + root check**
  - [ ] Deserialize public values; require `merkle_root` ∈ `CleanFundsRoot`
  - [ ] *Exit:* Bad root → `MerkleRootNotFound`

- [ ] **Day 25 — Nullifier gate**
  - [ ] Require `NullifierAccount` uninitialized; derive PDA from nullifier bytes
  - [ ] *Exit:* Second settle with same nullifier cannot proceed past this check

- [ ] **Day 26 — Vault mutations**
  - [ ] Zero-copy updates to reserves with checked math; reject overflow/underflow
  - [ ] *Exit:* Unit/integration test moves reserves correctly

- [ ] **Day 27 — SPL transfers**
  - [ ] `anchor_spl::token::transfer` signed by vault PDA; mint/ATA wiring
  - [ ] *Exit:* Balances change on successful settle path

- [ ] **Day 28 — Finalize nullifier + pause**
  - [ ] Write nullifier account after success
  - [ ] `pause`/`unpause` on `GlobalConfig` (authority-only); settle respects pause
  - [ ] *Exit:* Replay after success → `NullifierAlreadyUsed`; paused settle rejected

### Phase 6: CU & TX Size (4 sessions)

- [ ] **Day 29 — CU profile**
  - [ ] Measure settle CU; confirm verifier stays near budget (~280k); note hotspots

- [ ] **Day 30 — Compute budget helpers**
  - [ ] Client/helpers set CU limit (and price if needed) per tx shape

- [ ] **Day 31 — Zero-copy load path**
  - [ ] Account order + `bytemuck`/`AccountLoader` so settle avoids heap churn

- [ ] **Day 32 — Address Lookup Tables**
  - [ ] ALT setup so settle tx keys fit under 1232-byte MTU with proof payload

### Phase 7: Client + Localnet Verification (6 sessions)

- [ ] **Day 33 — TS client + Merkle util**
  - [ ] `client` Anchor provider to localhost
  - [ ] Poseidon/SHA-256 tree util matching guest

- [ ] **Day 34 — Prove bridge**
  - [ ] Client helper invokes host prove and returns proof + public bytes

- [ ] **Day 35 — Instruction wrappers**
  - [ ] Package ALT + compute budget + settle ix

- [ ] **Day 36 — Happy-path E2E**
  - [ ] Init → prove → settle → assert token balances + vault reserves

- [ ] **Day 37 — Negative E2E triad**
  - [ ] Flip 1 proof byte → `InvalidProof`
  - [ ] Replay → `NullifierAlreadyUsed`
  - [ ] Unregistered root → `MerkleRootNotFound`

- [ ] **Day 38 — `test:e2e` pipeline**
  - [ ] One command: validator, deploy, prove, settle, assertions
  - [ ] Optional confirmation listener

### Phase 8: Hardening (2 sessions)

- [ ] **Day 39 — Fuzz / invariant harness**
  - [ ] Trident or mollusk: arithmetic edges + random proof bytes (no verifier crash)

- [ ] **Day 40 — Constraint audit + Month 2 tag**
  - [ ] PDA `seeds`/`bump`/`has_one` pass
  - [ ] Strip noisy `msg!`
  - [ ] Tag `v0.2-month2`

---

## Month 3: Relayer, Benchmarks, Audit Readiness & Devnet (Sessions 41–60)

### Phase 9: Relayer & Client SDK (6 sessions)

- [ ] **Day 41 — Relayer scaffold**
  - [ ] `relayer/` Axum (or Actix) service skeleton + health endpoint

- [ ] **Day 42 — `/submit-proof` + worker queue**
  - [ ] Accept private intent; Tokio queue for prove jobs

- [ ] **Day 43 — Tx build + fee payer**
  - [ ] Relayer wraps proof into Solana tx and signs as fee payer

- [ ] **Day 44 — Fees + RPC retry**
  - [ ] bps fee from payout
  - [ ] Exponential backoff on send/confirm

- [ ] **Day 45 — `@zk-shield/sdk` surface**
  - [ ] `generateProofInputs`, `requestShieldedSwap`, `getVaultReserves`

- [ ] **Day 46 — SDK sync + docs**
  - [ ] WebSocket/`CleanFundsRoot` sync helper
  - [ ] Usage examples in package README

### Phase 10: Benchmarking (3 sessions)

- [ ] **Day 47 — E2E latency**
  - [ ] Prove → relayer → confirm; write numbers to `notes/latency.md`

- [ ] **Day 48 — Prove hardware + on-chain CU breakdown**
  - [ ] CPU vs GPU prove times
  - [ ] CU split (verify / checks / SPL)

- [ ] **Day 49 — Payload + throughput**
  - [ ] Minimize journal bytes
  - [ ] Concurrent localnet submission smoke

### Phase 11: Security Audit Readiness (4 sessions)

- [ ] **Day 50 — Threat model doc**
  - [ ] Circuit soundness, replay, pause, authority, root registry

- [ ] **Day 51 — Layout + signer audit**
  - [ ] `#[repr(C)]` / padding review
  - [ ] Every ix path checks signer/writable explicitly

- [ ] **Day 52 — Tooling scans**
  - [ ] `cargo-audit` (+ any chosen program analyzers)
  - [ ] Fix high findings or document waivers

- [ ] **Day 53 — Bytecode freeze prep**
  - [ ] Reproducible build notes; record program hash procedure

### Phase 12: Devnet Launch (7 sessions)

- [ ] **Day 54 — Devnet keys + SOL**
  - [ ] Funding, deploy keypair, cluster config

- [ ] **Day 55 — Deploy program**
  - [ ] Deploy to Devnet; record program ID

- [ ] **Day 56 — Init + vkey + roots**
  - [ ] `GlobalConfig`, vkey hash, seed `CleanFundsRoot`

- [ ] **Day 57 — Vault + token accounts**
  - [ ] Live vault ATAs on Devnet

- [ ] **Day 58 — Relayer on Devnet RPC**
  - [ ] Production-ish process config pointing at Devnet

- [ ] **Day 59 — Live shielded swap**
  - [ ] One successful swap via SDK
  - [ ] Explorer verification of balances + zero-copy state

- [ ] **Day 60 — Docs + `v1.0.0-devnet`**
  - [ ] README, architecture diagram, install scripts
  - [ ] Tag release; archive build logs, circuit keys, program artifacts
