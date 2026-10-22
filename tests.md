# Tests

Default tests use recorded fixtures. They must not talk to Succinct or a live cluster. Live prove is opt-in (`cargo run` + root `.env`). Devnet is opt-in (`DEVNET_ONCHAIN=1`).

Public fixture walkthrough: [`docs/tests.md`](./docs/tests.md). Commands after a crate edit: `.cursor/rules/05-tests-security.mdc`.

## CI

| Workflow | What it runs |
| --- | --- |
| [Unit tests](.github/workflows/unit.yml) | `cargo test --lib` for `zk_spot_shield`, `zk-circuit-io`, `client`, relayer. No zkVM, no BPF, no prover. |
| [Security](.github/workflows/security.yml) | `cargo audit` + `clippy -D warnings` on program / io / client / relayer. Weekly audit on Mondays. |
| [Program tests](.github/workflows/program-tests.yml) | `anchor build` + LiteSVM `cargo test --tests`. Still no Succinct. |

Rustc comes from `rust-toolchain.toml`. LiteSVM is pinned `=0.10.0`; do not merge lone bumps to 0.15+.

Invariant harness uses LiteSVM: `--lib` covers `apply_swap` arithmetic edges + mutated Groth16 bytes (no panic); `cargo test --tests` covers settle with random proof bytes (fail closed, reserves unchanged).

Empty-tree path: [`further_explanations/merkle_trees.md`](./further_explanations/merkle_trees.md). Host execute / prove: [`notes/proving.md`](./notes/proving.md).
