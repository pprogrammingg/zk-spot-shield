//! Append-only depth-20 Poseidon Merkle tree for deposit leaves.
//!
//! Indexer / client helper — not used inside the SP1 guest. Empty missing
//! subtrees use the same `empty[h]` hashes as `index0_empty_path`.

use crate::merkle_compute::{hash_nodes, verify_merkle_path};

pub const MERKLE_DEPTH: usize = 20;
pub const MAX_LEAVES: usize = 1 << MERKLE_DEPTH;

/// Sequential-insert deposit tree. Leaf `i` is inserted at index `i`.
#[derive(Clone, Debug)]
pub struct DepositTree {
    leaves: Vec<[u8; 32]>,
    /// `empty[h]` = root of a 2^h-wide all-zero subtree (`empty[0] = [0; 32]`).
    empty: [[u8; 32]; MERKLE_DEPTH],
}

impl Default for DepositTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DepositTree {
    pub fn new() -> Self {
        let mut empty = [[0u8; 32]; MERKLE_DEPTH];
        for h in 1..MERKLE_DEPTH {
            empty[h] = hash_nodes(&empty[h - 1], &empty[h - 1]);
        }
        Self {
            leaves: Vec::new(),
            empty,
        }
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Append `leaf` at the next index. Returns that index.
    pub fn insert(&mut self, leaf: [u8; 32]) -> Result<usize, DepositTreeError> {
        if self.leaves.len() >= MAX_LEAVES {
            return Err(DepositTreeError::TreeFull);
        }
        let index = self.leaves.len();
        self.leaves.push(leaf);
        Ok(index)
    }

    pub fn leaf_at(&self, index: usize) -> Option<[u8; 32]> {
        self.leaves.get(index).copied()
    }

    /// Inclusion path for `index` under the current tip (guest path shape).
    pub fn path(&self, index: usize) -> Result<[([u8; 32], bool); MERKLE_DEPTH], DepositTreeError> {
        if index >= self.leaves.len() {
            return Err(DepositTreeError::IndexOutOfRange);
        }
        let mut path = [([0u8; 32], false); MERKLE_DEPTH];
        let mut idx = index;
        for (level, entry) in path.iter_mut().enumerate() {
            let sibling_idx = idx ^ 1;
            let sibling = self.node_at(level, sibling_idx);
            // Guest: `is_right` ⇒ sibling on the right ⇒ `hash(current, sibling)`.
            let is_right = idx.is_multiple_of(2);
            *entry = (sibling, is_right);
            idx /= 2;
        }
        Ok(path)
    }

    pub fn root(&self) -> [u8; 32] {
        if self.leaves.is_empty() {
            // Full empty depth-20 tree: hash(empty[19], empty[19]).
            return hash_nodes(
                &self.empty[MERKLE_DEPTH - 1],
                &self.empty[MERKLE_DEPTH - 1],
            );
        }
        let leaf0 = self.leaves[0];
        let path = self.path(0).expect("non-empty");
        verify_merkle_path(leaf0, &path)
    }

    /// Node at Merkle `level` (0 = leaves) and `index` within that level.
    fn node_at(&self, level: usize, index: usize) -> [u8; 32] {
        if level == 0 {
            return self.leaves.get(index).copied().unwrap_or([0u8; 32]);
        }
        let subtree_width = 1usize << level;
        let start = index.saturating_mul(subtree_width);
        if start >= self.leaves.len() {
            return self.empty[level];
        }
        let left = self.node_at(level - 1, index * 2);
        let right = self.node_at(level - 1, index * 2 + 1);
        hash_nodes(&left, &right)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositTreeError {
    TreeFull,
    IndexOutOfRange,
}

/// Build guest-shaped private inputs after a deposit leaf was inserted.
pub fn private_inputs_from_deposit(
    secret: [u8; 32],
    user_address: [u8; 32],
    balance: u64,
    requested_swap_amount: u64,
    asset_id_mint: [u8; 32],
    tree: &DepositTree,
    leaf_index: usize,
) -> Result<crate::PrivateInputs, DepositTreeError> {
    let merkle_path = tree.path(leaf_index)?;
    let leaf = tree
        .leaf_at(leaf_index)
        .ok_or(DepositTreeError::IndexOutOfRange)?;
    let expected_root = tree.root();
    debug_assert_eq!(expected_root, verify_merkle_path(leaf, &merkle_path));
    Ok(crate::PrivateInputs {
        secret,
        user_address,
        merkle_path,
        balance,
        requested_swap_amount,
        asset_id_mint,
        expected_root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle_compute::{compute_leaf, index0_empty_path};

    #[test]
    fn first_leaf_matches_index0_empty_path_root() {
        let secret = [1u8; 32];
        let addr = [2u8; 32];
        let leaf = compute_leaf(&secret, &addr, 1_000);
        let mut tree = DepositTree::new();
        let idx = tree.insert(leaf).unwrap();
        assert_eq!(idx, 0);
        let path = tree.path(0).unwrap();
        assert_eq!(path, index0_empty_path());
        let root = tree.root();
        assert_eq!(root, verify_merkle_path(leaf, &path));
        assert_eq!(root, verify_merkle_path(leaf, &index0_empty_path()));
    }

    #[test]
    fn private_inputs_from_deposit_happy_note() {
        let secret = [1u8; 32];
        let addr = [2u8; 32];
        let leaf = compute_leaf(&secret, &addr, 1_000);
        let mut tree = DepositTree::new();
        let idx = tree.insert(leaf).unwrap();
        let inputs =
            private_inputs_from_deposit(secret, addr, 1_000, 100, [3u8; 32], &tree, idx).unwrap();
        assert!(inputs.balance >= inputs.requested_swap_amount);
        assert_eq!(
            inputs.expected_root,
            verify_merkle_path(leaf, &inputs.merkle_path)
        );
    }

    #[test]
    fn second_leaf_changes_root() {
        let mut tree = DepositTree::new();
        let a = compute_leaf(&[1u8; 32], &[2u8; 32], 1_000);
        let b = compute_leaf(&[9u8; 32], &[8u8; 32], 500);
        tree.insert(a).unwrap();
        let root0 = tree.root();
        tree.insert(b).unwrap();
        let root1 = tree.root();
        assert_ne!(root0, root1);
        let path_b = tree.path(1).unwrap();
        assert_eq!(verify_merkle_path(b, &path_b), root1);
        let path_a = tree.path(0).unwrap();
        assert_eq!(verify_merkle_path(a, &path_a), root1);
    }
}
