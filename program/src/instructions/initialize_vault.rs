use crate::{state::VaultState, SPOT_VAULT_SEED};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<VaultState>(),
        seeds = [SPOT_VAULT_SEED],
        bump,
    )]
    pub vault: AccountLoader<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_vault(
    ctx: Context<InitializeVault>,
) -> Result<()> {
    // Use load_init() when setting up a zero account for the first time
    let mut vault = ctx.accounts.vault.load_init()?;
    
    vault.authority = ctx.accounts.payer.key();
    vault.mint_a = Pubkey::default();
    vault.mint_b = Pubkey::default();
    vault.reserve_a = 0;
    vault.reserve_b = 0;
    vault.bump = ctx.bumps.vault;
    vault._padding = [0u8; 7];

    Ok(())
}