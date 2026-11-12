# Devnet

Operator record. Until these tables are filled, localnet and LiteSVM stay the default.

Env template: [`scripts/devnet.env.example`](../scripts/devnet.env.example). Copy to a gitignored repo-root `.env` or export in the shell. Never commit keypairs.

`npm run test:e2e` / `sdk/scripts/e2e-localnet.ts` is **local validator only**. Do not point it at `api.devnet.solana.com`.

Tag `v1.0.0-devnet` is on `30c35d7`. Paste Explorer URLs with `?cluster=devnet`.

## Hash the binary

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
unset CARGO_TARGET_DIR
anchor build --ignore-keys
./scripts/program-hash.sh
```

Hash on the machine that built. **Deploy this file:** `target/deploy/zk_spot_shield.so`. The SBPF path can hash differently — see [`budgets.md`](./budgets.md).

## Keys and SOL

```bash
solana config set --url https://api.devnet.solana.com
solana address
solana balance
solana airdrop 2
```

Need **at least ~3 SOL** for program deploy rent plus later PDAs. If the faucet rate-limits, use [faucet.solana.com](https://faucet.solana.com/). Optional: set `ANCHOR_WALLET` if you are not using `~/.config/solana/id.json`.

| Field | Value |
| --- | --- |
| Cluster | `https://api.devnet.solana.com` |
| Deploy / admin pubkey | `9sLGgVEotwD8gJgL5HhJdP7w61zeCrJ25NNkJPM3BAaX` |
| Balance after airdrop | 2.5 |

## Deploy / upgrade

Pinned program id (source + cluster): `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G`.

If the printed program id **equals** the pin, **upgrade** that address. If it **differs**, stop — the `.so` embeds `declare_id!` and a mismatch fails at runtime (`DeclaredProgramIdMismatch`). Align source to the **on-chain** id, rebuild, then upgrade:

1. `programs/zk_spot_shield/src/lib.rs` — `declare_id!("…")`
2. `Anchor.toml` — `[programs.localnet]` and `[programs.devnet]`
3. `sdk/idl/zk_spot_shield.json` — `"address"`
4. `ui/src/config.ts` — `DEFAULT_PROGRAM_ID`
5. `sdk/scripts/e2e-localnet.ts` — `PROGRAM_ID`
6. `scripts/devnet.env.example`, this note, README

Keep `target/deploy/zk_spot_shield-keypair.json` (gitignored). Its pubkey **is** the program id. Prefer upgrade so PDAs stay put. Init only after dump SHA-256 matches the local freeze.

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
unset CARGO_TARGET_DIR
anchor build --ignore-keys
./scripts/program-hash.sh

# First time only:
anchor deploy --provider.cluster devnet

# Same program id, new bytecode:
anchor upgrade target/deploy/zk_spot_shield.so \
 --Program-id 74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G \
 --Provider.cluster devnet

