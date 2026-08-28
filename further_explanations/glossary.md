# Quick Glossary

Terms from `roadmap_related.md`, the on-chain crates, and the Day 1 scaffold.

Each entry: **what it does**, **without it**, and **not the same as** when names collide.

Themes and items are alphabetical.

**Quick trio:** **owner** = which *program* may write the account’s data (runtime). **payer** = who funds *this* `init` (rent). **authority** = pubkey *stored in data* that later ixs treat as admin. Same wallet can be all three by choice; they are three jobs.

---

## Accounts and PDAs

### Account (Solana)

**What it does:** A 32-byte address pointing at `{ lamports, owner program, data bytes, executable flag }`. Wallets, program state, mints, and executables are all accounts.

**Without it:** There is nowhere to store SOL, token balances, or program state. The chain has no objects to pass into an instruction.

**Not the same as:** An Anchor `#[derive(Accounts)]` struct (that is an *instruction’s account list*, not one on-chain object). Not `Account<'info, T>` (that is a typed *view* of one account).

### `Account<'info, T>`

**What it does:** Anchor wrapper that copies account bytes into a Rust `T` on every instruction (standard `#[account]` path). Used for small state like `GlobalConfig`.

**Without it:** You either use raw `AccountInfo` and deserialize by hand, or `AccountLoader` for zero-copy. Wrong wrapper on a zero-copy type will fail to compile or deserialize.

**Not the same as:** `AccountLoader` (zero-copy map, no full copy). Not the Solana “account” primitive. Not the `Initialize` / `InitializeGlobalConfig` *Accounts* struct.

### Account meta

**What it does:** One slot in an instruction’s account list: pubkey + writable? + signer?. Clients must pass accounts in the order the Accounts struct expects.

**Without it:** The program cannot see payer, PDAs, or System Program. The runtime rejects missing/wrong flags (e.g. not writable when `init` needs a debit).

**Not the same as:** Account *data* (the bytes inside). Meta is only how this ix *references* an account.

### Bump

**What it does:** Extra byte (usually tried from 255 downward) so `hash(seeds + bump + program_id)` lands **off** the ed25519 curve. That address has no private key; only the program can sign as it via seeds+bump.

**Without it:** The derived address might be a normal keypair pubkey. Anyone with a matching secret could impersonate the “PDA,” or `find_program_address` would not give a program-only address.

**Not the same as:** Incrementing a toy `count` field (“bump the counter”). Not seeds (seeds *name* the PDA; bump *tweaks* it off-curve). The `bump` *field* on `VaultState` *stores* that byte for later CPIs.

### `CleanFundsRoot`

**What it does:** On-chain store of approved Merkle root(s). Settlement checks the proof’s public root against this registry (notes also describe a 32-slot ring buffer of historical roots so in-flight proofs stay valid after new deposits).

**Without it:** There is no whitelist of “this tree state is allowed.” Attackers could prove against a fake tree, or honest proofs would break the moment the live tree moved.

**Not the same as:** The Merkle *tree account* that holds leaves. Not the Merkle *path* (sibling hashes). `CleanFundsRoot` is the *allowed roots* list, not the tree itself.

### Discriminator (account)

**What it does:** First 8 bytes of account data; Anchor type tag so `GlobalConfig` cannot be parsed as `VaultState`.

**Without it:** Account bytes could be reinterpreted as the wrong struct. `init` space must include these 8 bytes (`8 + INIT_SPACE`).

**Not the same as:** **Instruction discriminator** (ix tag) — that lives in *instruction data* and picks the handler. Account discriminator lives in *account data* and picks the struct type.

### `GlobalConfig`

**What it does:** Singleton PDA holding `authority`, `vkey_hash`, and `pause_flag` — admin, trusted circuit fingerprint, and emergency freeze.

**Without it:** No single place to pin the SP1 verification key, pause settlement, or restrict who may rotate keys.

**Not the same as:** `VaultState` (reserves/mints). Not the executable program account. Config is admin/security state; vault is money state.

### `init` (account constraint)

