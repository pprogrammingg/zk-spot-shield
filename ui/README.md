# Swap UI

React + TypeScript + Vite. Your wallet pays the Solana fee. A relayer is optional.

```bash
cd ui
npm install
npm test
npm run dev
```

On **localnet**, load `~/.config/solana/id.json` in the keypair control. Do not use a mainnet secret. Paste the sample proof and journal (base64) from `zk-circuit/fixtures/happy/`. Token account addresses come from a localnet init (the SDK end-to-end bundle).

Static build (relayer can be off):

```bash
npm run build
npx --yes serve dist
```

The build uses relative URLs, so `dist/` works from a local folder or IPFS. Put more than one RPC URL in the list if you want fallback. Publish steps: [`notes/uncensorable-ui.md`](../notes/uncensorable-ui.md).
