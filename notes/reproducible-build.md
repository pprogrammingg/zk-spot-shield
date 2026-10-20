# Reproducible program hash

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
unset CARGO_TARGET_DIR
anchor build --ignore-keys
shasum -a 256 target/deploy/zk_spot_shield.so
# if the binary is only under SBPF:
shasum -a 256 target/sbpf-solana-solana/release/zk_spot_shield.so
```

Or run [`scripts/program-hash.sh`](../scripts/program-hash.sh).

| Field | Where to record it |
| --- | --- |
| Program id | `Anchor.toml` `[programs.localnet]` and `declare_id!` |
| `.so` SHA-256 | this note after a freeze build |
| Guest ELF | `cargo prove build` output (gitignored); verifying key in `constants::VKEY_HASH` |
| Groth16 fixtures | `zk-circuit/fixtures/happy/` |
| Anchor | `Anchor.toml` `toolchain.anchor_version` |
| Host rustc | `rust-toolchain.toml` |

Hash on the machine that built. An unrecorded hash is not a Devnet canonical until deploy is written in [`devnet.md`](./devnet.md).
