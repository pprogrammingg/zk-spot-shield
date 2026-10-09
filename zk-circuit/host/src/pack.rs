//! Settle instruction payload packing (Day 14).
//!
//! **Settle payload contract** — Anchor ix data is one length-prefixed blob:
//! ```text
//! [ u32 le proof_len ][ proof ][ u32 le journal_len ][ journal ]
//! ```
//! Journal bytes are the SP1 committed layout decoded by
//! [`zk_circuit_io::PublicOutputs::from_journal_bytes`] (same as `io::commit`).

use std::fmt;

/// Failed to decode a settle payload blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    Truncated { need: usize, got: usize },
    ProofLenMismatch { declared: usize, remaining: usize },
    JournalLenMismatch { declared: usize, remaining: usize },
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { need, got } => {
                write!(f, "settle payload truncated: need {need} bytes, got {got}")
            }
            Self::ProofLenMismatch {
                declared,
                remaining,
            } => write!(
                f,
                "proof_len {declared} exceeds remaining {remaining} bytes"
            ),
            Self::JournalLenMismatch {
                declared,
                remaining,
            } => write!(
                f,
                "journal_len {declared} exceeds remaining {remaining} bytes (or trailing junk)"
            ),
        }
    }
}

impl std::error::Error for PackError {}

/// Pack Groth16 proof + journal for `settle_shielded_spot` instruction data.
pub fn pack_settle_payload(proof: &[u8], journal: &[u8]) -> Vec<u8> {
    let proof_len = u32::try_from(proof.len()).expect("proof len fits u32");
    let journal_len = u32::try_from(journal.len()).expect("journal len fits u32");
    let mut out = Vec::with_capacity(8 + proof.len() + journal.len());
    out.extend_from_slice(&proof_len.to_le_bytes());
    out.extend_from_slice(proof);
    out.extend_from_slice(&journal_len.to_le_bytes());
    out.extend_from_slice(journal);
    out
}

/// Unpack settle ix data into `(proof, journal)`. Used to round-trip tests and
/// as the decode contract settle will mirror on-chain.
pub fn unpack_settle_payload(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), PackError> {
    if bytes.len() < 4 {
        return Err(PackError::Truncated {
            need: 4,
            got: bytes.len(),
        });
    }
    let proof_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let after_proof_len: usize = 4;
    let proof_end = after_proof_len
        .checked_add(proof_len)
        .ok_or(PackError::ProofLenMismatch {
            declared: proof_len,
            remaining: bytes.len().saturating_sub(after_proof_len),
        })?;
    if proof_end > bytes.len() {
        return Err(PackError::ProofLenMismatch {
            declared: proof_len,
            remaining: bytes.len() - after_proof_len,
        });
    }
    let proof = bytes[after_proof_len..proof_end].to_vec();

    let journal_len_start = proof_end;
    if bytes.len() < journal_len_start + 4 {
        return Err(PackError::Truncated {
            need: journal_len_start + 4,
            got: bytes.len(),
        });
    }
    let journal_len =
        u32::from_le_bytes(bytes[journal_len_start..journal_len_start + 4].try_into().unwrap())
            as usize;
    let journal_start = journal_len_start + 4;
    let journal_end = journal_start
        .checked_add(journal_len)
        .ok_or(PackError::JournalLenMismatch {
            declared: journal_len,
            remaining: bytes.len().saturating_sub(journal_start),
        })?;
    if journal_end != bytes.len() {
        return Err(PackError::JournalLenMismatch {
            declared: journal_len,
            remaining: bytes.len().saturating_sub(journal_start),
        });
    }
    let journal = bytes[journal_start..journal_end].to_vec();
    Ok((proof, journal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{happy_fixture_dir, happy_path_inputs};
    use std::fs;
    use zk_circuit_io::PublicOutputs;

    #[test]
    fn pack_unpack_round_trip_random_bytes() {
        let proof: Vec<u8> = (0u8..200).collect();
        let journal: Vec<u8> = (200u8..255).chain(0u8..49).collect();
        assert_eq!(journal.len(), PublicOutputs::JOURNAL_BYTE_LEN);
        let packed = pack_settle_payload(&proof, &journal);
        let (p2, j2) = unpack_settle_payload(&packed).expect("unpack");
        assert_eq!(p2, proof);
        assert_eq!(j2, journal);
    }

    #[test]
    fn pack_unpack_empty_proof_and_journal() {
        let packed = pack_settle_payload(&[], &[]);
        let (p, j) = unpack_settle_payload(&packed).expect("unpack");
        assert!(p.is_empty());
        assert!(j.is_empty());
    }

    #[test]
    fn unpack_rejects_truncated_and_trailing_junk() {
        assert!(matches!(
            unpack_settle_payload(&[1, 0, 0]),
            Err(PackError::Truncated { .. })
        ));
        let mut packed = pack_settle_payload(&[9, 9], &[1, 2, 3]);
        packed.push(0xff);
        assert!(matches!(
            unpack_settle_payload(&packed),
            Err(PackError::JournalLenMismatch { .. })
        ));
    }

    #[test]
    fn happy_fixture_unpack_reads_public_outputs() {
        let inputs = happy_path_inputs();
        let dir = happy_fixture_dir();
        let proof = fs::read(dir.join("groth16.bin")).expect("commit groth16.bin");
        let journal = fs::read(dir.join("journal.bin")).expect("commit journal.bin");
        let packed = pack_settle_payload(&proof, &journal);
        let (p2, j2) = unpack_settle_payload(&packed).expect("unpack fixture blob");
        assert_eq!(p2, proof);
        assert_eq!(p2.len(), 356);
        let public = PublicOutputs::from_journal_bytes(&j2).expect("journal layout");
        assert_eq!(public.requested_swap_amount, inputs.requested_swap_amount);
        assert_eq!(public.asset_id_mint, inputs.asset_id_mint);
        assert_eq!(public.merkle_root, inputs.expected_root);
        assert_ne!(public.nullifier, [0u8; 32]);
    }
}
