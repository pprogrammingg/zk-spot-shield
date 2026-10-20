# Relayer economics

The relayer pays Succinct **prove credits** (`$PROVE`) and Solana **SOL** fees, then refills from user fees.

```text
User fee (basis points / flat)
        │
        ▼
  Relayer treasury
     ├── SOL  → fee payer for settle (nullifier rent + tx fees)
     └── $PROVE → network Groth16 wrap (when wired; fixtures until then)
```

## Roles

| Role | Pays | Notes |
| --- | --- | --- |
| **Fee payer** (`payer` on settle) | SOL + nullifier account rent | Relayer key. Not the note owner. |
| **Note owner** | Private witness / intent | Bound by journal nullifier and root, not by the fee-payer pubkey. |
| **Recipient token account** | — | Receives the payout mint. Owner need not be the fee payer. |

The journal still binds **amount**, **mint**, **nullifier**, and **merkle_root**. Changing the fee payer does not change those fields.

## Fee stub

`FeeConfig { bps, flat }` → `compute_relayer_fee(amount)`. Env: `RELAYER_FEE_BPS` (default `30` = 0.30%) and `RELAYER_FEE_FLAT`.

On-chain skim to a treasury token account is **not** wired yet — stub and unit tests only. Direction: skim from payout or deposit to refill SOL and prove credits.

## RPC

Send/confirm uses exponential backoff (`BackoffConfig`: base 200 ms, cap 8 s, 5 attempts). See `relayer/src/rpc_retry.rs`.

## Prove spend

These paths use **committed fixtures** or client-supplied proof bytes. Do not set `SP1_USE_NETWORK` or spend `$PROVE` for local checks. Network prove is optional ([`proving.md`](./proving.md)).
