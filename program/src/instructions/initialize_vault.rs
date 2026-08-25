use crate::state::VaultState;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        zero, // Allocates account space & sets owner to program without copy overhead
        signer,
    )]
    pub vault: AccountLoader<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_vault(
    ctx: Context<InitializeVault>,
    bump: u8,
) -> Result<()> {
    // Use load_init() when setting up a zero account for the first time
    let mut vault = ctx.accounts.vault.load_init()?;
    
    vault.authority = ctx.accounts.payer.key();
    vault.mint_a = Pubkey::default();
    vault.mint_b = Pubkey::default();
    vault.reserve_a = 0;
    vault.reserve_b = 0;
    vault.bump = bump;
    vault._padding = [0u8; 7];

    Ok(())
}