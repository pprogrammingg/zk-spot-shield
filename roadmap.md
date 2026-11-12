# ZK Spot Shield — Session Roadmap

Each engineering Day is 1.5 hours (3 × 25 min). Mark `[x]` only when that item is done. A Day is **done** only when its Exit passes. One Day per session.

---

## Now

**Write pack** — each of Write-1/2/3 is a Medium/LinkedIn piece: **article + short click-through video**. Days **67–68** Devnet UI checklist + automated E2E are done (Day 67 script feeds the settle/public vids).

---

## Next

Do in this order. Do not pull a parked item mid-session.

| # | Day | What |
| --- | --- | --- |
| 1 | **Write-1** | Circuit article + prove-path video |
| 2 | **Write-2** | Settle article + UI/agent settle video |
| 3 | **Write-3** | Public artifact + public-vs-shielded (or README) video |

---

## Parked

| Item | Why |
| --- | --- |
| **70–72** IPFS CID / second host | Self-submit already works |
| **P3** Stripe MPP sandbox | After a real user |
| **Day 62** relayer send on Devnet | Agent/UI self-submit is enough |
| AMM / same-mint pool | Stay 1:1 spot |

Relayer = 402 merchant + optional SOL fee payer, **not** the agent. `$PROVE` is Succinct credits in gitignored `.env`, not a Solana wallet.

---

## 402 track

The **agent** holds notes and loops 402 → Groth16 → settle → `POST /402/pay`. The **wallet** deposits A and may pay SOL. The **relayer** quotes 402 and unlocks on a **nullifier**, not a payer pubkey.

| Milestone | Status |
| --- | --- |
| **P1** 402 unlocks on a settle journal (fixture) | Done (Day 50) |
| **P2** swap UI | Done (Days 66, 81) |
| **P4** Devnet agent loop | Done (Days 82–83). Eleven settle sigs; no note covers a further buy |
| **Day 84** Non-shielded A→B | Done — Devnet `27LEe6SB…` (11 A→B, unlock=wallet) |
| **P0** public vs shielded compare | Done — quote 11: ~49× wall, ~16× CU (public faster/cheaper) |
| **P3** Stripe MPP | Parked |

Dates: Stripe Sessions CFP **submitted 2 Nov** (decisions 16 Nov); Agentic Payments Hackathon **13–15 Nov**.

---

## Session rhythm (3 × 25)

```
[25m] Reopen yesterday's artifact. Name today's Exit.
[25m] Build that Exit.
[25m] Tests on crates you touched. Suggest a commit subject.
```

Do not split a Day across sessions. If you finish early, deepen tests or docs for **that** Day.

---

## Done (archive below)

Days **0–83**: Month 1–2, deposit, relayer, P1, benches, audit, Devnet 58–64, public README, UI, change notes, backup, unlink, localnet + Devnet agent. Checklists stay so Exit text is searchable.

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

### Phase 4: Prover Host & Remote Groth16 (5 sessions)

- [x] **Day 11 — Host driver + fixtures**
  - [x] `zk-circuit/host` with `sp1_sdk::ProverClient`
  - [x] Mock balances; build test Merkle trees and valid paths matching guest
  - [x] *Exit:* Host loads ELF / guest and runs execute (proof optional today)

- [x] **Day 12 — Remote Groth16 prove**
  - [x] Point host at Succinct (or equivalent) remote prover: `SP1_PROVER=network` + network key
  - [x] Request Groth16 wrap (not mock); save `sp1-artifacts/` (gitignored) and copy public happy-path bytes to `zk-circuit/fixtures/happy/` for host `--lib`
  - [x] Document hobby cost: network Groth16 is **not free** (`$PROVE`); execute stays $0
  - [x] *Exit:* Non-empty Groth16 + journal on disk from a **remote** prove; re-run command + cost note in `notes/proving.md`. If wrap/prove is blocked (e.g. RAM/Docker), write that honestly in the same note — Day 65’s public artifact may cite it; do not fake a Groth16 file.

