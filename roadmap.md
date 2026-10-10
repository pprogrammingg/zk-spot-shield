# ZK-Shielded Spot Settlement Engine — Session Roadmap (+ private agent payments over HTTP 402; payment-flow UI planned as a separate repo)

Calibrated for **~72 sessions** (Days **0–68** engineering + **3 writing checkpoints**). Each engineering Day is 1.5 hours (3 × 25 min). Writing checkpoints are also full sessions, but they are **not** design sprints — see the visual budget below.

Mark items `[x]` when done. A Day is **done** only when its *Exit* criteria pass — not when the code “mostly exists.”

---

## Track: private agent payments over HTTP 402 (hackathons · jobs · funding)

**Why:** ZK Spot Shield becomes the private payer for machine payments — an AI agent pays an API over HTTP 402 (Stripe MPP / Coinbase x402 on Solana) from a shielded note, so the payment settles without exposing the agent's wallet or balance. This is the rail Stripe and Coinbase are building, which makes the project a hackathon entry, a job-loop talking point and, if traction shows up, a fundable startup.

**Scope rule:** the core slice comes first (on-chain verify → settle → localnet E2E, target **Nov 8**). Nothing on this track starts before that, except the plain 402 demo (no ZK) built on Mondays.

### Milestones

- [ ] **P0 — Plain 402 demo (Mondays, from Oct 12):** agent hits a paid endpoint → `402` with price → pays USDC on Solana devnet (Circle faucet) → server verifies → returns data. Published early. No Stripe approval needed on devnet.
- [ ] **P1 — Shielded payer (after the Nov 8 E2E):** the agent's payment comes from a ZK Spot Shield settle instead of a public wallet. *Exit:* one paid request on localnet where the chain sees amount + nullifier, not the payer.
- [ ] **P2 — Payment-flow UI (separate repo):** small web app showing the flow end-to-end — request → 402 quote → shielded pay → response — with what the chain sees vs what stays private. Lives in its own repo (e.g. `zk-spot-shield-pay`) that depends on this one; this repo stays program + prover + SDK.
- [ ] **P3 — Stripe sandbox path (optional):** same demo through the Stripe MPP sandbox (cards / Tempo testnet). Real USDC-on-Solana into a Stripe balance needs approval via `machine-payments@stripe.com` — only once there is a real user.

### Where it gets shown

| When | Stage | What to show |
| --- | --- | --- |
| Oct 12 | Colosseum World's Fair submission | Core proof flow + docs site; 402 track as the roadmap |
| Weekly | Colosseum project updates / Eternal | What shipped, what's next (1-min video when due) |
| Nov 3 | Stripe Sessions 2027 talk proposal | "Private machine payments: agents paying over MPP on Solana with ZK proofs" + real numbers from P0/P1 |
| Nov 13–15 | Agentic Payments Hackathon (Toronto) | Mentor on Solana ZK + 402; demo P1 if ready |
| After E2E | Local Solana meetup showcase | P1 live demo |

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
  - [x] *Exit:* Non-empty Groth16 + journal on disk from a **remote** prove; re-run command + cost note in `notes/proving.md`. If wrap/prove is blocked (e.g. RAM/Docker), write that honestly in the same note — Day 61’s public artifact may cite it; do not fake a Groth16 file.

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

- [ ] **Day 39 — Fuzz / invariant harness**
  - [ ] Trident or mollusk: arithmetic edges + random proof bytes (no verifier crash)

- [ ] **Day 40 — Constraint audit + Month 2 tag**
  - [ ] PDA `seeds`/`bump`/`has_one` pass
  - [ ] Strip noisy `msg!`
  - [ ] Tag `v0.2-month2`
  - [ ] *Exit:* Tag exists. **Next session is Write-2** (settle article + LinkedIn), not Day 41.

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
  - [ ] *Exit:* Numbers live in `notes/` (or Day 29/18 docs); Day 61 README will link them

- [ ] **Day 49 — Payload + throughput**
  - [ ] Minimize journal bytes
  - [ ] Concurrent localnet submission smoke

### Phase 11: Security Audit Readiness (4 sessions)

- [ ] **Day 50 — Threat model doc**
  - [ ] Circuit soundness, replay, pause, authority, root registry
  - [ ] *Exit:* A named doc (e.g. `notes/threat-model.md`) a reviewer can open; Day 61 README links it