solana program show 74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G --url devnet
solana program dump 74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G dumped.so --url devnet
shasum -a 256 dumped.so target/deploy/zk_spot_shield.so
```

First deploy used a new keypair (`74UW…`). Source is synced; dump SHA-256 matches the local freeze.

| Field | Value |
| --- | --- |
| Program id | `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G` |
| `.so` SHA-256 (first pin upgrade) | `f9030f956eb2e2d967254c99355f2c49bd47215401aae8b3b082ca11ec473277` |
| Upgrade tx (pin) | `5bwFz3PfNHzBHNHmTznvLtt1ySvJLHZdrkR6wvyPN76ENMVN7JZxfno61jvGz6MN5vKPb5vLPxoDVY53Q3fbMAfn` |
| Current `.so` SHA-256 | `63beffdf5bac2ec2d36f010ff9911e786fe26916703a3daedd5ae9fcaaec2d6b` |
| Upgrade tx (change-leaf settle accounts) | `Pue1Vi2kzBfaL2J7QnfUufocfgJkYzjLmh3iFuBRoNxMp3C9b8RzMAmzVhVJK3KPJP2TJ8B9miMczC6ZmbuYFMv` |
| Upgrade tx (mints) | `3dDJRMHZxD7YcwnaH62J2Tk98kKToeXRoYGPk36ggtE5GNdXnJH5aRy6LXCkoSE6yqy1Lw67gb2uQfiodR3wQtak` |
| Upgrade tx (vkey) | `3rAxUJn5QLvajNuvEu8rVvXvN5cz4R5CSGC4kfuq3j8mEnhmwC9nXVWVetkYoPhhtfXj1fjFcHc2wKsXUMG9qsh9` |
| Upgrade tx (public_spot_swap) | `3gjNnDj2ouwznknJNH6cmRzdcnbs9XKyUGbMBAuCxFTj45X6tD8YMPRbn1aghnCiZhzj3cDTGYiT2U7okPYb1cH2` |
| Upgrade authority | `9sLGgVEotwD8gJgL5HhJdP7w61zeCrJ25NNkJPM3BAaX` |
| ProgramData | `9t5Z4LnoQxpnwb11g8hqDnhMrVzmH77oaLofSCQVgQr2` |
| IDL metadata | `HkAVd4rm1uHtKo1VSXS9wcF1MzsBbzDNtU83gZKZGajv` |
| Last deployed slot | `518044860` |
| Explorer (program) | https://explorer.solana.com/address/74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G?cluster=devnet |
| Explorer (upgrade tx) | https://explorer.solana.com/tx/Pue1Vi2kzBfaL2J7QnfUufocfgJkYzjLmh3iFuBRoNxMp3C9b8RzMAmzVhVJK3KPJP2TJ8B9miMczC6ZmbuYFMv?cluster=devnet |

## Init config and roots

Creates `GlobalConfig` (payer becomes `authority`; init writes program `VKEY_HASH`) and registers the happy-path Merkle root from `zk-circuit/fixtures/happy.json`. After a guest rebuild, protocol admin rotates the on-chain hash with `set_vkey_hash` (`NEW_VKEY_HASH=0x… npm run devnet:set-vkey`).

From `sdk/` (wallet must be the admin key; needs SOL for PDA rent). Script: [`sdk/scripts/devnet-init.ts`](../sdk/scripts/devnet-init.ts).

```bash
cd sdk && npm install
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json
npm run devnet:init-config
```

| Field | Value |
| --- | --- |
| `GlobalConfig` | `BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY` |
| Init config tx | `3beD7cUyN7SuqK629UVEW1bknb9Tcnijhwj8zKUU2V29ri5b14waMLxdaa8ZfzdLxjAyEGBMEAAWiYHNK2sn3dQe` |
| First `CleanFundsRoot` (toy happy `expected_root`) | `6CvBuqnxswcq8g7QJcybGRtGragupRzNdmW37fD7NMLA` |
| Register root tx | `vmXeNCSVYzBPRzkTonRHUaxQJpft48U5LDAVcSgv9pC71h6rQQgLNJ3cv3FMRzyiUisaCdEcJw2VcU8d1xPtvSx` |
| Explorer (config) | https://explorer.solana.com/address/BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY?cluster=devnet |
| Explorer (root) | https://explorer.solana.com/address/6CvBuqnxswcq8g7QJcybGRtGragupRzNdmW37fD7NMLA?cluster=devnet |

## Vault mints and ATAs

`set_vault_mints` (protocol admin, once) + two toy Devnet mints we control (so settle can `mintTo`) + vault ATAs. Not Circle USDC.

```bash
cd sdk
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json
npm run devnet:set-mints
```

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

## Relayer

Local process only. `/health` returned `{"status":"ok","service":"zk-spot-shield-relayer"}` on `127.0.0.1:8787` (dry-run; `RELAYER_RPC` unused). Not required for live settle — self-submit as fee payer.

## Live settle

Custody is on-chain (1.0 of each mint, 6 decimals, into vault ATAs). Live settle used a network Groth16 whose journal mint is vault `mint_a` (`DNNk…`), amount `100000`. On-chain `vkey_hash` is now the **current guest** (`0x00435696…`, same as `constants::VKEY_HASH`). The Oct 22 proof (old ELF) will no longer verify. A new Devnet settle still needs a mint-matched wrap (`DNNk…`), not the toy `0x03…03` journal.

```bash
cd sdk
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json
npm run devnet:deposit
```

Network prove whose journal `asset_id_mint` is vault `mint_a` or `mint_b`, amount ≤ deposited note, Merkle root registered. Then SDK settle as wallet fee payer. Do not commit note secrets. Command: [`proving.md`](./proving.md).

| Field | Value |
| --- | --- |
| Deposit A tx | `3iaybGa2mLo5GehJ5c3uqUW11MQp4SijNPJcTuyGa97FCdWD1X6Q1Tt4Jm7CEWWWCL42T4jRABZydvA7cSNn44VM` |
| Deposit B tx | `3zGiSJ6ykHikFTsT4bYZVA9c5xvdd4k64eMTBXjcRFjKW8kckz4nvfnfdRzTv8L4oDmywKcR8boFUz7SoxgdDJVD` |
| Deposit leaf A | `GfXdsoSDcqxvh44PdiaSm5VikMvy65ypFAn5DB7s5xW2` |
| Deposit leaf B | `DrJ4PCJqk6reSTU5jJ2fQRuePKtM2mHGmDZAPPTG8Mev` |
| Leaf A (hex) | `2376354f46c91e6ddf5851b57e3bc6444a62035fc046afa599dd7e8e3d5d7014` |
| Leaf B (hex) | `15c538a26305c6f989f281b98625f05b167eb906fd2d02b13fa2d4c622593784` |
| `reserve_a` / `reserve_b` after settle | `1000000` / `900000` (spent mint A; vault-only decrements payout mint B) |
| On-chain `vkey_hash` (current guest) | `0x00435696d666d305172306e3895a44017e188663de9b3876970ae270345fc240` |
| `set_vkey_hash` tx (current) | `ZvnaL9fygEo5XM4QZdGcRDEcKtx1Nnb1MUNuFZkNyRifHnRxz6cHdg1ejTPeeMtWiLLJo3BzAKM7br9wdDfmVwU` |
| Previous `vkey_hash` (Oct 22 wrap) | `0x00d6c588a6e51b5715930c0d9495cb6c0c87e8e32fd2775b0a371df81aca6991` |
| Previous `set_vkey_hash` tx | `CVVahBk8B7Z4k7GcuLUBnqLpa6Mp7zJjtUb6FrK5vLjY9xjMNCMs4Gm2dzp6L3MNzgVFWg2PpERUC4a6wds27LN` |
| Live `CleanFundsRoot` | `5ATcjDGsM8WPHbjQSFACPWAaraNZ5NNLGHkBTfW6cNwD` |
| Register live root tx | `3ho3tpmfzRydf6UiiaDRGxD8LW64Hche2vz53YmdBX9T7VSu7hWMTPJrhB7dmTnui9gwNPeEd85is7EK5CZPSa2e` |
| Merkle root (hex) | `08d4c239e1f6b3afcdbf68dd8e6b36e853fc6a418bb85b05dc8d14b23acad818` |
| Groth16 wrap | gitignored `sp1-artifacts/devnet-settle/` ([Succinct 0x3f8c…e344](https://explorer.succinct.xyz/request/0x3f8c073fc36fa195ad057d04164d4b60253467d334af3f321eaeb8ec4f52e344)) |
| Settle signature | `3V7YF2ULaHULbnSJinSTahDw5gJSMGkvB2GuQyLevnzh9A15d4yoEcXNbCMfvbkzAcdDBHStYpzkECMrbYGz5NyF` |
| `computeUnitsConsumed` (N=1) | **281,362** (`getTransaction` / `meta.computeUnitsConsumed`; slot `513744382`; fee `5000` lamports) |
| Explorer (tx) | https://explorer.solana.com/tx/3V7YF2ULaHULbnSJinSTahDw5gJSMGkvB2GuQyLevnzh9A15d4yoEcXNbCMfvbkzAcdDBHStYpzkECMrbYGz5NyF?cluster=devnet |

Re-read this (or a later) settle the same way. Do not quote LiteSVM **285,947** as cluster CU.

```bash
solana confirm -v 3V7YF2ULaHULbnSJinSTahDw5gJSMGkvB2GuQyLevnzh9A15d4yoEcXNbCMfvbkzAcdDBHStYpzkECMrbYGz5NyF \
  --url https://api.devnet.solana.com