- [x] **Day 13 — Negative inclusion test**
  - [x] Host test: address *not* in tree → guest/execution failure
  - [x] *Exit:* Test fails closed; CI-friendly assertion

- [x] **Day 14 — Proof packaging**
  - [x] Helpers to pack proof + journal into `Vec<u8>` for Anchor instruction data
  - [x] *Exit:* Round-trip serialize/deserialize unit test in host

- [x] **Day 15 — Vkey extract + latency notes**
  - [x] CLI/script to print SP1 vkey hash as bytes for `GlobalConfig`
  - [x] Record **remote** Groth16 wall time (and CPU/GPU if you ever prove locally) + execute cycle count in `notes/proving.md`
  - [x] *Exit:* Vkey bytes committed or documented; one measured prove time logged

### Month 1 buffer / integration (5 sessions)

- [x] **Day 16 — Wire program instruction stubs**
  - [x] Empty `initialize_vault` + `settle_shielded_spot` modules and enum entries

- [x] **Day 17 — Host ↔ guest fixture pack**
  - [x] Single shared fixture (tree, path, amounts) used by guest execute and host prove

- [x] **Day 18 — Size & CU budget doc**
  - [x] Capture expected account sizes and CU budget targets (~280k verifier)

- [x] **Day 19 — Month 1 review pass**
  - [x] Fix compile warnings, align naming, zero-copy padding review

- [x] **Day 20 — Checkpoint**
  - [x] Tag `v0.1-month1` *(create after Month 1 commit: `git tag v0.1-month1` — README documents the command)*
  - [x] README section: execute locally; Groth16 via remote prover (not this laptop’s Docker wrap)
  - [x] *Exit:* Tag + README commands exist.

---

## Month 2: On-Chain Verifier, Settlement & Localnet E2E (Sessions 21–40)

### Phase 5: ZK Verification & Settlement Instructions (8 sessions)

- [x] **Day 21 — `initialize_vault`**
  - [x] Initialize zero-copy `VaultState` + `GlobalConfig` (authority, vkey, pause=false)
  - [x] *Exit:* Localnet (or mollusk) test creates accounts with correct sizes

- [x] **Day 22 — `settle_shielded_spot` scaffold**
  - [x] Accounts + args: proof bytes, journal bytes, vault, nullifier PDA, clean-root, token accounts
  - [x] *Exit:* Instruction deserializes; fails loudly if accounts missing

- [x] **Day 23 — SP1 verifier CPI**
  - [x] Integrate `sp1-solana` verify against stored `vkey_hash`
  - [x] *Exit:* Valid fixture proof verifies; garbage proof fails

- [x] **Day 24 — Journal parse + root check**
  - [x] Deserialize public values; require `merkle_root` ∈ `CleanFundsRoot`
  - [x] *Exit:* Bad root → `MerkleRootNotFound`

- [x] **Day 25 — Nullifier gate**
  - [x] Require `NullifierAccount` uninitialized; derive PDA from nullifier bytes
  - [x] *Exit:* Second settle with same nullifier cannot proceed past this check

- [x] **Day 26 — Vault mutations**
  - [x] Zero-copy updates to reserves with checked math; reject overflow/underflow
  - [x] *Exit:* Unit/integration test moves reserves correctly

- [x] **Day 27 — SPL transfers**
  - [x] `anchor_spl::token::transfer` signed by vault PDA; mint/ATA wiring
  - [x] *Exit:* Balances change on successful settle path

- [x] **Day 28 — Finalize nullifier + pause**
  - [x] Write nullifier account after success
  - [x] `pause`/`unpause` on `GlobalConfig` (authority-only); settle respects pause
  - [x] *Exit:* Replay after success → `NullifierAlreadyUsed`; paused settle rejected

### Phase 6: CU & TX Size (4 sessions)

- [x] **Day 29 — CU profile**
  - [x] Measure settle CU; confirm verifier stays near budget (~280k); note hotspots

- [x] **Day 30 — Compute budget helpers**
  - [x] Client/helpers set CU limit (and price if needed) per tx shape

- [x] **Day 31 — Zero-copy load path**
  - [x] Account order + `bytemuck`/`AccountLoader` so settle avoids heap churn

