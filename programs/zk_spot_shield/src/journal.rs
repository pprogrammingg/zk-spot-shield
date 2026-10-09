//! On-chain journal decode (same byte layout as `zk_circuit_io::PublicOutputs`).

/// Packed `sp1_zkvm::io::commit` layout: LE `u64` + three `[u8; 32]`.
pub const JOURNAL_BYTE_LEN: usize = 8 + 32 + 32 + 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalPublic {
    pub requested_swap_amount: u64,
    pub asset_id_mint: [u8; 32],
    pub nullifier: [u8; 32],
    pub merkle_root: [u8; 32],
}

impl JournalPublic {
    pub fn from_journal_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != JOURNAL_BYTE_LEN {
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
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn happy_journal_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../zk-circuit/fixtures/happy/journal.bin");
        let bytes = std::fs::read(path).expect("journal.bin");
        let public = JournalPublic::from_journal_bytes(&bytes).expect("layout");
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.asset_id_mint, [3u8; 32]);
        assert_ne!(public.nullifier, [0u8; 32]);
        assert_ne!(public.merkle_root, [0u8; 32]);
    }
}