```

Or `getTransaction` and print `meta.computeUnitsConsumed`. Optional: after `sendAndConfirm`, log that field so later txs fill a table. **N=1 is enough** if you label it N=1.

## Agent settles

Same CLI/UI as localnet. Vault mint A is `DNNk…` (6 decimals). Do **not** prove with toy `0x03…03`.

```bash
# terminal 1 — relayer (402 merchant)
cargo run -p zk-spot-shield-relayer

# terminal 2 — agent on Devnet
cd sdk
AGENT_CLUSTER=devnet \
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
npm run agent

# terminal 3
cd ui && npm run dev
```

Agent **3** opens the UI with `?cluster=devnet`. Deposit / Buy use the agent. Type whole token amounts (`33` = 33 A). State file: gitignored `sdk/.agent-state-devnet.json`.

### Top up vault B (fix `VaultReserveUnderflow` / `0x177b`)

Settle pays **B 1:1** against an A spend. If `reserve_b` is short, mint+deposit B with the **admin / mint-authority** wallet (`~/.config/solana/id.json`). Aim for **≥ 10×** the A note you plan to spend (6 decimals → multiply human tokens by `1_000_000`).

```bash
cd sdk
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json

# Example: deposited / spending 50 A → put 500 B in the vault
npm run devnet:fund-b -- 500