- [x] **Day 32 — Address Lookup Tables**
  - [x] ALT setup so settle tx keys fit under 1232-byte MTU with proof payload

### Phase 7: Client + Localnet Verification (6 sessions)

- [x] **Day 33 — TS client + Merkle util**
  - [x] `client` Anchor provider to localhost
  - [x] Poseidon/SHA-256 tree util matching guest

- [x] **Day 34 — Prove bridge**
  - [x] Client helper invokes host prove and returns proof + public bytes

- [x] **Day 35 — Instruction wrappers**
  - [x] Package ALT + compute budget + settle ix

- [x] **Day 36 — Happy-path E2E**
  - [x] Init → prove → settle → assert token balances + vault reserves

- [x] **Day 37 — Negative E2E triad**
  - [x] Flip 1 proof byte → `InvalidProof`
  - [x] Replay → `NullifierAlreadyUsed`
  - [x] Unregistered root → `MerkleRootNotFound`

- [x] **Day 38 — `test:e2e` pipeline**
  - [x] One command: validator, deploy, prove, settle, assertions
  - [x] Optional confirmation listener

### Phase 8: Hardening (2 sessions)

- [x] **Day 39 — Fuzz / invariant harness**
  - [x] **LiteSVM** (not Mollusk/Trident — keeps pinned `litesvm = 0.10`): arithmetic edges (`apply_swap` `--lib`) + random/mutated proof bytes fail closed (verify `--lib` + settle LiteSVM; no verifier crash; reserves unchanged)
  - [x] *Exit:* `--lib` harness green; LiteSVM random-proof settle fails closed

- [x] **Day 40 — Constraint audit + Month 2 tag**
  - [x] PDA `seeds`/`bump`/`has_one` pass (`clean_funds_root` seeds on settle; `has_one` + `Unauthorized` on register)
  - [x] Strip noisy `msg!` (pause / settle success / verify debug logs)
  - [x] Tag `v0.2-month2`
  - [x] *Exit:* Tag exists.

---

## Month 3: Deposit, Relayer, P1 402, Benchmarks, Audit & Devnet (Sessions 41–64)

Deposit through P1, benches, audit, and Devnet **58–64** are done (`v1.0.0-devnet`, settle `3V7YF2UL…`). Skip 62 send. Match [Architecture](docs/architecture.md) phases A–E.

### Phase 9: Deposit → leaf → registered root (3 sessions) — done

Couples vault custody + Merkle leaf + `CleanFundsRoot` so settle is not fixture-only.

- [x] **Day 41 — Deposit instruction + vault custody**
  - [x] User SPL transfer into vault; program records public leaf commitment (not secret)
  - [x] Update vault reserves; fail closed if mint/amount invalid
  - [x] *Exit:* Localnet deposit moves tokens into vault and emits/stores a leaf the client can keep as a note stub

- [x] **Day 42 — Tree insert + path serve**
  - [x] Indexer (or in-repo tree helper) inserts leaf; serves Merkle path under current tip
  - [x] Client can rebuild `PrivateInputs` from deposit note + path (no hand-edited `happy.json` required for this path)
  - [x] *Exit:* After one deposit, host `execute` succeeds against the live path/root

- [x] **Day 43 — Register root from deposit tip**
  - [x] Protocol admin (or deposit pipeline) registers tip as `CleanFundsRoot`
  - [x] E2E: deposit → register → prove → settle (still may use recorded Groth16 if live prove is heavy)
  - [x] *Exit:* Settle succeeds against a root that came from a real deposit, not only the fixture tree.

### Phase 10: Relayer, vault-only settle & Client SDK (6 sessions) — done

Aligned with Architecture phase E. Relayer is merchant + optional fee payer.

- [x] **Day 44 — Relayer scaffold**
  - [x] `relayer/` Axum (or Actix) service skeleton + health endpoint

- [x] **Day 45 — `/submit-intent` + worker queue**
  - [x] Accept private intent/witness; Tokio queue for prove-then-settle jobs

