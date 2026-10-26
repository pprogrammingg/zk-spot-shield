# Swap UI

React + TypeScript + Vite. Your wallet pays the Solana fee. A relayer is optional.

```bash
cd ui
npm install
npm test
npm run dev
```

On **localnet**, load `~/.config/solana/id.json` in the keypair control. Do not use a mainnet secret. Paste the sample proof and journal (base64) from `zk-circuit/fixtures/happy/`. Token account addresses come from a localnet init (the SDK end-to-end bundle).

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
