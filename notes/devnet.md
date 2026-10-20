# Devnet launch

Fill the tables as you go. Until then, localnet and LiteSVM stay the default.

Env template: [`scripts/devnet.env.example`](../scripts/devnet.env.example).

## Keys and SOL

```bash
solana config set --url https://api.devnet.solana.com
solana address
solana airdrop 2
solana balance
```

If the public faucet rate-limits, use [faucet.solana.com](https://faucet.solana.com/) or a second wallet.

| Field | Value |
| --- | --- |
| Cluster | `https://api.devnet.solana.com` |
| Deploy / admin pubkey | |
| Balance after airdrop | |

## Deploy

```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
anchor build --ignore-keys
./scripts/program-hash.sh
anchor deploy --provider.cluster devnet
solana program show <PROGRAM_ID> --url devnet
```

| Field | Value |
| --- | --- |
| Program id | |
| `.so` SHA-256 | |
| Deploy tx / slot | |

Keep `declare_id!` and the IDL in sync if the deployed id differs from localnet.

## Init, verifying key, root

Create `GlobalConfig` (writes frozen `VKEY_HASH`) and register the happy-path Merkle root from `zk-circuit/fixtures/happy`. Use `sdk/scripts/e2e-localnet.ts` pointed at Devnet, or Anchor RPC. Authority is the wallet above.

| Field | Value |
| --- | --- |
| `GlobalConfig` | |
| Explorer (config) | |
| First `CleanFundsRoot` | |

## Vault and token accounts

Initialize `VaultState`, create vault token accounts for mint A and B, and fund reserves for a 1:1 spot.

| Field | Value |
| --- | --- |
| Vault | |
| Vault token account A | |
| Vault token account B | |
| Explorer (vault) | |

## Relayer

```bash
RELAYER_BIND=127.0.0.1:8787 \
RELAYER_FEE_PAYER=$HOME/.config/solana/id.json \
cargo run -p zk-spot-shield-relayer --release
curl -s http://127.0.0.1:8787/health
```

The relayer is an optional fee payer. The UI submits from the user’s wallet by default. CORS is on so the browser can call `/submit-intent`.

## One live swap (SDK, not UI)

After accounts exist, reuse the SDK end-to-end path against Devnet. The happy-path nullifier is single-use per cluster — prove a fresh Groth16 if it is already spent.

```bash
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npm run test:e2e
```

| Field | Value |
| --- | --- |
| Settle signature | |
| Explorer (tx) | |

## Docs and tag

1. Copy program id and hash into this note and the README Devnet pointers.
2. Keep the `anchor build` log, `./scripts/program-hash.sh` output, `VKEY_HASH`, and fixture paths.
3. Tag a freeze as `v1.0.0-devnet` when you mean it.

Record Explorer URLs for config, vault, and (if it ran) the settle transaction.
