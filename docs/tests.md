# Tests

| Suite | Command | What it checks | In CI |
| --- | --- | --- | --- |
| Program unit tests | `cargo test -p zk_spot_shield --lib` | Account sizes, PDA seeds | Yes |
| Circuit I/O tests | `cargo test -p zk-circuit-io --lib` | Poseidon, Merkle, recorded journal and proof | Yes |
| Client / relayer | `cargo test -p client --lib` and `cargo test -p zk-spot-shield-relayer --lib` | Compute budget, fee payer dry-run, HTTP 402 | Relayer yes |
| Program integration | `anchor build` + LiteSVM | Deposit and vault-only settle against a `.so` | Yes |
| SDK / UI | `cd sdk && npm test` / `cd ui && npm test` | Journal decode, settle encoding, RPC fallback | Local |
| Host tests | `cargo test -p zk-circuit-host --lib` | Fixture matches inputs; forged path rejected | Local only |
| Devnet | `DEVNET_ONCHAIN=1` in `sdk/` | Live cluster reads | Opt-in; skip in CI |

Default tests never call a live prover or cluster. The one real proof was recorded once and is reused as a fixture. Canonical dump: [`zk-circuit/fixtures/happy.json`](https://github.com/pprogrammingg/zk-spot-shield/blob/main/zk-circuit/fixtures/happy.json). Host builds the same pack via `happy_path_inputs()`.

## Happy path

Toy note: secret of all `0x01`, address of all `0x02`, mint of all `0x03`, balance `1000`, swap `100`. Index 0 of an otherwise empty depth-20 tree (level-0 sibling is zeros; every `is_right` flag is `true`). Full 20-sibling path is in `happy.json`.

| Field | Hex |
| --- | --- |
| Leaf | `0x263ff17437c665e4a47862859942f835fcf68c55642238cc4a2d3f82d09d6a76` |
| Root | `0x1aa60d6eb3b8b764ce14a2d49f45e3732bee6ddd962709eb3d26fdbcd814f354` |
| Nullifier | `0x1023e6c76d8f6adae4f07e565b323cd2e94fb547fe4c413c83664eb28314f8bd` |

Journal (also `journal.bin`): amount `100`, mint `0x03…03`, that nullifier, that root.

`happy_fixture_journal_matches_happy_path_inputs` (host) and `happy_journal_matches_io_hashes` (io) check the recorded journal matches these inputs.

## Corrupted Merkle path

Same inputs, but the level-0 sibling is all `0x04`. `expected_root` stays the real root. That models “prove membership against the real tree with a fake path.”

Leaf is unchanged. Forged path hashes to `0x1c7dd2b9b071543a5d2c1b5495b82726453a4853656421864a436e5a13619def`. Guest panics: `Merkle root mismatch: Computed path does not match state root`. No journal, so no proof.

- `corrupt_merkle_path_mismatches_expected_root` recomputes the root natively (milliseconds).
- `corrupt_merkle_path_execute_fails` runs the guest in the zkVM and asserts reject.

If the test recomputed `expected_root` from the forged path, path and root would agree and the guest would accept. The root must stay the real one.

Empty-tree path construction: [`further_explanations/merkle_trees.md`](https://github.com/pprogrammingg/zk-spot-shield/blob/main/further_explanations/merkle_trees.md).