- [x] **Day 46 — Tx build + fee payer**
  - [x] Relayer builds `settle_shielded_spot` tx (proof + journal + account metas) and **signs as fee payer** (user need not pay SOL for settle)
  - [x] Document: fee payer ≠ note owner; journal still binds amount/mint/nullifier/root
  - [x] *Exit:* LiteSVM settle with distinct fee payer (lamports decrease); relayer `sign_settle_as_fee_payer` + dry-run. Fixtures only (no `$PROVE`).

- [x] **Day 47 — Fees + RPC retry + prove-treasury note**
  - [x] bps (or flat) fee from payout / deposit to refill **relayer SOL** (and, when wired, Succinct **prove credits**)
  - [x] Exponential backoff on send/confirm
  - [x] Short `notes/relayer-economics.md`: user fee → treasury → SOL + $PROVE (Architecture phase E)
  - [x] *Exit:* Failed RPC retries; fee path stubbed with tests.

- [x] **Day 48 — Vault-only shielded settle (product gap)**
  - [x] Settle no longer SPL-transfers journal mint from the payer’s ATA into the vault after deposit funded custody
  - [x] Deposit-backed spend: nullify → `apply_vault_only_spend` → vault pays other mint only (1:1 spot)
  - [x] Fail-closed: underfunded vault, bad proof, used nullifier, pause
  - [x] *Exit:* LiteSVM deposit 300 A → fixture spend 100 → no second A debit; `--lib` + settle/deposit tests green

- [x] **Day 49 — `@zk-shield/sdk` surface + recipient hooks**
  - [x] `generateProofInputs`, `requestShieldedSwap`, `getVaultReserves`; WebSocket/`CleanFundsRoot` sync helper as needed
  - [x] Optional: settle account metas for **recipient ATA ≠ depositor** (hidden-destination groundwork; full unlink may need fresh ATA + relayer submit)
  - [x] Usage examples in package README; point at Architecture phases C–E
  - [x] *Exit:* SDK drives deposit-backed **vault-only** settle; relayer can be fee payer.

### Phase 11: Shielded 402 payer (1 session) — done

- [x] **Day 50 — P1 shielded payer on localnet**
  - [x] Agent (or script) hits paid endpoint → `402` quote → pays via ZK Spot Shield settle (nullifier path), not a public wallet transfer as the privacy story
  - [x] Prefer relayer fee payer (Day 46) + vault-only settle (Day 48) when available
  - [x] *Exit:* One paid request (HTTP 402 + fixture journal); credential is amount + nullifier. **P1** checked. Quote still 100 / `[3u8; 32]` (`pay402.rs`).

### Phase 12: Benchmarking (3 sessions) — done

- [x] **Day 51 — E2E latency**
  - [x] Prove → relayer → confirm; write numbers to `notes/budgets.md`

- [x] **Day 52 — Prove hardware + on-chain CU breakdown**
  - [x] CPU vs GPU prove times
  - [x] CU split (verify / checks / SPL)
  - [x] *Exit:* Numbers live in `notes/` (or Day 29/18 docs); README links them

- [x] **Day 53 — Payload + throughput**
  - [x] Minimize journal bytes
  - [x] Concurrent localnet submission smoke

### Phase 13: Security Audit Readiness (4 sessions) — done

- [x] **Day 54 — Threat model doc**
  - [x] Circuit soundness, replay, pause, protocol admin, root registry, deposit unbacked-leaf (A18)
  - [x] *Exit:* `docs/security.md` exists; README links it

- [x] **Day 55 — Layout + signer audit**
  - [x] `#[repr(C)]` / padding review
  - [x] Every ix path checks signer/writable explicitly

- [x] **Day 56 — Tooling scans**
  - [x] `cargo-audit` (+ any chosen program analyzers)
  - [x] Fix high findings or document waivers

- [x] **Day 57 — Bytecode freeze prep**
  - [x] Reproducible build notes; record program hash procedure

### Phase 14: Devnet Launch — done

Program + SDK on Devnet. Skip Day 62 send. UI is Days 66 / 81. Days 67–68 Devnet walkthrough + live tests done.

