# Devnet operator steps (Days 58–64)

This is the deploy guide the README calls **Operator steps**. Fill the tables as you go. Until they have values, localnet and LiteSVM stay the default.

Env template: [`scripts/devnet.env.example`](../scripts/devnet.env.example). Copy it to a gitignored repo-root `.env` or export in the shell. Never commit keypairs.

`npm run test:e2e` / `sdk/scripts/e2e-localnet.ts` is **local validator only**. It preloads accounts and does not talk to Devnet. Do not point it at `api.devnet.solana.com`.

Do these days in order. **58–63 done.** Day 62 health-checked (skip send). **Day 63 settle landed.** Next: Day 64 tag after you commit. Paste Explorer URLs with `?cluster=devnet`.

---

## Day 57 recap (hash procedure)

You already have this. Freeze the binary **before** Day 59 so the Devnet `.so` hash is the one you recorded.

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
unset CARGO_TARGET_DIR
anchor build --ignore-keys
./scripts/program-hash.sh
```

Record the SHA-256 in the Day 59 table below (and [`reproducible-build.md`](./reproducible-build.md)). Hash on the machine that built. **Deploy this file:** `target/deploy/zk_spot_shield.so`. The SBPF path can hash differently; do not treat it as the freeze unless that is what `anchor deploy` uploads.

---

## Day 58 — Keys + SOL

1. Point the Solana CLI at Devnet and print the deploy wallet (this pubkey becomes protocol admin if it pays init):

```bash
solana config set --url https://api.devnet.solana.com
solana address
solana balance
```

2. Fund it. Public faucet (2 SOL at a time; may rate-limit):

```bash
solana airdrop 2
solana balance
```

If that fails, use [faucet.solana.com](https://faucet.solana.com/) or a second wallet. You want **at least ~3 SOL** for program deploy rent plus later PDAs.

3. Optional: copy [`scripts/devnet.env.example`](../scripts/devnet.env.example) and set `ANCHOR_WALLET` if you are not using `~/.config/solana/id.json`.

| Field                 | Value                                          |
| --------------------- | ---------------------------------------------- |
| Cluster               | `https://api.devnet.solana.com`                |
| Deploy / admin pubkey | `9sLGgVEotwD8gJgL5HhJdP7w61zeCrJ25NNkJPM3BAaX` |
| Balance after airdrop | 2.5                                            |

**Paste back:** admin pubkey.

---

## Day 59 — Deploy program

Pinned program id (source + cluster): `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G`.

**Every deploy:** if the printed program id **equals** the pin, only the `.so` bytes changed — **upgrade** that same address (do not `anchor deploy` a second id). If the printed id **differs**, stop. The `.so` embeds `declare_id!`; a mismatch fails at runtime (`DeclaredProgramIdMismatch`). Align source to the **on-chain** id, rebuild, then **upgrade** that account:

1. `programs/zk_spot_shield/src/lib.rs` — `declare_id!("…")`
2. `Anchor.toml` — `[programs.localnet]` and `[programs.devnet]`
3. `sdk/idl/zk_spot_shield.json` — `"address"`
4. `ui/src/config.ts` — `DEFAULT_PROGRAM_ID`
5. `sdk/scripts/e2e-localnet.ts` — `PROGRAM_ID`
6. `scripts/devnet.env.example`, this note, README

Keep `target/deploy/zk_spot_shield-keypair.json` (gitignored). Its pubkey **is** the program id. Prefer upgrade over a new keypair so PDAs stay put. Init (Day 60) only after dump SHA-256 matches the local freeze.

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
unset CARGO_TARGET_DIR
anchor build --ignore-keys
./scripts/program-hash.sh

# First time only (creates the program account). After that, upgrade.
anchor deploy --provider.cluster devnet

# Same program id, new bytecode (including after a declare_id! pin):
anchor upgrade target/deploy/zk_spot_shield.so \
  --program-id 74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G \
  --provider.cluster devnet

solana program show 74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G --url devnet
solana program dump 74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G dumped.so --url devnet
shasum -a 256 dumped.so target/deploy/zk_spot_shield.so
```

First Devnet deploy used a new keypair (`74UW…`). Source is synced and the **upgrade landed** (2026-10-21): dump SHA-256 matches the local freeze. Day 60 init is unblocked.

Explorer: `https://explorer.solana.com/address/74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G?cluster=devnet`

