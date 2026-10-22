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

No IPFS CID is published yet. If someone seizes the marketing site, settle still works for anyone who has this bundle, the program id, and an RPC.
