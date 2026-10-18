# Architecture

The system has three parts. The **client** holds the user's secret note. The **ZK circuit** turns private data into a proof. The **Solana program** checks the proof and settles.

Dotted boxes and arrows are **(upcoming)** product direction — not shipped yet. Solid ones exist in the repo today.

## Components

```mermaid
flowchart TB
    User(["User"])
    Ui["Trading UI / web SDK"]
    User -->|"1 opens app"| Ui
    Ui -->|"connect wallet"| Client

    subgraph offchain [Off-chain]
        Client["Client / Phantom"]
        Indexer["Deposit tree / indexer"]
        Relayer["Relayer fee-payer"]
        subgraph zk [zk-circuit]
            Io["io: shared types and Poseidon"]
            Host["host: execute + prove driver"]
            Guest["guest: SP1 circuit"]
        end
        Prover["Succinct prover network"]
    end
    subgraph onchain [Solana]
        Program["zk_spot_shield program"]
        Config["GlobalConfig PDA"]
        Vault["VaultState PDA"]
        VaultAta["Vault token ATAs · mint A and B"]
        Leaves["DepositLeafAccount PDAs"]
        Roots["CleanFundsRoot PDAs"]
        Nullifiers["NullifierAccount PDAs"]
        Spl["SPL Token program"]
    end

    Client -->|"secret, amount, path"| Host
    Indexer -->|"Merkle path, root"| Host
    Host -->|"PrivateInputs then execute"| Guest
    Guest -->|uses| Io
    Host -->|uses| Io
    Host -->|"ELF + stdin"| Prover
    Prover -->|"Groth16 proof + journal"| Host
    Host -->|"proof + journal"| Client
    Client -->|"settle or hand to relayer"| Relayer
    Client -->|"settle transaction"| Program
    Relayer -.->|"upcoming pay SOL and prove costs"| Program
    Program --> Config
    Program --> Vault
    Program --> Leaves
    Program --> Roots
    Program --> Nullifiers
    Program -->|"CPI transfer"| Spl
    Spl --> VaultAta

    classDef planned stroke-dasharray: 5 5
    class Relayer planned
```

## Numbered product flow

Swap economics today are **1:1 spot** (journal `amount` of mint A ↔ same `amount` of mint B). An AMM-style “100 A for 50 B” curve is **(upcoming)**.

What the chain learns on settle: **amount**, **asset mint**, **nullifier**, **Merkle root**. What stays private in the proof: **secret**, **full note balance**, **Merkle path / leaf index**.

Step numbers **continue across phases** (1 → …). Each phase is its own diagram so the docs site can render them. Dotted arrows in phase E are **(upcoming)**.

| Phase | Steps | Status |
| --- | --- | --- |
| **A** Connect | 1–3 | UI polish **(upcoming)** |
| **B** Deposit | 4–12 | On-chain **built**; hosted indexer **(upcoming)** |
| **C** Prove | 13–19 | **Built** |
| **D** Settle today | 20–29 | **Built** (wallet legs for journal mint) |
| **E** Upcoming | 30–34 | **(upcoming)** |

### Phase A — Connect (steps 1–3)

```mermaid
sequenceDiagram
    actor U as User
    participant W as Web app / SDK
    participant Ph as Phantom

    U->>W: 1 Open web app
    U->>Ph: 2 Connect wallet and approve site
    Ph-->>W: 3 User pubkey available to SDK
```

### Phase B — Deposit (steps 4–12)

```mermaid
sequenceDiagram
    actor U as User
    participant W as Web app / SDK
    participant Ph as Phantom
    participant Prog as zk_spot_shield
    participant Spl as SPL Token
    participant Tree as Deposit tree

    U->>W: 4 Pick mint A and amount e.g. 300
    W->>W: 5 Create note secret; leaf equals Poseidon commitment
    W->>Ph: 6 Sign deposit_shielded
    Ph->>Prog: 7 deposit_shielded leaf and amount
    Prog->>Spl: 8 CPI user ATA to vault ATA for mint A
    Prog->>Prog: 9 Init DepositLeafAccount PDA
    Prog->>Prog: 10 Bump VaultState reserve
    Note over Prog: Leaf PDA holds leaf, amount, mint, depositor. Depositor is visible today.
    Prog-->>Tree: 11 Indexer inserts leaf into deposit tree
    Tree-->>Prog: 12 Authority register_clean_funds_root for tip
    Note over Tree,Prog: CleanFundsRoot PDA is the approved Merkle root fingerprint
```