**What it does:** Creates a new account this instruction: must not already exist; allocates `space` bytes; assigns **owner** to your program.

**Without it:** You cannot allocate the PDA in this ix. A later write would hit a missing or System-owned empty account.

**Not the same as:** The handler `handle_initialize_*` (that *fills fields* after creation). Not `Initialize*` Accounts structs (those *describe* the ix). `init` = allocate; handler = constructor body.

### `INIT_SPACE`

**What it does:** Const from `#[derive(InitSpace)]`: byte size of the struct **fields only** (e.g. `u64` + `Pubkey` = 40). Rent allocation is `8 + INIT_SPACE`.

**Without it:** You guess sizes by hand; too small fails at runtime, too large wastes rent.

**Not the same as:** `space = …` (that is the *full* allocation, including the 8-byte account discriminator). `INIT_SPACE` never includes those 8 bytes.

### `NullifierAccount`

**What it does:** PDA at seeds `[b"nullifier", nullifier_hash]`. Creating/using it marks that nullifier spent so the same withdrawal cannot replay. Seeds omit user pubkey so the address does not leak who spent.

**Without it:** The same proof/nullifier can be submitted twice (double-spend / replay).

**Not the same as:** The **nullifier hash** (a 32-byte value in the journal). The account is the on-chain *slot* whose *existence* means “already spent.” Not `CleanFundsRoot`.

### Owner (account owner)

**What it does:** Runtime field: **which program is allowed to change this account’s data**. After `init`, your program owns `GlobalConfig` / vault PDAs. System Program owns a normal wallet until it assigns owner on create.

**Without it:** No access control at the VM layer — any program could overwrite any data.

**Not the same as:** **`authority`** (app pubkey stored *inside* data — who may pause/admin). Not **`payer`** (who paid rent). Not SPL **token account owner** (who may spend tokens — a field in Token Program data). Three “owners” in the wild: program owner, stored authority, token-account owner.

### `payer = payer` (account constraint)

**What it does:** Debit the named `Signer` for the **rent-exempt deposit** when `init` creates the new account. Pays to *open the box this ix*, not “forever pays all txs.”

**Without it:** `init` has no funding source. Account creation fails. Later txs are still paid by the **transaction fee payer**.

**Not the same as:** **Fee payer** (tx fees every send). Not **authority** (stored ACL). Not **owner** (program that writes data). In the scaffold the same wallet is often payer *and* then copied into `authority`; that is a choice.

### PDA (Program Derived Address)

**What it does:** Deterministic address from seeds + program id + bump. No private key. Used for `GlobalConfig`, vault, nullifiers, roots.

**Without it:** State would live at a keypair you must keep secret, or at an address anyone could collide. Programs could not sign CPIs as “the vault.”

**Not the same as:** A wallet keypair account. Not “the program” (the executable is a different account). A PDA is an *address scheme*; what it *stores* is still just account data.

### Rent-exempt deposit

**What it does:** Minimum SOL left **on the new account** so its data size is allowed to persist. Taken from `payer` at `init`. Recoverable if the account is later closed.

**Without it:** Creation is rejected, or the account is not safely persistable at that size.

**Not the same as:** **Transaction fees** (paid to the network for including the tx). Rent deposit sits *on the account*; fees are spent/burned as tx cost. Both are lamports.

### Seeds

**What it does:** Byte strings that name a PDA (e.g. `b"global-config"`, `b"nullifier"` + hash). Same seeds + program id → same address.

**Without it:** PDAs would not be canonical. Callers could pass a random account and impersonate config/vault/nullifier if the program did not check seeds.

**Not the same as:** Bump (tweak byte). Not the 8-byte discriminator. Seeds are the *path*; bump is the *off-curve tweak*.

### Signer / `Signer<'info>`

**What it does:** This account **signed this transaction**. For `payer`, that authorizes debiting rent (and proves identity so the handler may set `authority = payer`).

**Without it:** Anyone could include someone else’s pubkey as payer and drain them, or claim to be admin without a signature.

**Not the same as:** **Authority** (persisted in state; next tx that person must *sign again*). Signing once does not leave a badge except what you *write* into data. Not **owner**.

