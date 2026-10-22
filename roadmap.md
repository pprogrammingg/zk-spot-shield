# ZK-Shielded Spot Settlement Engine — Session Roadmap (+ private agent payments over HTTP 402; payment-flow UI planned as a separate repo)

Calibrated for **~83 sessions** (Days **0–79** engineering + **3 writing checkpoints**). Each engineering Day is 1.5 hours (3 × 25 min). Writing checkpoints are also full sessions, but they are **not** design sprints — see the visual budget below.

Mark items `[x]` when done. A Day is **done** only when its *Exit* criteria pass — not when the code “mostly exists.”

---

## Build priority calendar (do not reorder)

Product north star: **private settlement rail** → **shielded HTTP 402 payer**. Core localnet E2E (Days 36–40) and **deposit → leaf → root (Days 41–43)** plus relayer scaffold (Days 44–45) are done. Docs architecture phases A–E spell the same story (built vs upcoming).

Next engineering follows this order — not “whatever Day number looks fun.”

| Rank | Priority | Calendar home | Why |
| --- | --- | --- | --- |
| **1** | **Relayer fee payer + fee economics** | **Days 46–47** ✓ | Relayer builds/signs settle as fee payer; fee stub + RPC backoff; `notes/relayer-economics.md` |
| **2** | **Vault-only shielded settle** | **Day 48** ✓ | Deposit-backed spend: vault pays other mint only — **no** second user→vault of journal mint |
| **3** | **SDK + optional hidden recipient hooks** | **Day 49** ✓ | Drive deposit-backed settle; recipient ATA ≠ depositor; relayer settle-intent encode |
| **4** | **Shielded 402 payer (P1)** | **Day 50** ✓ | HTTP 402 quote → pay with settle journal (nullifier path), fixtures — no `$PROVE` |
| — | **Live Devnet swap (SDK)** | **Day 63** ✓ | Settle `3V7YF2UL…`; journal mint `DNNk…`; `reserve_b` 1e6 → 9e5 |
| — | **Docs + `v1.0.0-devnet` tag** | **Day 64** ✓ | Tag on `30c35d7`; settle recorded in `notes/devnet.md` |
| **Now** | **Change notes** (spend 100, keep 900) | **Days 73–75** | Product gap. After Devnet 64. **Do not wait** for uncensorable UI |
| **Next** | **Encrypted note backup** | **Day 76** | Wiped device can restore `{secret, balance, address, leaf_index}`. After change notes so the backup format includes remainder notes |
| **7** | **Hide / unlink depositor** | **Days 77–79** | Leaf PDA without raw depositor; settle recipient ≠ deposit wallet. After backup so remaining notes are restorable when the deposit link is cut |
| **8** | **Uncensorable UI / extra RPC** | **Days 69–72** | Self-submit, multi-RPC, IPFS mirrors — **back seat** until 73–79 Exit |
| — | **Relayer on Devnet RPC** | **Day 62** skip | Local dry-run server only; `RELAYER_RPC` unread. UI self-submit is enough |
| — | AMM curve (“100 A → 50 B”) | Not a Day | Stay **1:1 spot** unless product explicitly pivots |
| — | Same-mint deposit/withdraw pool | Parked | 1:1 A→B is a dummy; USDC-in/USDC-out is a later track, not ahead of 73–79 |

**Hard rule:** Next session is **Days 73–75** (change notes). Skip **62** send. Do not start 76 mid-session. After 73–75 Exit: **76 → 77–79 → 69–72**. Do not invent AMM Days.

---

## Track: private agent payments over HTTP 402 (hackathons · jobs · funding)

**Why:** ZK Spot Shield becomes the private payer for machine payments — an AI agent pays an API over HTTP 402 (Stripe MPP / Coinbase x402 on Solana) from a shielded note, so the payment settles without exposing the agent's wallet or balance. This is the rail Stripe and Coinbase are building, which makes the project a hackathon entry, a job-loop talking point and, if traction shows up, a fundable startup.

**Scope rule:** Core slice (verify → settle → localnet E2E) and deposit (41–43) are **done**. P0 plain 402 continues on Mondays. **P1 shielded payer is Day 50** (after relayer fee payer + vault-only settle + SDK Days 46–49). Do not start P2 UI repo before Day 50 Exit.

