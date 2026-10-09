//! SP1 **v6.1.0** Groth16 verify on Solana (`alt_bn128` via `groth16-solana`).
//!
//! crates.io `sp1-solana` 0.1.0 only ships VKs through SP1 v5 and a 2-input
//! `verify_proof`. Our Day 12 fixture is v6 (`proof.bytes()` = 356 B envelope).
//! This module mirrors the v6 path used by Succinct's Ethereum verifier / the
//! community `verify_proof_v6` patch: 5 public inputs + Solana BN254 precompiles.

use ark_bn254::{Fq, G1Affine};
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;
use groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};
use sha2::{Digest, Sha256};

/// Groth16 VK for SP1 circuits v6.1.0 (`~/.sp1/circuits/groth16/v6.1.0/`).
pub const GROTH16_VK_V6_1_0_BYTES: &[u8] =
    include_bytes!("../vk/groth16_vk_v6_1_0.bin");

/// SP1 v6.1.0 recursion vk merkle root (must match proof envelope bytes 36..68).
pub const SP1_V6_1_0_VK_ROOT: [u8; 32] = [
    0x00, 0x2f, 0x85, 0x0e, 0xe9, 0x98, 0x97, 0x4d, 0x6c, 0xc0, 0x0e, 0x50, 0xcd, 0x08, 0x14,
    0xb0, 0x98, 0xc0, 0x5b, 0xfa, 0xde, 0x46, 0x6d, 0x28, 0x57, 0x32, 0x40, 0xd0, 0x57, 0xf2,
    0x53, 0x52,
];

/// `[4 selector | 32 exit | 32 vkRoot | 32 nonce | 256 groth16]`
pub const SP1_V6_PROOF_BYTE_LEN: usize = 4 + 32 + 32 + 32 + 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    InvalidProofLen,
    Groth16VkeyHashMismatch,
    NonZeroExitCode,
    VkRootMismatch,
    ProofLoad,
    VerificationFailed,
}

struct Proof {
    pi_a: [u8; 64],
    pi_b: [u8; 128],
    pi_c: [u8; 64],
}

struct VerificationKey {
    vk_alpha_g1: [u8; 64],
    vk_beta_g2: [u8; 128],
    vk_gamma_g2: [u8; 128],
    vk_delta_g2: [u8; 128],
    vk_ic: Vec<[u8; 64]>,
    nr_pubinputs: u32,
}

/// Verify an SP1 v6 Groth16 proof (`proof.bytes()`) against `sp1_vkey_hash`
/// (program `vk.bytes32()` / `GlobalConfig.vkey_hash`) and the journal bytes.
pub fn verify_sp1_groth16_v6(
    proof: &[u8],
    journal: &[u8],
    sp1_vkey_hash: &[u8; 32],
) -> Result<(), VerifyError> {
    if proof.len() != SP1_V6_PROOF_BYTE_LEN {
        return Err(VerifyError::InvalidProofLen);
    }

    let groth16_vk = GROTH16_VK_V6_1_0_BYTES;
    let groth16_vk_hash: [u8; 4] = Sha256::digest(groth16_vk)[..4]
        .try_into()
        .map_err(|_| VerifyError::ProofLoad)?;
    if groth16_vk_hash != proof[..4] {
        return Err(VerifyError::Groth16VkeyHashMismatch);
    }

    let exit_code: [u8; 32] = proof[4..36]
        .try_into()
        .map_err(|_| VerifyError::ProofLoad)?;
    let vk_root: [u8; 32] = proof[36..68]
        .try_into()
        .map_err(|_| VerifyError::ProofLoad)?;
    let nonce: [u8; 32] = proof[68..100]
        .try_into()
        .map_err(|_| VerifyError::ProofLoad)?;

    if exit_code != [0u8; 32] {
        return Err(VerifyError::NonZeroExitCode);
    }
    if vk_root != SP1_V6_1_0_VK_ROOT {
        return Err(VerifyError::VkRootMismatch);
    }

    let mut padded_vkey = [0u8; 32];
    padded_vkey[1..].copy_from_slice(&sp1_vkey_hash[1..]);
    let committed = hash_public_inputs(journal);
    let public_inputs = [
        padded_vkey,
        committed,
        exit_code,
        vk_root,
        nonce,
    ];

    verify_raw_v6(&proof[100..], &public_inputs, groth16_vk)
}

