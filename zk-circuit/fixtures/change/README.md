# Change-note fixture

Remote Groth16 of the **current** guest (spend 100 / change 900). Same toy witness as [`../happy.json`](../happy.json). Journal is **144 bytes** (legacy 104 + `change_amount` + `change_leaf`).

Do **not** overwrite [`../happy/`](../happy/). That pack is the frozen 104-byte CI proof (`vkey` `0x00b3a15c…35d7`). This wrap has a **new** verifying key.

| Field | Value |
| --- | --- |
| Date (UTC) | 2026-10-24T14:00Z |
| Groth16 wall time | 25.1 s |
| Proof length | 356 bytes |
| Journal | 144 bytes; amount `100`; change `900` |
| Quoted base fee | 0.4032 `$PROVE` |
| Request | [0x3cf7…5e65](https://explorer.succinct.xyz/request/0x3cf702a04e3296db58c5f6411140d8af9686d8f69de5ad0e719300ab69e55e65) |
| `vk.bytes32()` | `0x00435696d666d305172306e3895a44017e188663de9b3876970ae270345fc240` |

Rebuild:

```bash
cargo prove build -p guest --elf-name guest --output-directory target/elf
SKIP_HAPPY_FIXTURE=1 ARTIFACT_DIR=sp1-artifacts/change \
  SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
# Then copy journal.bin groth16.bin vkey.bytes32.txt into this directory.
# Do not commit proof.bin.
```
