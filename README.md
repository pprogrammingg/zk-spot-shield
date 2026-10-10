# ZK Spot Shield

[![Unit tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/unit.yml/badge.svg)](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/unit.yml)
[![Security](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/security.yml/badge.svg)](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/security.yml)
[![Program tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/program-tests.yml/badge.svg)](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/program-tests.yml)
[![Docs](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/docs.yml/badge.svg)](https://pprogrammingg.github.io/zk-spot-shield/)

ZK Spot Shield is a Solana program that settles spot swaps behind SP1 zero-knowledge proofs: the chain verifies a Groth16 proof and public journal, then updates zero-copy vault state and moves SPL tokens. Compliance membership and unlinkability are enforced off-chain in the circuit (Merkle inclusion + nullifier); on-chain logic checks the proof, registered Merkle root, and unused nullifier before settlement.

**Read the docs:** [pprogrammingg.github.io/zk-spot-shield](https://pprogrammingg.github.io/zk-spot-shield/)

## Table of contents

- [Flow summary (user swap request → settle)](#flow-summary-user-swap-request--settle)
- [Installation](#installation)
- [Build and run (Month 1 checkpoint)](#build-and-run-month-1-checkpoint)
- [Tests](#tests)
- [Docs site](#docs-site)
- [Cursor / agents](#cursor--agents)

## Flow summary (user swap request → settle)

End-to-end path once the protocol is live. **Client** = wallet / `client/` SDK. **Backend** = off-chain operator (tree indexer + optional relayer; not in-repo yet). **ZK** = `zk-circuit/` (shared I/O, SP1 guest, host prover). **Solana** = `programs/zk_spot_shield/` + SPL token program.

Some later crates (relayer, on-chain tree insert) are still on the roadmap; the **objects and I/O** below match the circuit journal (`PublicOutputs`) and on-chain accounts (`GlobalConfig`, `CleanFundsRoot`, `NullifierAccount`, `VaultState`).

```text
User/client → backend (path + root) → ZK host/guest (proof + journal) → Solana settle → SPL
```

| Step | App section | Constructs | Input | Output |
| --- | --- | --- | --- | --- |
| 0. One-time setup | **Solana** (admin txs) | `GlobalConfig` PDA (`vkey_hash`, pause, authority); `VaultState`; empty `CleanFundsRoot` ring | Admin keypair; SP1 verifying-key hash of the guest ELF | Initialized PDAs. Later settles refuse proofs whose vkey does not match `vkey_hash`. |
| 1. Create shielded note | **Client** (local, stays private) | **Leaf** = Poseidon(`secret`, `user_address`, `balance`) via `zk-circuit-io` | User `secret` (32 bytes), pubkey, note `balance`, `asset_id_mint` | **Leaf** (32-byte commitment). Secret never goes on-chain. |
| 2. Insert membership | **Backend** builds next tree state; **Solana** records the public fingerprint | Merkle **tree** (leaves); new **MerkleRoot**; append-only tree account (planned); `CleanFundsRoot.roots[]` | Public **leaf** (not the secret); previous tree | Updated tree; **MerkleRoot** pushed into the 32-slot `CleanFundsRoot` buffer so in-flight proofs still match an old root. **No ZK** on insert. |
| 3. User submits swap | **Client** | Swap intent (not yet a proof) | `requested_swap_amount` (≤ balance), `asset_id_mint`, recipient / routing as the product adds them | A request the backend/prover can turn into a witness. |
| 4. Fetch inclusion data | **Backend** (indexer) | **Merkle path**: 20 × (`sibling` hash, `is_right`); **expected_root** | Which **leaf** / `leaf_index` (index is public at insert, **private witness** at prove) | `merkle_path` + a **MerkleRoot** that still sits in `CleanFundsRoot`. Path stays off-chain. |
| 5. Assemble witness | **ZK host** (`zk-circuit/host`) | `PrivateInputs` | `secret`, `user_address`, `merkle_path`, `balance`, `requested_swap_amount`, `asset_id_mint`, `expected_root` | Stdin blob for the guest. Host may `execute` first (no proof) to catch panics. |
| 6. Guest constraints | **ZK guest** (`zk-circuit/guest`, RISC-V ELF) | Recomputes **leaf**; **computed_root** = hash-up(`leaf`, path); **nullifier** = Poseidon(`secret`, `leaf`, `asset_id_mint`); `PublicOutputs` journal | `PrivateInputs` via `sp1_zkvm::io::read()` | If solvency and `computed_root == expected_root` hold: `commit` **journal** `{ requested_swap_amount, asset_id_mint, nullifier, merkle_root }`. Else panic (no proof). |
| 7. Prove | **ZK host** + SP1 prover (CPU / network) | **Groth16 proof** `(A, B, C)`; same **journal** bound as public inputs | Guest ELF + stdin + proving key (`setup` once per ELF) | Proof bytes + journal bytes. Verifier can check these without the path or secret. |
| 8. Pack instruction | **Client** (later packing helpers on host) | Ix data: length-prefixed **proof** + **journal** | Proof + `PublicOutputs` | `Vec<u8>` for `settle_shielded_spot`. |
| 9. Submit settle | **Solana** program (`sp1-solana` CPI) | Reads `GlobalConfig`, `CleanFundsRoot`, `VaultState`; derives `NullifierAccount` PDA `[b"nullifier", nullifier]` | Tx accounts + proof + journal. Fee payer = user or **relayer**. | Pairing check vs `vkey_hash`. Journal `merkle_root` must be in `CleanFundsRoot`. Unused nullifier → create **NullifierAccount** (spend tag). Fail: `InvalidProof` / `MerkleRootNotFound` / `NullifierAlreadyUsed`. |
| 10. Move funds | **Solana** + **SPL** token program | Vault reserve mutation (zero-copy); SPL transfer | Journal `requested_swap_amount` + `asset_id_mint`; vault token accounts | Tokens moved; vault reserves updated. Chain never learned `secret`, **leaf** preimage, or **Merkle path**. |

**What stays private vs public**

| Private (witness / client) | Public (journal or chain) |
| --- | --- |
| `secret`, full **Merkle path**, note `balance`, `leaf_index` at prove time | **MerkleRoot**, **nullifier**, swap amount, mint |
| **Leaf** preimage (`secret` ∥ address ∥ balance) | **Leaf** as an opaque 32-byte blob if inserted on-chain |
| Proving key, guest stdin | **Groth16 proof**, `vkey_hash`, `NullifierAccount` PDA existence |

Terms: [`docs/glossary.md`](./docs/glossary.md) (also on the docs site). ZK tool map: `further_explanations/zero-knowledge.md`. Happy-path Merkle (empty tree): `further_explanations/merkle_trees.md`.

## Installation

For someone who needs to build and run this repo locally (localnet).

| Topic | Task | Version / target | Verify |
| --- | --- | --- | --- |
| Rust | Install via [rustup](https://rustup.rs/) | **1.94.1** host (`rust-toolchain.toml` for `sp1-sdk`); **program BPF** uses Solana’s own rustc (~1.89) — `program` MSRV is `1.87`, not 1.94 | `rustc --version` · `cargo --version` |
| Solana CLI | In the repo, `avm solana install` | Mapped from the Anchor release in `Anchor.toml`: **3.1.10** for Anchor 1.1.2 (Agave 3.x, platform-tools v1.52). Agave 4 / SBPFv3 waits for the LiteSVM and platform-tools bump in `programs/zk_spot_shield/Cargo.toml` ([Agave install](https://docs.anza.xyz/cli/install)) | `solana --version` |
| Anchor CLI | Install [AVM](https://www.anchor-lang.com/docs/references/avm), then that release | **1.1.2**, `[toolchain] anchor_version` in `Anchor.toml` (same as `anchor-lang` in `programs/zk_spot_shield/`) | `anchor --version` |
| SP1 CLI | Install + update via `sp1up` | current SP1 release | `cargo prove --version` |
| Node.js | Runtime for Anchor/TS client | `20+` LTS | `node --version` · `npm --version` |
| Local keypair | Create wallet for localnet | any fresh keypair | `solana-keygen new` |
| Cluster config | Point CLI at local validator | `localhost` | `solana config set --url localhost` · `solana config get` |

Install snippets:

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Anchor version manager, from the release named in Anchor.toml.
# Requires Python 3.11+ (tomllib). Git HEAD of AVM can need a newer rustc.
anchor_version=$(python3 -c 'import tomllib; print(tomllib.load(open("Anchor.toml","rb"))["toolchain"]["anchor_version"])')
cargo install --git https://github.com/solana-foundation/anchor --tag "v${anchor_version}" avm --locked --force
avm install "$anchor_version"
# Solana CLI from AVM's Anchor→Solana map (3.1.10 for Anchor 1.1.2).
avm solana install
# add ~/.local/share/solana/install/active_release/bin to PATH if `solana` is not found

# SP1
curl -L https://sp1.succinct.xyz | bash
sp1up

# Node (example: nvm)
nvm install --lts
```

You are ready when every **Verify** command succeeds and `solana config get` shows `localhost`.

## Build and run (Month 1 checkpoint)

Stranger path after toolchain install. Tag restore point: `v0.1-month1` (create once Month 1 work is committed: `git tag v0.1-month1`).

### Solana program

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
anchor build
# artifact: target/deploy/zk_spot_shield.so
```

### Host execute (local, free)

Rebuild the guest ELF only if `zk-circuit/guest` or `zk-circuit/io` changed; the host embeds it at compile time.

```bash
mkdir -p target/elf
cargo prove build -p guest --elf-name guest --output-directory target/elf
# path may nest: find target/elf -type f

RUST_LOG=info cargo run -p zk-circuit-host --release
```

Default run is **execute only** (happy-path fixture → guest → journal asserts). No Groth16, no `$PROVE`.

Offline vkey from the frozen fixture:

```bash
cargo run -p zk-circuit-host -- --print-vkey
```

### Groth16 via remote prover (not this laptop)

Local Docker wrap OOMs a 16 GB machine. Use Succinct network (needs deposited `$PROVE` + gitignored root `.env`). Details: [`notes/proving.md`](./notes/proving.md), [`further_explanations/prove_network.md`](./further_explanations/prove_network.md).

```bash
# root .env: SP1_USE_NETWORK=1, NETWORK_PRIVATE_KEY=… (and related Succinct vars)
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

Writes gitignored `sp1-datas/`; copies public happy-path bytes to `zk-circuit/fixtures/happy/` for CI. Budgets for settle (sizes / ~280k CU / ALT): [`notes/budgets.md`](./notes/budgets.md).

### TypeScript SDK (`sdk/`, Days 33–38)

Anchor provider for **localhost**, Poseidon Merkle util, prove bridge (fixture or optional host network prove), settle ix + compute-budget wrappers, localnet E2E.

```bash
cd sdk && npm install && npm test         # unit tests (no validator)
cd sdk && npm run test:e2e                # validator + deploy preload + settle E2E
# IDL copy: sdk/idl/zk_spot_shield.json (refresh after `anchor build` if the API changes)
```

`test:e2e` starts `solana-test-validator` with the program at `declare_id!` plus preloaded config/vault/SPL accounts, runs happy + negative settle cases, then tears down.

## Tests

See **[tests.md](./tests.md)** for the full map: ZK guest/host/io coverage, stubs vs live prove, CI workflows, and local commands.

## Docs site

Live site: [https://pprogrammingg.github.io/zk-spot-shield/](https://pprogrammingg.github.io/zk-spot-shield/)

The public docs live in `docs/` as Markdown and are built with [MkDocs Material](https://squidfunk.github.io/mkdocs-material/). Diagrams are Mermaid blocks inside the pages, so they change in the same commit as the text.

**Deploy is automatic.** The [Docs workflow](.github/workflows/docs.yml) runs on every push to `main` that touches `docs/**`, `mkdocs.yml`, or the workflow itself. It runs `mkdocs build --strict` (broken links or nav entries fail the build) and publishes to GitHub Pages. You can also start it by hand from the Actions tab (`workflow_dispatch`).

One-time repo setup: **Settings → Pages → Source: GitHub Actions**.

**Edit or add a page:**

1. Write or change a Markdown file in `docs/`.
2. For a new page, add it to `nav:` in `mkdocs.yml`.
3. Preview locally, then push to `main`.

```bash
python3 -m venv .venv-docs && .venv-docs/bin/pip install -r docs/requirements.txt
.venv-docs/bin/mkdocs serve           # live preview at http://127.0.0.1:8000
.venv-docs/bin/mkdocs build --strict  # same check CI runs
```

The build output (`site/`) is gitignored.

## Cursor / agents

See `AGENTS.md` and `.cursor/rules/` for monorepo, zero-copy, SP1, and session rules. Follow `roadmap.md` one Day at a time. Generated trees (`target/`, `.anchor/`, ledgers, proofs) are listed in `.cursorignore` so they stay out of agent context.
