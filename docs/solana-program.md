# Solana Program and Anchor

The on-chain crate is `program/` (program name `zk_spot_shield`), written with the Anchor framework. Today it has the account layer in place: the config, the vault, and the accounts that record approved roots and spent nullifiers. Proof verification and settlement build on top of these.

| Piece | Kind | Seeds | Size (data) | Purpose |
| --- | --- | --- | --- | --- |
| `GlobalConfig` | `Account<T>` (Borsh) | `["global-config"]` | 65 bytes | Admin key, trusted circuit fingerprint, pause switch |
| `VaultState` | zero-copy | `["spot_vault"]` | 120 bytes | Token mints and reserves the settlement moves |
| `CleanFundsRoot` | zero-copy | `["clean_funds_root", root]` | 32 bytes | One account per approved Merkle root |
| `NullifierAccount` | zero-copy | `["nullifier", nullifier]` | 32 bytes | One account per spent nullifier |

Every account also carries Anchor's 8-byte type tag in front of the data, so allocations are `8 + size`.

**Anchor sets the program's shape.** Each instruction declares the accounts it needs in a `#[derive(Accounts)]` struct, and Anchor checks them before the handler runs: right owner, right seeds, signer present, writable when it must be. Two instructions exist today, `initialize_global_config` and `initialize_vault`. Settlement will be a third, `settle_shielded_spot`, using the same pattern.

**`GlobalConfig` pins which circuit the program trusts.** It stores `authority` (the admin key), `vkey_hash` (the fingerprint of the SP1 guest program), and `pause_flag`. A proof only counts if it was made for the circuit whose fingerprint is stored here, so swapping in a different circuit is impossible without the admin. The account is small and read rarely, so it uses the ordinary `Account<T>` wrapper, which copies the data into a Rust struct. The stored hash is still a zero placeholder until vkey extraction lands (Day 15).

**`VaultState` is zero-copy because settlement touches it on every trade.** Marked `#[account(zero_copy)]` and loaded through `AccountLoader`, the program reads and writes the account's bytes in place instead of copying them onto the heap and back. That saves compute units on the hot path. The trade-off is a strict memory layout: `#[repr(C)]`, fixed-size fields only, and explicit padding.

**Explicit padding keeps the layout honest.** `VaultState` is three 32-byte keys, two `u64` reserves, a 1-byte `bump`, and 7 bytes of `_padding`, which brings it to 120 bytes, a multiple of 8. Compile-time assertions (`const _: () = { assert!(...) }`) fail the build if anyone changes a field and breaks the size or alignment. A layout bug becomes a compiler error instead of corrupted on-chain data.

**PDAs give the program addresses only it can sign for.** A Program Derived Address is computed from seeds plus the program ID and has no private key. `GlobalConfig` and `VaultState` use fixed seeds, so there is exactly one of each and any client can find them without a lookup. The `bump` saved in `VaultState` lets the program sign token transfers as the vault later.

**The nullifier PDA turns double-spend protection into "does this account exist?"** The seeds are `["nullifier", nullifier]`, where the nullifier comes from the proof's public output. Settlement creates this account with Anchor's `init`. If it already exists, `init` fails and the whole transaction rolls back. The check costs one account lookup, needs no growing list, and cannot be raced, because creation is atomic.

**The nullifier seed deliberately leaves out the user's key.** If the seeds included the wallet, the account address would link a spend to a person, which defeats the point. The nullifier itself is a Poseidon hash of the user's secret, their leaf, and the mint. It is unique per note and token, yet reveals nothing about who produced it.

**`CleanFundsRoot` makes "is this root approved?" the same kind of lookup.** Each approved Merkle root gets its own 32-byte account at `["clean_funds_root", root]`. To accept a proof, settlement derives the address from the proof's root and checks the account exists. Older roots stay valid as long as their accounts exist, so a proof made just before new deposits change the tree still settles.

**Errors name the exact rule that failed.** `InvalidProof`, `MerkleRootNotFound`, and `NullifierAlreadyUsed` map one-to-one onto the three settlement checks, and `ZeroCopyDeserializationError` covers a malformed account. A failed transaction says which guarantee stopped it.

**The invariants have their own tests.** Native unit tests check account sizes, that PDA seed prefixes never collide, and that the same nullifier or root always maps to the same address while different ones never do. LiteSVM tests build the real program binary and run both initialize instructions in an in-memory Solana runtime.