| Field | Value |
| --- | --- |
| Program id (on-chain + source) | `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G` |
| `.so` SHA-256 (first pin upgrade) | `f9030f956eb2e2d967254c99355f2c49bd47215401aae8b3b082ca11ec473277` |
| Upgrade tx (pin) | `5bwFz3PfNHzBHNHmTznvLtt1ySvJLHZdrkR6wvyPN76ENMVN7JZxfno61jvGz6MN5vKPb5vLPxoDVY53Q3fbMAfn` |
| Current `.so` SHA-256 | `b14e59e6cd0ecd67de4cc46c912f123749d81ee02046e7956945384be74c8c9e` (local `target/deploy`; Day 63 `set_vkey_hash`) |
| Upgrade tx (Day 61) | `3dDJRMHZxD7YcwnaH62J2Tk98kKToeXRoYGPk36ggtE5GNdXnJH5aRy6LXCkoSE6yqy1Lw67gb2uQfiodR3wQtak` |
| Upgrade tx (Day 63) | `3rAxUJn5QLvajNuvEu8rVvXvN5cz4R5CSGC4kfuq3j8mEnhmwC9nXVWVetkYoPhhtfXj1fjFcHc2wKsXUMG9qsh9` |
| Upgrade authority | `9sLGgVEotwD8gJgL5HhJdP7w61zeCrJ25NNkJPM3BAaX` |
| ProgramData | `9t5Z4LnoQxpnwb11g8hqDnhMrVzmH77oaLofSCQVgQr2` |
| IDL metadata | `HkAVd4rm1uHtKo1VSXS9wcF1MzsBbzDNtU83gZKZGajv` |
| Last deployed slot | Day 63 upgrade (extend +10 KB then resume buffer `H1gP…`) |
| Explorer (program) | https://explorer.solana.com/address/74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G?cluster=devnet |
| Explorer (upgrade tx) | https://explorer.solana.com/tx/3rAxUJn5QLvajNuvEu8rVvXvN5cz4R5CSGC4kfuq3j8mEnhmwC9nXVWVetkYoPhhtfXj1fjFcHc2wKsXUMG9qsh9?cluster=devnet |

**Paste back:** program id + `.so` SHA-256.

---

## Day 60 — Init config, vkey, root

Creates `GlobalConfig` (payer becomes `authority`; init writes program `VKEY_HASH`) and registers the happy-path Merkle root from `zk-circuit/fixtures/happy.json` (`expected_root`). After a guest rebuild, protocol admin rotates the on-chain hash with `set_vkey_hash` (`NEW_VKEY_HASH=0x… npm run devnet:set-vkey`).

From `sdk/` (wallet must be the Day 58 key; needs SOL for PDA rent). Script: [`sdk/scripts/devnet-init.ts`](../sdk/scripts/devnet-init.ts).

```bash
cd sdk && npm install
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json
npm run devnet:init-config
```

Explorer (config): `https://explorer.solana.com/address/BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY?cluster=devnet`

| Field | Value |
| --- | --- |
| `GlobalConfig` | `BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY` |
| Explorer (config) | https://explorer.solana.com/address/BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY?cluster=devnet |
| Init config tx | `3beD7cUyN7SuqK629UVEW1bknb9Tcnijhwj8zKUU2V29ri5b14waMLxdaa8ZfzdLxjAyEGBMEAAWiYHNK2sn3dQe` |
| First `CleanFundsRoot` (toy happy `expected_root`) | `6CvBuqnxswcq8g7QJcybGRtGragupRzNdmW37fD7NMLA` |
| Register root tx | `vmXeNCSVYzBPRzkTonRHUaxQJpft48U5LDAVcSgv9pC71h6rQQgLNJ3cv3FMRzyiUisaCdEcJw2VcU8d1xPtvSx` |
| Explorer (root) | https://explorer.solana.com/address/6CvBuqnxswcq8g7QJcybGRtGragupRzNdmW37fD7NMLA?cluster=devnet |

**Paste back:** GlobalConfig pubkey + Explorer URL.

---

## Day 61 — Set vault mints + ATAs (done)

`set_vault_mints` (protocol admin, once) + two toy Devnet mints we control (so Day 63 can `mintTo`) + vault ATAs. Not Circle USDC.

```bash
cd sdk
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json
npm run devnet:set-mints
```

Explorer (vault): `https://explorer.solana.com/address/A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd?cluster=devnet`

| Field | Value |
| --- | --- |
| Vault | `A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd` |
| Init vault tx | `4nK1iSerMzPCHoQA5b9CJ18NvDR28kF9bWUkVwPdyvtRtYMWgankLmyBxwRjh3xJx7aE18BbbpSGwcxDhbACAJww` |
| `mint_a` | `DNNkZidPWCHxoyuaQ3h5N8FUCWaJZddEYvF7JyHHKbVE` |
| `mint_b` | `HfgizYEd9aVxk6hAXjEtoac1jcVHA7KyrqyJYdm5wqzg` |
| Vault token account A | `EVmRfTNFecvTBtW8Zcv5BrQJwZTA5HbSJMVBUJTEdTGQ` |
| Vault token account B | `8wBhwWC9c52D2qgSzYxNsCEN2YMJgsM916c8kBFrMTso` |
| `set_vault_mints` tx | `2oadPEEozV4ZXK824MuGqhS6ev43sB8Aj78wsvvfFmmV8MEeZzsFrXSFbXqhcj8VXsHNKFTnVxWrt5PscqNjaQnm` |
| Create ATAs tx | `2VZzevEVyEcPuY8jfx9EAs8hNuajfu6M5dve5E4QZLF6ETzngKp6vWHKZJzDcoHrEboSv7FkUdhm5bEpi35oEgYU` |
| Explorer (vault) | https://explorer.solana.com/address/A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd?cluster=devnet |
| Explorer (mint A) | https://explorer.solana.com/address/DNNkZidPWCHxoyuaQ3h5N8FUCWaJZddEYvF7JyHHKbVE?cluster=devnet |
| Explorer (mint B) | https://explorer.solana.com/address/HfgizYEd9aVxk6hAXjEtoac1jcVHA7KyrqyJYdm5wqzg?cluster=devnet |