### System Program

**What it does:** Built-in program that **creates accounts** and **transfers SOL**. `init` and SOL `transfer` CPI through it. You pass `system_program` so the runtime can invoke it.

**Without it:** `init` and native SOL transfers cannot run. The instruction has no kernel to allocate accounts.

**Not the same as:** *Your* program (`zk_spot_shield`). Not Token Program (SPL). System Program is the “malloc + native SOL” service.

### `VaultState`

**What it does:** Zero-copy PDA for spot vault: authority, two mints, two reserves, bump, padding.

**Without it:** Nowhere to keep AMM/spot reserves and mint wiring for settlement.

**Not the same as:** SPL token accounts (those hold token balances; vault PDA *authorizes* transfers out of vault ATAs). Not `GlobalConfig`.

---

## Anchor Surface

### `#[account]` (on a struct)

**What it does:** Marks `T` as Anchor account state: discriminator, serialize/deserialize, `Account<'info, T>`. Default path copies bytes onto the stack/heap.

**Without it:** Anchor will not treat the struct as account data. Clients/IDL will not know the layout.

**Not the same as:** `#[account(init, seeds, …)]` on an *Accounts field* (constraints). One is “this type is state”; the other is “this *ix account* must pass these checks.”

### `#[account(...)]` (on an Accounts field)

**What it does:** Constraint DSL: `mut`, `init`, `payer`, `space`, `seeds`, `bump`. Checked **before** the handler.

**Without it:** You must manually check signer, writable, PDA derivation, and creation. Easy to skip a check and get drained or spoofed.

**Not the same as:** `#[account]` on the *state struct*. Field constraints do not write `authority`; the handler does.

### `#[account(zero_copy)]`

**What it does:** Layout for mapping account bytes in place (use with `#[repr(C)]` and `AccountLoader`). Suited to larger/hot accounts.

**Without it:** Large state is copied every ix (`Account<'info, T>`), hitting stack/heap limits (~10 KB class) and extra CU.

**Not the same as:** Plain `#[account]` (copy deserialize). Pair with `AccountLoader`, not `Account<'info, T>`.

### `#[constant]`

**What it does:** Exports a const (seeds, `VKEY_HASH`) into the IDL for clients.

**Without it:** The value still works in Rust; clients must hardcode seeds/hashes and can drift.

**Not the same as:** `declare_id!` (program pubkey). Not `vkey_hash` *stored on-chain* in `GlobalConfig` (that can be rotated; `#[constant]` is compile-time).

### `#[derive(Accounts)]`

**What it does:** Turns a struct into the instruction’s **account list schema**. Generates validation and client types (`accounts::InitializeGlobalConfig`).

**Without it:** No typed `Context<T>`. You parse `&[AccountInfo]` by index yourself.

**Not the same as:** State structs (`GlobalConfig`, `Counter`). `InitializeGlobalConfig` is “who must be in *this call*,” not “the bytes we store forever.” Naming the Accounts struct `Initialize*` is Anchor convention matching `fn initialize_*`.

### `#[derive(InitSpace)]`

**What it does:** Generates `T::INIT_SPACE` from field sizes.

**Without it:** Manual `space = 8 + …` that bitrots when fields change.

**Not the same as:** `size_of::<T>()` for zero-copy `#[repr(C)]` types (those often skip `InitSpace` and assert `size_of` instead).

### `#[error_code]`

**What it does:** Maps enum variants to on-chain error codes + `#[msg]` strings (`InvalidProof`, `NullifierAlreadyUsed`, …).

**Without it:** Failures are generic panics or untyped custom errors; clients cannot match `NullifierAlreadyUsed`.

**Not the same as:** `msg!` logs (those are not error codes). Not instruction discriminators.

### `#[program]`

**What it does:** Builds the instruction table and entrypoint. Each `pub fn` gets an 8-byte **instruction discriminator**.

**Without it:** No generated dispatcher. The runtime has nothing named `initialize_vault` to call.

**Not the same as:** `declare_id!`. Not System Program. This macro is *your* crate’s public methods.