fn hash_public_inputs(public_inputs: &[u8]) -> [u8; 32] {
    let mut result = Sha256::digest(public_inputs);
    // BN254 field: clear top 3 bits (same as SP1 Ethereum / sp1-solana).
    result[0] &= 0x1F;
    result.into()
}

fn verify_raw_v6(
    proof_bytes: &[u8],
    public_inputs: &[[u8; 32]; 5],
    groth16_vk: &[u8],
) -> Result<(), VerifyError> {
    let proof = load_proof_from_bytes(proof_bytes)?;
    let vk = load_groth16_verifying_key_from_bytes(groth16_vk)?;

    let vk = Groth16Verifyingkey {
        nr_pubinputs: vk.nr_pubinputs as usize,
        vk_alpha_g1: vk.vk_alpha_g1,
        vk_beta_g2: vk.vk_beta_g2,
        vk_gamme_g2: vk.vk_gamma_g2,
        vk_delta_g2: vk.vk_delta_g2,
        vk_ic: vk.vk_ic.as_slice(),
    };

    let mut verifier = Groth16Verifier::new(
        &proof.pi_a,
        &proof.pi_b,
        &proof.pi_c,
        public_inputs,
        &vk,
    )
    .map_err(|_| VerifyError::VerificationFailed)?;

    verifier
        .verify()
        .map_err(|_| VerifyError::VerificationFailed)
}

fn convert_endianness<const CHUNK_SIZE: usize, const ARRAY_SIZE: usize>(
    bytes: &[u8; ARRAY_SIZE],
) -> [u8; ARRAY_SIZE] {
    bytes
        .chunks_exact(CHUNK_SIZE)
        .flat_map(|chunk| chunk.iter().rev().copied())
        .enumerate()
        .fold([0u8; ARRAY_SIZE], |mut acc, (i, v)| {
            acc[i] = v;
            acc
        })
}

const GNARK_MASK: u8 = 0b11 << 6;
const GNARK_COMPRESSED_POSITIVE: u8 = 0b10 << 6;
const GNARK_COMPRESSED_NEGATIVE: u8 = 0b11 << 6;
const GNARK_COMPRESSED_INFINITY: u8 = 0b01 << 6;
const ARK_MASK: u8 = 0b11 << 6;
const ARK_COMPRESSED_POSITIVE: u8 = 0b00 << 6;
const ARK_COMPRESSED_NEGATIVE: u8 = 0b10 << 6;
const ARK_COMPRESSED_INFINITY: u8 = 0b01 << 6;

fn gnark_flag_to_ark_flag(msb: u8) -> Result<u8, VerifyError> {
    let ark_flag = match msb & GNARK_MASK {
        GNARK_COMPRESSED_POSITIVE => ARK_COMPRESSED_POSITIVE,
        GNARK_COMPRESSED_NEGATIVE => ARK_COMPRESSED_NEGATIVE,
        GNARK_COMPRESSED_INFINITY => ARK_COMPRESSED_INFINITY,
        _ => return Err(VerifyError::ProofLoad),
    };
    Ok(msb & !ARK_MASK | ark_flag)
}

fn gnark_compressed_x_to_ark_compressed_x(x: &[u8]) -> Result<Vec<u8>, VerifyError> {
    if x.len() != 32 && x.len() != 64 {
        return Err(VerifyError::ProofLoad);
    }
    let mut x_copy = x.to_owned();
    x_copy[0] = gnark_flag_to_ark_flag(x_copy[0])?;
    x_copy.reverse();
    Ok(x_copy)
}

fn decompress_g1(g1_bytes: &[u8; 32]) -> Result<[u8; 64], VerifyError> {
    let g1_bytes = gnark_compressed_x_to_ark_compressed_x(g1_bytes)?;
    let g1_bytes = convert_endianness::<32, 32>(&g1_bytes.as_slice().try_into().unwrap());
    groth16_solana::decompression::decompress_g1(&g1_bytes).map_err(|_| VerifyError::ProofLoad)
}

fn decompress_g2(g2_bytes: &[u8; 64]) -> Result<[u8; 128], VerifyError> {
    let g2_bytes = gnark_compressed_x_to_ark_compressed_x(g2_bytes)?;
    let g2_bytes = convert_endianness::<64, 64>(&g2_bytes.as_slice().try_into().unwrap());
    groth16_solana::decompression::decompress_g2(&g2_bytes).map_err(|_| VerifyError::ProofLoad)
}

