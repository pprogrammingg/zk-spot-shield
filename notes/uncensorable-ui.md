# Static swap UI

The program on Solana still works if the website is taken down. The UI should too.

## Self-submit

Default: **your wallet (or a localnet keypair) pays the fee**. Relayer URL is optional.

```bash
cd ui
npm install
npm run build
npx --yes serve dist
# or open dist/index.html as a local file (the injected wallet still talks to RPC)
```

## Multiple RPC endpoints

In the UI, list RPC URLs separated by commas or newlines. The first endpoint that answers JSON-RPC `getHealth` is used (`sdk/src/rpcFallback.ts`).

## Publish a content-addressed build

```bash
cd ui && npm run build
shasum -a 256 dist/index.html dist/assets/*
# optional: ipfs add -r dist
```

| Field | Value |
| --- | --- |
| Bundle hash (`dist/index.html`) | after `npm run build` |
| IPFS content id | not published yet |
| Second host | e.g. GitHub Pages |

If someone seizes the marketing site, settle still works for anyone who has this bundle, the program id, and an RPC.
