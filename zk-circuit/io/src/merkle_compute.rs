use ark_bn254::Fr;
use ark_ff_04::{BigInteger, PrimeField};
use light_poseidon::{Poseidon, PoseidonHasher};

/// Interpret big-endian bytes as an integer and reduce into BN254 Fr.
///
/// `light_poseidon::hash_bytes_be` rejects values ≥ the scalar field (~2^254).
/// Solana pubkeys, mint addresses, and random secrets often sit above that.
/// Circom / `poseidon-lite` reduce the same way (`x mod p`), so TypeScript
/// leaves stay consistent with this crate.
fn bytes32_to_fr(bytes: &[u8]) -> Fr {
    Fr::from_be_bytes_mod_order(bytes)
}

fn hash_fields(inputs: &[Fr]) -> [u8; 32] {
    let mut hasher =
        Poseidon::<Fr>::new_circom(inputs.len()).expect("Failed to initialize Poseidon hasher");
    let out = hasher.hash(inputs).expect("Failed to compute Poseidon hash");
    let be = out.into_bigint().to_bytes_be();
    be.try_into().expect("BN254 Fr encodes as 32 bytes")
}

/// 2-to-1 Poseidon hash for Merkle tree nodes
pub fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    hash_fields(&[bytes32_to_fr(left), bytes32_to_fr(right)])
}

/// Private leaf commitment from note data
pub fn compute_leaf(secret: &[u8; 32], user_address: &[u8; 32], balance: u64) -> [u8; 32] {
    let balance_bytes = balance.to_be_bytes();
    let mut balance_padded = [0u8; 32];
    balance_padded[24..32].copy_from_slice(&balance_bytes);

    hash_fields(&[
        bytes32_to_fr(secret),
        bytes32_to_fr(user_address),
        bytes32_to_fr(&balance_padded),
    ])
}

/// Climb the depth-20 Merkle path to a root
pub fn verify_merkle_path(leaf: [u8; 32], path: &[([u8; 32], bool); 20]) -> [u8; 32] {
    let mut current = leaf;

    for (sibling, is_right) in path.iter() {
        if *is_right {
            current = hash_nodes(&current, sibling);
        } else {
            current = hash_nodes(sibling, &current);
        }
    }

    current
}

/// Depth-20 Merkle path for **leaf index 0** when every other leaf is zero.
///
/// Each hash is 32 bytes (bn254 field). `empty[h]` is the hash of a 2^h-wide
/// all-zero subtree: `empty[0] = [0; 32]`, `empty[h] = hash(empty[h-1], empty[h-1])`
/// (left and right are the *same* empty child, not `h` and `h+1`).
/// Index 0 always combines as `hash(current, sibling)`, so the sibling is on the
/// right (`true`) — same flag `verify_merkle_path` uses.
pub fn index0_empty_path() -> [([u8; 32], bool); 20] {
    let mut empty = [[0u8; 32]; 20];
    for h in 1..20 {
        empty[h] = hash_nodes(&empty[h - 1], &empty[h - 1]);
    }
    let mut path = [([0u8; 32], false); 20];
    for h in 0..20 {
        path[h] = (empty[h], true);
    }
    path
}

/// Unique 32-byte nullifier (double-spend tag)
pub fn compute_nullifier(secret: &[u8; 32], leaf: &[u8; 32], asset_id: &[u8; 32]) -> [u8; 32] {
    hash_fields(&[
        bytes32_to_fr(secret),
        bytes32_to_fr(leaf),
        bytes32_to_fr(asset_id),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use light_poseidon::PoseidonBytesHasher;

    #[test]
    fn leaf_is_stable_and_balance_endianness_matters() {
        let secret = [1u8; 32];
        let addr = [2u8; 32];
        let a = compute_leaf(&secret, &addr, 1000);
        let b = compute_leaf(&secret, &addr, 1000);
        let c = compute_leaf(&secret, &addr, 1001);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, [0u8; 32]);
    }

    #[test]
    fn index0_inclusion_path_matches_verify() {
        let leaf = compute_leaf(&[1u8; 32], &[2u8; 32], 1_000);
        let path = index0_empty_path();
        let root = verify_merkle_path(leaf, &path);
        assert_eq!(path[0], ([0u8; 32], true));
        assert_eq!(path[1].0, hash_nodes(&[0u8; 32], &[0u8; 32]));
        assert_ne!(root, leaf);
    }

    #[test]
    fn nullifier_binds_secret_leaf_and_mint() {
        let secret = [1u8; 32];
        let leaf = compute_leaf(&secret, &[2u8; 32], 1_000);
        let mint_a = [3u8; 32];
        let mint_b = [4u8; 32];
        let n = compute_nullifier(&secret, &leaf, &mint_a);
        assert_ne!(n, [0u8; 32]);
        assert_eq!(n, compute_nullifier(&secret, &leaf, &mint_a));
        assert_ne!(n, compute_nullifier(&secret, &leaf, &mint_b));
        assert_ne!(n, compute_nullifier(&[9u8; 32], &leaf, &mint_a));
    }

    #[test]
    fn sibling_side_changes_parent_hash() {
        let left = [1u8; 32];
        let right = [2u8; 32];
        assert_ne!(hash_nodes(&left, &right), hash_nodes(&right, &left));
    }

    #[test]
    fn in_field_inputs_match_hash_bytes_be() {
        let left = [1u8; 32];
        let right = [2u8; 32];
        let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
        let strict = hasher
            .hash_bytes_be(&[left.as_slice(), right.as_slice()])
            .unwrap();
        assert_eq!(hash_nodes(&left, &right), strict);
    }

    #[test]
    fn solana_pubkey_sized_bytes_hash() {
        let secret = [0xffu8; 32];
        let addr = [0xeeu8; 32];
        let mint = [0xddu8; 32];
        let leaf = compute_leaf(&secret, &addr, 1_000_000);
        assert_eq!(
            leaf,
            [
                0x0d, 0xad, 0x9f, 0x6a, 0x7d, 0xc1, 0x8f, 0x49, 0xa0, 0xa3, 0x03, 0xd3, 0xc0, 0xec,
                0xce, 0xad, 0xd7, 0x74, 0x6e, 0x48, 0xc9, 0xb9, 0xc5, 0x02, 0xec, 0x07, 0xca, 0x95,
                0xcc, 0xf1, 0x2d, 0x9e,
            ]
        );
        assert_eq!(leaf, compute_leaf(&secret, &addr, 1_000_000));
        let n = compute_nullifier(&secret, &leaf, &mint);
        assert_ne!(n, [0u8; 32]);
    }

    #[test]
    fn reduce_then_hash_matches_canonical_bytes() {
        let huge = [0xffu8; 32];
        let reduced = bytes32_to_fr(&huge);
        let canon: [u8; 32] = reduced.into_bigint().to_bytes_be().try_into().unwrap();
        assert_ne!(canon, huge);
        let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
        let via_bytes = hasher
            .hash_bytes_be(&[canon.as_slice(), canon.as_slice()])
            .unwrap();
        assert_eq!(hash_nodes(&huge, &huge), via_bytes);
    }
}
