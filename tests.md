# Tests

Source of truth for what this repo tests, what CI runs, and what stays offline (no real prover in default CI).

## Principle

Every default test should use a **stub / recorded fixture** and must **not** talk to a live system (Succinct prover, Devnet, etc.). Live prove is opt-in (`cargo run` + root `.env`). Devnet / end-to-end against real clusters come later as explicit workflows.

## ZK surface (guest / host / io)

| Layer | In CI (`unit.yml`)? | What it checks | Talks to prover? |
| --- | --- | --- | --- |
| **`zk-circuit-io`** | Yes | Poseidon leaf / path / nullifier; journal decode; committed happy `journal.bin` / `groth16.bin` / vkey string | No — pure Rust + files on disk |
| **Guest** | No | Circuit logic only runs inside SP1 (RISC-V ELF). No guest `--lib` suite | No in CI. Local: `cargo run -p zk-circuit-host` → `execute` |
| **Host** | No (CI: do not compile host) | One optional `--lib` test mirrors io (reads same fixture files). Live path is `main`: fixture → `execute`; Groth16 only with network env | Network only on explicit `SP1_USE_NETWORK=1` + `SP1_GROTH16=1` + key |
| **Program** | Yes (`--lib`) | PDAs, zero-copy sizes, errors — not proof verify yet | No |

### Stubs / fixtures

| Artifact | Role |
| --- | --- |
| Happy-path witness (`secret=[1;32]`, empty-tree index-0 path) | Deterministic inputs in `zk_circuit_host::fixtures::happy_path_inputs` / shared io helpers — not a mock prover |
| `zk-circuit/fixtures/happy/{groth16,journal}.bin` + `vkey.bytes32.txt` | Recorded bytes from one remote wrap. Tests assert layout / hashes / length; they do **not** re-prove. **Tracked in git** (`.gitignore` allowlist) so CI checkout sees them — toy fixture, not user funds |
| `SP1_PROVER=mock` | Exists in the SDK; **not** used for Day 12 exit and **not** CI |

CI tests **shared crypto + frozen proof artifacts**, not guest zkVM and not Succinct. Guest/host execute is manual (or later host negative tests); still offline unless you opt into network prove.

Empty-tree path used by the happy fixture: [`further_explanations/merkle_trees.md`](./further_explanations/merkle_trees.md).

## CI workflows

| Workflow | What it runs | Why it matters |
| --- | --- | --- |
| [Unit tests](.github/workflows/unit.yml) | `cargo test --lib` for `zk_spot_shield`, `zk-circuit-io`, `client` | Fast invariants (PDA seeds, vault layout, Poseidon/Merkle, happy fixture bytes). No zkVM, no BPF, no prover |
| [Security](.github/workflows/security.yml) | `cargo audit` + `clippy -D warnings` on program / io / client | Advisories and obvious unsoundness. Weekly audit on Mondays |
| [Program tests](.github/workflows/program-tests.yml) | `anchor build` + LiteSVM `cargo test --tests` | On-chain instruction smoke tests. Needs Solana + Anchor CLIs. Still no Succinct |

Rustc comes from `rust-toolchain.toml` (`rustup show` after checkout). Do not pin a second copy in workflows or VS Code.

## Local commands (same as CI unit + security bar)

```bash
cargo test -p zk_spot_shield --lib --locked
cargo test -p zk-circuit-io --lib --locked
cargo test -p client --lib --locked
cargo clippy -p zk_spot_shield -p zk-circuit-io -p client --locked --all-targets -- -D warnings
cargo audit
```

After editing a crate, run `--lib` + clippy for **that** crate first (see `.cursor/rules/05-tests-security.mdc`).

Host execute / prove (not CI-default):

```bash
# execute only (offline)
RUST_LOG=info cargo run -p zk-circuit-host --release

# one-shot remote Groth16 (costs $PROVE; see notes/proving.md)
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info cargo run -p zk-circuit-host --release
```

## Local pre-push check (same as CI, no `act`)

`act` is not required. From repo root, run the Unit + Security job commands:

```bash
rustup show   # installs channel from rust-toolchain.toml if needed
cargo test -p zk_spot_shield --lib --locked
cargo test -p zk-circuit-io --lib --locked
cargo test -p client --lib --locked
cargo clippy -p zk_spot_shield -p zk-circuit-io -p client --locked --all-targets -- -D warnings
cargo audit
```

Program tests (`anchor build` + LiteSVM) need Solana/Anchor CLIs — same as `.github/workflows/program-tests.yml`.