Recorded cluster: `https://api.devnet.solana.com`. Tables: [`notes/devnet.md`](notes/devnet.md).

- [x] **Day 58 — Devnet keys + SOL**
  - [x] Cluster Devnet; admin `9sLGgVEotwD8gJgL5HhJdP7w61zeCrJ25NNkJPM3BAaX` (`~/.config/solana/id.json`); funded
  - [x] *Exit:* pubkey + balance in `notes/devnet.md`

- [x] **Day 59 — Deploy program**
  - [x] Program `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G`; source `declare_id!` pinned; upgrade dump SHA-256 `f9030f95…` matched local freeze
  - [x] *Exit:* program id + hash in `notes/devnet.md` + `notes/budgets.md`

- [x] **Day 60 — Init + vkey + roots**
  - [x] `GlobalConfig` `BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY`; toy `CleanFundsRoot` `6CvBuqnxswcq8g7QJcybGRtGragupRzNdmW37fD7NMLA` (happy fixture `expected_root`)
  - [x] *Exit:* both pubkeys in `notes/devnet.md`

- [x] **Day 61 — Set vault mints + ATAs**
  - [x] `VaultState` PDA `A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd` (`initialize_vault`)
  - [x] Admin `set_vault_mints` writes `mint_a` / `mint_b` once
  - [x] `--lib` + clippy; `anchor build`; upgrade `74UW…`; dump SHA-256 `381ba3fb…` matched
  - [x] Toy mints `DNNk…` / `Hfgiz…`; vault ATAs `EVmR…` / `8wBh…`
  - [x] *Exit:* non-default mints + ATA addresses in `notes/devnet.md`

- [x] **Day 62 — Relayer on Devnet RPC** (skip send)
  - [x] Local `/health` ok; dry-run; `RELAYER_RPC` unused
  - [x] *Skip send:* self-submit as fee payer

- [x] **Day 63 — Live shielded swap (SDK, not UI)**
  - [x] Deposit 1e6 of each mint into vault ATAs (`reserve_*` = 1000000)
  - [x] Poseidon 32-byte inputs reduce into BN254 Fr; admin `set_vkey_hash`
  - [x] Groth16 journal mint = vault `mint_a` (`DNNk…`); wrap in `sp1-artifacts/devnet-settle/`
  - [x] `settle_shielded_spot` `3V7YF2UL…`; `reserve_a`/`reserve_b` = 1000000 / 900000
  - [x] *Exit:* settle signature in `notes/devnet.md`

- [x] **Day 64 — Docs + `v1.0.0-devnet`** (after 63 Exit only)
  - [x] README + `notes/devnet.md` tables for 58–63; current program hash recorded
  - [x] Tag `v1.0.0-devnet` on `30c35d7` (live settle recorded)
  - [x] *Exit:* A stranger can deploy/init from docs.

---

## Month 3+ buffer: Public artifact, UI, change notes, backup, unlink, agent (Sessions 65–83)

Days **65–84** + **P0** + **67–68** done. Toy pack stays localnet. No AMM.

### Phase 15: Public shield artifact (1 session) — done

Assemble what already exists (execute, Groth16 or honest block, threat model, CU) into **one** repo front door. A staff engineer should understand the system from README + linked notes without a browser app.

- [x] **Day 65 — Public artifact (staff README)**
  - [x] Root `README.md` a staff engineer respects: what the protocol does, trust model, crate map, how to **execute** the guest, how to **prove** (remote Groth16 command) **or** a dated honest “blocked on RAM / no Groth16 file” in `notes/proving.md` — never a fake proof
  - [x] Link Day 54 threat model (circuit, replay, pause, protocol admin, root registry, deposit)
  - [x] Link CU notes (Days 18 / 29 / 52): verifier budget, measured settle CU if you have it
  - [x] How to build program + run `--lib` tests; program id / Devnet pointers if Days 58–64 ran
  - [x] *Exit:* A cold reader can follow README → execute → (Groth16 **or** documented block) → threat model → CU numbers. No UI this Day.

### Phase 16: Basic swap UI (1 session) — done

