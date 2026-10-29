# Happy-path fixture

One recorded witness feeds host execute, remote Groth16 prove, and the TypeScript client.

| File | Role |
| --- | --- |
| [`../happy.json`](../happy.json) | Hex dump of private inputs plus derived leaf and nullifier |
| Rust `happy_path_inputs()` | Same values, hashed in `zk-circuit-io` |
| `journal.bin` / `groth16.bin` / `vkey.bytes32.txt` | Recorded proof artifacts for that witness only |

Test bytes only (`secret = 0x01…`). Not user funds. Guest I/O stays `PrivateInputs` / `PublicOutputs` in `zk-circuit-io`.

Toy demo: spend **100**, keep change **900**. Journal is **144 bytes**. `vk.bytes32()` is `0x00435696…c240` (`programs/zk_spot_shield` `VKEY_HASH`).

Spend-all still encodes a **104-byte** journal (`change_amount = 0`); that encoding is not this Groth16 pack.

To regenerate proof artifacts (needs `$PROVE` and a root `.env`): see [`prove_network.md`](../../further_explanations/prove_network.md).

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

Writes `groth16.bin` (356 bytes), `journal.bin` (144 bytes), and `vkey.bytes32.txt`. If guest/io change, prove again and refresh `happy.json` if inputs change. Numbers: [`notes/proving.md`](../../../notes/proving.md).

A live Devnet wrap (arbitrary amounts, vault mint) must use `SKIP_HAPPY_FIXTURE=1 ARTIFACT_DIR=…` so it does not clobber this toy pack. Then `set_vkey_hash` to that wrap’s `vk.bytes32()`.

Default CI does **not** wrap. Host `--lib` checks that `happy.json` and `journal.bin` match `happy_path_inputs()`.
