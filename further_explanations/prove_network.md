# Succinct prover network — MetaMask and one-shot fixture

Day 12. Official: [Prover Network Quickstart (MetaMask)](https://docs.succinct.xyz/docs/sp1/prover-network/quickstart). Session checklist: [`roadmap_related.md`](./roadmap_related.md). Never commit `.env`, keys, or live `sp1-artifacts/`.

Succinct’s site **does not create a wallet**. You use MetaMask, send **`$PROVE` on Ethereum**, **Deposit** into the network, then the host reads **root `.env`**.

---

## After MetaMask: where `$PROVE` lives

- Create a **new** MetaMask account. Do not reuse a funded main wallet. That account’s address is the **requester**.
- **Yes — `$PROVE` must sit on that Ethereum address first** (buy/bridge on **Ethereum mainnet**, not Solana). Leaving zero PROVE on that address means you cannot deposit.
- **Do not leave it only on L1 for proving.** Connect the **same** account on [explorer Account](https://explorer.succinct.xyz), click **Deposit**. After indexing (~5 min), spendable balance is the **network** PROVE, not leftover L1.
- If PROVE is on a different wallet: Deposit from that wallet, then **Transfer** to the requester address on the explorer.
- Proving does **not** pull from MetaMask automatically. The CLI signs with a key in **`.env`**.

---

## Env file and keys (repo root)

- File: **`.env`** at the **repo root** (already gitignored). Copy [`.env.example`](../.env.example). Do not put `.env` under `zk-circuit/` unless you also load it there — the host looks at **root**.
- Required for a **live** wrap: `NETWORK_PRIVATE_KEY` (MetaMask **exported** private key, `0x…`), `SP1_PROVER=network`, `SP1_GROTH16=1`, `SP1_USE_NETWORK=1`.
- Default / later tests: leave `SP1_USE_NETWORK` unset or `0`. Host **execute** only. After **you** wrap once, public bytes land in `zk-circuit/fixtures/happy/` (this fixture only). No key needed to re-read them.
- Never commit `NETWORK_PRIVATE_KEY`. Export from MetaMask only into `.env` or a shell `export`.

---

## Steps (≤50 words each)

- **Wallet.** Install MetaMask. Create a new account. That ETH address is the requester. [Quickstart → Metamask](https://docs.succinct.xyz/docs/sp1/prover-network/quickstart).
- **Get `$PROVE`.** Buy or bridge PROVE onto **that same address on Ethereum mainnet** (see Succinct token overview from the quickstart). Tokens live on L1 until you deposit.
- **Deposit.** Open explorer Account, connect this account, **Deposit**. Wait until **network** PROVE shows. Proving spends this, not undeposited L1.
- **Export key.** MetaMask: export this account’s private key. Put it in root `.env` as `NETWORK_PRIVATE_KEY`. The explorer cannot run `cargo run` for you.
- **You prove once.** Copy the requester MetaMask private key into root `.env` as `NETWORK_PRIVATE_KEY`. Then `SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info cargo run -p zk-circuit-host --release`. That is the wrap — not `cargo test`.
- **Save once.** Success writes gitignored `sp1-artifacts/` and public `groth16.bin` / `journal.bin` / `vkey.bytes32.txt` into `zk-circuit/fixtures/happy/`. Change the guest, prove again.
- **Later tests.** Default CI still only `--lib` on program / io / client (`zk-circuit-io` reads the saved happy files). Future packing/settle tests can too. Do not fake Groth16. Wrap log: [`notes/proving.md`](../notes/proving.md).

---

## What “register” means

| What | How |
| --- | --- |
| You | MetaMask address + **deposited** network PROVE + `NETWORK_PRIVATE_KEY` in `.env`. |
| Guest ELF | First `prove` for that ELF; logs `Registered program 0x…`. No dashboard upload. |
