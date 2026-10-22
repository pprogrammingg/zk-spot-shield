use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode, CLEAN_FUNDS_ROOT_SEED_PREFIX, DEPOSIT_LEAF_SEED_PREFIX, NULLIFIER_SEED_PREFIX,
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

impl GlobalConfig {
    /// Protocol admin rotates the circuit fingerprint after a guest rebuild.
    pub fn rotate_vkey_hash(&mut self, vkey_hash: [u8; 32]) -> Result<()> {
        require!(vkey_hash != [0u8; 32], ErrorCode::InvalidVkeyHash);
        self.vkey_hash = vkey_hash;
        Ok(())
    }
}

#[account(zero_copy)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct VaultState {
    pub authority: Pubkey, // 32 bytes
    pub mint_a: Pubkey,    // 32 bytes
    pub mint_b: Pubkey,    // 32 bytes
    pub reserve_a: u64,    // 8 bytes
    pub reserve_b: u64,    // 8 bytes
    pub bump: u8,          // 1 byte
    pub _padding: [u8; 7], // 7 bytes (explicit padding to align to 8-byte boundary)
}

// Static alignment and size checks compile-time
const _: () = {
    // Total size: 32 + 32 + 32 + 8 + 8 + 1 + 7 = 120 bytes
    assert!(size_of::<VaultState>() == 120);
    assert!(size_of::<VaultState>().is_multiple_of(8));
};

impl VaultState {
    pub fn find_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[crate::SPOT_VAULT_SEED], program_id)
    }

    /// Legacy wallet-leg swap: vault takes `asset_mint` and pays the other.
    /// Prefer [`Self::apply_vault_only_spend`] after deposit-backed custody.
    pub fn apply_swap(&mut self, amount: u64, asset_mint: &Pubkey) -> Result<()> {
        require!(amount > 0, ErrorCode::ZeroSwapAmount);

        if asset_mint == &self.mint_a {
            let new_a = self
                .reserve_a
                .checked_add(amount)
                .ok_or(error!(ErrorCode::VaultReserveOverflow))?;
            let new_b = self
                .reserve_b
                .checked_sub(amount)
                .ok_or(error!(ErrorCode::VaultReserveUnderflow))?;
            self.reserve_a = new_a;
            self.reserve_b = new_b;
        } else if asset_mint == &self.mint_b {
            let new_b = self
                .reserve_b
                .checked_add(amount)
                .ok_or(error!(ErrorCode::VaultReserveOverflow))?;
            let new_a = self
                .reserve_a
                .checked_sub(amount)
                .ok_or(error!(ErrorCode::VaultReserveUnderflow))?;
            self.reserve_a = new_a;
            self.reserve_b = new_b;
        } else {
            return err!(ErrorCode::UnknownVaultMint);
        }
        Ok(())
    }

    /// Deposit-backed 1:1 spend — journal mint already sits in vault reserves/ATA.
    /// Only subtract the **payout** mint; leave journal-mint reserve unchanged.
    /// Fail-closed on zero amount, unknown mint, or payout underflow.
    pub fn apply_vault_only_spend(&mut self, amount: u64, asset_mint: &Pubkey) -> Result<()> {
        require!(amount > 0, ErrorCode::ZeroSwapAmount);

        if asset_mint == &self.mint_a {
            // Spend custodied A → pay B.
            self.reserve_b = self
                .reserve_b
                .checked_sub(amount)
                .ok_or(error!(ErrorCode::VaultReserveUnderflow))?;
        } else if asset_mint == &self.mint_b {
            // Spend custodied B → pay A.
            self.reserve_a = self
                .reserve_a
                .checked_sub(amount)
                .ok_or(error!(ErrorCode::VaultReserveUnderflow))?;
        } else {
            return err!(ErrorCode::UnknownVaultMint);
        }
        Ok(())
    }

    /// Custody-side deposit — add `amount` of `mint` to the matching reserve.
    pub fn apply_deposit(&mut self, amount: u64, mint: &Pubkey) -> Result<()> {
        require!(amount > 0, ErrorCode::ZeroDepositAmount);
        if mint == &self.mint_a {
            self.reserve_a = self
                .reserve_a
                .checked_add(amount)
                .ok_or(error!(ErrorCode::VaultReserveOverflow))?;
        } else if mint == &self.mint_b {
            self.reserve_b = self
                .reserve_b
                .checked_add(amount)
                .ok_or(error!(ErrorCode::VaultReserveOverflow))?;
        } else {
            return err!(ErrorCode::InvalidDepositMint);
        }
        Ok(())
    }

    /// Write the two pair mints once. Fail closed if already set, equal, or default.
    pub fn set_mints(&mut self, mint_a: &Pubkey, mint_b: &Pubkey) -> Result<()> {
        require!(
            self.mint_a == Pubkey::default() && self.mint_b == Pubkey::default(),
            ErrorCode::VaultMintsAlreadySet
        );
        require!(
            *mint_a != Pubkey::default() && *mint_b != Pubkey::default(),
            ErrorCode::InvalidVaultMint
        );
        require!(mint_a != mint_b, ErrorCode::InvalidVaultMint);
        self.mint_a = *mint_a;
        self.mint_b = *mint_b;
        Ok(())
    }
}

