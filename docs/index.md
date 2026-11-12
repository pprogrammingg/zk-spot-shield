---
title: Private spot settlement on Solana
description: Private spot swaps on Solana with SP1 zero-knowledge proofs — Merkle inclusion, solvency, and nullifiers settled on-chain.
---

# ZK Spot Shield

A Solana program for **private spot swaps**. You prove you may trade and that you can afford it, without showing the chain who you are or how much you hold. The program checks a short proof and settles.

## Why

Every public transfer shows who sent it, how much, and to whom. That is a poor fit for trading. Hiding everything does not work either: a venue still has to know funds are clean and that nobody spends the same money twice.

## How

- **Off-chain**, your machine builds a zero-knowledge proof: the funds are in the approved list, the balance covers the trade, and a one-time tag marks this spend. Private details stay private.
- **On-chain**, the program checks the proof, that the approved list is one it recognizes, and that the one-time tag is unused. Only then does it move tokens.

The chain learns the trade amount, the token, and the one-time tag. It never learns the secret, the full balance, or where the funds sit in the list.

| Idea | What it gives you |
| --- | --- |
| **Merkle tree of approved funds** | One 32-byte root stands for a large list of deposits |
| **Zero-knowledge proof (SP1 + Groth16)** | Proves membership and solvency without revealing which entry |
| **Nullifier** | One-time spend tag, so the same funds cannot be spent twice |
| **Zero-copy accounts** | Hot state is read in place, keeping settlement cheap |

## Status

| Area | Status |
| --- | --- |
| Circuit, Groth16 fixtures, forged-path reject | Done |
| Program: config, vault, roots, nullifiers, deposit | Done |
| Vault-only 1:1 settle (proof → vault pays the other mint) | Done |
| Relayer fee payer + HTTP 402 stub | Done. Default quote is fixture 100 / toy mint. `GET /402/resource?amount=&mint=` sets the price (vault mint A on Devnet). Relayer terminal prints quote vs journal (amount/mint MATCH or MISS) and the nullifier receipt — no wallet |
| TypeScript SDK + swap UI | Localnet + Devnet toggle (`?cluster=`). Agent fills ATAs + notes. Human amounts (Devnet 6 decimals). Day 67 Devnet click-through: `notes/devnet.md` |
| Hidden recipient | Payout ATA may differ from the depositor; deposit tx still on the ledger |
| Change notes | 144-byte pack (spend 100 / keep 900). Second spend needs an admin root |
| Encrypted note backup | Done (wipe → restore → localnet settle) |
| Hide / unlink depositor | Done. Leaf PDA has no depositor pubkey. Settle can pay a different wallet. Deposit tx still on the ledger |
| Devnet | Live (`74UW…`). UI checklist + E2E Vitest (faucet/mint helpers, done-markers) in `notes/devnet.md` |
| Extra RPC / IPFS | Parked |
| Agent 402 → prove → verify → settle | Localnet + Devnet (`cd sdk && npm run agent`). Non-shielded path live. Same-quote 11 A→B in `notes/budgets.md`: public ~1 s / 18k CU vs shielded ~48 s / 295k CU |

Start with [Architecture](architecture.md) for the flow, [Solana Program](solana-program.md) for accounts and instructions, or [Tests](tests.md) for fixtures.
