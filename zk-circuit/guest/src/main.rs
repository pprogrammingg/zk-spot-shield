#![no_main]
sp1_zkvm::entrypoint!(main);

mod merkle_compute;

use merkle_compute::{compute_leaf, compute_nullifier, verify_merkle_path};
use serde::{Deserialize, Serialize};

/// Private inputs read from the Host (Witness)
/// Uses fixed-size arrays to guarantee stack allocation and zero dynamic memory (heap) overhead.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateInputs {
    /// 32-byte secret key controlling the shielded note
    pub secret: [u8; 32],
    
    /// 32-byte public key of the account holder
    pub user_address: [u8; 32],
    
    /// Depth-20 Merkle path: Array of 20 sibling hashes paired with a direction flag.
    /// (bool = true means sibling is on the right; false means left)
    pub merkle_path: [([u8; 32], bool); 20],
    
    /// Total balance stored inside the private note commitment
    pub balance: u64,
    
    /// The trade or withdrawal amount requested by the user
    pub requested_swap_amount: u64,
    
    /// The 32-byte SPL token mint address being spent
    pub asset_id_mint: [u8; 32],
    
    /// The expected on-chain Merkle root hash
    pub expected_root: [u8; 32],
}

/// Public outputs committed to the SP1 Journal
/// Verified on-chain by the Solana Anchor program.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PublicOutputs {
    /// Publicly verified trade amount
    pub requested_swap_amount: u64,
    
    /// Publicly verified SPL token mint
    pub asset_id_mint: [u8; 32],
    
    /// Unique 32-byte hash preventing double spending
    pub nullifier: [u8; 32],
    
    /// The state root this note belonged to
    pub merkle_root: [u8; 32],
}

pub fn main() {
    // 1. Read Private Witness from Host
    let inputs: PrivateInputs = sp1_zkvm::io::read();

    // 2. Solvency Check: Ensure user owns enough funds
    assert!(
        inputs.balance >= inputs.requested_swap_amount,
        "Solvency violation: Insufficient balance"
    );

    // 3. Reconstruct Private Leaf Commitment
    let leaf = compute_leaf(&inputs.secret, &inputs.user_address, inputs.balance);

    // 4. Merkle Path Traversal & Root Verification
    let computed_root = verify_merkle_path(leaf, &inputs.merkle_path);
    assert_eq!(
        computed_root, inputs.expected_root, 
        "Merkle root mismatch: Computed path does not match state root"
    );

    // 5. Derive Unique Spend Marker (Nullifier)
    let nullifier = compute_nullifier(&inputs.secret, &leaf, &inputs.asset_id_mint);

    // 6. Construct & Commit Public Outputs to SP1 Journal
    let outputs = PublicOutputs {
        requested_swap_amount: inputs.requested_swap_amount,
        asset_id_mint: inputs.asset_id_mint,
        nullifier,
        merkle_root: computed_root,
    };

    sp1_zkvm::io::commit(&outputs);
}