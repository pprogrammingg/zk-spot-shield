use anchor_lang::prelude::*;

use crate::{CLEAN_FUNDS_ROOT_SEED_PREFIX, NULLIFIER_SEED_PREFIX};


#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub vkey_hash: [u8; 32],
    pub pause_flag: bool,
}

#[account(zero_copy)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VaultState {
    pub authority: Pubkey,       // 32 bytes
    pub mint_a: Pubkey,          // 32 bytes
    pub mint_b: Pubkey,          // 32 bytes
    pub reserve_a: u64,          // 8 bytes
    pub reserve_b: u64,          // 8 bytes
    pub bump: u8,                // 1 byte
    pub _padding: [u8; 7],       // 7 bytes (explicit padding to align to 8-byte boundary)
}

// Static alignment and size checks compile-time
const _: () = {
    // Total size: 32 + 32 + 32 + 8 + 8 + 1 + 7 = 120 bytes
    assert!(size_of::<VaultState>() == 120);
    assert!(size_of::<VaultState>() % 8 == 0);
};

impl VaultState {
    pub fn find_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[crate::SPOT_VAULT_SEED],
            program_id,
        )
    }
}

/// PDA seeds:
/// ["nullifier", nullifier]
///
/// A unique account is created for each nullifier.
/// Account existence is later used to prevent replay.
#[account(zero_copy)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct NullifierAccount {
    /// The 32-byte nullifier hash.
    pub nullifier: [u8; 32],
}

const _: () = {
    assert!(size_of::<NullifierAccount>() == 32);
};

impl NullifierAccount {
    pub fn find_pda(
        program_id: &Pubkey,
        nullifier: &[u8; 32],
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[NULLIFIER_SEED_PREFIX, nullifier],
            program_id,
        )
    }
}

/// PDA seeds:
/// ["clean_funds_root"]
///
/// A single account is used to store the approved Merkle root.
/// This root is used to verify that a transaction is valid.
#[account(zero_copy)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct CleanFundsRoot {
    /// An approved 32-byte Merkle root.
    pub root: [u8; 32],
}

const _: () = {
    assert!(size_of::<CleanFundsRoot>() == 32);
};

impl CleanFundsRoot {
    pub fn find_pda(
        program_id: &Pubkey,
        root: &[u8; 32],
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[CLEAN_FUNDS_ROOT_SEED_PREFIX, root],
            program_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nullifier_account_size_is_32_bytes() {
        assert_eq!(std::mem::size_of::<NullifierAccount>(), 32);
    }

    #[test]
    fn clean_funds_root_size_is_32_bytes() {
        assert_eq!(std::mem::size_of::<CleanFundsRoot>(), 32);
    }

    #[test]
    fn same_nullifier_produces_same_pda() {
        let program_id = Pubkey::new_unique();
        let nullifier = [1u8; 32];

        let (pda1, bump1) =
            NullifierAccount::find_pda(&program_id, &nullifier);

        let (pda2, bump2) =
            NullifierAccount::find_pda(&program_id, &nullifier);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn different_nullifiers_produce_different_pdas() {
        let program_id = Pubkey::new_unique();

        let nullifier_a = [1u8; 32];
        let nullifier_b = [2u8; 32];

        let (pda_a, _) =
            NullifierAccount::find_pda(&program_id, &nullifier_a);

        let (pda_b, _) =
            NullifierAccount::find_pda(&program_id, &nullifier_b);

        assert_ne!(pda_a, pda_b);
    }

    #[test]
    fn same_root_produces_same_pda() {
        let program_id = Pubkey::new_unique();
        let root = [1u8; 32];

        let (pda1, bump1) =
            CleanFundsRoot::find_pda(&program_id, &root);

        let (pda2, bump2) =
            CleanFundsRoot::find_pda(&program_id, &root);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn different_roots_produce_different_pdas() {
        let program_id = Pubkey::new_unique();

        let root_a = [1u8; 32];
        let root_b = [2u8; 32];

        let (pda_a, _) =
            CleanFundsRoot::find_pda(&program_id, &root_a);

        let (pda_b, _) =
            CleanFundsRoot::find_pda(&program_id, &root_b);

        assert_ne!(pda_a, pda_b);
    }
}