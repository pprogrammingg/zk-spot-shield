//! Day 28: authority-only pause / unpause on `GlobalConfig`.

use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::GlobalConfig, GLOBAL_CONFIG_SEED};

#[derive(Accounts)]
pub struct SetPause<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
        has_one = authority @ ErrorCode::Unauthorized,
    )]
    pub global_config: Account<'info, GlobalConfig>,
}

pub fn handle_pause(ctx: Context<SetPause>) -> Result<()> {
    ctx.accounts.global_config.pause_flag = true;
    msg!("global_config: paused");
    Ok(())
}

pub fn handle_unpause(ctx: Context<SetPause>) -> Result<()> {
    ctx.accounts.global_config.pause_flag = false;
    msg!("global_config: unpaused");
    Ok(())
}
