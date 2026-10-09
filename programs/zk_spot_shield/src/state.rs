use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode, CLEAN_FUNDS_ROOT_SEED_PREFIX, NULLIFIER_SEED_PREFIX,
};


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
    assert!(size_of::<VaultState>().is_multiple_of(8));
};

impl VaultState {
    pub fn find_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[crate::SPOT_VAULT_SEED],
            program_id,
        )
    }

    /// Day 26: 1:1 spot accounting on zero-copy reserves.
    /// Vault takes `asset_mint` (`checked_add`) and pays the other mint (`checked_sub`).
    pub fn apply_swap(&mut self, amount: u64, asset_mint: &Pubkey) -> Result<()> {
        require!(amount > 0, ErrorCode::ZeroSwapAmount);

        if asset_mint == &self.mint_a {
            self.reserve_a = self
                .reserve_a
                .checked_add(amount)
                .ok_or(error!(ErrorCode::VaultReserveOverflow))?;
            self.reserve_b = self
                .reserve_b
                .checked_sub(amount)
                .ok_or(error!(ErrorCode::VaultReserveUnderflow))?;
        } else if asset_mint == &self.mint_b {
            self.reserve_b = self
                .reserve_b
                .checked_add(amount)
                .ok_or(error!(ErrorCode::VaultReserveOverflow))?;
            self.reserve_a = self
                .reserve_a
                .checked_sub(amount)
                .ok_or(error!(ErrorCode::VaultReserveUnderflow))?;
        } else {
            return err!(ErrorCode::UnknownVaultMint);
        }
        Ok(())
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
/// ["clean_funds_root", root]
///
/// One account per approved Merkle root. Existence means the root is allow-listed
/// for settle (journal `merkle_root` must match this PDA).
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
    fn vault_state_is_120_bytes_and_aligned() {
        assert_eq!(std::mem::size_of::<VaultState>(), 120);
        assert!(std::mem::size_of::<VaultState>().is_multiple_of(8));
    }

    fn sample_vault(reserve_a: u64, reserve_b: u64) -> VaultState {
        let mint_a = Pubkey::new_from_array([3u8; 32]);
        let mint_b = Pubkey::new_from_array([4u8; 32]);
        VaultState {
            authority: Pubkey::new_unique(),
            mint_a,
            mint_b,
            reserve_a,
            reserve_b,
            bump: 255,
            _padding: [0u8; 7],
        }
    }

    #[test]
    fn apply_swap_moves_reserves_one_to_one() {
        let mut vault = sample_vault(1_000, 1_000);
        let mint_a = vault.mint_a;
        vault.apply_swap(100, &mint_a).unwrap();
        assert_eq!(vault.reserve_a, 1_100);
        assert_eq!(vault.reserve_b, 900);
    }

    #[test]
    fn apply_swap_rejects_underflow() {
        let mut vault = sample_vault(1_000, 50);
        let mint_a = vault.mint_a;
        let err = vault.apply_swap(100, &mint_a).unwrap_err();
        assert_eq!(err, error!(ErrorCode::VaultReserveUnderflow));
    }

    #[test]
    fn apply_swap_rejects_overflow() {
        let mut vault = sample_vault(u64::MAX, 1_000);
        let mint_a = vault.mint_a;
        let err = vault.apply_swap(1, &mint_a).unwrap_err();
        assert_eq!(err, error!(ErrorCode::VaultReserveOverflow));
    }

    #[test]
    fn apply_swap_rejects_unknown_mint() {
        let mut vault = sample_vault(1_000, 1_000);
        let err = vault
            .apply_swap(1, &Pubkey::new_from_array([9u8; 32]))
            .unwrap_err();
        assert_eq!(err, error!(ErrorCode::UnknownVaultMint));
    }

    #[test]
    fn global_config_init_space_is_65() {
        // authority (32) + vkey_hash (32) + pause_flag (1); rent space = 8 + INIT_SPACE.
        assert_eq!(GlobalConfig::INIT_SPACE, 65);
    }

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