### Milestones

- [ ] **P0 — Plain 402 demo (Mondays, from Oct 12):** agent hits a paid endpoint → `402` with price → pays USDC on Solana devnet (Circle faucet) → server verifies → returns data. Published early. No Stripe approval needed on devnet.
- [x] **P1 — Shielded payer (= Day 50):** the agent's payment comes from a ZK Spot Shield settle instead of a public wallet. *Exit:* one paid request on localnet where the chain sees amount + nullifier, not the payer. Requires Days 41–43 deposit Exit.
- [ ] **P2 — Payment-flow UI (separate repo, after Day 50):** small web app showing the flow end-to-end — request → 402 quote → shielded pay → response — with what the chain sees vs what stays private. Lives in its own repo (e.g. `zk-spot-shield-pay`) that depends on this one; this repo stays program + prover + SDK.
- [ ] **P3 — Stripe sandbox path (optional):** same demo through the Stripe MPP sandbox (cards / Tempo testnet). Real USDC-on-Solana into a Stripe balance needs approval via `machine-payments@stripe.com` — only once there is a real user.

### Where it gets shown

| When | Stage | What to show |
| --- | --- | --- |
| Oct 12 | Colosseum World's Fair submission | Core proof flow + docs site; 402 track as the roadmap |
| Weekly | Colosseum project updates / Eternal | What shipped, what's next (1-min video when due) |
| Nov 3 | Stripe Sessions 2027 talk proposal | "Private machine payments…" + P0 numbers; P1 if Day 50 landed |
| Nov 13–15 | Agentic Payments Hackathon (Toronto) | Mentor on Solana ZK + 402; demo P1 (Day 50) if ready |
| After Day 50 | Local Solana meetup showcase | P1 live demo |

### Funding (only if traction)

Colosseum Eternal → accelerator (up to US$250K pre-seed, very selective). Not a goal on its own: the primary outcome is a top-tier job offer; funding is considered only if real users or a design partner appear. Revisit after P1.

---

### Session rhythm (3 × 25)

```
[P1 25m] Context + design: reopen yesterday's artifact, name today's exit criteria.
[P2 25m] Build: implement the core of today's deliverable.
[P3 25m] Prove: compile, run the smallest meaningful test, commit.
```

**After commit (2 min):** post a Colosseum project update (Updates → Post an update): what shipped today, one line on what's next. Add a 1-min video when the week's update is due.

Do not split a Day across sessions. If you finish early, deepen tests or docs for *that* deliverable — do not pull tomorrow's Day forward mid-session. If the leftover is writing, you may start that checkpoint’s **LinkedIn draft only** (not a new diagram set).

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
  - [x] *Exit:* Tag + README commands exist. **Next session is Write-1** (circuit article + LinkedIn), not Day 21.

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
  - [x] *Exit:* Tag exists. **Next session is Write-2** (settle article + LinkedIn), then **Day 41 deposit** (not relayer — see priority calendar).

---

## Month 3: Deposit, Relayer, P1 402, Benchmarks, Audit & Devnet (Sessions 41–64)

