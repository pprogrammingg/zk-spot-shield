# Tests

| Suite | Command | What it checks | In CI |
| --- | --- | --- | --- |
| Program unit tests | `cargo test -p zk_spot_shield --lib` | Account sizes, PDA seeds and derivation | Yes |
| Circuit I/O tests | `cargo test -p zk-circuit-io --lib` | Poseidon leaf, Merkle path, nullifier, recorded journal and proof bytes | Yes |
| Program integration | `anchor build` + LiteSVM | Initialize instructions in an in-memory runtime | Yes |
| Host tests | `cargo test -p zk-circuit-host --lib` | Happy fixture matches inputs; forged path rejected by the guest | Local only |

Default tests never call a live prover or cluster. The one real proof was recorded once and is reused as a fixture.

All 32-byte values below are shown as hex. The JSON mirrors the Rust struct `PrivateInputs`. Each `merkle_path` entry is a `sibling` hash and an `is_right` flag (`true` means the sibling sits on the right, so the parent is `hash(current, sibling)`).

## Happy Path Example

The test note: secret of all `0x01` bytes, user address of all `0x02`, balance `1000`, trading `100` units of a mint of all `0x03`. The note sits at position 0 of an otherwise empty depth-20 tree. In an empty tree, each sibling is the hash of an all-zero subtree of the matching height, which is why level 0 is all zeros and every flag is `true`.

```json
{
  "secret":        "0x0101010101010101010101010101010101010101010101010101010101010101",
  "user_address":  "0x0202020202020202020202020202020202020202020202020202020202020202",
  "balance": 1000,
  "requested_swap_amount": 100,
  "asset_id_mint": "0x0303030303030303030303030303030303030303030303030303030303030303",
  "merkle_path": [
    { "level": 0,  "sibling": "0x0000000000000000000000000000000000000000000000000000000000000000", "is_right": true },
    { "level": 1,  "sibling": "0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864", "is_right": true },
    { "level": 2,  "sibling": "0x1069673dcdb12263df301a6ff584a7ec261a44cb9dc68df067a4774460b1f1e1", "is_right": true },
    { "level": 3,  "sibling": "0x18f43331537ee2af2e3d758d50f72106467c6eea50371dd528d57eb2b856d238", "is_right": true },
    { "level": 4,  "sibling": "0x07f9d837cb17b0d36320ffe93ba52345f1b728571a568265caac97559dbc952a", "is_right": true },
    { "level": 5,  "sibling": "0x2b94cf5e8746b3f5c9631f4c5df32907a699c58c94b2ad4d7b5cec1639183f55", "is_right": true },
    { "level": 6,  "sibling": "0x2dee93c5a666459646ea7d22cca9e1bcfed71e6951b953611d11dda32ea09d78", "is_right": true },
    { "level": 7,  "sibling": "0x078295e5a22b84e982cf601eb639597b8b0515a88cb5ac7fa8a4aabe3c87349d", "is_right": true },
    { "level": 8,  "sibling": "0x2fa5e5f18f6027a6501bec864564472a616b2e274a41211a444cbe3a99f3cc61", "is_right": true },
    { "level": 9,  "sibling": "0x0e884376d0d8fd21ecb780389e941f66e45e7acce3e228ab3e2156a614fcd747", "is_right": true },
    { "level": 10, "sibling": "0x1b7201da72494f1e28717ad1a52eb469f95892f957713533de6175e5da190af2", "is_right": true },
    { "level": 11, "sibling": "0x1f8d8822725e36385200c0b201249819a6e6e1e4650808b5bebc6bface7d7636", "is_right": true },
    { "level": 12, "sibling": "0x2c5d82f66c914bafb9701589ba8cfcfb6162b0a12acf88a8d0879a0471b5f85a", "is_right": true },
    { "level": 13, "sibling": "0x14c54148a0940bb820957f5adf3fa1134ef5c4aaa113f4646458f270e0bfbfd0", "is_right": true },
    { "level": 14, "sibling": "0x190d33b12f986f961e10c0ee44d8b9af11be25588cad89d416118e4bf4ebe80c", "is_right": true },
    { "level": 15, "sibling": "0x22f98aa9ce704152ac17354914ad73ed1167ae6596af510aa5b3649325e06c92", "is_right": true },
    { "level": 16, "sibling": "0x2a7c7c9b6ce5880b9f6f228d72bf6a575a526f29c66ecceef8b753d38bba7323", "is_right": true },
    { "level": 17, "sibling": "0x2e8186e558698ec1c67af9c14d463ffc470043c9c2988b954d75dd643f36b992", "is_right": true },
    { "level": 18, "sibling": "0x0f57c5571e9a4eab49e2c8cf050dae948aef6ead647392273546249d1c1ff10f", "is_right": true },
    { "level": 19, "sibling": "0x1830ee67b5fb554ad5f63d4388800e1cfe78e310697d46e43c9ce36134f72cca", "is_right": true }
  ],
  "expected_root": "0x1aa60d6eb3b8b764ce14a2d49f45e3732bee6ddd962709eb3d26fdbcd814f354"
}
```

Inside the guest:

```json
{
  "leaf":          "0x263ff17437c665e4a47862859942f835fcf68c55642238cc4a2d3f82d09d6a76",
  "computed_root": "0x1aa60d6eb3b8b764ce14a2d49f45e3732bee6ddd962709eb3d26fdbcd814f354",
  "root_matches_expected": true,
  "solvent": true
}
```

The guest commits this journal, which is also what the recorded `journal.bin` decodes to:

