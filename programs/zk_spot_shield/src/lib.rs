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

declare_id!("H6X5T8TP6T8hyTySiDVAug8bfULbKGMhCTV5PA9VMHPv");

#[program]
pub mod zk_spot_shield {
    use super::*;

    /// Create `GlobalConfig` PDA: authority, frozen `VKEY_HASH`, pause=false.
    pub fn initialize_global_config(ctx: Context<InitializeGlobalConfig>) -> Result<()> {
        crate::instructions::initialize_global_config::handle_initialize_global_config(ctx, VKEY_HASH)
    }

    /// Create zero-copy escrow `VaultState` PDA (mints/reserves; token ATAs wired later).
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault::handle_initialize_vault(ctx)
    }

    /// Authority registers a `CleanFundsRoot` PDA for an approved Merkle root.
    pub fn register_clean_funds_root(
        ctx: Context<RegisterCleanFundsRoot>,
        root: [u8; 32],
    ) -> Result<()> {
        instructions::register_clean_funds_root::handle_register_clean_funds_root(ctx, root)
    }

    /// Settle: verify + root + vault reserves + SPL + nullifier (respects pause).
    pub fn settle_shielded_spot(
        ctx: Context<SettleShieldedSpot>,
        proof: Vec<u8>,
        journal: Vec<u8>,
    ) -> Result<()> {
        instructions::settle_shielded_spot::handle_settle_shielded_spot(ctx, proof, journal)
    }

    /// Authority sets `GlobalConfig.pause_flag = true` (Day 28).
    pub fn pause(ctx: Context<SetPause>) -> Result<()> {
        instructions::set_pause::handle_pause(ctx)
    }

    /// Authority clears `GlobalConfig.pause_flag` (Day 28).
    pub fn unpause(ctx: Context<SetPause>) -> Result<()> {
        instructions::set_pause::handle_unpause(ctx)
    }
}