Order is fixed by the [Build priority calendar](#build-priority-calendar-do-not-reorder): deposit through P1 and bench/audit are **done**. Devnet **58–64** recorded (live settle + `v1.0.0-devnet`). Skip 62 send. **Next: Days 73–75**. Match [Architecture](docs/architecture.md) phases A–E.

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
  - [x] *Exit:* Settle succeeds against a root that came from a real deposit, not only the fixture tree. **Next: Phase 10 relayer.**

### Phase 10: Relayer, vault-only settle & Client SDK (6 sessions)

Aligned with Architecture **phase E** (upcoming) and remaining ranks 1–3.

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
  - [x] *Exit:* Failed RPC retries; fee path stubbed with tests. **Next was Day 48 vault-only settle.**

- [x] **Day 48 — Vault-only shielded settle (product gap)**
  - [x] Settle no longer SPL-transfers journal mint from the payer’s ATA into the vault after deposit funded custody
  - [x] Deposit-backed spend: nullify → `apply_vault_only_spend` → vault pays other mint only (1:1 spot)
  - [x] Fail-closed: underfunded vault, bad proof, used nullifier, pause
  - [x] *Exit:* LiteSVM deposit 300 A → fixture spend 100 → no second A debit; `--lib` + settle/deposit tests green

- [x] **Day 49 — `@zk-shield/sdk` surface + recipient hooks**
  - [x] `generateProofInputs`, `requestShieldedSwap`, `getVaultReserves`; WebSocket/`CleanFundsRoot` sync helper as needed
  - [x] Optional: settle account metas for **recipient ATA ≠ depositor** (hidden-destination groundwork; full unlink may need fresh ATA + relayer submit)
  - [x] Usage examples in package README; point at Architecture phases C–E
  - [x] *Exit:* SDK drives deposit-backed **vault-only** settle; relayer can be fee payer. **Next session is Day 50 (P1).**

### Phase 11: Shielded 402 payer (1 session) — priority 4 on calendar / demo rank 1

- [x] **Day 50 — P1 shielded payer on localnet**
  - [x] Agent (or script) hits paid endpoint → `402` quote → pays via ZK Spot Shield settle (nullifier path), not a public wallet transfer as the privacy story
  - [x] Prefer relayer fee payer (Day 46) + vault-only settle (Day 48) when available
  - [x] *Exit:* One paid request (HTTP 402 + fixture journal); chain-facing credential is amount + nullifier, not a wallet. Milestone **P1** checked. Mondays may still polish P0.

### Phase 12: Benchmarking (3 sessions)

- [x] **Day 51 — E2E latency**
  - [x] Prove → relayer → confirm; write numbers to `notes/budgets.md`

- [x] **Day 52 — Prove hardware + on-chain CU breakdown**
  - [x] CPU vs GPU prove times
  - [x] CU split (verify / checks / SPL)
  - [x] *Exit:* Numbers live in `notes/` (or Day 29/18 docs); Day 65 README will link them

- [x] **Day 53 — Payload + throughput**
  - [x] Minimize journal bytes
  - [x] Concurrent localnet submission smoke

### Phase 13: Security Audit Readiness (4 sessions)

- [x] **Day 54 — Threat model doc**
  - [x] Circuit soundness, replay, pause, protocol admin, root registry, deposit unbacked-leaf (A18)
  - [x] *Exit:* A named doc (`docs/security.md`) a reviewer can open; Day 65 README links it

- [x] **Day 55 — Layout + signer audit**
  - [x] `#[repr(C)]` / padding review
  - [x] Every ix path checks signer/writable explicitly

- [x] **Day 56 — Tooling scans**
  - [x] `cargo-audit` (+ any chosen program analyzers)
  - [x] Fix high findings or document waivers

- [x] **Day 57 — Bytecode freeze prep**
  - [x] Reproducible build notes; record program hash procedure

### Phase 14: Devnet Launch (remaining **1** session: 64 tag)

Program + SDK on Devnet. **No browser UI yet** (that is Day 66). Day 65 README already exists. Finished-product Devnet tests are Days 67–68 (after 63). Skip Day 62.

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
  - [x] *Exit:* settle signature in `notes/devnet.md`. **Not** a browser product (UI is Day 66+)

- [x] **Day 64 — Docs + `v1.0.0-devnet`** (after 63 Exit only)
  - [x] README + `notes/devnet.md` tables for 58–63; current program hash recorded
  - [x] Tag `v1.0.0-devnet` on `30c35d7` (live settle recorded)
  - [x] *Exit:* A stranger can deploy/init from docs. Browser product + Devnet UI tests are Days 66–68. **Next: Day 73**

---

## Month 3+ buffer: Public artifact, basic UI, then change notes + backup + unlink (Sessions 65–79)

Follow the [calendar](#build-priority-calendar-do-not-reorder), not Day number. **Next session is Days 73–75** (change notes). Then **76 → 77–79**. Days **69–72** wait until those Exit. Skip **62** send.

Day **65** (README) is done. Day **66** has a swap screen; Exit still needs a localnet click-through. Devnet **58–64** are recorded (program `74UW…`, live settle `3V7YF2UL…`, tag `v1.0.0-devnet`). Do **not** pull AMM work.

### Phase 15: Public shield artifact (1 session) — **before UI**

Assemble what already exists (execute, Groth16 or honest block, threat model, CU) into **one** repo front door. A staff engineer should understand the system from README + linked notes without a browser app.

- [x] **Day 65 — Public artifact (staff README)**
  - [x] Root `README.md` a staff engineer respects: what the protocol does, trust model, crate map, how to **execute** the guest, how to **prove** (remote Groth16 command) **or** a dated honest “blocked on RAM / no Groth16 file” in `notes/proving.md` — never a fake proof
  - [x] Link Day 54 threat model (circuit, replay, pause, protocol admin, root registry, deposit)
  - [x] Link CU notes (Days 18 / 29 / 52): verifier budget, measured settle CU if you have it
  - [x] How to build program + run `--lib` tests; program id / Devnet pointers if Days 58–64 ran
  - [x] *Exit:* A cold reader can follow README → execute → (Groth16 **or** documented block) → threat model → CU numbers. **No UI this Day.** Clone-and-read is the deliverable. **Write-3** is the public-artifact article. (Current next session is **Days 73–75** change notes.)

### Phase 16: Basic swap UI (1 session)

Ship a normal swap screen first. Relayer and a local HTTPS/static server are allowed. This is **not** the uncensorable phase.

- [ ] **Day 66 — Basic swap UI**
  - [x] Minimal web UI: wallet connect, show vault reserves / clean root / pause, swap amount + asset (1:1 spot — not AMM), submit settle (via relayer **or** wallet as fee payer)
  - [x] Load or paste Groth16 + journal (in-browser prove is out of scope; host execute + remote prove stay off the critical UI path)
  - [x] Talks to the SDK from Day 49; works against **localnet** today
  - [x] Optional UI copy: “shielded settle” vs clear note that deposit still shows depositor on-chain until a later privacy Day
  - [ ] *Exit:* You can click through a swap in the browser on localnet and see balances change in the UI (not only logs / explorer). No IPFS, no multi-RPC, no seizure runbook today. **Needs your localnet click-through.**

### Phase 17: Finished product on Devnet — see it and test it (2 sessions) — after 58–64; does not block 73–76

Needs Day **61** Exit (done) and a Day **63** settle. Skip until the settle signature is in `notes/devnet.md`. Do **not** start this instead of Day 63.

- [ ] **Day 67 — Manual Devnet walkthrough (see the end product)**
  - [ ] Point the Day 66 UI at Devnet (program id + RPC from Days 58–61); fund a wallet if needed
  - [ ] Human checklist in `notes/devnet.md`: connect wallet → read on-chain vault/root in the UI → submit (or dry-run + one live settle if a proof is available) → open Solana Explorer links for accounts + tx
  - [ ] Record at least one Explorer URL for vault/config and, if settle ran, the tx signature
  - [ ] *Exit:* A person who is not the session agent can follow the checklist and **see** the product (UI + live accounts). Screenshots optional; the written checklist + Explorer URLs are required.

- [ ] **Day 68 — Automated on-chain Devnet tests**
  - [ ] Integration tests (TS and/or Rust) with **cluster = Devnet** — `solana-test-validator` / LiteSVM do **not** count
  - [ ] Tests must **interact on-chain**: fetch `GlobalConfig` + `VaultState` (and ATAs) from Devnet; assert layout / reserves / vkey
  - [ ] At least one **write** that lands on Devnet (protocol-admin no-op such as pause→unpause, or a real `settle_shielded_spot` if a Groth16 fixture exists)
  - [ ] One command documented (e.g. `ANCHOR_PROVIDER_URL=https://api.devnet.solana.com …`); skip-with-fail if key/SOL missing so CI-default stays localnet
  - [ ] *Exit:* That command passes against live Devnet; tx signatures or account pubkeys from the run are in the test output or `notes/devnet.md`

### Phase 18: Decentralized / uncensorable swap UI (4 sessions) — calendar rank 8, after Day 79

Self-submit and a fallback RPC list are already in `ui/`. This phase is **mirrors + CID + seizure runbook**. Park it until change notes (73–75), note backup (76), and unlink (77–79) Exit.

- [ ] **Day 69 — Static self-submit (no required backend)**
  - [x] Same swap UI as Day 66: works from `file://` or a local static server **without** `relayer/` running
  - [x] Default path is **user wallet as fee payer**; relayer URL stays optional
  - [ ] *Exit:* Localnet settle can be sent with relayer down. This is hardening, not the first UI. **Needs your check (relayer down).**

- [ ] **Day 70 — Multi-RPC + no single origin**
  - [x] Configurable RPC list (fallback if one endpoint censors or dies)
  - [x] Document IPFS/Arweave (or equivalent) publish of the **same** static bundle; pin CID in README
  - [ ] *Exit:* UI works against a second RPC; a content-addressed build hash/CID is recorded. **CID not published yet — hash `ui/dist` after build.**

- [ ] **Day 71 — Relayer-optional settle**
  - [ ] SDK/UI: “submit myself” vs “ask relayer”; shutdown of relayer must not block the self-submit path
  - [ ] Optional: two public relayer URLs, client tries in order
  - [ ] *Exit:* Written threat note: seizing the marketing site ≠ seizing settle; users keep CID + program id

- [ ] **Day 72 — Mirrors + seizure runbook**
  - [ ] README: how to rebuild the UI, where CIDs live, how to verify the bundle hash
  - [ ] At least two independent hosts (e.g. IPFS gateway + GitHub Pages / Pages-from-CID)
  - [ ] *Exit:* Runbook a stranger can follow if the primary URL is gone; record it in `ui/README.md`.

**Out of scope here:** fully trustless in-browser Groth16 on a phone. Prove can stay remote; uncensorable means **anyone can host the UI and submit the tx**, not that prove is free or local.

### Phase 19: Change notes (3 sessions) — calendar rank 5 — after Day 64

Partial spend without burning the whole note balance. UTXO-style: nullify old note, insert change leaf for remainder. Day **48** vault-only settle already landed. Start after Devnet **61 → 63 → 64**. Do **not** wait for Days 69–72.

- [ ] **Day 73 — Circuit: change output**
  - [ ] Guest: after solvency, commit change note leaf (`balance - amount`) under same or new secret; journal/public shape updated as needed
  - [ ] Fail closed if change + spend ≠ old balance
  - [ ] *Exit:* `execute` happy path with spend 100 / change 900; underfunded still panics

- [ ] **Day 74 — Settle: insert change leaf + register tip**
  - [ ] On successful settle, protocol path inserts change leaf (or emits event the indexer must append) and registers new root when required
  - [ ] Old nullifier still one-time; change note is a new spendable membership
  - [ ] *Exit:* Localnet: deposit 1000 → settle 100 → client holds change note 900 that can be proven later

- [ ] **Day 75 — Change-note E2E + docs**
  - [ ] Negative: replay old note → `NullifierAlreadyUsed`; spend change note succeeds once
  - [ ] Docs/glossary: note vs change note; update Architecture phase E + threat model if journal fields grew
  - [ ] *Exit:* Checklist + `--lib` / e2e green; roadmap change-notes rank done. **Next: Day 76 note backup.**

### Phase 20: Encrypted note backup (1 session) — calendar rank 6

Wiped phone or lost browser profile should not mean the note is gone. Export an encrypted blob the user can store elsewhere; decrypt locally to restore `{secret, balance, address, leaf_index}` (and change-note fields from Days 73–75). Not an NFT. Not plaintext balance on-chain.

- [ ] **Day 76 — Encrypted note backup**
  - [ ] Client export / import of an encrypted note blob (passphrase or similar); round-trip restore in tests
  - [ ] Cover a change note (remainder after spend), not only a fresh deposit
  - [ ] *Exit:* Wipe the in-memory note, restore from the blob, prove/spend the restored note once on localnet. **Next: Days 77–79 (hide/unlink depositor).**

### Phase 21: Hide / unlink depositor (3 sessions) — calendar rank 7 — after Day 76

Deposit still writes the user’s wallet on `DepositLeafAccount`. Settle still lists recipient token accounts. SDK Day 49 only allows a different ATA — it does not hide the deposit. Do this **after** backup so a user who rotates receive addresses can still restore notes. Not a mixer: the deposit tx remains on the ledger.

- [ ] **Day 77 — Omit depositor from the leaf PDA**
  - [ ] Stop storing a raw depositor pubkey on `DepositLeafAccount` (omit or replace with a non-identifying commitment). `init` uniqueness stays `["deposit_leaf", leaf]`
  - [ ] Indexer / SDK still get leaf, amount, mint from the account or `DepositEvent`
  - [ ] *Exit:* `--lib` + LiteSVM deposit; account data has no wallet pubkey; `docs/security.md` A22 updated

- [ ] **Day 78 — Settle recipient unlink**
  - [ ] Default happy path: payout ATAs are **not** the depositing wallet’s ATAs (fresh receive account + relayer or self-submit as fee payer)
  - [ ] Optional: bind recipient in the journal so a relayer cannot redirect (if the journal grows, new Groth16 fixture)
  - [ ] *Exit:* Localnet: deposit from wallet D, settle pays wallet R; leaf PDA and nullifier PDA do not contain D

- [ ] **Day 79 — Unlink E2E + docs**
  - [ ] Negative: observer with only the leaf PDA cannot read a depositor field
  - [ ] Architecture phase E, glossary, `docs/security.md` A22: what is still linkable (deposit tx, unique amounts, token-account graph)
  - [ ] *Exit:* Checklist + tests green. **Then** Days 69–72 if you still want UI resilience. Same-mint pool stays parked.

---

## Tech debt (extra — not a Day)

Do when they unblock toolchain/CI; do not pull them into a session Mid-Day. Do **not** promote these over the [Build priority calendar](#build-priority-calendar-do-not-reorder).

- [x] **Anchor `programs/*/` layout** — Move on-chain crate from `program/` → `programs/zk_spot_shield/` so AVM/Anchor discover `anchor-lang` under the standard path. Workspace + `Anchor.toml` members, LiteSVM `.so` path, docs/rules/CI updated. *Exit:* `cargo test -p zk_spot_shield --lib` and `anchor build --ignore-keys` succeed.

---

## Writing checkpoints (3 sessions)

- [ ] **Write-1 — after Day 20 — What the circuit proves**
  - [ ] Series (≤3 parts): private vs public I/O, Merkle inclusion + solvency, nullifier / unlinkability. This session: outline all parts + **ship Part 1**
  - [ ] Hero diagram (1): guest reads private path, commits journal only — digestible in 10s
  - [ ] LinkedIn: 120–200 words, same diagram, no extra graphic
  - [ ] *Exit:* `notes/articles/01-circuit/part-1.md` + `diagram.md` (or mermaid in the article) + `linkedin.md`. ChainTribe copy is optional until the blogs folder exists.

- [ ] **Write-2 — after Day 40 — Settle on Solana**
  - [ ] Series (≤3 parts): verify Groth16, registered root, unused nullifier, vault + SPL. This session: outline + **Part 1**
  - [ ] Hero diagram (1): those four gates in order (reuse on Day 64 README — do not redraw)
  - [ ] Optional CU callout from Days 18/29 (numbers only; no new benchmark)
  - [ ] LinkedIn from Part 1
  - [ ] *Exit:* `notes/articles/02-settle/part-1.md` + one diagram + `linkedin.md`. **Next engineering Day is 41 (deposit), not relayer.**

- [ ] **Write-3 — after Day 65 — Public artifact (honest prove)**
  - [ ] Series (≤3 parts): membership ≠ mixer; execute vs remote Groth16 vs “blocked on RAM”; how to read the README. This session: outline + **Part 1**
  - [ ] Hero diagram: **reuse** README / Write-1 / Write-2 — new picture only if something is still confusing
  - [ ] LinkedIn from Part 1 (good “here is the repo” post)
  - [ ] *Exit:* `notes/articles/03-artifact/part-1.md` + `linkedin.md`. Still **no UI required** (UI is Day 66).

**Do not add** Write-4, a Devnet essay, or an uncensorable-UI miniseries unless you drop one of the three above. One LinkedIn per shipped part is enough; do not batch-write a week of posts in a writing session.
