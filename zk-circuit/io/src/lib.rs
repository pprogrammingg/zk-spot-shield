//! Shared circuit I/O and Poseidon helpers.
//! Used by the SP1 guest and the native host. No `sp1-zkvm` — keep this crate zkVM-safe.

pub mod merkle_compute;

pub use merkle_compute::{compute_leaf, compute_nullifier, hash_nodes, verify_merkle_path};

use serde::{Deserialize, Serialize};

/// Private inputs read from the host (witness).
/// Fixed-size arrays so the guest hot path stays stack-allocated.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateInputs {
    /// 32-byte secret controlling the shielded note
    pub secret: [u8; 32],
    /// 32-byte public key of the account holder
    pub user_address: [u8; 32],
    /// Depth-20 Merkle path: sibling hash + direction (`true` = sibling on the right)
    pub merkle_path: [([u8; 32], bool); 20],
    /// Total balance stored inside the private note commitment
    pub balance: u64,
    /// Trade or withdrawal amount requested
    pub requested_swap_amount: u64,
    /// 32-byte SPL token mint being spent
    pub asset_id_mint: [u8; 32],
    /// Expected on-chain Merkle root
    pub expected_root: [u8; 32],
}

/// Public outputs committed to the SP1 journal (verified on-chain later).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PublicOutputs {
    pub requested_swap_amount: u64,
    pub asset_id_mint: [u8; 32],
    pub nullifier: [u8; 32],
    pub merkle_root: [u8; 32],
}