```json
{
  "requested_swap_amount": 100,
  "asset_id_mint": "0x0303030303030303030303030303030303030303030303030303030303030303",
  "nullifier":     "0x1023e6c76d8f6adae4f07e565b323cd2e94fb547fe4c413c83664eb28314f8bd",
  "merkle_root":   "0x1aa60d6eb3b8b764ce14a2d49f45e3732bee6ddd962709eb3d26fdbcd814f354"
}
```

The tests `happy_fixture_journal_matches_happy_path_inputs` (host) and `happy_journal_matches_io_hashes` (io) check that the recorded journal matches these inputs exactly.

## Corrupted Merkle Path

The test starts from the happy input and changes one fact: the level-0 sibling becomes all `0x04` bytes. Everything else, including `expected_root`, stays the same. This models someone claiming membership with a path that does not belong to the approved tree.

```json hl_lines="8 28"
{
  "secret":        "0x0101010101010101010101010101010101010101010101010101010101010101",
  "user_address":  "0x0202020202020202020202020202020202020202020202020202020202020202",
  "balance": 1000,
  "requested_swap_amount": 100,
  "asset_id_mint": "0x0303030303030303030303030303030303030303030303030303030303030303",
  "merkle_path": [
    { "level": 0,  "sibling": "0x0404040404040404040404040404040404040404040404040404040404040404", "is_right": true },
    { "level": 1,  "sibling": "0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864", "is_right": true },
    { "level": 2,  "sibling": "0x1069673dcdb12263df301a6ff584a7ec261a44cb9dc68df067a4774460b1f1e1", "is_right": true },
    { "level": 3,  "sibling": "0x18f43331537ee2af2e3d758d50f72106467c6eea50371dd528d57eb2b856d238", "is_right": true },
    { "level": 4,  "sibling": "0x07f9d837cb17b0d36320ffe93ba52345f1b728571a568265caac97559dbc952a", "is_right": true },
    { "level": 5,  "sibling": "0x2b94cf5e8746b3f5c9631f4c5df32907a699c58c94b2ad4d7b5cec1639183f55", "is_right": true },
    { "level": 6,  "sibling": "0x2dee93c5a666459646ea7d22cca9e1bcfed71e6951b953611d11dda32ea09d78", "is_right": true },
    { "level": 7,  "sibling": "0x078295e5a22b84e982cf601eb639597b8b0515a88cb5ac7fa8a4aabe3c87349d", "is_right": true },
    { "level": 8,  "sibling": "0x2fa5e5f18f6027a6501bec864564472a616b2e274a41211a444cbe3a99f3cc61", "is_right": true },
    { "level": 9,  "sibling": "0x0e884376d0d8fd21ecb780389e941f66e45e7acce3e228ab3e2156a614fcd747", "is_right": true },
    { "level": 10, "sibling": "0x1b7201da72494f1e28717ad1a52eb469f95892f957713533de6175e5da190af2", "is_right": true },
    { "level": 11, "sibling": "0x1f8d8822725e36385200c0b201249819a6e6e1e4650808b5bebc6bface7d7636", "is_right": true },
    { "level": 12, "sibling": "0x2c5d82f66c914bafb9701589ba8cfcfb6162b0a12acf88a8d0879a0471b5f85a", "is_right": true },
    { "level": 13, "sibling": "0x14c54148a0940bb820957f5adf3fa1134ef5c4aaa113f4646458f270e0bfbfd0", "is_right": true },
    { "level": 14, "sibling": "0x190d33b12f986f961e10c0ee44d8b9af11be25588cad89d416118e4bf4ebe80c", "is_right": true },
    { "level": 15, "sibling": "0x22f98aa9ce704152ac17354914ad73ed1167ae6596af510aa5b3649325e06c92", "is_right": true },
    { "level": 16, "sibling": "0x2a7c7c9b6ce5880b9f6f228d72bf6a575a526f29c66ecceef8b753d38bba7323", "is_right": true },
    { "level": 17, "sibling": "0x2e8186e558698ec1c67af9c14d463ffc470043c9c2988b954d75dd643f36b992", "is_right": true },
    { "level": 18, "sibling": "0x0f57c5571e9a4eab49e2c8cf050dae948aef6ead647392273546249d1c1ff10f", "is_right": true },
    { "level": 19, "sibling": "0x1830ee67b5fb554ad5f63d4388800e1cfe78e310697d46e43c9ce36134f72cca", "is_right": true }
  ],
  "expected_root": "0x1aa60d6eb3b8b764ce14a2d49f45e3732bee6ddd962709eb3d26fdbcd814f354"
}
```

Inside the guest, the leaf is unchanged, but hashing up the forged path lands on a different root:

```json hl_lines="3 4"
{
  "leaf":          "0x263ff17437c665e4a47862859942f835fcf68c55642238cc4a2d3f82d09d6a76",
  "computed_root": "0x1c7dd2b9b071543a5d2c1b5495b82726453a4853656421864a436e5a13619def",
  "root_matches_expected": false,
  "solvent": true
}
```

The guest panics with `Merkle root mismatch: Computed path does not match state root`. No journal is committed, so no proof can exist for this input.

Two host tests cover this case:

- `corrupt_merkle_path_mismatches_expected_root` recomputes the root natively and asserts it differs from `expected_root`. It runs in milliseconds.
- `corrupt_merkle_path_execute_fails` runs the real guest in the zkVM and asserts the run is rejected: either an error, or an empty journal because `commit` never ran.

!!! note "Why `expected_root` is not recomputed"
    If the test recomputed `expected_root` from the forged path, path and root would agree again and the guest would accept the input. The attack being modelled is "prove membership against the real root with a fake path", so the root must stay the real one.
