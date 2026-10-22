//! Protocol admin rotates `GlobalConfig.vkey_hash` after a guest ELF rebuild.

use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::GlobalConfig, GLOBAL_CONFIG_SEED};

#[derive(Accounts)]
pub struct SetVkeyHash<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
        has_one = authority @ ErrorCode::Unauthorized,
    )]
    pub global_config: Account<'info, GlobalConfig>,
}

pub fn handle_set_vkey_hash(ctx: Context<SetVkeyHash>, vkey_hash: [u8; 32]) -> Result<()> {
    ctx.accounts.global_config.rotate_vkey_hash(vkey_hash)
}
