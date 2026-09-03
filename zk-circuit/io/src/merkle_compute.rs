use ark_bn254::Fr;
use light_poseidon::{Poseidon, PoseidonBytesHasher};

/// 2-to-1 Poseidon hash for Merkle tree nodes
pub fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Poseidon::<Fr>::new_circom(2).expect("Failed to initialize Poseidon hasher");

    hasher
        .hash_bytes_be(&[left.as_slice(), right.as_slice()])
        .expect("Failed to compute node hash")
}

/// Private leaf commitment from note data
pub fn compute_leaf(secret: &[u8; 32], user_address: &[u8; 32], balance: u64) -> [u8; 32] {
    let mut hasher = Poseidon::<Fr>::new_circom(3).expect("Failed to initialize Poseidon hasher");

    let balance_bytes = balance.to_be_bytes();
    let mut balance_padded = [0u8; 32];
    balance_padded[24..32].copy_from_slice(&balance_bytes);

    hasher
        .hash_bytes_be(&[
            secret.as_slice(),
            user_address.as_slice(),
            balance_padded.as_slice(),
        ])
        .expect("Failed to compute leaf hash")
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

/// Unique 32-byte nullifier (double-spend tag)
pub fn compute_nullifier(secret: &[u8; 32], leaf: &[u8; 32], asset_id: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Poseidon::<Fr>::new_circom(3).expect("Failed to initialize Poseidon hasher");

    hasher
        .hash_bytes_be(&[secret.as_slice(), leaf.as_slice(), asset_id.as_slice()])
        .expect("Failed to compute nullifier hash")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index0_path(leaf: [u8; 32]) -> ([([u8; 32], bool); 20], [u8; 32]) {
        let mut empty = [[0u8; 32]; 20];
        empty[0] = [0u8; 32];
        for h in 1..20 {
            empty[h] = hash_nodes(&empty[h - 1], &empty[h - 1]);
        }
        let mut path = [([0u8; 32], false); 20];
        for h in 0..20 {
            path[h] = (empty[h], true);
        }
        let root = verify_merkle_path(leaf, &path);
        (path, root)
    }

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
        let (path, root) = index0_path(leaf);
        assert_eq!(verify_merkle_path(leaf, &path), root);
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
}
