# Prove hardware and settle compute units

## Prove hardware

Local Groth16 wrapping **runs out of memory** on a 16 GB Mac ([`proving.md`](./proving.md)). No local GPU wrap was measured.

| Path | Hardware | Result |
| --- | --- | --- |
| Host **execute** (CPU) | This laptop | **11,093,598** cycles |
| Local Docker Groth16 | CPU, 16 GB | Out of memory — do not use |
| Succinct **network** Groth16 | Remote | **30.7 s**, 356-byte proof |
| Local GPU prove | — | Not measured |

For this circuit, the only Groth16 wall time we have is the network wrap. Execute stays on the CPU zkVM and is free.

## On-chain compute units (LiteSVM, vault-only settle)

Solana meters a transaction in **compute units** (a CPU budget). The cap is **1,400,000**. Our full settle used **285,947**.

Almost all of that is checking the Groth16 proof on-chain (~270,000–280,000). The rest (~6,000–16,000) is the token transfer, creating the nullifier account, and a few checks. We cannot measure Groth16 alone: it runs inside `settle_shielded_spot`, not as its own instruction. The ~270–280k band is “total minus the small leftover,” not a second on-chain counter.

| Band | Est. compute units | Basis |
| --- | --- | --- |
| SP1 v6 Groth16 verify | **~270k–280k** | Remainder of settle is small |
| Vault → user token transfer | **~3k–8k** | One Token program call |
| Nullifier account create + write | **~3k–6k** | System program alloc |
| Pause / journal / root / mint checks | **~1k–3k** | Zero-copy loads |
| **Measured total** | **285,947** | LiteSVM 2026-10-20 |

Non-verify work is about **6–16k** compute units (a few percent of the transaction). Groth16 dominates.

```bash
cargo test -p zk_spot_shield --test test_settle_scaffold settle_cu_profile --locked -- --nocapture
```
