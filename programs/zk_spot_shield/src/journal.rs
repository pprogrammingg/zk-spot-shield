//! On-chain journal decode (same byte layout as `zk_circuit_io::PublicOutputs`).

/// Packed layout of a **legacy** journal (no change note): LE `u64` + three `[u8; 32]`.
pub const JOURNAL_BYTE_LEN: usize = 8 + 32 + 32 + 32;
/// Packed layout with change note: legacy + LE `u64` change amount + 32-byte change leaf.
pub const JOURNAL_BYTE_LEN_WITH_CHANGE: usize = JOURNAL_BYTE_LEN + 8 + 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalPublic {
    pub requested_swap_amount: u64,
    pub asset_id_mint: [u8; 32],
    pub nullifier: [u8; 32],
    pub merkle_root: [u8; 32],
    pub change_amount: u64,
    pub change_leaf: [u8; 32],
}

impl JournalPublic {
    pub fn from_journal_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != JOURNAL_BYTE_LEN && bytes.len() != JOURNAL_BYTE_LEN_WITH_CHANGE {
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
        if bytes.len() == JOURNAL_BYTE_LEN_WITH_CHANGE {
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

    /// Remainder note bound by a 144-byte journal. `None` for legacy 104-byte journals
    /// and for spend-all (`change_amount == 0`).
    pub fn change_note(&self) -> Option<([u8; 32], u64)> {
        if self.change_amount == 0 {
            return None;
        }
        if self.change_leaf == [0u8; 32] {
            return None;
        }
        Some((self.change_leaf, self.change_amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn journal_is_packed_104_bytes() {
        assert_eq!(JOURNAL_BYTE_LEN, 104);
        let mut bytes = [0u8; JOURNAL_BYTE_LEN];
        bytes[0..8].copy_from_slice(&100u64.to_le_bytes());
        let public = JournalPublic::from_journal_bytes(&bytes).expect("packed");
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.change_amount, 0);
        assert!(public.change_note().is_none());
        assert!(JournalPublic::from_journal_bytes(&bytes[..103]).is_none());
        assert!(JournalPublic::from_journal_bytes(&[0u8; 105]).is_none());
    }

    #[test]
    fn journal_parses_144_byte_change_note() {
        let mut bytes = vec![0u8; JOURNAL_BYTE_LEN_WITH_CHANGE];
        bytes[0..8].copy_from_slice(&100u64.to_le_bytes());
        bytes[8..40].copy_from_slice(&[3u8; 32]);
        bytes[104..112].copy_from_slice(&900u64.to_le_bytes());
        let leaf = [9u8; 32];
        bytes[112..144].copy_from_slice(&leaf);
        let public = JournalPublic::from_journal_bytes(&bytes).expect("144");
        assert_eq!(public.change_amount, 900);
        assert_eq!(public.change_note(), Some((leaf, 900)));
        let mut zero_leaf = bytes.clone();
        zero_leaf[112..144].fill(0);
        assert!(JournalPublic::from_journal_bytes(&zero_leaf).is_none());
    }

    #[test]
    fn happy_journal_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../zk-circuit/fixtures/happy/journal.bin");
        let bytes = std::fs::read(path).expect("journal.bin");
        let public = JournalPublic::from_journal_bytes(&bytes).expect("layout");
        assert_eq!(bytes.len(), JOURNAL_BYTE_LEN_WITH_CHANGE);
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.asset_id_mint, [3u8; 32]);
        assert_ne!(public.nullifier, [0u8; 32]);
        assert_ne!(public.merkle_root, [0u8; 32]);
        assert_eq!(public.change_amount, 900);
        assert_eq!(public.change_note(), Some((public.change_leaf, 900)));
        assert_ne!(public.change_leaf, [0u8; 32]);
    }
}
