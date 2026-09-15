//! Shared circuit I/O and Poseidon helpers.
//! Used by the SP1 guest and the native host. No `sp1-zkvm` — keep this crate zkVM-safe.

pub mod merkle_compute;

pub use merkle_compute::{
    compute_leaf, compute_nullifier, hash_nodes, index0_empty_path, verify_merkle_path,
};

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

impl PublicOutputs {
    /// Packed `sp1_zkvm::io::commit` layout for this struct: LE `u64` then three `[u8; 32]`.
    pub const JOURNAL_BYTE_LEN: usize = 8 + 32 + 32 + 32;

    /// Decode a committed journal (`journal.bin`). Not a second serde layout.
    pub fn from_journal_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::JOURNAL_BYTE_LEN {
            return None;
        }
        let requested_swap_amount = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let mut asset_id_mint = [0u8; 32];
        asset_id_mint.copy_from_slice(&bytes[8..40]);
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&bytes[40..72]);
        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&bytes[72..104]);
        Some(Self {
            requested_swap_amount,
            asset_id_mint,
            nullifier,
            merkle_root,
        })
    }
}

#[cfg(test)]
mod happy_fixture_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn happy_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/happy")
    }

    #[test]
    fn happy_journal_matches_io_hashes() {
        let bytes = fs::read(happy_dir().join("journal.bin"))
            .expect("commit zk-circuit/fixtures/happy/journal.bin from a remote Groth16 wrap");
        let public = PublicOutputs::from_journal_bytes(&bytes).expect("journal layout");
        let secret = [1u8; 32];
        let leaf = compute_leaf(&secret, &[2u8; 32], 1_000);
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.asset_id_mint, [3u8; 32]);
        assert_eq!(public.merkle_root, verify_merkle_path(leaf, &index0_empty_path()));
        assert_eq!(
            public.nullifier,
            compute_nullifier(&secret, &leaf, &[3u8; 32])
        );
    }

    #[test]
    fn happy_groth16_is_nonempty_remote_wrap() {
        let groth16 = fs::read(happy_dir().join("groth16.bin"))
            .expect("commit zk-circuit/fixtures/happy/groth16.bin from a remote Groth16 wrap");
        assert_eq!(groth16.len(), 356, "SP1 6.5 proof.bytes() for this ELF");
        assert!(groth16.iter().any(|b| *b != 0), "do not fake an all-zero Groth16");
        let vkey = fs::read_to_string(happy_dir().join("vkey.bytes32.txt"))
            .expect("commit vkey.bytes32.txt")
            .trim()
            .to_string();
        assert_eq!(
            vkey,
            "0x00b3a15ce4c0ea94e3b0267473c6b7543a80c72d209b2c947f71886c6a5735d7"
        );
    }
}