Ship a swap screen. Relayer optional.

- [x] **Day 66 — Basic swap UI**
  - [x] Minimal web UI: wallet connect, show vault reserves / clean root / pause, swap amount + asset (1:1 spot — not AMM), submit settle (via relayer **or** wallet as fee payer)
  - [x] Load or paste Groth16 + journal (in-browser prove is out of scope; host execute + remote prove stay off the critical UI path)
  - [x] **Load localnet demo** fills ATAs + 144-byte happy pack; UI shows vault + recipient token amounts (not only explorer)
  - [x] Talks to the SDK from Day 49; works against **localnet** today
  - [x] Optional UI copy: “shielded settle” vs clear note that deposit still shows depositor on-chain until a later privacy Day
  - [x] *Exit:* Localnet click-through: vault B `1000 → 900`, recipient B `0 → 100`, tx `4vS2MTi6…`. Fixture only (spend 100 read-only; no Deposit button). This is P2 — not P4.

### Phase 17: Devnet UI walkthrough + tests (2 sessions)

P0 compare is in `notes/budgets.md`.

- [x] **Day 67 — Manual Devnet walkthrough**
  - [x] Point the Day 66 UI at Devnet (program id + RPC; `?cluster=devnet` / agent **3**)
  - [x] Human checklist in `notes/devnet.md`: start processes → On-chain panel → Explorer vault/config → optional wallet + public/deposit/shielded submit
  - [x] Explorer URLs for program, config, vault, ATAs + sample public/shielded txs
  - [x] *Exit:* Checklist + Explorer URLs in `notes/devnet.md` (Day 67 section)

- [x] **Day 68 — Automated on-chain Devnet tests**
  - [x] TS Vitest against live Devnet (`sdk/test/devnet/onchain.devnet.test.ts`) — not LiteSVM
  - [x] Factored helpers (`sdk/test/devnet/helpers.ts`): faucet SOL, mint A/B, deposit, done-markers
  - [x] Positive: `reads`, `pause`, `deposit_a`/`deposit_b`, `public_swap`
  - [x] Negative: `neg_zero_swap`, `neg_underflow`, `neg_paused_swap` (shielded settle stays manual / Groth16)
  - [x] Passed cases ignored via `sdk/.devnet-e2e-done.json`; catalog + commands in `notes/devnet.md`
  - [x] Command: `npm run test:devnet` (`DEVNET_FORCE=1` / `DEVNET_CASES=` to re-run or subset); CI skips when unset
  - [x] *Exit:* Live E2E green; sample pause/unpause + case table recorded in `notes/devnet.md`

### Phase 18: Mirrors / CID (parked)

Self-submit and a fallback RPC list are already in `ui/`. CID and a second host are **not** this demo.

- [x] **Day 69 — Static self-submit (no required backend)**
  - [x] Same swap UI as Day 66: works from `file://` or a local static server **without** `relayer/` running
  - [x] Default path is **user wallet as fee payer**; relayer URL stays optional
  - [x] *Exit:* Localnet settle can be sent with relayer down. This is hardening, not the first UI. UI self-submit with Relayer URL empty; tx `4vS2MTi6…`.

- [ ] **Day 70 — Multi-RPC + CID**
  - [x] Configurable RPC fallback list
  - [x] `ui/dist` sha256 in `ui/README.md`
  - [ ] Publish a CID
  - [ ] *Exit:* Second RPC + CID recorded

- [ ] **Day 71 — Relayer-optional settle**
  - [x] Self-submit with relayer down
  - [ ] Optional: two public relayer URLs
  - [ ] *Exit:* Seizure note already in `ui/README.md`; CID still unpublished

- [ ] **Day 72 — Mirrors + seizure runbook**
  - [x] Rebuild + hash runbook in `ui/README.md`
  - [ ] Second independent host
  - [ ] *Exit:* Stranger can follow the runbook if the primary URL is gone

Prove stays remote. Anyone can host the UI and submit the tx.

### Phase 19: Change notes (3 sessions) — done

Partial spend: nullify old note, insert change leaf for remainder.

