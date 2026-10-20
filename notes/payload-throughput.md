# Payload and throughput

## Journal size

The public journal is **104 bytes**: little-endian `u64` amount plus three 32-byte fields (mint, nullifier, root). There is no serde and no length prefix inside the blob. Shrinking it further means dropping a public field (circuit, verifying key, and on-chain parse). Out of scope.

Settle instruction data is **476 bytes**: 8-byte discriminator + Borsh `Vec` proof (356) + `Vec` journal (104). See [`budgets.md`](./budgets.md).

## Concurrent smoke

Relayer: 16 parallel `process_intent_dry_run` jobs (fixture-shaped settle JSON). Sign only; no cluster.

On-chain, one happy-path nullifier is single-use. A second settle of the **same** fixture must fail `NullifierAlreadyUsed`. Distinct notes need distinct Groth16 proofs.

```bash
cargo test -p zk-spot-shield-relayer --lib concurrent --locked -- --nocapture
```
