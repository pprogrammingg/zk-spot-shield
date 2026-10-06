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
| Circuit: Merkle inclusion, solvency check, nullifier | Done |
| Remote Groth16 proof of the happy-path example | Done, stored as test fixtures |
| Negative test: a forged Merkle path is rejected | Done |
| Solana program: config and vault accounts, nullifier and root accounts | Done (account layer) |
| On-chain proof verification and settlement | Next |
| Client SDK and token transfers | Planned |

Start with [Architecture](architecture.md) for the full picture, or jump to [Tests](tests.md) to see real inputs and outputs.