# Or agent Admin (same wallet must be mint authority):
#   npm run agent → 5 → 1 → enter human B amount
# UI: open Admin (collapsed) → Deposit B to vault
```

Tiny top-up (0.1 B) via the E2E case: `DEVNET_ONCHAIN=1 DEVNET_FORCE=1 DEVNET_CASES=deposit_b npm run test:devnet`.

Wallet `9sLGgV…`. Eleven `SettleShieldedSpot` txs (Status Ok). Distinct from the SDK settle `3V7YF2UL…` above. All 11 notes spent (`spent: true`); no unspent note covers a buy.

| # | Signature | CU |
| --- | --- | --- |
| 1 | [`3CpfyWL6…Ek87h`](https://explorer.solana.com/tx/3CpfyWL6BuE62zxrMu9VKFjf9hFEf7u6Y8iSXWFVsHnkr6Rqch7VhC92m8VSVSphwJNoVjkjpMQ6eTtqgmDEk87h?cluster=devnet) | 294,537 |
| 2 | [`25PKqmaH…oQWBy`](https://explorer.solana.com/tx/25PKqmaH1WFyYgMfccPbQkvhHTAFtfBRTx8isBJvJ4EFfv1emtLRPSZhNUGJjFVvb9btMBhRmt5panxfMfVoQWBy?cluster=devnet) | 297,541 |
| 3 | [`petW4Kmj…7XZKb`](https://explorer.solana.com/tx/petW4Kmj9Nin87Espc3CYazY5r9eQzc7nZbfWsaQu1xK4ZKiCvEHDB7orbKUSUDJ8wPyi6QEtrJzfDueGq7XZKb?cluster=devnet) | 293,046 |
| 4 | [`nQQ9fgu8…NRpBu`](https://explorer.solana.com/tx/nQQ9fgu8EkK5QxADZiRYsaUobvhMdcBr9wuKj4bb6PzZR5qhQpvTeLqfaatHjSfnbNvJGPpELYg2JoLggJNRpBu?cluster=devnet) | 296,042 |
| 5 | [`2vFkhR9S…mfpyjs`](https://explorer.solana.com/tx/2vFkhR9SZQr7GUtXrgmfKwKsxDYGMoRndvMMUgSXhzpUcbQJbANaKB1bTK6AJdMiA3NF26AEPAhwPyexvxmfpyjs?cluster=devnet) | 294,580 |
| 6 | [`26eDe1zK…ALbYi7`](https://explorer.solana.com/tx/26eDe1zKZ4FqcvutkUPoA4o4eXS8TVnwu1Lm1gxJY5WDK1Seqx3SMBZMJuQcnvpNtNZsdMmFPhm65RTq6GALbYi7?cluster=devnet) | 299,035 |
| 7 | [`FcReL2PK…WBfG3`](https://explorer.solana.com/tx/FcReL2PKAoukUxFe7QaGtUtyANHVdcjT9zEwsiVpCTMuLvgnP3dnnQphNbosGos5oLNheQnqRkbE859oPgWBfG3?cluster=devnet) | 289,831 |
| 8 | [`2EKYyCBY…VLLtYn`](https://explorer.solana.com/tx/2EKYyCBYATbZ78MsbnQdGKKHJFYh5UBsnEChgtJ9RH5Ev4SCdoiREuAG2sPoWHiNfqzscHYXr2xzUnZ6K5VLLtYn?cluster=devnet) | 288,309 |
| 9 | [`4TdRCF6h…26SW1M`](https://explorer.solana.com/tx/4TdRCF6h516goiwuRZaXab3C2mgZMGxxEdvFW68nBAzJgu1tkhQU4vbuibx2Sb5HWhgk8kHPzDvkARXUn326SW1M?cluster=devnet) | 291,302 |
| 10 | [`52M1V6f5…ypLyea`](https://explorer.solana.com/tx/52M1V6f529xnFrzihPjVtGBxnifoGXyejsGkrqRBkSGZpFw7XqtE1SkCCqg1ibmYu7o3zuztsrhp35pLKUypLyea?cluster=devnet) | 288,324 |
| 11 | [`3op3aRxp…i1N8gC`](https://explorer.solana.com/tx/3op3aRxpaH8dYhykV35egXULhCCijiviYLVTUZoiikVqHqDtvkJkyZDqkvA9go5vKGQZTVC1Kx5Mt6Ye5Si1N8gC?cluster=devnet) | 288,314 |

```bash
solana confirm -v 3CpfyWL6BuE62zxrMu9VKFjf9hFEf7u6Y8iSXWFVsHnkr6Rqch7VhC92m8VSVSphwJNoVjkjpMQ6eTtqgmDEk87h \
  --url https://api.devnet.solana.com
