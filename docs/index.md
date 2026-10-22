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
| Relayer fee payer + HTTP 402 stub | Done |
| TypeScript SDK + swap UI | Done (UI still needs a localnet click-through) |
| Hidden recipient | Partial — deposit still records the depositor |
| Change notes + encrypted note backup | Next |
| Hide / unlink depositor | After backup |
| Devnet | Live (program `74UW…`, settle `3V7YF2UL…`, tag `v1.0.0-devnet`) |
| Extra RPC / IPFS | Later |

Start with [Architecture](architecture.md) for the flow, [Solana Program](solana-program.md) for accounts and instructions, or [Tests](tests.md) for fixtures.
