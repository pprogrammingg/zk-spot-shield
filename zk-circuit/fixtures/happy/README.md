# Happy-path Groth16 fixture (this guest only)

Inputs are `zk_circuit_host::fixtures::happy_path_inputs()` (fixed test secret, address, amount, Merkle path). Not user funds.

**Generate once** (needs deposited `$PROVE` + root `.env`): see [`prove_network.md`](../../further_explanations/prove_network.md).

```bash
SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info cargo run -p zk-circuit-host --release
```

Writes `groth16.bin` (356 bytes), `journal.bin` (104 bytes), `vkey.bytes32.txt` here. Guest/io change → prove again. Numbers for this wrap: [`notes/proving.md`](../../../notes/proving.md).

Default CI does **not** wrap. `zk-circuit-io --lib` reads these files. You produce them with `cargo run` (MetaMask key in root `.env`), not `cargo test`.
