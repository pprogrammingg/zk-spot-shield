# Swap UI

React + TypeScript + Vite. Your wallet pays the Solana fee. A relayer is optional.

```bash
cd ui
npm install
npm test
npm run dev
```

On **localnet**, do **not** run `npm run test:e2e` first — that spends the happy nullifier. Use the demo validator (root unused):

```bash
# terminal 1 — leave running
cd sdk && npm run demo:localnet

# terminal 2
cd ui && npm run dev
```

**Load localnet demo.** Teal card = private note of 1000 mint A. Gold card = 900 **change note** after spend 100 (not extra tokens in the mint-A ATA). Fee payer: `~/.config/solana/id.json` (Finder hides `.config` — ⌘⇧G). Relayer URL empty. Expect user B `0 → 100`, vault B `1000 → 900`. One Groth16 + one 144-byte journal.

Optional 402: `cargo run -p zk-spot-shield-relayer`, Relayer URL `http://127.0.0.1:8787`. Default quote is 100 of mint `0x03…03`. Pass `?amount=` and `?mint=0x…` (vault mint A on Devnet) to set the price.

This screen is a human click-through of one fixture settle. **Localnet | Devnet** in the header picks RPC (`?cluster=`). Localnet auto-loads the demo pack (teal 1000 fixture). Both clusters **poll** On-chain data: vault + user ATAs (Devnet every 15s, localnet 8s) and CLI notes from the running agent `GET /status` (`127.0.0.1:8790`, every 4s, no Solana RPC). Nullifier sync is the agent (boot, before buy, every 60s) — not the UI poll. Vault, user, and note-card balances are on-chain whole units (no decimal split). Agent **3** Balance opens the matching cluster. **Deposit Token A** / **Buy Token B** POST integers to that agent — same paths as CLI `2` / `1`. While Groth16 runs, the chosen note card turns orange with **Locked for Spending ....**; after settle it greys out. **Load localnet demo** is localnet-only. Devnet agent: `AGENT_CLUSTER=devnet`.

## Static build

The program on Solana still works if the website is taken down. The UI should too.

```bash
npm run build
npx --yes serve dist
# or open dist/index.html as a local file (the injected wallet still talks to RPC)
```

The build uses relative URLs, so `dist/` works from a local folder or IPFS. In the UI, list RPC URLs separated by commas or newlines. The first endpoint that answers JSON-RPC `getHealth` is used (`sdk/src/rpcFallback.ts`).

```bash
shasum -a 256 dist/index.html dist/assets/*
# optional: ipfs add -r dist
```

No IPFS CID is published yet. Record a content hash after `npm run build`:

```bash
cd ui && npm run build
shasum -a 256 dist/index.html dist/assets/*
# optional: ipfs add -r dist
```

Recorded after a local `npm run build` (rebuild changes hashed asset names):

| File | sha256 |
| --- | --- |
| `dist/index.html` | `3ff6156b284ad8e0a1c90d86ba2f362ca441f4244fc4d8fa741ea8bf9a624783` |
| `dist/assets/index-B5MT5q31.js` | `4fd0aa945b4f0045ed013f860359770fe11a0f30eee8779826800671d7e13d56` |
| `dist/assets/index-fhcqvVit.css` | `5de8de572b491c54a21815c699c8a86b0fa2536f2e746a99e7e8f491165e7e8f` |

## Seizure runbook

Seizing the marketing site is **not** seizing settle. The program id and vault PDAs live on Solana. Anyone who has this `ui/dist` bundle (or can rebuild it), an RPC, and a wallet can self-submit.

1. Rebuild: `cd ui && npm install && npm run build`. Confirm the `shasum` of `dist/index.html` against a hash you saved earlier.
2. Host the same files somewhere else: local `npx serve dist`, GitHub Pages, or `ipfs add -r dist` and an IPFS gateway.
3. In the UI, set RPC URLs (comma or newline separated). Self-submit is the default fee-payer path. Do **not** depend on `relayer/` being up.
4. Verify the program id in the UI matches `programs/zk_spot_shield` / `Anchor.toml` before signing.
5. If the primary URL is gone, keep: this repo (or a pin of `dist/`), the program id, at least two RPC endpoints, and your encrypted note backup.

A CID is not published from this repo yet. Pin one when you run `ipfs add -r dist` and write it here.
