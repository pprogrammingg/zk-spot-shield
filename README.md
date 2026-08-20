# ZK Spot Shield

ZK Spot Shield is a Solana program that settles spot swaps behind SP1 zero-knowledge proofs: the chain verifies a Groth16 proof and public journal, then updates zero-copy vault state and moves SPL tokens. Compliance membership and unlinkability are enforced off-chain in the circuit (Merkle inclusion + nullifier); on-chain logic checks the proof, registered Merkle root, and unused nullifier before settlement.

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

## Cursor / agents

See `AGENTS.md` and `.cursor/rules/` for monorepo, zero-copy, SP1, and session rules. Follow `roadmap.md` one Day at a time.
