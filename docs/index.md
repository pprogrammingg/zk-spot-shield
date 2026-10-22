---
title: Private spot settlement on Solana
description: Private spot swaps on Solana with SP1 zero-knowledge proofs — Merkle inclusion, solvency, and nullifiers settled on-chain.
---

# ZK Spot Shield

ZK Spot Shield is a Solana program for **private spot swaps**. A user proves they are allowed to trade, and that they have enough funds, without showing the chain who they are or how much they hold. The chain checks a short cryptographic proof and settles the trade.

## The problem

On a public blockchain, every transfer is visible: who sent it, how much, and to whom. That is a poor fit for trading. Competitors can watch positions, and anyone can link a wallet to a person. Hiding everything does not work either, because a venue still has to know that funds are clean and that nobody spends the same money twice.

## The approach

ZK Spot Shield splits the job in two:

- **Off-chain, the user's machine builds a zero-knowledge proof.** The proof says: "my funds are in the approved list, my balance covers this trade, and here is a one-time tag for this spend." It reveals none of the private details behind those statements.
- **On-chain, a Solana program checks the proof.** It confirms the proof is valid, the approved list it refers to is one the program recognizes, and the one-time tag has not been used before. Only then does it move tokens.

The chain learns the trade amount, the token, and the one-time tag. It never learns the user's secret, their full balance, or where their funds sit in the approved list.

## Key ideas in one line each

| Idea | What it gives you |
| --- | --- |
| **Merkle tree of approved funds** | A single 32-byte fingerprint (the *root*) stands for a large list of approved deposits |
| **Zero-knowledge proof (SP1 + Groth16)** | Proves "I am in the list and solvent" without revealing which entry or how much |
| **Nullifier** | A one-time tag per spend, so the same funds cannot be spent twice |
| **Zero-copy accounts** | On-chain state is read in place, keeping settlement cheap |

## What is built so far

| Area | Status |
| --- | --- |
| Circuit, Groth16 fixtures, forged-path reject | Done |
| Program: config, vault, roots, nullifiers, deposit | Done |
| Vault-only 1:1 settle (proof → vault pays the other mint) | Done |
| Relayer fee payer + HTTP 402 stub | Done |
| TypeScript SDK + basic swap UI | Done (UI Exit needs a localnet click-through) |
| Hidden recipient | Partial (SDK hooks; Days 77–79 omit depositor + settle unlink) |
| Change notes + encrypted note backup | After Devnet 63 settle + 64 tag (Days 73–76) |
| Hide / unlink depositor | After backup (Days 77–79) |
| Devnet launch | 58–63 done (program `74UW…`, live settle `3V7YF2UL…`); **next: Day 64** tag |
| Extra RPC / IPFS | Later (Days 69–72) |

Start with [Architecture](architecture.md) for the numbered flow, [Accounts deep dive](accounts-deep-dive.md) for PDA/ATA vocabulary, or [Tests](tests.md) for fixtures.