```

## Day 67 — Manual Devnet UI checklist

For someone who is **not** the session agent. Goal: open the same UI, see live Devnet accounts, optionally land one tx, and verify on Solana Explorer. Screenshots optional.

Pinned program id: `74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G`. Amounts are **human tokens** (type `11` for 11 A; Devnet mints use 6 decimals under the hood).

### 0. Prerequisites

| Need | Why |
| --- | --- |
| Devnet SOL in a wallet (Phantom or `~/.config/solana/id.json`) | Fees for deposit / public swap / settle |
| Two terminals for a full click-through | Relayer (402) + agent; UI alone can **read** chain without them |
| Optional: mint authority / admin key | Only if you must top up vault B for a large public swap |

### 1. Start processes

```bash
# terminal 1 — 402 merchant (optional for read-only; required for deposit / buy / public swap)
cargo run -p zk-spot-shield-relayer

# terminal 2 — agent (holds notes; talks to Devnet)
cd sdk
AGENT_CLUSTER=devnet \
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
npm run agent

# terminal 3 — UI
cd ui && npm install && npm run dev
```

Agent menu **3** (Balance) opens the UI with `?cluster=devnet`, or open:

`http://localhost:5173/?cluster=devnet#on-chain`

Confirm the header toggle shows **Devnet** pressed (not Localnet).

### 2. See on-chain vault / config in the UI

1. Expand **On-chain** (`#on-chain`) if collapsed.
2. Click the refresh control (↻) or wait for the Devnet poll (~15s).
3. Expect non-empty **Vault A / B** (human amounts), **Paused** = `no`, and program id matching the pin above.
4. Open the Explorer links under the stats (or paste from this table):

| Account | Explorer |
| --- | --- |
| Program | https://explorer.solana.com/address/74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G?cluster=devnet |
| `GlobalConfig` | https://explorer.solana.com/address/BQbrv7CVnh571XjvGv4SMfo5beUcrgF3wtsHrzWAU3CY?cluster=devnet |
| `VaultState` | https://explorer.solana.com/address/A3h8dhmMipthngLSS5AmhsxYvt9sbgLmuUf1wfpJDqmd?cluster=devnet |
| Vault ATA A | https://explorer.solana.com/address/EVmRfTNFecvTBtW8Zcv5BrQJwZTA5HbSJMVBUJTEdTGQ?cluster=devnet |
| Vault ATA B | https://explorer.solana.com/address/8wBhwWC9c52D2qgSzYxNsCEN2YMJgsM916c8kBFrMTso?cluster=devnet |

Read-only stop is enough for “I can see the product on Devnet.” Continue for a live tx.

### 3. Optional: connect a wallet

In the UI, **Connect wallet** (Phantom set to Devnet) **or** load the local keypair JSON used as fee payer. Relayer URL may stay empty — self-submit is the default.

### 4. Optional: one live action

Pick **one** path (agent + relayer must be up). After success, note the status line signature and open **Explorer tx** in the UI (or the links below).

| Path | UI control | What you should see |
| --- | --- | --- |
| **A — Public (fast)** | **Swap Non-Shielded**, amount e.g. `1` | ~1s; vault B drops / A rises 1:1; 402 unlock=wallet |
| **B — Deposit note** | **Deposit Token A**, amount e.g. `1` | New note card; no Groth16 |
| **C — Shielded buy** | **Buy Token B**, amount ≤ an open note | Orange **Locked…** card → grey spent; ~40s Groth16 in the agent terminal |