- [ ] **Day 51 — Layout + signer audit**
  - [ ] `#[repr(C)]` / padding review
  - [ ] Every ix path checks signer/writable explicitly

- [ ] **Day 52 — Tooling scans**
  - [ ] `cargo-audit` (+ any chosen program analyzers)
  - [ ] Fix high findings or document waivers

- [ ] **Day 53 — Bytecode freeze prep**
  - [ ] Reproducible build notes; record program hash procedure

### Phase 12: Devnet Launch (7 sessions)

Program + relayer + SDK go live. **No browser UI yet** (that is Day 62). Day 61 is the staff README (public artifact). Finished-product Devnet tests (manual + automated) are Days 63–64.

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

- [ ] **Day 59 — Live shielded swap (SDK, not UI)**
  - [ ] One successful swap via SDK / CLI against the Devnet program
  - [ ] Explorer verification of balances + zero-copy state
  - [ ] *Exit:* A Devnet tx signature is recorded; this Day is **not** a browser product (UI is Day 62+)

- [ ] **Day 60 — Docs + `v1.0.0-devnet`**
  - [ ] README, install scripts; **one** digestible diagram (reuse Write-2 hero if it exists — do not draw a new poster)
  - [ ] Tag release; archive build logs, circuit keys, program artifacts
  - [ ] *Exit:* A stranger can deploy/init from docs. Staff-quality **shield artifact** is Day 61 (before UI). Browser product + Devnet UI tests are Days 62–64.

---

## Month 3+ buffer: Public artifact, then UI, then Devnet product tests, then uncensorable UI (Sessions 61–68)

Days **54–60** put the **program + relayer + SDK** on Devnet. **Day 61** is the GitHub README a staff engineer respects (no UI required). Days **62–64** are the **product you can see and test on-chain**. Days **65–68** harden that UI so a seized website cannot stop settle. Do **not** start IPFS/mirrors before the basic swap UI exists.

### Phase 13: Public shield artifact (1 session) — **before UI**

Assemble what already exists (execute, Groth16 or honest block, threat model, CU) into **one** repo front door. A staff engineer should understand the system from README + linked notes without a browser app.

- [ ] **Day 61 — Public artifact (staff README)**
  - [ ] Root `README.md` a staff engineer respects: what the protocol does, trust model, crate map, how to **execute** the guest, how to **prove** (remote Groth16 command) **or** a dated honest “blocked on RAM / no Groth16 file” in `notes/proving.md` — never a fake proof
  - [ ] Link Day 50 threat model (circuit, replay, pause, authority, root registry)
  - [ ] Link CU notes (Days 18 / 29 / 48): verifier budget, measured settle CU if you have it
  - [ ] How to build program + run `--lib` tests; program id / Devnet pointers if Days 54–60 ran
  - [ ] *Exit:* A cold reader can follow README → execute → (Groth16 **or** documented block) → threat model → CU numbers. **No UI this Day.** Clone-and-read is the deliverable. **Next session is Write-3** (public-artifact article + LinkedIn), then Day 62 UI.

### Phase 14: Basic swap UI (1 session)

Ship a normal swap screen first. Relayer and a local HTTPS/static server are allowed. This is **not** the uncensorable phase.

- [ ] **Day 62 — Basic swap UI**
  - [ ] Minimal web UI: wallet connect, show vault reserves / clean root / pause, swap amount + asset, submit settle (via relayer **or** wallet as fee payer)
  - [ ] Load or paste Groth16 + journal (in-browser prove is out of scope)
  - [ ] Talks to the SDK from Day 45; works against **localnet** today
  - [ ] *Exit:* You can click through a swap in the browser on localnet and see balances change in the UI (not only logs / explorer). No IPFS, no multi-RPC, no seizure runbook today.

### Phase 15: Finished product on Devnet — see it and test it (2 sessions)

The stack from Days 54–62 (program + vault + UI) must be **exercised on Devnet**, not only LiteSVM / local validator. Automated tests must send or confirm **real cluster RPCs**. A human must also open the UI and look at the product.

- [ ] **Day 63 — Manual Devnet walkthrough (see the end product)**
  - [ ] Point the Day 62 UI at Devnet (program id + RPC from Days 54–57); fund a wallet if needed
  - [ ] Human checklist in `notes/manual-devnet.md`: connect wallet → read on-chain vault/root in the UI → submit (or dry-run + one live settle if a proof is available) → open Solana Explorer links for accounts + tx
  - [ ] Record at least one Explorer URL for vault/config and, if settle ran, the tx signature
  - [ ] *Exit:* A person who is not the session agent can follow the checklist and **see** the product (UI + live accounts). Screenshots optional; the written checklist + Explorer URLs are required.

