# Automated Devnet tests

CI stays on localnet. This command talks to **live Devnet**.

From `sdk/`:

```bash
DEVNET_ONCHAIN=1 \
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
npx vitest run test/devnet/onchain.devnet.test.ts
```

Optional write (pause then unpause). The wallet must be `GlobalConfig.authority` and hold SOL:

```bash
DEVNET_ONCHAIN=1 DEVNET_WRITE=1 \
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=$HOME/.config/solana/id.json \
npx vitest run test/devnet/onchain.devnet.test.ts
```

Leave `DEVNET_ONCHAIN` unset so CI skips. If you set it before config and vault exist on Devnet, the test fails.

## Last run

| Field | Value |
| --- | --- |
| Date | |
| `globalConfig` | |
| `vault` | |
| Write signatures | |
