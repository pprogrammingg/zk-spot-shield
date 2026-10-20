# Proving log

Remote Succinct Groth16 for the **happy-path fixture** (`zk_circuit_host::fixtures::happy_path_inputs`). This is a real wrap, not a mock. Local Docker wrapping runs out of memory on a 16 GB Mac.

## Re-run

From repo root. Needs network `$PROVE` and a gitignored `.env`:

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

If guest/io are unchanged, skip `cargo prove build`. If the guest changed, rebuild the ELF, then recompile the host (it `include_bytes!` the ELF). Re-prove updates fixtures, this note, and `constants::VKEY_HASH`.

## Print verifying key without proving

Frozen fixture only (offline, free):

```bash
cargo run -p zk-circuit-host -- --print-vkey
```

Prints `vk.bytes32()` hex and a Rust `[u8; 32]` for `GlobalConfig` / `VKEY_HASH`.

## This wrap

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-09-14T02:36Z |
| Circuit version | v6.1.0 (SP1 6.5 host) |
| Execute cycles | 11,093,598 |
| Groth16 wall time | 30.7 s |
| Proof length | 356 bytes |
| Journal | 104 bytes (little-endian `u64` amount + mint + nullifier + root) |
| Quoted base fee | 0.4168 `$PROVE` |
| Max price per proving unit | 0.7900 `$PROVE` |
| Request | [0x6af4…d4a8](https://explorer.succinct.xyz/request/0x6af4b9d854a22672c20045c61d23c6a5193a6cbea4cb5ecdfa6c8e37c51ad4a8) |

**Verifying key** (`vk.bytes32()`):

```text
0x00b3a15ce4c0ea94e3b0267473c6b7543a80c72d209b2c947f71886c6a5735d7
```

Same key as Rust `VKEY_HASH` in `programs/zk_spot_shield/src/constants.rs`.

Public bytes (commit these, not `sp1-datas/`): `zk-circuit/fixtures/happy/{groth16,journal}.bin` and `vkey.bytes32.txt`. On-chain packing uses `groth16.bin` (`proof.bytes()`), not the SDK suitcase `proof.bin`.
