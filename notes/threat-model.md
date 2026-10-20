# Threat model

Engineer review of the settle path. Full attack table: [`docs/security.md`](../docs/security.md) (A01–A25).

## Circuit soundness

The guest must recompute `expected_root` from the Merkle path, require `requested_swap_amount ≤ balance`, and bind `nullifier = Poseidon(secret, leaf, mint)`. Failure produces no journal. On-chain Groth16 binds **proof + journal** to `GlobalConfig.vkey_hash` (A06–A08, A11).

## Replay

The program creates a program-derived account at seeds `["nullifier", nullifier]`. The first settle initializes it; a second settle hits `NullifierAlreadyUsed` (A09). Seeds do not include a wallet pubkey.

## Pause

Only the protocol admin can set `GlobalConfig.pause_flag` (`has_one = authority`). Settle refuses while it is set (A19).

## Protocol admin

`GlobalConfig.authority` may pause, unpause, register roots, and (via upgrade) change the program. It is not the vault account and not the fee payer. Stolen admin key: A05.

## Root registry

A `CleanFundsRoot` account at `["clean_funds_root", root]` is created only by the admin. Settle requires that account for the journal’s `merkle_root` (A03, A04).

## Unbacked deposit leaf (A18)

On-chain `deposit_shielded` moves SPL tokens into the vault and records the leaf in one instruction. An off-chain tree insert **without** that instruction must not be registered as a `CleanFundsRoot`. Admins should only register tips that match funded deposits.

## Relayer and HTTP 402

The relayer is an optional fee payer (A16). HTTP 402 unlocks on the **journal nullifier**, not a wallet. Changing the journal would break Groth16.

## Honest limits

The deposit transaction still shows the depositor’s token account. Settle lists the token accounts in the transaction. A remote prover sees the witness (A15). This is not a mixer.