### `Context<T>`

**What it does:** After constraints pass: `ctx.accounts` is the validated bundle `T`, plus bumps and remaining accounts.

**Without it:** Handler would take raw account infos with no guaranteed checks.

**Not the same as:** `CpiContext` (wrapper for calling *another* program). `Context<T>` is *this* instruction’s validated accounts.

### `declare_id!`

**What it does:** Embeds the program’s public key. Must match `Anchor.toml` and the deploy keypair.

**Without it:** PDA derivation and clients target the wrong program. Deployed `.so` will not match declared id.

**Not the same as:** `authority` or `payer` pubkeys. This is *which executable* you are, not who signed.

### Instruction discriminator (ix tag)

**What it does:** First 8 bytes of **instruction data** (`sha256("global:<fn_name>")[0..8]`). Selects `initialize_global_config` vs `initialize_vault`.

**Without it:** One program cannot tell which handler to run. Clients sending empty data would be ambiguous.

**Not the same as:** **Account discriminator** (8 bytes at the start of *account* data). Ix tag selects *method*; account tag selects *struct type*.

### `space = 8 + T::INIT_SPACE`

**What it does:** Byte length allocated at `init`: 8-byte account discriminator + payload.

**Without it:** Account too small → serialize/init fails; too large → extra rent with no benefit.

**Not the same as:** `INIT_SPACE` alone. The `8` is required for standard `#[account]` types.

---

## Memory Alignment / Efficiency

### `AccountLoader<'info, T>`

**What it does:** Loads `#[account(zero_copy)]` types via `.load()` / `.load_mut()` without copying the whole struct.

**Without it:** You cannot safely use zero-copy accounts in the Accounts struct (`Account<'info, T>` copies).

**Not the same as:** `Account<'info, T>`. Loader = map; Account = copy. Mixing them with the wrong `#[account]` style fails.

### Avalanche effect (circuit inputs)

**What it does:** Flipping one bit of leaf, path, or order changes hashes so the off-chain proof will not generate (or will not verify).

**Without it:** Broken or tampered Merkle data could still look like a valid transition.

**Not the same as:** On-chain CU failure. Avalanche is a *hash* property of the circuit inputs, not the Solana runtime.

### `bytemuck::Pod` / `Zeroable`

**What it does:** Marks a type as plain bytes: no invalid bit patterns, safe to cast from an account buffer. Required mental model (and often derive bounds) for zero-copy.

**Without it:** Casting account bytes to a Rust struct can hit padding holes or invalid enums — undefined / rejected layouts.

**Not the same as:** `InitSpace` / Borsh copy accounts. Pod is for *in-place* casts; Borsh `#[account]` is a different serialization path.

### Compute units (CU)

**What it does:** CPU budget for one instruction. Groth16 verify is expensive (~hundreds of thousands of CU).

**Without it / over budget:** The transaction fails even if logic is correct. No “run longer” on Solana.

**Not the same as:** Transaction **fees** (lamports). CU is *time/compute*; fees are *money*. You can have enough SOL and still hit CU limit.

### Explicit padding (`_padding: [u8; 7]`)

**What it does:** Fills the struct so size is a multiple of 8 (or the alignment you chose). No silent compiler-inserted gaps.

**Without it:** `#[repr(C)]` may still insert implicit padding. Size/alignment asserts fail; different compilers or bytemuck casts disagree.

**Not the same as:** PDA **bump** byte (crypto). Padding is unused alignment bytes in the *struct layout*.

### `#[repr(C)]`

**What it does:** C field order and alignment. Required for stable zero-copy layouts across builds.

**Without it:** Rust may reorder fields. On-chain bytes would not match the struct; zero-copy reads garbage or trap.

**Not the same as:** `#[account(zero_copy)]` (Anchor’s account behavior). You typically need **both**: `repr(C)` for layout, `zero_copy` for how Anchor loads it.

### Size / alignment `assert!`

**What it does:** Compile-time check e.g. `size_of::<VaultState>() == 120` and `% 8 == 0`.

