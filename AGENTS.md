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
| `01-anchor-zero-copy.mdc` | `programs/**/*.rs` — zero-copy / `AccountLoader` |
| `02-sp1-circuit.mdc` | `zk-circuit/**` — guest/host SP1 constraints |
| `03-session-roadmap.mdc` | Always — one Day per session, checklist hygiene |
| `06-readme-toolchain.mdc` | Always — README Installation pins point at source files, not hardcoded versions |
| `07-commit-messages.mdc` | Always — suggest ≤50-char subjects + ` - YES`; user commits/pushes |
| `04-context-hygiene.mdc` | Always — skip `target/` and other generated trees |
| `05-tests-security.mdc` | Always — after editing a crate, `--lib` + clippy on **that** crate |
| `08-docs-sync.mdc` | Always — after a behavior change, update the matching `docs/` / `notes/` page |

Layout, ignore lists, and stack pins live in `00-project.mdc`. Test commands live in `05-tests-security.mdc`. Toolchain verify and `anchor build` live in `README.md`.
