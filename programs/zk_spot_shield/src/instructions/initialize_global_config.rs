use anchor_lang::prelude::*;

use crate::{constants::*, state::GlobalConfig};

#[derive(Accounts)]
pub struct InitializeGlobalConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + GlobalConfig::INIT_SPACE,
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_global_config(
    ctx: Context<InitializeGlobalConfig>,
    vkey_hash: [u8; 32],
) -> Result<()> {
    let config = &mut ctx.accounts.global_config;

    config.authority = ctx.accounts.payer.key();
    config.vkey_hash = vkey_hash;
    config.pause_flag = false;

    Ok(())
}