**Without it:** Layout bugs show up only on-chain or in LiteSVM after deploy.

**Not the same as:** `INIT_SPACE` (Borsh/copy path). Zero-copy uses `size_of`; copy accounts use `8 + INIT_SPACE`.

### Zero-copy

**What it does:** Treat account `data` as the struct in place. Good for `VaultState` and large buffers (up to ~10 MB class vs ~10 KB copy path).

**Without it:** Every settle copies the vault; CU and stack blow up as state grows.

**Not the same as:** “Zero-knowledge.” Zero-*copy* is a memory trick. Zero-*knowledge* is a proof system.

---

## Privacy and Zero-Knowledge

### Commitment `C = Hash(S || N)`

**What it does:** Deposit binds secret `S` and nullifier seed `N` without revealing them. Inserted as a Merkle leaf (notes: plus 10 SOL in that mixer-style sketch).

**Without it:** The tree would store identities or spend secrets in the clear, or deposits would not be later provable.

**Not the same as:** **Nullifier hash** (spend tag, often `Hash(S || N || leaf_index)`). Commitment goes *into the tree*; nullifier is shown *at withdraw* and burned via PDA.

### Groth16 / SP1 proof

**What it does:** Compact proof that the guest ran the circuit (inclusion, solvency, nullifier). Verified on-chain against `vkey_hash`.

**Without it:** The chain cannot check private Merkle/solvency claims; you would leak witnesses or skip verification.

**Not the same as:** **Journal** (the public outputs *inside* / beside the proof). Not **`vkey_hash`** (which circuit is trusted). Proof = evidence; journal = claimed public result; vkey = which circuit.

### Journal / public values

**What it does:** Public outputs committed by the guest (amount, mint, nullifier, merkle root, often recipient). Bound into the proof.

**Without it:** On-chain cannot know *what* was proved. A frontrunner could swap the recipient; the verifier would have nothing to check.

**Not the same as:** Private witnesses (secrets/path). Not `msg!` logs. Journal is *cryptographically bound*; logs are not.

### `leaf_index`

**What it does:** Public slot when depositing. In the circuit at withdraw it is a **private witness**: picks the Merkle path and is hashed into the nullifier so two deposits from the same `S, N` still differ.

**Without it:** Path is ambiguous; multiple deposits with the same master secrets collide on one nullifier PDA.

**Not the same as:** The leaf *value* (commitment). Index is *where* in the tree; commitment is *what* is stored there.

### Merkle path / root

**What it does:** Path proves a leaf is in the tree; root is the public fingerprint of that tree state. Historical roots live in `CleanFundsRoot`.

**Without it:** No inclusion proof. Anyone could claim a leaf that was never deposited.

**Not the same as:** Path = siblings (usually private in the circuit). Root = one 32-byte hash (public, whitelisted). Tree account vs `CleanFundsRoot` registry.

### Nullifier hash `Hash(S || N || leaf_index)`

**What it does:** Public spend tag. Shown on withdraw; never reveals `S`. Unique per deposit even if `S` and `N` repeat.

**Without it:** Same user/secrets → same nullifier → second deposit cannot be spent, or spends link together.

**Not the same as:** **Commitment** (deposit). Not **`NullifierAccount`** (the PDA that records the hash as used). Hash is the value; account is the on-chain lock.

### Private witness

**What it does:** Values the circuit sees that never appear in the on-chain ix (secrets, path, `leaf_index` at withdraw).

**Without it:** Withdrawals would publish the mixer secrets; anonymity set collapses.

**Not the same as:** Journal / public values. Witness stays in the guest; journal is what the chain is allowed to see.

### Prover (host) vs verifier (on-chain)

**What it does:** Host (`zk-circuit/host`) **generates** the Groth16 proof off-chain. On-chain `sp1-solana` CPI **checks** it against stored `vkey_hash`.

**Without it:** Either you trust the client with no check, or you try to prove on-chain (not feasible for this circuit).

**Not the same as:** Each other. Proving is expensive and private-input-heavy; verifying is cheaper and public. Guest = circuit; host = driver that runs the prover.

