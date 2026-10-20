# Relayer economics (Day 47)

Architecture **phase E** steps **30–31**: the relayer pays Succinct **prove credits** (`$PROVE`) and Solana **SOL** fees, then refills from user fees.

```text
User fee (bps / flat)
        │
        ▼
  Relayer treasury
     ├── SOL  → fee payer for settle (nullifier rent + tx fees)
     └── $PROVE → network Groth16 wrap (when wired; fixtures until then)
```

## Roles

| Role | Pays | Notes |
| --- | --- | --- |
| **Fee payer** (`payer` on settle) | SOL + nullifier PDA rent | Relayer key. **≠ note owner** |
| **Note owner** | Private witness / intent | Bound by journal nullifier + root, not by fee-payer pubkey |
| **Recipient ATA** | — | Receives payout mint; owner need not be fee payer (Day 46+) |

Journal still binds **amount**, **mint**, **nullifier**, **merkle_root**. Changing the fee payer does not change those fields.

## Fee stub (Day 47)

`FeeConfig { bps, flat }` → `compute_relayer_fee(amount)`. Env: `RELAYER_FEE_BPS` (default `30` = 0.30%), `RELAYER_FEE_FLAT`.

On-chain skim / treasury ATA CPI is **not** wired yet — stub + unit tests only. Product direction: skim from payout or deposit to refill SOL + prove credits.

## RPC

Send/confirm uses exponential backoff (`BackoffConfig`: base 200ms, cap 8s, 5 attempts). See `relayer/src/rpc_retry.rs`.

## Prove spend

Day 46–47 paths use **committed fixtures** or client-supplied proof bytes. Do **not** set `SP1_USE_NETWORK` / spend `$PROVE` for local Exit checks. Network prove remains optional (`notes/proving.md`).
