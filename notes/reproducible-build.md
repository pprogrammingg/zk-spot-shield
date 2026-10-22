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
| Program id | `Anchor.toml` `[programs.localnet]` / `[programs.devnet]` and `declare_id!` — `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G` |
| `.so` SHA-256 (deploy, current local) | `b14e59e6cd0ecd67de4cc46c912f123749d81ee02046e7956945384be74c8c9e` — `target/deploy/zk_spot_shield.so` (2026-10-21, Day 63 `set_vkey_hash`) |
| `.so` SHA-256 (SBPF, same build) | `7790bcd2cc8de10d6579fe4f478cd8372d0bd809742d1df01ef61e9cb5770327` — **not** the deploy path |
| `.so` SHA-256 (Devnet dump) | not re-dumped this session; upgrade tx `3rAxUJn5…` |
| Guest ELF | `cargo prove build` output (gitignored); verifying key in `constants::VKEY_HASH` |
| Groth16 fixtures | `zk-circuit/fixtures/happy/` |
| Anchor | `Anchor.toml` `toolchain.anchor_version` |
| Host rustc | `rust-toolchain.toml` |

Hash on the machine that built. Compare Devnet dumps to the **deploy** digest. Cluster canonical only after `anchor deploy` is written in [`devnet.md`](./devnet.md).
