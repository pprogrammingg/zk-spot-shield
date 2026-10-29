//! Shared circuit I/O and Poseidon helpers.
//! Used by the SP1 guest and the native host. No `sp1-zkvm` — keep this crate zkVM-safe.

pub mod deposit_tree;
pub mod merkle_compute;

pub use deposit_tree::{
    private_inputs_from_deposit, DepositTree, DepositTreeError, MERKLE_DEPTH as DEPOSIT_TREE_DEPTH,
};
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

/// Why a native guest run was rejected (no journal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitReject {
    /// `requested_swap_amount > balance`
    Underfunded,
    /// Recomputed Merkle root ≠ `expected_root`
    MerkleRootMismatch,
    /// `change_amount + spend != balance` (should be unreachable if change is derived)
    Conservation,
}

/// Public outputs committed to the SP1 journal (verified on-chain later).
///
/// The committed happy-path Groth16 pack is 144 bytes (`change_amount` +
/// `change_leaf`). Spend-all still encodes 104 bytes (`change_amount = 0`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PublicOutputs {
    pub requested_swap_amount: u64,
    pub asset_id_mint: [u8; 32],
    pub nullifier: [u8; 32],
    pub merkle_root: [u8; 32],
    /// Remainder after spend (`balance - requested_swap_amount`). Zero on a
    /// 104-byte legacy journal (remainder was burned, not re-inserted).
    pub change_amount: u64,
    /// Poseidon leaf for the remainder note. All-zero on a legacy journal.
    pub change_leaf: [u8; 32],
}

impl PublicOutputs {
    /// Packed layout of a **legacy** journal (no change note): LE `u64` then three `[u8; 32]`.
    pub const JOURNAL_BYTE_LEN: usize = 8 + 32 + 32 + 32;
    /// Packed layout with change note: legacy + LE `u64` change amount + 32-byte change leaf.
    pub const JOURNAL_BYTE_LEN_WITH_CHANGE: usize = Self::JOURNAL_BYTE_LEN + 8 + 32;

    /// Decode a committed journal (`journal.bin`). Accepts 104 (legacy) or 144 (change note).
    pub fn from_journal_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::JOURNAL_BYTE_LEN
            && bytes.len() != Self::JOURNAL_BYTE_LEN_WITH_CHANGE
        {
            return None;
        }
        let requested_swap_amount = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let mut asset_id_mint = [0u8; 32];
        asset_id_mint.copy_from_slice(&bytes[8..40]);
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&bytes[40..72]);
        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&bytes[72..104]);
        let mut change_amount = 0u64;
        let mut change_leaf = [0u8; 32];
        if bytes.len() == Self::JOURNAL_BYTE_LEN_WITH_CHANGE {
            change_amount = u64::from_le_bytes(bytes[104..112].try_into().ok()?);
            change_leaf.copy_from_slice(&bytes[112..144]);
            if change_amount > 0 && change_leaf == [0u8; 32] {
                return None;
            }
        }
        Some(Self {
            requested_swap_amount,
            asset_id_mint,
            nullifier,
            merkle_root,
            change_amount,
            change_leaf,
        })
    }

    /// Packed bytes the guest / host / chain share. 104 bytes when there is no
    /// remainder note; 144 when `change_amount > 0`.
    pub fn to_journal_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::JOURNAL_BYTE_LEN_WITH_CHANGE);
        out.extend_from_slice(&self.requested_swap_amount.to_le_bytes());
        out.extend_from_slice(&self.asset_id_mint);
        out.extend_from_slice(&self.nullifier);
        out.extend_from_slice(&self.merkle_root);
        if self.change_amount > 0 {
            out.extend_from_slice(&self.change_amount.to_le_bytes());
            out.extend_from_slice(&self.change_leaf);
        }
        out
    }
}

/// Remainder after a spend, or `None` if the spend exceeds the note.
pub fn change_balance(balance: u64, spend: u64) -> Option<u64> {
    balance.checked_sub(spend)
}