/// PDA seeds: `["deposit_leaf", leaf]` — public leaf commitment after a deposit.
#[account(zero_copy)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DepositLeafAccount {
    pub leaf: [u8; 32],
    pub amount: u64,
    pub mint: Pubkey,
    pub depositor: Pubkey,
}

const _: () = {
    assert!(size_of::<DepositLeafAccount>() == 104);
    assert!(size_of::<DepositLeafAccount>().is_multiple_of(8));
};

impl DepositLeafAccount {
    pub fn find_pda(program_id: &Pubkey, leaf: &[u8; 32]) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[DEPOSIT_LEAF_SEED_PREFIX, leaf], program_id)
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
    pub fn find_pda(program_id: &Pubkey, nullifier: &[u8; 32]) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[NULLIFIER_SEED_PREFIX, nullifier], program_id)
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
    pub fn find_pda(program_id: &Pubkey, root: &[u8; 32]) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[CLEAN_FUNDS_ROOT_SEED_PREFIX, root], program_id)
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

    #[test]
    fn vault_padding_sits_after_bump() {
        assert_eq!(core::mem::offset_of!(VaultState, bump), 112);
        assert_eq!(core::mem::offset_of!(VaultState, _padding), 113);
        assert_eq!(std::mem::size_of_val(&[0u8; 7]), 7);
        assert_eq!(32 + 32 + 32 + 8 + 8 + 1 + 7, 120);
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
    fn apply_vault_only_spend_pays_other_mint_only() {
        let mut vault = sample_vault(1_000, 1_000);
        let mint_a = vault.mint_a;
        vault.apply_vault_only_spend(100, &mint_a).unwrap();
        assert_eq!(vault.reserve_a, 1_000);
        assert_eq!(vault.reserve_b, 900);
    }

    #[test]
    fn apply_vault_only_spend_rejects_underflow_and_unknown() {
        let mut vault = sample_vault(1_000, 50);
        let mint_a = vault.mint_a;
        assert_eq!(
            vault.apply_vault_only_spend(100, &mint_a).unwrap_err(),
            error!(ErrorCode::VaultReserveUnderflow)
        );
        assert_eq!(vault.reserve_b, 50);
        assert_eq!(
            vault
                .apply_vault_only_spend(1, &Pubkey::new_from_array([9u8; 32]))
                .unwrap_err(),
            error!(ErrorCode::UnknownVaultMint)
        );
    }

    #[test]
    fn apply_vault_only_spend_edge_harness() {
        // (reserve_a, reserve_b, amount, expect_ok) when spending mint_a (pay B).
        let edges: &[(u64, u64, u64, bool)] = &[
            (1_000, 1_000, 0, false),
            (1_000, 0, 1, false),
            (1_000, 1, 1, true),
            (1_000, 1, 2, false),
            (0, 500, 100, true),
            (500, 500, 500, true),
        ];
        for &(ra, rb, amount, expect_ok) in edges {
            let mut vault = sample_vault(ra, rb);
            let mint_a = vault.mint_a;
            let result = vault.apply_vault_only_spend(amount, &mint_a);
            if expect_ok {
                assert!(result.is_ok(), "ra={ra} rb={rb} amt={amount}");
                assert_eq!(vault.reserve_a, ra);
                assert_eq!(vault.reserve_b, rb - amount);
            } else {
                assert!(result.is_err(), "ra={ra} rb={rb} amt={amount}");
                assert_eq!(vault.reserve_a, ra);
                assert_eq!(vault.reserve_b, rb);
            }
        }
    }

    #[test]
    fn apply_deposit_adds_reserve() {
        let mut vault = sample_vault(100, 200);
        let mint_a = vault.mint_a;
        vault.apply_deposit(50, &mint_a).unwrap();
        assert_eq!(vault.reserve_a, 150);
        assert_eq!(vault.reserve_b, 200);
    }

    #[test]
    fn apply_deposit_rejects_zero_and_unknown_mint() {
        let mut vault = sample_vault(100, 200);
        let mint_a = vault.mint_a;
        assert_eq!(
            vault.apply_deposit(0, &mint_a).unwrap_err(),
            error!(ErrorCode::ZeroDepositAmount)
        );
        assert_eq!(
            vault
                .apply_deposit(1, &Pubkey::new_from_array([9u8; 32]))
                .unwrap_err(),
            error!(ErrorCode::InvalidDepositMint)
        );
        assert_eq!(vault.reserve_a, 100);
        assert_eq!(vault.reserve_b, 200);
    }

    fn unset_mints_vault() -> VaultState {
        VaultState {
            authority: Pubkey::new_unique(),
            mint_a: Pubkey::default(),
            mint_b: Pubkey::default(),
            reserve_a: 0,
            reserve_b: 0,
            bump: 255,
            _padding: [0u8; 7],
        }
    }

    #[test]
    fn rotate_vkey_hash_writes_and_rejects_zero() {
        let mut config = GlobalConfig {
            authority: Pubkey::new_unique(),
            vkey_hash: [1u8; 32],
            pause_flag: false,
        };
        assert_eq!(
            config.rotate_vkey_hash([0u8; 32]).unwrap_err(),
            error!(ErrorCode::InvalidVkeyHash)
        );
        let next = [2u8; 32];
        config.rotate_vkey_hash(next).unwrap();
        assert_eq!(config.vkey_hash, next);
    }

    #[test]
    fn set_mints_writes_once() {
        let mut vault = unset_mints_vault();
        let a = Pubkey::new_from_array([3u8; 32]);
        let b = Pubkey::new_from_array([4u8; 32]);
        vault.set_mints(&a, &b).unwrap();
        assert_eq!(vault.mint_a, a);
        assert_eq!(vault.mint_b, b);
        assert_eq!(
            vault.set_mints(&a, &b).unwrap_err(),
            error!(ErrorCode::VaultMintsAlreadySet)
        );
    }

    #[test]
    fn set_mints_rejects_default_and_equal() {
        let mut vault = unset_mints_vault();
        let a = Pubkey::new_from_array([3u8; 32]);
        assert_eq!(
            vault.set_mints(&Pubkey::default(), &a).unwrap_err(),
            error!(ErrorCode::InvalidVaultMint)
        );
        assert_eq!(
            vault.set_mints(&a, &a).unwrap_err(),
            error!(ErrorCode::InvalidVaultMint)
        );
        assert_eq!(vault.mint_a, Pubkey::default());
        assert_eq!(vault.mint_b, Pubkey::default());
    }

    #[test]
    fn deposit_leaf_account_size_is_104() {
        assert_eq!(std::mem::size_of::<DepositLeafAccount>(), 104);
    }

    /// Arithmetic edges — success keeps `reserve_a + reserve_b` invariant;
    /// overflow / underflow / zero / unknown mint fail closed (no panic).
    #[test]
    fn apply_swap_arithmetic_edge_harness() {
        // (reserve_a, reserve_b, amount, expect_ok) for mint_a = receive A / pay B.
        let edges: &[(u64, u64, u64, bool)] = &[
            (1_000, 1_000, 0, false),     // zero amount
            (0, 1_000, 1, true),          // receive into empty A
            (1_000, 0, 1, false),         // pay-side empty
            (1_000, 1, 1, true),          // exact pay drain
            (1_000, 1, 2, false),         // pay underflow
            (u64::MAX, 1_000, 1, false),  // receive overflow
            (u64::MAX - 10, 10, 5, true), // near max receive (sum stays ≤ u64::MAX)
            (0, 0, 1, false),             // both empty → pay underflow
            (500, 500, 500, true),        // equal drain
        ];

        for &(ra, rb, amount, expect_ok) in edges {
            let mut vault = sample_vault(ra, rb);
            let mint_a = vault.mint_a;
            let result = vault.apply_swap(amount, &mint_a);
            if expect_ok {
                assert!(result.is_ok(), "expected ok ra={ra} rb={rb} amt={amount}");
                let sum_before = ra.checked_add(rb).expect("start sum");
                assert_eq!(
                    vault.reserve_a.checked_add(vault.reserve_b),
                    Some(sum_before),
                    "1:1 sum invariant"
                );
                assert_eq!(vault.reserve_a, ra.checked_add(amount).unwrap());
                assert_eq!(vault.reserve_b, rb.checked_sub(amount).unwrap());
            } else {
                assert!(result.is_err(), "expected err ra={ra} rb={rb} amt={amount}");
                assert_eq!(vault.reserve_a, ra, "fail closed: reserve_a unchanged");
                assert_eq!(vault.reserve_b, rb, "fail closed: reserve_b unchanged");
            }

            // mint_b direction mirrors mint_a with reserves swapped.
            let mut vault_b = sample_vault(rb, ra);
            let mint_b = vault_b.mint_b;
            let result_b = vault_b.apply_swap(amount, &mint_b);
            assert_eq!(result_b.is_ok(), expect_ok);
            if result_b.is_err() {
                assert_eq!(vault_b.reserve_a, rb);
                assert_eq!(vault_b.reserve_b, ra);
            }
        }
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

        let (pda1, bump1) = NullifierAccount::find_pda(&program_id, &nullifier);

        let (pda2, bump2) = NullifierAccount::find_pda(&program_id, &nullifier);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn different_nullifiers_produce_different_pdas() {
        let program_id = Pubkey::new_unique();

        let nullifier_a = [1u8; 32];
        let nullifier_b = [2u8; 32];

        let (pda_a, _) = NullifierAccount::find_pda(&program_id, &nullifier_a);

        let (pda_b, _) = NullifierAccount::find_pda(&program_id, &nullifier_b);

        assert_ne!(pda_a, pda_b);
    }

    #[test]
    fn same_root_produces_same_pda() {
        let program_id = Pubkey::new_unique();
        let root = [1u8; 32];

        let (pda1, bump1) = CleanFundsRoot::find_pda(&program_id, &root);

        let (pda2, bump2) = CleanFundsRoot::find_pda(&program_id, &root);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn different_roots_produce_different_pdas() {
        let program_id = Pubkey::new_unique();

        let root_a = [1u8; 32];
        let root_b = [2u8; 32];

        let (pda_a, _) = CleanFundsRoot::find_pda(&program_id, &root_a);

        let (pda_b, _) = CleanFundsRoot::find_pda(&program_id, &root_b);

        assert_ne!(pda_a, pda_b);
    }
}
