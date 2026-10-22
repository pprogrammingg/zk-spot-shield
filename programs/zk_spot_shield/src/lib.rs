pub mod constants;
pub mod error;
pub mod instructions;
pub mod journal;
pub mod state;
pub mod verify_sp1;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("74UW993wrDasAMyBQ53PjHpYLK5vMkWbRRWFZ7kCmM3G");

#[program]
pub mod zk_spot_shield {
    use super::*;

    /// Create `GlobalConfig` PDA: authority, initial `VKEY_HASH`, pause=false.
    pub fn initialize_global_config(ctx: Context<InitializeGlobalConfig>) -> Result<()> {
        crate::instructions::initialize_global_config::handle_initialize_global_config(
            ctx, VKEY_HASH,
        )
    }

    /// Protocol admin rotates `vkey_hash` after a guest ELF rebuild.
    pub fn set_vkey_hash(ctx: Context<SetVkeyHash>, vkey_hash: [u8; 32]) -> Result<()> {
        instructions::set_vkey_hash::handle_set_vkey_hash(ctx, vkey_hash)
    }

    /// Create zero-copy escrow `VaultState` PDA (mints unset; token ATAs wired later).
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault::handle_initialize_vault(ctx)
    }

    /// Protocol admin writes `mint_a` / `mint_b` once. Fails if already set.
    pub fn set_vault_mints(
        ctx: Context<SetVaultMints>,
        mint_a: Pubkey,
        mint_b: Pubkey,
    ) -> Result<()> {
        instructions::set_vault_mints::handle_set_vault_mints(ctx, mint_a, mint_b)
    }

    /// Authority registers a `CleanFundsRoot` PDA for an approved Merkle root.
    pub fn register_clean_funds_root(
        ctx: Context<RegisterCleanFundsRoot>,
        root: [u8; 32],
    ) -> Result<()> {
        instructions::register_clean_funds_root::handle_register_clean_funds_root(ctx, root)
    }

    /// Transfer SPL into vault and record a public leaf commitment PDA.
    pub fn deposit_shielded(
        ctx: Context<DepositShielded>,
        leaf: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        instructions::deposit_shielded::handle_deposit_shielded(ctx, leaf, amount)
    }

    /// Settle: verify + root + vault reserves + SPL + nullifier (respects pause).
    pub fn settle_shielded_spot(
        ctx: Context<SettleShieldedSpot>,
        proof: Vec<u8>,
        journal: Vec<u8>,
    ) -> Result<()> {
        instructions::settle_shielded_spot::handle_settle_shielded_spot(ctx, proof, journal)
    }

    /// Authority sets `GlobalConfig.pause_flag = true`.
    pub fn pause(ctx: Context<SetPause>) -> Result<()> {
        instructions::set_pause::handle_pause(ctx)
    }

    /// Authority clears `GlobalConfig.pause_flag`.
    pub fn unpause(ctx: Context<SetPause>) -> Result<()> {
        instructions::set_pause::handle_unpause(ctx)
    }
}
