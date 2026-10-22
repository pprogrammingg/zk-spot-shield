//! Protocol admin writes `mint_a` / `mint_b` once on an existing vault PDA.

use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode,
    state::{GlobalConfig, VaultState},
    GLOBAL_CONFIG_SEED, SPOT_VAULT_SEED,
};

#[derive(Accounts)]
pub struct SetVaultMints<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
        has_one = authority @ ErrorCode::Unauthorized,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [SPOT_VAULT_SEED],
        bump = vault.load()?.bump,
    )]
    pub vault: AccountLoader<'info, VaultState>,
}

pub fn handle_set_vault_mints(
    ctx: Context<SetVaultMints>,
    mint_a: Pubkey,
    mint_b: Pubkey,
) -> Result<()> {
    let mut vault = ctx.accounts.vault.load_mut()?;
    vault.set_mints(&mint_a, &mint_b)
}
