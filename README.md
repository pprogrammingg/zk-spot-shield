# ZK Spot Shield

[![Unit tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/unit.yml/badge.svg)](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/unit.yml)
[![Security](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/security.yml/badge.svg)](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/security.yml)
[![Program tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/program-tests.yml/badge.svg)](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/program-tests.yml)

ZK Spot Shield is a Solana program that settles spot swaps behind SP1 zero-knowledge proofs: the chain verifies a Groth16 proof and public journal, then updates zero-copy vault state and moves SPL tokens. Compliance membership and unlinkability are enforced off-chain in the circuit (Merkle inclusion + nullifier); on-chain logic checks the proof, registered Merkle root, and unused nullifier before settlement.

## Flow summary (user swap request → settle)

End-to-end path once the protocol is live. **Client** = wallet / `client/` SDK. **Backend** = off-chain operator (tree indexer + optional relayer; not in-repo yet). **ZK** = `zk-circuit/` (shared I/O, SP1 guest, host prover). **Solana** = `program/` (`zk_spot_shield`) + SPL token program.

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

Terms: `further_explanations/glossary.md`. ZK tool map: `further_explanations/zero-knowledge.md`.

## Installation

For someone who needs to build and run this repo locally (localnet).

| Topic | Task | Version / target | Verify |
| --- | --- | --- | --- |
| Rust | Install via [rustup](https://rustup.rs/) | `1.75+` (stable) | `rustc --version` · `cargo --version` |
| Solana CLI | Install Anza release tools | `stable` channel ([install](https://docs.anza.xyz/cli/install)) | `solana --version` |
| Anchor CLI | Install AVM, then Anchor | latest via `avm` | `anchor --version` |
| SP1 CLI | Install + update via `sp1up` | current SP1 release | `cargo prove --version` |
| Node.js | Runtime for Anchor/TS client | `20+` LTS | `node --version` · `npm --version` |
| Local keypair | Create wallet for localnet | any fresh keypair | `solana-keygen new` |
| Cluster config | Point CLI at local validator | `localhost` | `solana config set --url localhost` · `solana config get` |

Install snippets:

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Solana
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
# add active_release/bin to PATH if the installer prompts you

# Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install latest && avm use latest

# SP1
curl -L https://sp1.succinct.xyz | bash
sp1up

# Node (example: nvm)
nvm install --lts
```

You are ready when every **Verify** command succeeds and `solana config get` shows `localhost`.

## CI

| Workflow | What it runs | Why it matters |
| --- | --- | --- |
| [Unit tests](.github/workflows/unit.yml) | `cargo test --lib` for `zk_spot_shield`, `zk-circuit-io`, `client` | Fast invariants (PDA seeds, vault layout, Poseidon/Merkle). No zkVM, no BPF. |
| [Security](.github/workflows/security.yml) | `cargo audit` + `clippy -D warnings` on program / io / client | Advisories and obvious unsoundness. Weekly audit on Mondays. |
| [Program tests](.github/workflows/program-tests.yml) | `anchor build` + LiteSVM `cargo test --tests` | On-chain instruction smoke tests. Needs Solana + Anchor CLIs. |

Local equivalents (unit + security are the default bar):

```bash
cargo test -p zk_spot_shield --lib --locked
cargo test -p zk-circuit-io --lib --locked
cargo test -p client --lib --locked
cargo clippy -p zk_spot_shield -p zk-circuit-io -p client --locked --all-targets -- -D warnings
cargo audit
```

Do not use `zk-circuit/host` execute as a stand-in for `--lib` tests. Keep `Cargo.lock` committed so `--locked` and audit stay reproducible.

## Cursor / agents

See `AGENTS.md` and `.cursor/rules/` for monorepo, zero-copy, SP1, and session rules. Follow `roadmap.md` one Day at a time. Generated trees (`target/`, `.anchor/`, ledgers, proofs) are listed in `.cursorignore` so they stay out of agent context.
