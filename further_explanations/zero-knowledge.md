# Zero-knowledge notes

How the proving stack fits this repo. Protocol terms (journal, vkey, nullifier) live in `glossary.md`. Session tasks live in `roadmap_related.md`.

This project: **SP1 guest** proves Merkle inclusion + solvency + nullifier; **host** executes then (Day 12) wraps as **Groth16**; **Solana** verifies that Groth16 proof against `vkey_hash`.

## Tools and technologies

| Name | What it does | Competitors | Pros | Cons |
| --- | --- | --- | --- | --- |
| **SP1** (Succinct) | RISC-V **zkVM**. Write a Rust **guest**, compile to ELF, host proves “this program ran.” This repo’s circuit (`zk-circuit/guest` + `host`). Optional Groth16/PLONK wrap for chains. | RISC Zero, Jolt, Valida, Nexus | Rust instead of a hand-rolled circuit; `execute` then `prove`; Groth16 wrap fits Solana CU; matches `sp1-sdk` / `cargo prove` / `sp1-solana` | Prove is slow on CPU; guest change → new vkey; `execute` is not on-chain evidence; SDK APIs shift (sync vs async) |
| **RISC Zero** | Same job as SP1: RISC-V zkVM (guest, host, receipts, optional Groth16 wrap / Bonsai). Not used here. | SP1, Jolt, other zkVMs | Mature docs; same mental model (stdin / journal / image ID ≈ vkey) | Different proof bytes and verifier than `sp1-solana`; swap = new guest glue + vkey + CPI |
| **Arkworks** (`ark-*`) | Rust **math + circuit** libraries: fields, curves, R1CS, Groth16, Marlin. You write *constraints*, not a CPU program. | gnark, Circom, Halo2, Bellman | Fine-grained circuits; no RISC-V overhead; battle-tested pairings | You maintain gadgets/witnesses; Merkle+Poseidon+nullifier as R1CS is more work than the SP1 guest; not the Day 11 host |
| **Groth16** | Pairing **SNARK**: tiny proof, fast verify, per-circuit proving/verifying keys (trusted setup). **On-chain format** for an SP1 run in this repo. | PLONK, STARKs, Halo2, Bulletproofs | Small bytes; verify ~hundreds of kCU on Solana (roadmap ~280k); widely supported | Trusted setup / circuit-specific keys; new guest ELF → rotate `vkey_hash`; wrapping a zkVM is slower to *prove* than a tiny native Groth16 circuit |
| **PLONK** (incl. SP1 PLONK wrap) | Universal-setup SNARK family. SP1 can emit PLONK instead of Groth16. | Groth16, Halo2, Marlin | One setup for many circuits (up to a size); easier circuit iteration | Often heavier verify than Groth16 on Solana; this roadmap’s on-chain verifier expects Groth16 |
| **STARKs** (FRI / native zkVM proof) | Hash-based proofs of execution or circuits. No pairings. SP1/RISC Zero prove this *first*, then compress. | Groth16/PLONK as the *on-chain* layer; Miden, Stone | Transparent setup; large programs; hash-based | Fat proofs; native verify blows Solana packet + CU limits — hence the Groth16 wrap |
| **Circom + snarkjs** | DSL + JS/CLI for R1CS Groth16 (classic mixer / Merkle circuits). | Arkworks, gnark, Noir, Halo2 | Lots of examples; explicit signals; easy to audit a *small* circuit | Not Rust; not this repo’s guest/host layout; witness gen is a separate stack |
| **Halo2** | PLONKish proving (custom gates, lookups). Zcash / PSE / Ethereum-heavy. | Groth16+Ark, Circom, Noir, zkVMs | Flexible gadgets; no per-circuit Groth16 setup | Steeper API; not what `sp1-solana` verifies in this project |
| **Noir** (Aztec) | Rust-like language → ACIR circuit (Barretenberg etc.). “Write a program, get a circuit,” not a full RISC-V VM. | Circom, Leo, Cairo, zkVMs | Nice language; growing ecosystem | Different backend and verifier; would *replace* the guest, not drop in |
| **gnark** (Consensys) | Go library for Groth16/PLONK circuits (Arkworks’ cousin in Go). | Arkworks, Circom, Halo2 | Fast; production EVM use | Go, not this workspace; still a hand-written circuit, not a zkVM |
| **Jolt** | zkVM aimed at high-performance RISC-V proving (lookup-heavy). | SP1, RISC Zero | Research/perf angle on the same “prove a CPU” idea | Less turnkey Solana Groth16 path than SP1 in this roadmap |
| **Poseidon** (this circuit’s hash) | SNARK-friendly hash used for leaves, Merkle nodes, nullifier in `zk-circuit/io`. | SHA-256, Keccak, Rescue, MiMC | Cheap inside a circuit/zkVM vs SHA | Must match on host and guest; not the same as Solana’s SHA syscalls |

### How the pieces stack in *this* repo

| Layer | Tool | Job |
| --- | --- | --- |
| Guest program | SP1 + `sp1-zkvm` | `read` private inputs → asserts → `commit` journal |
| Shared types / hash | `zk-circuit-io` (Poseidon Merkle) | Same leaf/path/nullifier on host and guest |
| Host driver | `sp1-sdk` | Fixture → stdin → `execute` (Day 11) → `.groth16()` (Day 12) |
| On-chain proof | Groth16 | Bytes the program actually verifies |
| On-chain check | `sp1-solana` CPI + `vkey_hash` | Accept proof iff it matches the registered circuit |

**Not interchangeable:** SP1 vs RISC Zero is a *zkVM product* choice. Arkworks vs Circom is a *hand-written circuit* choice. Groth16 vs STARK is a *proof format* choice (here: STARK-style execution, Groth16 postage stamp for Solana).