- [ ] **Day 64 — Automated on-chain Devnet tests**
  - [ ] Integration tests (TS and/or Rust) with **cluster = Devnet** — `solana-test-validator` / LiteSVM do **not** count
  - [ ] Tests must **interact on-chain**: fetch `GlobalConfig` + `VaultState` (and ATAs) from Devnet; assert layout / reserves / vkey
  - [ ] At least one **write** that lands on Devnet (authority no-op such as pause→unpause, or a real `settle_shielded_spot` if a Groth16 fixture exists)
  - [ ] One command documented (e.g. `ANCHOR_PROVIDER_URL=https://api.devnet.solana.com …`); skip-with-fail if key/SOL missing so CI-default stays localnet
  - [ ] *Exit:* That command passes against live Devnet; tx signatures or account pubkeys from the run are in the test output or `notes/devnet-tests.md`

### Phase 16: Decentralized / uncensorable swap UI (4 sessions)

The program on Solana already survives a seized website. The **basic UI** does not, unless users can swap without one operator’s HTTPS origin. This phase is **mirrors + self-submit** of the Day 62 UI, not a new chain and not the first time a swap screen exists.

- [ ] **Day 65 — Static self-submit (no required backend)**
  - [ ] Same swap UI as Day 62: works from `file://` or a local static server **without** `relayer/` running
  - [ ] Default path is **user wallet as fee payer**; relayer URL stays optional
  - [ ] *Exit:* Localnet settle can be sent with relayer down. This is hardening, not the first UI.

- [ ] **Day 66 — Multi-RPC + no single origin**
  - [ ] Configurable RPC list (fallback if one endpoint censors or dies)
  - [ ] Document IPFS/Arweave (or equivalent) publish of the **same** static bundle; pin CID in README
  - [ ] *Exit:* UI works against a second RPC; a content-addressed build hash/CID is recorded

- [ ] **Day 67 — Relayer-optional settle**
  - [ ] SDK/UI: “submit myself” vs “ask relayer”; shutdown of relayer must not block the self-submit path
  - [ ] Optional: two public relayer URLs, client tries in order
  - [ ] *Exit:* Written threat note: seizing the marketing site ≠ seizing settle; users keep CID + program id

- [ ] **Day 68 — Mirrors + seizure runbook**
  - [ ] README: how to rebuild the UI, where CIDs live, how to verify the bundle hash
  - [ ] At least two independent hosts (e.g. IPFS gateway + GitHub Pages / Pages-from-CID)
  - [ ] *Exit:* Runbook a stranger can follow if the primary URL is gone; tag or note `uncensorable-ui`

**Out of scope here:** fully trustless in-browser Groth16 on a phone. Prove can stay remote; uncensorable means **anyone can host the UI and submit the tx**, not that prove is free or local.

---

## Tech debt (extra — not a Day)

Tasks outside the numbered Day sequence. Do when they unblock toolchain/CI; do not pull them into a session Mid-Day.

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
  - [ ] Hero diagram (1): those four gates in order (reuse on Day 60 README — do not redraw)
  - [ ] Optional CU callout from Days 18/29 (numbers only; no new benchmark)
  - [ ] LinkedIn from Part 1
  - [ ] *Exit:* `notes/articles/02-settle/part-1.md` + one diagram + `linkedin.md`

- [ ] **Write-3 — after Day 61 — Public artifact (honest prove)**
  - [ ] Series (≤3 parts): membership ≠ mixer; execute vs remote Groth16 vs “blocked on RAM”; how to read the README. This session: outline + **Part 1**
  - [ ] Hero diagram: **reuse** README / Write-1 / Write-2 — new picture only if something is still confusing
  - [ ] LinkedIn from Part 1 (good “here is the repo” post)
  - [ ] *Exit:* `notes/articles/03-artifact/part-1.md` + `linkedin.md`. Still **no UI required** (UI is Day 62).

**Do not add** Write-4, a Devnet essay, or an uncensorable-UI miniseries unless you drop one of the three above. One LinkedIn per shipped part is enough; do not batch-write a week of posts in a writing session.
