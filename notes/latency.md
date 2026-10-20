# End-to-end latency

Measured **2026-10-20** on this laptop. Network Groth16 was **not** re-run; that wall time is still the figure in [`proving.md`](./proving.md).

Pipeline: **load fixture proof → relayer sign / HTTP 402 accept → LiteSVM confirm**.

| Stage | What | Wall |
| --- | --- | --- |
| Load Groth16 + journal fixtures | `zk-circuit/fixtures/happy/{groth16,journal}.bin` | **0.58 ms** (Rust) / **0.1 ms** (SDK) |
| Witness `generateProofInputs` | Poseidon tree insert + path (depth 20) | **4.2 ms** |
| Relayer `process_intent_dry_run` | Build and sign settle as fee payer | **0.59 ms** |
| HTTP 402 `accept_journal` | Quote match on amount and mint | **0.06 ms** |
| SDK `requestShieldedSwap` | Fixture + instruction plan | **6.4 ms** |
| LiteSVM settle confirm | On-chain Groth16 verify + vault payout | **84.8 ms** (**285,947 compute units**) |
| **Network Groth16 wrap** | Succinct (not re-run) | **30.7 s** + ~0.42 `$PROVE` |

With fixtures, proving is disk I/O; the bottleneck is **on-chain verify** (~85 ms local, ~286k compute units). In production, remote wrap still dominates (~31 s) unless you already have a proof.

Relayer dry-run is not cluster confirm. A live `sendTransaction` adds round-trip time (backoff on failure is in the relayer).

```bash
cargo test -p zk-spot-shield-relayer --lib day51 --locked -- --nocapture
cargo test -p zk_spot_shield --test test_settle_scaffold settle_cu_profile --locked -- --nocapture
cd sdk && npx vitest run test/latency.test.ts --reporter=verbose
```
