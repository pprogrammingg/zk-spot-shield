# Tooling scans

```bash
cargo audit
cargo clippy -p zk_spot_shield -p zk-circuit-io -p client --locked --all-targets -- -D warnings
```

CI: `.github/workflows/security.yml`. Ignore list: `.cargo/audit.toml`.

| Advisory | Action |
| --- | --- |
| RUSTSEC-2026-0344 (`ruzstd` via `sp1-sdk` → `object`) | Ignored. Host only; we do not decode untrusted zstd. Revisit on `sp1-sdk` bump. |
| RUSTSEC-2026-0382 (`rustls-webpki` 0.103.15) | Bumped to **0.103.16** in `Cargo.lock`. |
| RUSTSEC-2026-0381 (`webpki-roots` 1.0.9 / 0.26.11) | 1.0.x bumped to **1.0.10**. 0.26.11 stays via `tonic` 0.12 (`sp1-prover`); waived — host TLS only. |