### Phase C — Prove (steps 13–19)

```mermaid
sequenceDiagram
    actor U as User
    participant W as Web app / SDK
    participant Tree as Deposit tree
    participant Host as ZK host
    participant Net as Succinct network

    U->>W: 13 Choose shield swap amount of A for B at 1 to 1
    W->>Tree: 14 Fetch Merkle path under a registered root
    Tree-->>W: 15 Path and root
    W->>Host: 16 Witness secret, balance, amount, mint, path, root
    Host->>Host: 17 Cheap execute guest; fail closed if invalid
    Host->>Net: 18 Request Groth16 wrap
    Net-->>Host: Groth16 proof and journal
    Host-->>W: 19 Proof and journal ready for settle
```

### Phase D — Settle today (steps 20–29)

```mermaid
sequenceDiagram
    participant W as Web app / SDK
    participant Ph as Phantom
    participant Prog as zk_spot_shield
    participant Spl as SPL Token

    W->>Ph: 20 Sign settle_shielded_spot or hand pack to relayer
    Ph->>Prog: 21 settle_shielded_spot proof and journal
    Note over Prog: Proof and journal are instruction data, not VaultState fields
    Prog->>Prog: 22 Reject if paused
    Prog->>Prog: 23 Verify Groth16 vs GlobalConfig vkey_hash
    Prog->>Prog: 24 Journal merkle_root must match CleanFundsRoot
    Prog->>Prog: 25 Nullifier PDA must be empty
    Prog->>Prog: 26 apply_swap on VaultState reserves 1 to 1
    Prog->>Spl: 27 User pays journal mint into vault ATA
    Prog->>Spl: 28 Vault PDA pays other mint to user ATA
    Prog->>Prog: 29 Create NullifierAccount PDA
    Note over Prog: Today step 27 still debits the payer wallet even after deposit funded the vault
```

### Phase E — Upcoming (steps 30–34)

```mermaid
sequenceDiagram
    participant Rel as Relayer
    participant Net as Succinct network
    participant Prog as zk_spot_shield
    participant W as Web app / SDK

    Rel-->>Net: 30 Relayer treasury pays prove credits
    Rel-->>Prog: 31 Relayer submits settle and pays SOL fees
    Note over Rel: User fees refill relayer for PROVE and SOL
    Note over W,Prog: 32 Hidden recipient / unlink deposit from withdraw
    Note over W,Prog: 33 Change note for leftover balance after partial spend
    Note over W,Prog: 34 Vault-only spend without second user to vault of mint A
```

## What stays private

```mermaid
flowchart TB
    subgraph privateSide [Private: never leaves the prover]
        Secret[secret]
        Balance[full balance]
        Path[Merkle path]
        Index[leaf position]
    end
    subgraph publicSide [Public: journal and chain]
        Amount[swap amount]
        Mint[token mint]
        Nullifier[nullifier]
        Root[Merkle root]
        Proof[Groth16 proof]
        DepositLink[deposit tx + depositor on leaf PDA]
    end
    privateSide -->|"zero-knowledge proof"| publicSide
```

Deposit today still links a **wallet** to a leaf on-chain. Settle hides the secret and leaf index, but the settle transaction still lists **token accounts** (and often the fee payer). Full unlink is **(upcoming)**.

## Repository layout

| Folder | Role |
| --- | --- |
| `programs/zk_spot_shield/` | Anchor program `zk_spot_shield` (on-chain) |
| `zk-circuit/io/` | Shared input/output types and Poseidon Merkle helpers |
| `zk-circuit/guest/` | SP1 guest circuit |
| `zk-circuit/host/` | Host: execute and/or request a proof |
| `zk-circuit/fixtures/happy/` | Recorded Groth16 proof, journal, and verifying key |
| `client/` | Rust CU / ALT helpers |
| `relayer/` | Axum fee-payer scaffold (`/health`, `/submit-proof` queue) |
| `sdk/` | TypeScript Anchor provider, Merkle helpers, settle bridge |

See [Accounts deep dive](accounts-deep-dive.md) for PDA vs ATA vs instruction data, and [Solana Program](solana-program.md) for instruction semantics. See [Security](security.md) for the attack-surface table.