- [x] **Day 73 — Circuit: change output**
  - [x] Guest: after solvency, commit change note leaf (`balance - amount`) under same or new secret; journal/public shape updated as needed
  - [x] Fail closed if change + spend ≠ old balance
  - [x] *Exit:* `execute` happy path with spend 100 / change 900; underfunded still panics. Host execute printed `change=900`. Groth16 wrap landed later as the 144-byte happy pack.

- [x] **Day 74 — Settle: insert change leaf + register tip**
  - [x] On successful settle, protocol path inserts change leaf (or emits event the indexer must append) and registers new root when required
  - [x] Old nullifier still one-time; change note is a new spendable membership
  - [x] *Exit:* Settle inserts the 900 change-leaf PDA (`fixtures/happy/` 144-byte pack). Spend-of-change still needs admin `register_clean_funds_root` (P4 loop blocker).

- [x] **Day 75 — Change-note E2E + docs**
  - [x] Negative: replay old note → `NullifierAlreadyUsed`; spend change note succeeds once
  - [x] Docs/glossary: note vs change note; update Architecture phase E + threat model if journal fields grew
  - [x] *Exit:* `--lib` uses the 144-byte happy pack. `sdk npm run test:e2e` 4/4.

### Phase 20: Encrypted note backup (1 session) — done

Export an encrypted blob; decrypt locally to restore `{secret, balance, address, leaf_index}` (and change-note fields). Not plaintext on-chain.

- [x] **Day 76 — Encrypted note backup**
  - [x] Client export / import of an encrypted note blob (passphrase or similar); round-trip restore in tests
  - [x] Cover a change note (remainder after spend), not only a fresh deposit
  - [x] *Exit:* Wipe in-memory note, restore from the blob, spend once on localnet. `sdk` `npm run test:e2e` 4/4 — happy path restores fixture note then settles (`2bDC9vCw…`).

### Phase 21: Hide / unlink depositor (3 sessions) — done

Deposit still writes the user’s wallet on `DepositLeafAccount`. Settle still lists recipient token accounts. SDK Day 49 only allows a different ATA — it does not hide the deposit. Do this **after** backup so a user who rotates receive addresses can still restore notes. Not a mixer: the deposit tx remains on the ledger.

- [x] **Day 77 — Omit depositor from the leaf PDA**
  - [x] Stop storing a raw depositor pubkey on `DepositLeafAccount` (omit or replace with a non-identifying commitment). `init` uniqueness stays `["deposit_leaf", leaf]`
  - [x] Indexer / SDK still get leaf, amount, mint from the account or `DepositEvent`
  - [x] *Exit:* `--lib` 39 + LiteSVM `--tests` 23 (deposit 3 + init 5 + settle 15). Leaf PDA has no depositor wallet. A22 already in `docs/security.md`.

- [x] **Day 78 — Settle recipient unlink**
  - [x] Default happy path: payout ATAs are **not** the depositing wallet’s ATAs (fresh receive account + relayer or self-submit as fee payer)
  - [ ] Optional: bind recipient in the journal so a relayer cannot redirect (if the journal grows, new Groth16 fixture)
  - [x] *Exit:* Localnet e2e settle credits `recipientTokenB` (owner R); change-leaf + nullifier PDAs omit D. Journal-bound recipient skipped (would need a new Groth16).

- [x] **Day 79 — Unlink E2E + docs**
  - [x] Negative: observer with only the leaf PDA cannot read a depositor field
  - [x] Architecture phase E, glossary, `docs/security.md` A22: what is still linkable (deposit tx, unique amounts, token-account graph)
  - [x] *Exit:* Matches Days 77–78. Phase E + A22: leaf PDA has no depositor; settle may pay wallet R; still linkable = deposit tx, unique amounts, token-account graph. LiteSVM + e2e already green.

### Phase 22: Agentic buy loop — done

Same wallet for CLI and UI. Relayer is merchant + optional SOL fee payer, not the agent. `$PROVE` is Succinct credits in gitignored `.env`.

