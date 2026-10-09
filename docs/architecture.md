# Architecture

The system has three parts. The **client** holds the user's secret note. The **ZK circuit** turns private data into a proof. The **Solana program** checks the proof and settles.

## Components

Dotted boxes and arrows are planned and not built yet. Solid ones exist in the repo today.

```mermaid
flowchart TB
    User(["User"])
    Ui["Trading UI (planned)"]
    User -.->|"opens"| Ui
    Ui -.->|"swap request"| Client

    subgraph offchain [Off-chain]
        Client["Client / wallet"]
        Indexer["Backend indexer"]
        subgraph zk [zk-circuit]
            Io["io: shared types and Poseidon"]
            Host["host: prover driver"]
            Guest["guest: SP1 circuit"]
        end
        Prover["Succinct prover network"]
    end
    subgraph onchain [Solana]
        Program["zk_spot_shield program"]
        Config["GlobalConfig PDA"]
        Vault["VaultState PDA"]
        Roots["CleanFundsRoot PDAs"]
        Nullifiers["NullifierAccount PDAs"]
        Spl["SPL Token program"]
    end

    Client -->|"secret, amount"| Host
    Indexer -->|"Merkle path, root"| Host
    Host -->|"PrivateInputs"| Guest
    Guest -->|uses| Io
    Host -->|uses| Io
    Host -->|"ELF + stdin"| Prover
    Prover -->|"Groth16 proof + journal"| Host
    Host -->|"proof + journal"| Client
    Client -->|"settle transaction"| Program
    Program --> Config
    Program --> Vault
    Program --> Roots
    Program --> Nullifiers
    Program -->|"transfer"| Spl

    classDef planned stroke-dasharray: 5 5
    class Ui planned
```

## Settlement flow

```mermaid
sequenceDiagram
    participant U as Client
    participant I as Indexer
    participant H as ZK host
    participant G as ZK guest
    participant P as Prover network
    participant S as Solana program

    U->>I: which leaf is mine?
    I-->>U: Merkle path + root
    U->>H: secret, balance, amount, mint, path, root
    H->>G: PrivateInputs
    G->>G: check balance covers amount
    G->>G: recompute leaf and root
    G->>G: compute nullifier
    G-->>H: journal (amount, mint, nullifier, root)
    H->>P: request Groth16 proof
    P-->>H: proof bytes
    H-->>U: proof + journal
    U->>S: settle(proof, journal)
    S->>S: verify proof against vkey hash
    S->>S: root must be registered
    S->>S: nullifier account must not exist
    S->>S: create nullifier, update vault, transfer tokens
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
    end
    privateSide -->|"zero-knowledge proof"| publicSide
```

## Repository layout

| Folder | Role |
| --- | --- |
| `programs/zk_spot_shield/` | Anchor program `zk_spot_shield` (on-chain) |
| `zk-circuit/io/` | Shared input/output types and Poseidon Merkle helpers, used by both guest and host |
| `zk-circuit/guest/` | SP1 guest: the circuit that runs inside the zkVM |
| `zk-circuit/host/` | Host: feeds inputs to the guest, runs execute or requests a proof |
| `zk-circuit/fixtures/happy/` | Recorded Groth16 proof, journal, and verifying key from one real remote proof |
| `client/` | Client helpers (placeholder for the future SDK) |

See [Security](security.md) for the attack-surface table (threat, mitigation, recovery).
