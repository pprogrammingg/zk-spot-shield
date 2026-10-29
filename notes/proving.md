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

## Devnet witness

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

Toy witness: spend **100**, keep change **900**. This is `VKEY_HASH` / `fixtures/happy/`.

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-10-24T14:00Z |
| Circuit version | v6.1.0 (SP1 6.5 host) |
| Execute cycles | 11,743,748 |
| Groth16 wall time | 25.1 s |
| Proof length | 356 bytes |
| Journal | 144 bytes (amount + mint + nullifier + root + change amount + change leaf); amount `100`; change `900` |
| Quoted base fee | 0.4032 `$PROVE` |
| Max price per proving unit | 0.7600 `$PROVE` |
| Request | [0x3cf7…5e65](https://explorer.succinct.xyz/request/0x3cf702a04e3296db58c5f6411140d8af9686d8f69de5ad0e719300ab69e55e65) |
| Artifacts | `zk-circuit/fixtures/happy/{groth16,journal}.bin` and `vkey.bytes32.txt` |

**Verifying key** (`vk.bytes32()`):

```text
0x00435696d666d305172306e3895a44017e188663de9b3876970ae270345fc240
```

Same key as Rust `VKEY_HASH` in `programs/zk_spot_shield/src/constants.rs`. Devnet `GlobalConfig` was rotated to this hash (`set_vkey_hash` `ZvnaL9f…`). Localnet / local UI demo this pack (mint `0x03…03`). A live wrap still needs vault mint `DNNk…` and stays in `sp1-artifacts/` (`SKIP_HAPPY_FIXTURE=1` / `ARTIFACT_DIR`).

Public bytes (commit these, not `sp1-datas/`): `zk-circuit/fixtures/happy/{groth16,journal}.bin` and `vkey.bytes32.txt`. On-chain packing uses `groth16.bin` (`proof.bytes()`), not the SDK suitcase `proof.bin`.
