# Manual Devnet walkthrough

For a person using the UI against live Devnet. Needs the program, config, and vault from [`devnet.md`](./devnet.md).

## Checklist

1. `cd ui && npm install && npm run dev`
2. Set the RPC list to `https://api.devnet.solana.com` (optional second URL on the next line).
3. Confirm the program id matches [`devnet.md`](./devnet.md).
4. Connect Phantom on Devnet, or skip the wallet and only refresh (read-only).
5. Click **Refresh on-chain**. Vault reserves, pause flag, and config pubkey should appear.
6. Open the Explorer links for vault and `GlobalConfig`.
7. If you have an unused Groth16 proof and journal: paste them, set token accounts, click **Submit myself**, confirm in the wallet.
8. Refresh reserves. One side should drop and the other rise (1:1).
9. Open the transaction Explorer URL.

The relayer can stay off. Self-submit is the default.

## Recorded

| Item | URL / value |
| --- | --- |
| UI origin | `http://127.0.0.1:5173` |
| Explorer `GlobalConfig` | |
| Explorer vault | |
| Settle signature (if run) | |
