# Happy-path fixture (Day 17)

**One pack** feeds host execute, remote Groth16 prove, and (later) the TS client.

| File | Role |
| --- | --- |
| [`../happy.json`](../happy.json) | Portable dump (hex) of the private inputs + derived leaf/nullifier |
| Rust [`happy_path_inputs()`](../../host/src/fixtures.rs) | Runtime builder — same values, via `zk-circuit-io` hashes |
| `journal.bin` / `groth16.bin` / `vkey.bytes32.txt` | Recorded proof artifacts for that pack only |

Test bytes only (`secret = 0x01…`, not user funds). Guest I/O shape stays `PrivateInputs` / `PublicOutputs` in `zk-circuit-io` — do not fork a second guest fixture.

**Generate proof artifacts once** (needs deposited `$PROVE` + root `.env`): see [`prove_network.md`](../../further_explanations/prove_network.md).

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
  cargo run -p zk-circuit-host --features network --release
```

Writes `groth16.bin` (356 bytes), `journal.bin` (104 bytes), `vkey.bytes32.txt` here. Guest/io change → prove again and refresh `happy.json` if inputs change. Numbers for this wrap: [`notes/proving.md`](../../../notes/proving.md).

Default CI does **not** wrap. Host `--lib` asserts `happy.json` + `journal.bin` match `happy_path_inputs()`.
