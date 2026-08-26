pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("H6X5T8TP6T8hyTySiDVAug8bfULbKGMhCTV5PA9VMHPv");

#[program]
pub mod zk_spot_shield {
    use super::*;

    pub fn initialize_global_config(ctx: Context<InitializeGlobalConfig>) -> Result<()> {
        crate::instructions::initialize_global_config::handle_initialize_global_config(ctx, VKEY_HASH)
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault::handle_initialize_vault(ctx)
    }
}