Example live txs already on Devnet (open any if you skip submitting today):

| Kind | Explorer |
| --- | --- |
| Public swap 11 A→B | https://explorer.solana.com/tx/27LEe6SBTrkt2WX7RiRXaHwUWBLbnESSdNgLjsTjNbzYrsN3eHEmimaBThwxiLNRAzV7bT4UhEV3A9snSr3sk9qt?cluster=devnet |
| Shielded settle 11 A→B | https://explorer.solana.com/tx/N2DjaypMXxtaKciQGPmfjovXHGT4zoqTM3ZSueQ6XLyi8WZvVicszDCkZTU57oSAk1hxEN11gCnW1kecLRPESkY?cluster=devnet |
| Earlier SDK settle | https://explorer.solana.com/tx/3V7YF2ULaHULbnSJinSTahDw5gJSMGkvB2GuQyLevnzh9A15d4yoEcXNbCMfvbkzAcdDBHStYpzkECMrbYGz5NyF?cluster=devnet |

Same-quote wall/CU: [`budgets.md`](./budgets.md) (P0).

### 5. Exit check (Day 67)

- [x] UI opens on Devnet (`?cluster=devnet`)
- [x] On-chain panel shows vault / pause / program id; Explorer URLs for vault + config recorded above
- [x] At least one Explorer tx URL recorded (public and/or shielded samples above; or your own submit)
- [x] A stranger can follow §§1–2 without localnet

## Day 68 — Live-cluster E2E tests

CI stays on localnet (`DEVNET_ONCHAIN` unset → suite skipped). Helpers: `sdk/test/devnet/helpers.ts` (faucet SOL, mint A/B, deposit, done-markers). Suite: `sdk/test/devnet/onchain.devnet.test.ts`.

Passed cases are recorded in gitignored `sdk/.devnet-e2e-done.json` and **skipped on later runs** unless you force or name them.

### Case catalog

| Id | Kind | What |
| --- | --- | --- |
| `reads` | + | Fetch `GlobalConfig` + `VaultState` + vault ATAs; assert pins / vkey / reserves == ATA |
| `pause` | + | Admin pause → unpause |
| `deposit_a` | + | Faucet if needed → mint A → `deposit_shielded` |
| `deposit_b` | + | Mint B → deposit (vault liquidity) |
| `public_swap` | + | Mint A → `public_spot_swap` A→B |
| `neg_zero_swap` | − | `public_spot_swap(0)` → `ZeroSwapAmount` |
| `neg_underflow` | − | Quote > `reserve_b` → `VaultReserveUnderflow` |
| `neg_paused_swap` | − | Pause → swap fails `ProtocolPaused` → always unpause |
| *(manual)* | + | Shielded settle / buy — agent `1` / UI (needs Groth16 + `$PROVE`); not in Vitest |

### Commands

```bash
cd sdk
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=$HOME/.config/solana/id.json

# Run only cases not yet marked done
npm run test:devnet

# Re-run everything (ignore done file)
npm run test:devnet:force

# Clear done markers, then run pending (= all)
npm run test:devnet:reset

# Subset (still skips if already done unless FORCE)
DEVNET_ONCHAIN=1 DEVNET_CASES=reads,public_swap npm run test:devnet
DEVNET_ONCHAIN=1 DEVNET_FORCE=1 DEVNET_CASES=neg_paused_swap npm run test:devnet

# Show markers
cat .devnet-e2e-done.json
```

Wallet must be `GlobalConfig.authority` and mint authority for toy mints `DNNk…` / `Hfgiz…`. Faucet tops up SOL when balance is low (rate limits possible — fund manually if airdrop fails).

### Earlier Day 68 sample (pause write)

| Field | Value |
| --- | --- |
| Pause tx | [`3ccRn2Q1…jvcahqG`](https://explorer.solana.com/tx/3ccRn2Q1iEiaByaPrLsisN22nz1GqkphvbFJjn6g5KyRp9coXMvtGdo15b47fBDwZKmbiPZQTiJxBW7PQjvcahqG?cluster=devnet) |
| Unpause tx | [`3q5D78vA…uGJdHy`](https://explorer.solana.com/tx/3q5D78vACE85f5qmmLmNnWiXyWRayNMtJxX6iNUucZPxq8nuSMjHVFXsGVVcqVJ7722dMe8Kmr6DiGvm4TuGJdHy?cluster=devnet) |