fn uncompressed_bytes_to_g1_point(buf: &[u8]) -> Result<G1Affine, VerifyError> {
    if buf.len() != 64 {
        return Err(VerifyError::ProofLoad);
    }
    let (x_bytes, y_bytes) = buf.split_at(32);
    let x = Fq::from_be_bytes_mod_order(x_bytes);
    let y = Fq::from_be_bytes_mod_order(y_bytes);
    Ok(G1Affine::new_unchecked(x, y))
}

fn negate_g1(g1_bytes: &[u8; 64]) -> Result<[u8; 64], VerifyError> {
    let g1 = -uncompressed_bytes_to_g1_point(g1_bytes)?;
    let mut out = [0u8; 64];
    g1.serialize_uncompressed(&mut out[..])
        .map_err(|_| VerifyError::ProofLoad)?;
    Ok(convert_endianness::<32, 64>(
        &out.as_slice().try_into().unwrap(),
    ))
}

fn load_proof_from_bytes(buffer: &[u8]) -> Result<Proof, VerifyError> {
    if buffer.len() != 256 {
        return Err(VerifyError::ProofLoad);
    }
    Ok(Proof {
        pi_a: negate_g1(buffer[..64].try_into().map_err(|_| VerifyError::ProofLoad)?)?,
        pi_b: buffer[64..192]
            .try_into()
            .map_err(|_| VerifyError::ProofLoad)?,
        pi_c: buffer[192..256]
            .try_into()
            .map_err(|_| VerifyError::ProofLoad)?,
    })
}

fn load_groth16_verifying_key_from_bytes(buffer: &[u8]) -> Result<VerificationKey, VerifyError> {
    let g1_alpha = decompress_g1(buffer[..32].try_into().unwrap())?;
    let g2_beta = decompress_g2(buffer[64..128].try_into().unwrap())?;
    let g2_gamma = decompress_g2(buffer[128..192].try_into().unwrap())?;
    let g2_delta = decompress_g2(buffer[224..288].try_into().unwrap())?;

    let num_k = u32::from_be_bytes([buffer[288], buffer[289], buffer[290], buffer[291]]);
    let mut k = Vec::new();
    let mut offset = 292;
    for _ in 0..num_k {
        let point = decompress_g1(&buffer[offset..offset + 32].try_into().unwrap())?;
        k.push(point);
        offset += 32;
    }

    let num_of_array_of_public_and_commitment_committed = u32::from_be_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ]);
    offset += 4;
    for _ in 0..num_of_array_of_public_and_commitment_committed {
        let num = u32::from_be_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        offset += 4;
        for _ in 0..num {
            offset += 4;
        }
    }

    Ok(VerificationKey {
        vk_alpha_g1: g1_alpha,
        vk_beta_g2: g2_beta,
        vk_gamma_g2: g2_gamma,
        vk_delta_g2: g2_delta,
        vk_ic: k,
        nr_pubinputs: num_of_array_of_public_and_commitment_committed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn happy_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zk-circuit/fixtures/happy")
    }

    #[test]
    fn happy_fixture_verifies_against_vkey_hash() {
        let proof = std::fs::read(happy_dir().join("groth16.bin")).expect("groth16.bin");
        let journal = std::fs::read(happy_dir().join("journal.bin")).expect("journal.bin");
        assert_eq!(proof.len(), SP1_V6_PROOF_BYTE_LEN);
        verify_sp1_groth16_v6(&proof, &journal, &crate::constants::VKEY_HASH)
            .expect("Day 12 fixture must verify on SP1 v6.1.0 path");
    }

    #[test]
    fn garbage_proof_fails() {
        let journal = std::fs::read(happy_dir().join("journal.bin")).expect("journal.bin");
        let mut proof = std::fs::read(happy_dir().join("groth16.bin")).expect("groth16.bin");
        // Flip a byte inside the Groth16 body (after the 100-byte envelope).
        proof[120] ^= 0xff;
        assert!(verify_sp1_groth16_v6(&proof, &journal, &crate::constants::VKEY_HASH).is_err());
    }

    #[test]
    fn wrong_vkey_hash_fails() {
        let proof = std::fs::read(happy_dir().join("groth16.bin")).expect("groth16.bin");
        let journal = std::fs::read(happy_dir().join("journal.bin")).expect("journal.bin");
        let bad = [0u8; 32];
        assert!(verify_sp1_groth16_v6(&proof, &journal, &bad).is_err());
    }
}
