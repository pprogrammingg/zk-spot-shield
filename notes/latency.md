# E2E latency (Day 51)

Measured **2026-10-20** on this laptop. **No `$PROVE` re-run.** Network Groth16 wall time stays the Day 15 figure in `notes/proving.md`.

Pipeline: **fixture prove → relayer sign / 402 accept → LiteSVM confirm**.

| Stage | What | Wall |
| --- | --- | --- |
| Fixture Groth16+journal load | `zk-circuit/fixtures/happy/{groth16,journal}.bin` | **0.58 ms** (Rust) / **0.1 ms** (SDK) |
| Witness `generateProofInputs` | Poseidon tree insert + path (depth 20) | **4.2 ms** |
| Relayer `process_intent_dry_run` | Build + sign settle as fee payer | **0.59 ms** |
| HTTP 402 `accept_journal` | Quote match on amount+mint | **0.06 ms** |
| SDK `requestShieldedSwap` | Fixture + ix/tx plan | **6.4 ms** |
| LiteSVM settle confirm | On-chain Groth16 verify + vault-only SPL | **84.8 ms** (**285_947 CU**) |
| **Network Groth16 wrap** | Succinct (Day 15, not re-run) | **30.7 s** + ~0.42 `$PROVE` |

## How to read this

- **P1 402 demo** with fixtures: prove is disk I/O; bottleneck is **on-chain verify** (~85 ms local LiteSVM, ~286k CU).
- **Production prove** still dominates: ~**31 s** remote wrap unless you already have a proof.
- Relayer dry-run is **not** RPC confirm. Live `sendTransaction` + cluster confirm will add RTT (Day 47 backoff on failure).

## Re-measure

```bash
cargo test -p zk-spot-shield-relayer --lib day51 --locked -- --nocapture
cargo test -p zk_spot_shield --test test_settle_scaffold settle_cu_profile --locked -- --nocapture
cd sdk && npx vitest run test/latency.test.ts --reporter=verbose
```
