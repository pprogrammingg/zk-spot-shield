# ZK Spot Shield

![Unit tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/unit.yml/badge.svg)
![Security](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/security.yml/badge.svg)
![Program tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/program-tests.yml/badge.svg)
![Docs](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/docs.yml/badge.svg)

Private spot settlement on Solana. An SP1 circuit proves you are in the deposit tree and can afford the trade; the program checks a short Groth16 proof, a registered root, and a one-time nullifier (mechanism used to prevent double-spend), then moves SPL tokens.

**Docs:** [pprogrammingg.github.io/zk-spot-shield](https://pprogrammingg.github.io/zk-spot-shield/)

## Flow

```text
Client → indexer (Merkle path + tree root) → ZK prove (Groth16 + journal) → Solana settle → SPL
```

Approved deposits sit in a **Merkle tree**: each note is hashed into a **leaf**, and those leaves roll up into one public **root** the chain can check. When you spend, your wallet keeps a **path** of sibling hashes that show your leaf is in that tree—without putting the path or your secret on Solana.

Diagrams and Flow Sequence: **[Architecture](https://pprogrammingg.github.io/zk-spot-shield/architecture/)** · `[docs/architecture.md](./docs/architecture.md)`

## Installation

Install these tools, then point Solana at localnet. You are ready when the verify commands succeed and `solana config get` shows `localhost`.


| Tool                                                            | Version                                 | Verify                  |
| --------------------------------------------------------------- | --------------------------------------- | ----------------------- |
| Rust ([rustup](https://rustup.rs/))                             | host **1.94.1** (`rust-toolchain.toml`) | `rustc --version`       |
| Anchor ([AVM](https://www.anchor-lang.com/docs/references/avm)) | **1.1.2**                               | `anchor --version`      |
| Solana CLI (`avm solana install`)                               | **3.1.10** with this Anchor             | `solana --version`      |
| SP1 (`sp1up`)                                                   | current                                 | `cargo prove --version` |
| Node.js                                                         | **20+**                                 | `node --version`        |


```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Needs Python 3.11+. Installs Anchor AVM, then Solana for that release.
anchor_version=$(python3 -c 'import tomllib; print(tomllib.load(open("Anchor.toml","rb"))["toolchain"]["anchor_version"])')
cargo install --git https://github.com/solana-foundation/anchor --tag "v${anchor_version}" avm --locked --force
avm install "$anchor_version"
avm solana install
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

curl -L https://sp1.succinct.xyz | bash
sp1up

# Node: use nvm, brew, or your usual installer, then:
# nvm install --lts

solana-keygen new
solana config set --url localhost
solana config get
```



## Build and run

Work from the repo root after the tools above are installed.

**Program**

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
anchor build
```

The binary is written to `target/deploy/zk_spot_shield.so`.

**Circuit host (execute only, free)**

Rebuild the guest when `zk-circuit/guest` or `zk-circuit/io` changes.

```bash
mkdir -p target/elf
cargo prove build -p guest --elf-name guest --output-directory target/elf
RUST_LOG=info cargo run -p zk-circuit-host --release
```

Print the frozen verifying key with:

```bash
cargo run -p zk-circuit-host -- --print-vkey
```

**Groth16 (remote, costs $PROVE token)**

Local wrap often runs out of memory. Use Succinct’s network with `$PROVE` and a root `.env` — see `[notes/proving.md](./notes/proving.md)`.

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

**TypeScript SDK**

```bash
cd sdk
npm install
npm test
npm run test:e2e
```

`npm test` is unit-only. `npm run test:e2e` starts a local validator, deploys, and runs settle checks.

## Tests

See **[tests.md](./tests.md)** for crate coverage and CI commands.

## Docs

Published site: [https://pprogrammingg.github.io/zk-spot-shield/](https://pprogrammingg.github.io/zk-spot-shield/). Sources live in `docs/` (MkDocs).

```bash
python3 -m venv .venv-docs
.venv-docs/bin/pip install -r docs/requirements.txt
.venv-docs/bin/mkdocs serve
```

Open [http://127.0.0.1:8000](http://127.0.0.1:8000) while `serve` is running.