- [x] **Day 80 — Agent CLI (localnet)**
  - [x] HELP: `1` Buy Token B · `2` Deposit Token A · `3` Balance · `EXIT`
  - [x] Deposit A: SPL into vault + leaf (no Groth16)
  - [x] Buy: 402 → pick covering note → Groth16 → vault-only settle → change + admin root
  - [x] *Exit:* Localnet buys with live Groth16. Same keypair as the UI. `cd sdk && npm run agent`

- [x] **Day 81 — UI same loop**
  - [x] Deposit / buy / balances via agent loopback (`127.0.0.1:8790`)
  - [x] *Exit:* Same loop as Day 80 from the swap UI

- [x] **Day 82 — First Devnet agent buy**
  - [x] Mint-matched wrap (`DNNk…`). Toy `0x03…03` is localnet only
  - [x] `AGENT_CLUSTER=devnet` loads vault `DNNk…` / `Hfgiz…` and the wallet’s ATAs
  - [x] Nullifier sync at boot, before buy, and every 60s (skip notes whose PDA already exists)
  - [x] *Exit:* Settle `3CpfyWL6…` (and five more) in `notes/devnet.md`

- [x] **Day 83 — Devnet loop**
  - [x] Repeat buys until no unspent note covers a quote; all 11 notes spent
  - [x] *Exit:* Eleven Devnet settle signatures in `notes/devnet.md`

- [x] **Day 84 — Non-shielded Swap A→B**
  - [x] Program `public_spot_swap`: payer transfers mint A in; vault pays mint B (1:1); respects pause
  - [x] Agent HELP `4` + HTTP `POST /swap-public`; UI row **Swap Non-Shielded**
  - [x] Relayer `POST /402/pay-public` + `GET /402/resource?signature=` (`unlock=wallet`)
  - [x] *Exit:* Devnet public swap `27LEe6SB…` (11 A→B, unlock=wallet); shielded buy still works (`0xbe631e5f…` prove + settle earlier same session)

---

## Tech debt (extra — not a Day)

Do when they unblock toolchain/CI. Do not pull them mid-Day.

- [x] **Anchor `programs/*/` layout** — Move on-chain crate from `program/` → `programs/zk_spot_shield/` so AVM/Anchor discover `anchor-lang` under the standard path. Workspace + `Anchor.toml` members, LiteSVM `.so` path, docs/rules/CI updated. *Exit:* `cargo test -p zk_spot_shield --lib` and `anchor build --ignore-keys` succeed.

---

## Writing checkpoints (after Day 83 + P0 + 67–68)

Each Write is **one Medium/LinkedIn piece**: markdown article + a short click-through video (UI and/or terminal) that the post can embed or link. Day 67 checklist is the Devnet UI script; reuse it where the topic is on-chain settle / public path.

Video: ≤ ~2–3 min, silent captions OK; store path or URL in the article frontmatter / a one-line `video:` note (do not commit large binaries — link YouTube/Loom/unlisted or keep local under gitignored `notes/articles/_video/`).

- [ ] **Write-1 — What the circuit proves**
  - [ ] Private vs public I/O, Merkle inclusion + solvency, nullifier
  - [ ] Short video: prove path (agent Groth16 / host log) or diagram walk — not a full settle unless short
  - [ ] *Exit:* `docs/articles/zk-spot-shield-part1.md` (+ HTML preview) + video link in that file (`docs/articles/assets/` for PNGs)

- [ ] **Write-2 — Settle on Solana**
  - [ ] Verify Groth16, registered root, unused nullifier, vault + SPL
  - [ ] Short video: Devnet or localnet UI/agent settle click-through (Day 67 script)
  - [ ] *Exit:* `docs/articles/zk-spot-shield-part2.md` + video link in that file

- [ ] **Write-3 — Public artifact**
  - [ ] Membership ≠ mixer; execute vs remote Groth16; how to read the README
  - [ ] Short video: public vs shielded contrast (e.g. Swap Non-Shielded vs Buy) or README tour
  - [ ] *Exit:* `docs/articles/zk-spot-shield-part3.md` + video link in that file; ready to paste Medium + LinkedIn
