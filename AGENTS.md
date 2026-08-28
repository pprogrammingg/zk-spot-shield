# AGENTS.md

Guidance for AI agents working in **zk-spot-shield**.

## Read first

1. `roadmap.md` — current Day / Exit criteria
2. `README.md` — install + toolchain verify
3. `.cursor/rules/` — persistent project rules (MDC)

## Rules map

| File | When it applies |
| --- | --- |
| `00-project.mdc` | Always — layout, stack, do-nots |
| `01-anchor-zero-copy.mdc` | `program/**/*.rs` — zero-copy / `AccountLoader` |
| `02-sp1-circuit.mdc` | `zk-circuit/**` — guest/host SP1 constraints |
| `03-session-roadmap.mdc` | Always — one Day per session, checklist hygiene |
| `04-context-hygiene.mdc` | Always — skip `target/` and other generated trees |

## Repo shape

```text
program/              # Anchor crate zk_spot_shield
zk-circuit/guest/     # SP1 guest circuit
zk-circuit/host/      # SP1 host prover driver
client/               # client placeholder → SDK later
Anchor.toml           # localnet + program id
Cargo.toml            # workspace
further_explanations/ # glossary + session notes
```

Ignore for search/context: `target/`, `.anchor/`, `test-ledger/` (see `.cursorignore`).

## Verify toolchain

```bash
rustc --version && cargo --version
solana --version && anchor --version
cargo prove --version
solana config get   # expect localhost
```

## Build

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
anchor build
```

Deploy artifact: `target/deploy/zk_spot_shield.so` (required for program integration tests that `include_bytes!` it).
