# Proving log (Day 12)

Remote Succinct Groth16 for the **happy-path fixture** (`zk_circuit_host::fixtures::happy_path_inputs`). Not a mock. Local Docker wrap OOMs this 16 GB Mac.

## Re-run

From repo root (needs deposited network `$PROVE` + gitignored `.env`):

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info cargo run -p zk-circuit-host --release
```

Guest/io unchanged → skip `cargo prove build`. Guest changed → rebuild ELF, then recompile host (it `include_bytes!` the ELF).

## This wrap

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-09-14T02:36Z |
| Circuit version | v6.1.0 (SP1 6.5 host) |
| Execute cycles | 11_093_598 |
| Groth16 wall time | 30.7 s |
| `proof.bytes().len()` | 356 |
| Journal | 104 bytes (LE `u64` amount + mint + nullifier + root) |
| Quoted base fee | 0.4168 `$PROVE` |
| Max price per bPGU | 0.7900 `$PROVE` |
| Request | [0x6af4…d4a8](https://explorer.succinct.xyz/request/0x6af4b9d854a22672c20045c61d23c6a5193a6cbea4cb5ecdfa6c8e37c51ad4a8) |

**vkey** (`vk.bytes32()`):

```text
0x00b3a15ce4c0ea94e3b0267473c6b7543a80c72d209b2c947f71886c6a5735d7
```

Public bytes (commit these, not `sp1-artifacts/`): `zk-circuit/fixtures/happy/{groth16,journal}.bin` + `vkey.bytes32.txt`. On-chain packing still uses `groth16.bin` (`proof.bytes()`), not the SDK suitcase `proof.bin`.
