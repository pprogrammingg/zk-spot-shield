# ZK Spot Shield

![Unit tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/unit.yml/badge.svg)
![Security](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/security.yml/badge.svg)
![Program tests](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/program-tests.yml/badge.svg)
![Docs](https://github.com/pprogrammingg/zk-spot-shield/actions/workflows/docs.yml/badge.svg)

Private spot settlement on Solana. A zero-knowledge circuit proves you deposited and can afford the trade. The on-chain program checks a short proof, a registered Merkle root, and a one-time nullifier, then the vault pays the other token 1:1.

This is **not a mixer**. The deposit transaction still shows who deposited.

**Docs:** [pprogrammingg.github.io/zk-spot-shield](https://pprogrammingg.github.io/zk-spot-shield/) · [Architecture](./docs/architecture.md) · [Glossary](./docs/glossary.md)

## Trust model

| You trust | You do not need to trust |
| --- | --- |
| The circuit and the verifying key stored on-chain (`VKEY_HASH`) | The relayer (optional fee payer; it cannot change the proof journal) |
| Protocol admin (`GlobalConfig.authority`) for pause and which Merkle roots are allowed | The website that hosts the UI (you can submit the transaction yourself) |
| The remote proving service, if you use Succinct (it sees your private witness) | A single RPC endpoint (the UI can fall back to another) |

Attack table: [`docs/security.md`](./docs/security.md).

Local Groth16 wrapping runs out of memory on a 16 GB laptop. Use free **execute** to check the circuit, or pay `$PROVE` for a remote Groth16 proof. Do not invent a proof file. Log: [`notes/proving.md`](./notes/proving.md).

## Repository

```text
programs/zk_spot_shield/ on-chain program
zk-circuit/io/ Merkle hashing and public journal layout
zk-circuit/guest/ circuit (runs in SP1)
zk-circuit/host/ execute locally, or prove remotely
client/ compute-budget helpers
relayer/ optional fee payer and HTTP 402
sdk/ TypeScript client
ui/ swap screen (React)
```

Program id (`declare_id!` / localnet / Devnet): `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G`. Deploy record: [`notes/devnet.md`](./notes/devnet.md).

## Measured cost

| Number | Where |
| --- | --- |
| Live Devnet settle **281,362** CU (N=1, Groth16 is most of that); LiteSVM **285,947** CU; tx cap **1,400,000** | [`notes/budgets.md`](./notes/budgets.md) · tx in [`notes/devnet.md`](./notes/devnet.md) |
| Public journal **144 bytes** (104 on spend-all) | [`notes/budgets.md`](./notes/budgets.md) |
| Remote Groth16 **25.1 s** / **11,743,748** cycles (toy 144-byte pack) | [`notes/proving.md`](./notes/proving.md) |

## Installation

Version pins live in the files below, not duplicated here.

| Tool | Source of truth | Verify |
| --- | --- | --- |
| Host Rust | [`rust-toolchain.toml`](./rust-toolchain.toml) `channel` | `rustc --version` |
| Anchor | [`Anchor.toml`](./Anchor.toml) `toolchain.anchor_version` | `anchor --version` |
| Solana CLI | `avm solana install` for that Anchor | `solana --version` |
| Node.js | [`sdk/package.json`](./sdk/package.json) `engines.node` | `node --version` |
| SP1 | `sp1up` (current) | `cargo prove --version` |

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Python 3.11+. Installs Anchor, then Solana for that release.
anchor_version=$(python3 -c 'import tomllib; print(tomllib.load(open("Anchor.toml","rb"))["toolchain"]["anchor_version"])')
cargo install --git https://github.com/solana-foundation/anchor --tag "v${anchor_version}" avm --locked --force
avm install "$anchor_version"
avm solana install
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

curl -L https://sp1.succinct.xyz | bash
sp1up

solana-keygen new
solana config set --url localhost
solana config get
```

## Build, execute, prove

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
anchor build
```

Binary: `target/deploy/zk_spot_shield.so`. Hash it with [`scripts/program-hash.sh`](./scripts/program-hash.sh).

**Execute** the circuit (free):

```bash
mkdir -p target/elf
cargo prove build -p guest --elf-name guest --output-directory target/elf
RUST_LOG=info cargo run -p zk-circuit-host --release
cargo run -p zk-circuit-host -- --print-vkey
```

**Groth16** (remote, costs `$PROVE`). Or stop after execute and keep the committed fixtures:

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
 cargo run -p zk-circuit-host --features network --release
```

**Unit tests** (CI default):

```bash
cargo test -p zk_spot_shield --lib --locked
cargo test -p zk-circuit-io --lib --locked
cargo test -p client --lib --locked
cargo test -p zk-spot-shield-relayer --lib --locked
```

See [`docs/tests.md`](./docs/tests.md) and `.cursor/rules/05-tests-security.mdc`.

## SDK, relayer, UI

```bash
cd sdk && npm install && npm test
cd ../relayer && cargo run -p zk-spot-shield-relayer
cd ../ui && npm install && npm run dev
```

Wallet is the default fee payer; relayer is optional. Localnet click-through: [`ui/README.md`](./ui/README.md) (`npm run demo:localnet`, then Load localnet demo). Do not run `sdk` `test:e2e` on that validator first — it spends the happy nullifier.

## Devnet

Operator record: [`notes/devnet.md`](./notes/devnet.md). Live-cluster tests: `DEVNET_ONCHAIN=1` from `sdk/`.

## Docs site

```bash
python3 -m venv .venv-docs
.venv-docs/bin/pip install -r docs/requirements.txt
.venv-docs/bin/mkdocs serve
```
