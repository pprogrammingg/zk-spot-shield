//! Authority registers an approved Merkle root PDA for settle (Day 24).

use anchor_lang::prelude::*;

use crate::{
    state::{CleanFundsRoot, GlobalConfig},
    CLEAN_FUNDS_ROOT_SEED_PREFIX, GLOBAL_CONFIG_SEED,
};

#[derive(Accounts)]
#[instruction(root: [u8; 32])]
pub struct RegisterCleanFundsRoot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
        has_one = authority,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<CleanFundsRoot>(),
        seeds = [CLEAN_FUNDS_ROOT_SEED_PREFIX, root.as_ref()],
        bump,
    )]
    pub clean_funds_root: AccountLoader<'info, CleanFundsRoot>,

    pub system_program: Program<'info, System>,
}

pub fn handle_register_clean_funds_root(
    ctx: Context<RegisterCleanFundsRoot>,
    root: [u8; 32],
) -> Result<()> {
    let mut acc = ctx.accounts.clean_funds_root.load_init()?;
    acc.root = root;
    Ok(())
}