/// Guest constraints as a native function (same hashes the zkVM guest uses).
pub fn public_outputs_from_private(inputs: &PrivateInputs) -> Result<PublicOutputs, CircuitReject> {
    let change_amount = change_balance(inputs.balance, inputs.requested_swap_amount)
        .ok_or(CircuitReject::Underfunded)?;
    if change_amount.checked_add(inputs.requested_swap_amount) != Some(inputs.balance) {
        return Err(CircuitReject::Conservation);
    }
    let leaf = compute_leaf(&inputs.secret, &inputs.user_address, inputs.balance);
    let computed_root = verify_merkle_path(leaf, &inputs.merkle_path);
    if computed_root != inputs.expected_root {
        return Err(CircuitReject::MerkleRootMismatch);
    }
    let nullifier = compute_nullifier(&inputs.secret, &leaf, &inputs.asset_id_mint);
    let change_leaf = compute_leaf(&inputs.secret, &inputs.user_address, change_amount);
    Ok(PublicOutputs {
        requested_swap_amount: inputs.requested_swap_amount,
        asset_id_mint: inputs.asset_id_mint,
        nullifier,
        merkle_root: computed_root,
        change_amount,
        change_leaf,
    })
}

#[cfg(test)]
mod happy_fixture_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn happy_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/happy")
    }

    /// Same bytes as host `happy_path_inputs` / `fixtures/happy.json`.
    #[test]
    fn happy_journal_matches_io_hashes() {
        let bytes = fs::read(happy_dir().join("journal.bin"))
            .expect("commit zk-circuit/fixtures/happy/journal.bin from a remote Groth16 wrap");
        let public = PublicOutputs::from_journal_bytes(&bytes).expect("journal layout");
        let secret = [1u8; 32];
        let leaf = compute_leaf(&secret, &[2u8; 32], 1_000);
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.asset_id_mint, [3u8; 32]);
        assert_eq!(
            public.merkle_root,
            verify_merkle_path(leaf, &index0_empty_path())
        );
        assert_eq!(
            public.nullifier,
            compute_nullifier(&secret, &leaf, &[3u8; 32])
        );
        assert_eq!(public.change_amount, 900);
        assert_eq!(bytes.len(), PublicOutputs::JOURNAL_BYTE_LEN_WITH_CHANGE);
        let inputs = happy_private_inputs();
        let expected = public_outputs_from_private(&inputs).expect("funded");
        assert_eq!(public, expected);
        assert_ne!(public.change_leaf, [0u8; 32]);
    }

    #[test]
    fn happy_groth16_is_nonempty_remote_wrap() {
        let groth16 = fs::read(happy_dir().join("groth16.bin"))
            .expect("commit zk-circuit/fixtures/happy/groth16.bin from a remote Groth16 wrap");
        assert_eq!(groth16.len(), 356, "SP1 6.5 proof.bytes() for this ELF");
        assert!(
            groth16.iter().any(|b| *b != 0),
            "do not fake an all-zero Groth16"
        );
        let vkey = fs::read_to_string(happy_dir().join("vkey.bytes32.txt"))
            .expect("commit vkey.bytes32.txt")
            .trim()
            .to_string();
        assert_eq!(
            vkey,
            "0x00435696d666d305172306e3895a44017e188663de9b3876970ae270345fc240"
        );
    }

    fn happy_private_inputs() -> PrivateInputs {
        let secret = [1u8; 32];
        let user_address = [2u8; 32];
        let asset_id_mint = [3u8; 32];
        let balance = 1_000;
        let requested_swap_amount = 100;
        let leaf = compute_leaf(&secret, &user_address, balance);
        let merkle_path = index0_empty_path();
        let expected_root = verify_merkle_path(leaf, &merkle_path);
        PrivateInputs {
            secret,
            user_address,
            merkle_path,
            balance,
            requested_swap_amount,
            asset_id_mint,
            expected_root,
        }
    }

    #[test]
    fn spend_100_of_1000_commits_change_leaf_900() {
        let inputs = happy_private_inputs();
        let public = public_outputs_from_private(&inputs).expect("funded");
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.change_amount, 900);
        assert_eq!(
            public
                .change_amount
                .checked_add(public.requested_swap_amount),
            Some(inputs.balance)
        );
        let want_leaf = compute_leaf(&inputs.secret, &inputs.user_address, 900);
        assert_eq!(public.change_leaf, want_leaf);
        assert_ne!(public.change_leaf, [0u8; 32]);
        let packed = public.to_journal_bytes();
        assert_eq!(packed.len(), PublicOutputs::JOURNAL_BYTE_LEN_WITH_CHANGE);
        let round = PublicOutputs::from_journal_bytes(&packed).expect("144");
        assert_eq!(round, public);
    }

    #[test]
    fn underfunded_spend_is_rejected() {
        let mut inputs = happy_private_inputs();
        inputs.requested_swap_amount = inputs.balance + 1;
        assert_eq!(
            public_outputs_from_private(&inputs),
            Err(CircuitReject::Underfunded)
        );
        assert_eq!(
            change_balance(inputs.balance, inputs.requested_swap_amount),
            None
        );
    }

    #[test]
    fn corrupt_merkle_path_is_rejected() {
        let mut inputs = happy_private_inputs();
        inputs.merkle_path[0].0 = [4u8; 32];
        assert_eq!(
            public_outputs_from_private(&inputs),
            Err(CircuitReject::MerkleRootMismatch)
        );
    }

    #[test]
    fn spend_all_encodes_legacy_104_byte_journal() {
        let mut inputs = happy_private_inputs();
        inputs.requested_swap_amount = inputs.balance;
        let public = public_outputs_from_private(&inputs).expect("spend-all");
        assert_eq!(public.change_amount, 0);
        let packed = public.to_journal_bytes();
        assert_eq!(packed.len(), PublicOutputs::JOURNAL_BYTE_LEN);
        let round = PublicOutputs::from_journal_bytes(&packed).expect("104");
        assert_eq!(round.change_amount, 0);
        assert_eq!(round.change_leaf, [0u8; 32]);
    }

    #[test]
    fn change_note_is_new_membership_with_new_nullifier() {
        let spent = happy_private_inputs();
        let first = public_outputs_from_private(&spent).expect("spend 100");
        let mut tree = DepositTree::new();
        let old_leaf = compute_leaf(&spent.secret, &spent.user_address, spent.balance);
        tree.insert(old_leaf).expect("old");
        let change_idx = tree.insert(first.change_leaf).expect("change");
        let mut change_spend = spent.clone();
        change_spend.balance = 900;
        change_spend.requested_swap_amount = 50;
        change_spend.merkle_path = tree.path(change_idx).expect("path");
        change_spend.expected_root = tree.root();
        let second = public_outputs_from_private(&change_spend).expect("spend change");
        assert_ne!(second.nullifier, first.nullifier);
        assert_eq!(second.change_amount, 850);
        assert_eq!(
            second.change_leaf,
            compute_leaf(&spent.secret, &spent.user_address, 850)
        );
    }

    #[test]
    fn journal_rejects_unknown_lengths_and_zero_change_leaf() {
        assert!(PublicOutputs::from_journal_bytes(&[0u8; 105]).is_none());
        assert!(PublicOutputs::from_journal_bytes(&[0u8; 143]).is_none());
        let mut bad = vec![0u8; PublicOutputs::JOURNAL_BYTE_LEN_WITH_CHANGE];
        bad[0..8].copy_from_slice(&100u64.to_le_bytes());
        bad[104..112].copy_from_slice(&900u64.to_le_bytes());
        // change_leaf stays all-zero while change_amount > 0
        assert!(PublicOutputs::from_journal_bytes(&bad).is_none());
    }
}
