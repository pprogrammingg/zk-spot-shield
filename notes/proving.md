# Proving log

Remote Succinct Groth16 for the **happy-path fixture** (`zk_circuit_host::fixtures::happy_path_inputs`). This is a real wrap, not a mock. Local Docker wrapping runs out of memory on a 16 GB Mac.

## Re-run

From repo root. Needs network `$PROVE` and a gitignored `.env`:

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

If guest/io are unchanged, skip `cargo prove build`. If the guest changed, rebuild the ELF, then recompile the host (it `include_bytes!` the ELF). Re-prove updates fixtures, this note, and `constants::VKEY_HASH`. Devnet settle can rotate the **on-chain** hash with `set_vkey_hash` without rewriting the happy fixture.

## Print verifying key without proving

Frozen happy fixture only (offline, free):

```bash
cargo run -p zk-circuit-host -- --print-vkey
```

Live guest ELF after `cargo prove build` (local setup, no `$PROVE`):

```bash
cargo run -p zk-circuit-host --release -- --print-live-vkey
```

Prints `vk.bytes32()` hex and a Rust `[u8; 32]` for `GlobalConfig` / `VKEY_HASH`.

## Devnet witness (Day 63)

Gitignored `.devnet-witness.json` + `WITNESS_JSON` / `ARTIFACT_DIR`. Do **not** copy that wrap into `zk-circuit/fixtures/happy/`.

```bash
WITNESS_JSON=.devnet-witness.json cargo run -p zk-circuit-host --release
WITNESS_JSON=.devnet-witness.json ARTIFACT_DIR=sp1-artifacts/devnet-settle \
  SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

## Devnet wrap (2026-10-22)

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-10-22T01:27Z |
| Circuit version | v6.1.0 (SP1 6.5 host) |
| Execute cycles | 11,056,815 |
| Groth16 wall time | 45.4 s |
| Proof length | 356 bytes |
| Journal | 104 bytes; mint `DNNk…`; amount `100000`; root `08d4c239…` |
| Quoted base fee | 0.3928 `$PROVE` |
| Max price per proving unit | 0.7400 `$PROVE` |
| Request | [0x3f8c…e344](https://explorer.succinct.xyz/request/0x3f8c073fc36fa195ad057d04164d4b60253467d334af3f321eaeb8ec4f52e344) |
| On-chain `vkey_hash` | `0x00d6c588a6e51b5715930c0d9495cb6c0c87e8e32fd2775b0a371df81aca6991` |
| Artifacts | gitignored `sp1-artifacts/devnet-settle/` (not copied to `fixtures/happy/`) |

## Happy-path wrap (CI fixture)

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