### `vkey_hash`

**What it does:** 32-byte fingerprint of the circuit verification key in `GlobalConfig`. Proofs from a different circuit fail.

**Without it:** A rogue circuit could produce “valid” proofs the verifier would accept.

**Not the same as:** The proof bytes. Not the journal. Not `declare_id!`. Rotating `vkey_hash` is a governance/admin action (`authority`), not a deploy of a new program id (though you may do both).

---

## Protocol Control

### `authority` (stored pubkey)

**What it does:** App-level ACL in account data (who may pause, rotate vkey, increment toy counter). Set in the handler from `payer.key()` in the scaffold.

**Without it:** Anyone could call admin instructions if you only checked “some signer exists.”

**Not the same as:** **Owner** (program that may mutate account *bytes*). Not **payer** (rent for this `init`). Not **signer** (this tx only). Next admin ix: that pubkey must sign *again*; storing authority is not a standing signature.

### Circuit breaker / `pause_flag`

**What it does:** Global freeze: settlement ix should abort when paused.

**Without it:** A known bug keeps draining the vault until you upgrade or the market empties it.

**Not the same as:** Closing accounts or pausing the whole Solana cluster. Only *your* settle path, and only if every settle ix actually reads the flag.

### CPI (cross-program invocation)

**What it does:** This program calls another (System transfer, Token transfer, SP1 verifier) mid-instruction.

**Without it:** You cannot create accounts, move SPL tokens, or verify Groth16 inside settle.

**Not the same as:** A **syscall** in general (CPI is one kind of runtime call). Not a new transaction (same tx, nested call).

### Fee payer (transaction)

**What it does:** Wallet that pays **tx fees** (and often signs). Can be the user, a relayer, or the same key as `init`’s `payer`.

**Without it:** The network will not include the transaction.

**Not the same as:** `payer = payer` on `init` (rent-exempt deposit onto the *new* account). Relayer as fee payer still does not automatically become `authority`.

### `pause_flag` / `is_paused`

**What it does:** Boolean on `GlobalConfig` for the circuit breaker.

**Without it:** No instant halt; you rely on deploy delays or authority transferring everything out.

**Not the same as:** `pause_flag` vs circuit breaker: same mechanism, two names. Not the PDA bump.

---

## Runtime and Transactions

### Instruction (ix)

**What it does:** One call: program id + data (discriminator + args) + account metas. A transaction is an atomic list of ixs.

**Without it:** No way to invoke `initialize_vault` or settle.

**Not the same as:** The whole **transaction**. Not the **Accounts** struct (that is source-level schema for one ix).

### Lamports

**What it does:** Smallest SOL unit. Rent, fees, and the toy 1-lamport transfer use lamports.

**Without it:** No native currency granularity on Solana.

**Not the same as:** SPL token amounts (those are mint decimals, not lamports). Not CU.

### `msg!`

**What it does:** Writes a line into **transaction logs** (explorers / `getTransaction`). Not stored in account data. Costs CU.

**Without it:** Harder to debug; no “Hello world” on the explorer. Production often strips noisy logs.

**Not the same as:** Journal (ZK public values). Not `#[msg]` on errors (that string is the error description). Logs are not authenticated the way a proof journal is.

### `Program<'info, System>`

**What it does:** Typed handle that the account is the System Program executable.

**Without it:** CPI target could be a fake program id; `init`/transfer would not hit the real System Program.

**Not the same as:** `#[program]` on *your* module. This field is “pass the System Program account into *this ix*.”

### Syscall

**What it does:** Sandboxed program asks the runtime to log, CPI, read clock, etc.

**Without it:** The program can only mutate the `AccountInfo` buffers it was given — no System/Token/verifier calls.

**Not the same as:** CPI specifically (CPI is the “call another program” syscall). `AccountInfo` is the *handle* you pass; syscall is the *trap* into the runtime.

### Transaction

**What it does:** Atomic bundle of instructions. All succeed or none persist.

**Without it:** You cannot change chain state from a wallet/client.

**Not the same as:** A single instruction. Not a block (many txs).