---

## Day 62 — Relayer on Devnet RPC (skip send)

Local process only. `/health` returned `{"status":"ok","service":"zk-spot-shield-relayer"}` on `127.0.0.1:8787` (dry-run; `RELAYER_RPC` unused). Not required for Day 63 — self-submit as fee payer.

---

## Day 63 — One live swap (SDK, not UI)

**Not** `npm run test:e2e`. That script is localnet.

Custody is on-chain (1.0 of each mint, 6 decimals, into vault ATAs). Live settle used a network Groth16 whose journal mint is vault `mint_a` (`DNNk…`), amount `100000`. Guest ELF rebuilt (Poseidon reduces 32-byte keys into BN254 Fr); on-chain `vkey_hash` rotated with `set_vkey_hash`. Happy fixture (`0x0303…03`) was not overwritten.

```bash
cd sdk
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json
npm run devnet:deposit
```

**Manual (you):** network prove whose journal `asset_id_mint` is vault `mint_a` or `mint_b`, amount ≤ deposited note, Merkle root registered. Then SDK settle as wallet fee payer. Do not commit note secrets. Command: [`notes/proving.md`](./proving.md).

| Field | Value |
| --- | --- |
| Deposit A tx | `3iaybGa2mLo5GehJ5c3uqUW11MQp4SijNPJcTuyGa97FCdWD1X6Q1Tt4Jm7CEWWWCL42T4jRABZydvA7cSNn44VM` |
| Deposit B tx | `3zGiSJ6ykHikFTsT4bYZVA9c5xvdd4k64eMTBXjcRFjKW8kckz4nvfnfdRzTv8L4oDmywKcR8boFUz7SoxgdDJVD` |
| Deposit leaf A | `GfXdsoSDcqxvh44PdiaSm5VikMvy65ypFAn5DB7s5xW2` |
| Deposit leaf B | `DrJ4PCJqk6reSTU5jJ2fQRuePKtM2mHGmDZAPPTG8Mev` |
| Leaf A (hex) | `2376354f46c91e6ddf5851b57e3bc6444a62035fc046afa599dd7e8e3d5d7014` |
| Leaf B (hex) | `15c538a26305c6f989f281b98625f05b167eb906fd2d02b13fa2d4c622593784` |
| `reserve_a` / `reserve_b` after settle | `1000000` / `900000` (spent mint A; vault-only decrements payout mint B) |
| On-chain `vkey_hash` | `0x00d6c588a6e51b5715930c0d9495cb6c0c87e8e32fd2775b0a371df81aca6991` |
| `set_vkey_hash` tx | `CVVahBk8B7Z4k7GcuLUBnqLpa6Mp7zJjtUb6FrK5vLjY9xjMNCMs4Gm2dzp6L3MNzgVFWg2PpERUC4a6wds27LN` |
| Live `CleanFundsRoot` | `5ATcjDGsM8WPHbjQSFACPWAaraNZ5NNLGHkBTfW6cNwD` |
| Register live root tx | `3ho3tpmfzRydf6UiiaDRGxD8LW64Hche2vz53YmdBX9T7VSu7hWMTPJrhB7dmTnui9gwNPeEd85is7EK5CZPSa2e` |
| Merkle root (hex) | `08d4c239e1f6b3afcdbf68dd8e6b36e853fc6a418bb85b05dc8d14b23acad818` |
| Groth16 wrap | gitignored `sp1-artifacts/devnet-settle/` ([Succinct 0x3f8c…e344](https://explorer.succinct.xyz/request/0x3f8c073fc36fa195ad057d04164d4b60253467d334af3f321eaeb8ec4f52e344)) |
| Settle signature | `3V7YF2ULaHULbnSJinSTahDw5gJSMGkvB2GuQyLevnzh9A15d4yoEcXNbCMfvbkzAcdDBHStYpzkECMrbYGz5NyF` |
| Explorer (tx) | https://explorer.solana.com/tx/3V7YF2ULaHULbnSJinSTahDw5gJSMGkvB2GuQyLevnzh9A15d4yoEcXNbCMfvbkzAcdDBHStYpzkECMrbYGz5NyF?cluster=devnet |

---

## Day 64 — Docs and tag (after a Day 63 settle signature)

1. Program id + current hash are in this note, README, and [`reproducible-build.md`](./reproducible-build.md).
2. Keep the `anchor build` log, `./scripts/program-hash.sh` output, `VKEY_HASH`, and fixture paths.
3. Tag `v1.0.0-devnet` **after** you commit (Day 63 settle signature is already in the table above).

Related: UI click-through [`manual-devnet.md`](./manual-devnet.md) (Days 66–67). Automated reads [`devnet-tests.md`](./devnet-tests.md) (`DEVNET_ONCHAIN=1`, Day 68).
