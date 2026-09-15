# Merkle trees (happy-path tests)

How the **empty depth-20 tree** works in this repo’s fixtures, and why tests use it instead of a hand-built multi-leaf tree.

Code: `zk_circuit_io::index0_empty_path`, `verify_merkle_path`, `compute_leaf` (`zk-circuit/io`). Host fixture: `happy_path_inputs()`.

---

## Why an “empty” tree in tests?

| Goal | Empty index-0 tree | Concrete multi-leaf tree |
| --- | --- | --- |
| Prove inclusion logic + journal hashes | Yes | Yes |
| Deterministic, no indexer | Yes | Need a builder + fixed leaves |
| Tiny fixture (no 2²⁰ leaves stored) | Yes — only **20 siblings** | Still only 20 siblings, but you must invent other leaves |
| Match a real product tree later | No — toy only | Closer, but easy to drift |

We are **not** claiming the product tree is empty. We need one leaf that is a real Poseidon commitment, and a path that hashes to a stable root. Putting that leaf at **index 0** and treating every other leaf as the zero hash `[0u8; 32]` is the smallest such tree: siblings are empty-subtree hashes, computed once, no randomness, no “user account” from production.

The Groth16 bytes in `zk-circuit/fixtures/happy/` prove **this** witness. They are public test artifacts (toy secret `[1u8; 32]`), not user funds — safe and useful to **commit** so CI can read them offline.

---

## Shapes (data structures)

```text
Leaf / hash / root / sibling / nullifier
  → [u8; 32]     // one bn254 field element as 32 bytes

empty (empty-subtree table)
  → [[u8; 32]; 20]
     empty[h] = hash of a perfect 2^h-wide all-zero subtree

merkle_path (witness path for one leaf)
  → [([u8; 32], bool); 20]
     path[h] = (sibling_hash, is_right)
     is_right == true  → hash(current, sibling)   // sibling on the right
     is_right == false → hash(sibling, current)   // sibling on the left

PrivateInputs.merkle_path
  → same [([u8; 32], bool); 20]

Journal PublicOutputs (after prove)
  → { amount: u64, mint: [u8;32], nullifier: [u8;32], merkle_root: [u8;32] }
```

Depth **20** ⇒ up to **2²⁰** leaves. Tests only place a real leaf at **index 0**.

---

## Tiny picture (depth 3 instead of 20)

Same idea; height labels match `empty[h]`.

```text
                    root = hash(N1, empty[2])
                   /                    \
              N1 = hash(L, empty[0])    empty[2] = hash(empty[1], empty[1])
             /              \            (entire right half is "empty")
            L              empty[0]
         (our leaf)       = [0; 32]
```

For index **0**, every step the live node is on the **left**, so the sibling is always on the **right** (`is_right = true`).

Depth 20 is the same ladder, twenty times.

---

## Step by step (what the test builds)

### Step 0 — Fixed toy note (not a real user)

```text
secret         = [1u8; 32]     // 32 bytes, all 0x01
user_address   = [2u8; 32]
asset_id_mint  = [3u8; 32]
balance        = 1000_u64
requested_swap = 100_u64       // ≤ balance → solvency ok
```

### Step 1 — Leaf commitment

```text
leaf: [u8; 32] = Poseidon3(secret, user_address, balance_as_32_be)
```

`balance` is big-endian in the low 8 bytes of a 32-byte pad (see `compute_leaf`).

### Step 2 — Build `empty[0..20)` (empty subtrees)

```text
empty: [[u8; 32]; 20]

empty[0] = [0u8; 32]                          // "empty leaf"
empty[1] = hash(empty[0], empty[0])           // two empty leaves
empty[2] = hash(empty[1], empty[1])           // four empty leaves
...
empty[h] = hash(empty[h-1], empty[h-1])       // 2^h empty leaves
```

Left and right are the **same** child on purpose: a full empty subtree is two identical empty half-trees. Not `empty[h]` with `empty[h+1]`.

```text
h=0:  ○                 ○ = [0;32]
h=1:  hash(○,○)
h=2:  hash( h1 , h1 )
h=3:  hash( h2 , h2 )
...
```

### Step 3 — Path for leaf index 0

```text
path: [([u8; 32], bool); 20]

for h in 0..20:
  path[h] = (empty[h], true)   // sibling = empty subtree at this height; right side
```

At height 0, sibling is the other leaf under the same parent → empty leaf `[0;32]`.  
At height 1, sibling is the empty pair-of-leaves hash `empty[1]`, and so on.

### Step 4 — Climb to root (`verify_merkle_path`)

```text
current: [u8; 32] = leaf

for h in 0..20:
  (sibling, is_right) = path[h]
  // is_right always true for index 0:
  current = hash(current, sibling)

expected_root: [u8; 32] = current   // after 20 steps
```

Visual for the first two steps:

```text
step h=0:
  current = hash( leaf , empty[0] )     // sibling right

step h=1:
  current = hash( current , empty[1] )

...
step h=19:
  current = hash( current , empty[19] ) → root
```

### Step 5 — What the guest checks

```text
assert balance >= requested_swap_amount
recompute leaf from (secret, address, balance)
recompute root from (leaf, merkle_path)
assert root == expected_root
nullifier = Poseidon3(secret, leaf, asset_id_mint)
commit PublicOutputs { amount, mint, nullifier, merkle_root }
```

### Step 6 — What CI tests (offline)

| Check | Input shape | Assert |
| --- | --- | --- |
| Journal matches hashes | `journal.bin` (104 bytes) | amount=100, mint=`[3;32]`, root = Step 4, nullifier = Step 5 |
| Groth16 present | `groth16.bin` (356 bytes) | non-empty, not all zeros; vkey string matches wrap |

No prover call. Same empty path as above.

---

## Mental model: one filled slot in a huge empty array

```text
Leaf slots (conceptual), depth 20 → 2^20 positions:

index:  0      1      2      3     ...   2^20-1
        [L]   [0]    [0]    [0]   ...   [0]

L = our leaf. Everything else = zero leaf.
The Merkle path only needs the sibling at each level of that sparse tree.
We never allocate 2^20 leaves in memory — only empty[0..20] and path[0..20].
```

---

## What this is *not*

- Not the live Devnet / production membership tree.
- Not a privacy claim about the toy `secret` (it is fixed and public in tests).
- Not a mock Groth16: `groth16.bin` is a real wrap of **this** empty-tree witness, frozen for offline CI.
