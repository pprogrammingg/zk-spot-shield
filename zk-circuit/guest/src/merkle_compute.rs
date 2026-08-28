use ark_bn254::Fr;
use light_poseidon::{Poseidon, PoseidonBytesHasher};

/// Computes a 2-to-1 Poseidon hash for Merkle tree nodes
pub fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Poseidon::<Fr>::new_circom(2).expect("Failed to initialize Poseidon hasher");
    
    hasher
        .hash_bytes_be(&[left.as_slice(), right.as_slice()])
        .expect("Failed to compute node hash")
}

/// Derives the private leaf commitment from note data
pub fn compute_leaf(secret: &[u8; 32], user_address: &[u8; 32], balance: u64) -> [u8; 32] {
    let mut hasher = Poseidon::<Fr>::new_circom(3).expect("Failed to initialize Poseidon hasher");
    
    let balance_bytes = balance.to_be_bytes();
    let mut balance_padded = [0u8; 32];
    balance_padded[24..32].copy_from_slice(&balance_bytes);

    hasher
        .hash_bytes_be(&[secret.as_slice(), user_address.as_slice(), balance_padded.as_slice()])
        .expect("Failed to compute leaf hash")
}

/// Climbs the 20-depth Merkle tree to compute the root
pub fn verify_merkle_path(
    leaf: [u8; 32], 
    path: &[([u8; 32], bool); 20]
) -> [u8; 32] {
    let mut current = leaf;

    for (sibling, is_right) in path.iter() {
        if *is_right {
            // Sibling is on the right -> Current is on the left
            current = hash_nodes(&current, sibling);
        } else {
            // Sibling is on the left -> Current is on the right
            current = hash_nodes(sibling, &current);
        }
    }

    current
}

/// Computes a unique 32-byte nullifier to prevent double-spending
pub fn compute_nullifier(secret: &[u8; 32], leaf: &[u8; 32], asset_id: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Poseidon::<Fr>::new_circom(3).expect("Failed to initialize Poseidon hasher");

    hasher
        .hash_bytes_be(&[secret.as_slice(), leaf.as_slice(), asset_id.as_slice()])
        .expect("Failed to compute nullifier hash")
}