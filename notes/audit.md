# Tooling scans

CI: `.github/workflows/security.yml`. Ignore list: `.cargo/audit.toml`. Commands: `.cursor/rules/05-tests-security.mdc`.

Bumped in `Cargo.lock` when a compatible release exists. Waived only when Solana/SP1 pin the crate (one line per ID in `audit.toml`).

| Advisory | Action |
| --- | --- |
| RUSTSEC-2026-0344 (`ruzstd` / `object`) | Waived. Host/SP1 only. |
| RUSTSEC-2026-0381 (`webpki-roots` 0.26 via `tonic` 0.12) | Waived. Host TLS. 1.0.x path is 1.0.10. |
| RUSTSEC-2026-0382 (`rustls-webpki`) | Bumped to **0.103.16**. |
| RUSTSEC-2026-0385 (`ctutils` 0.4.2) | Bumped to **0.4.4**. |
| RUSTSEC-2026-0387 (`crypto-bigint` 0.5 / `elliptic-curve` 0.13) | Waived. Needs k256/SP1 stack bump (≥0.7.6). |
| RUSTSEC-2026-0391 / 0398 / 0403 / 0410 (`zstd*` via `sp1-sdk` 6.5) | Waived. Host prove artifacts only. Needs `sp1-sdk` ≥6.6. |
