//! Public 1:1 spot: payer SPL-transfers mint A into the vault; vault pays mint B.
//! No proof, no nullifier, no leaf. Contrast with `settle_shielded_spot`.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode,
    state::{GlobalConfig, VaultState},
    GLOBAL_CONFIG_SEED, SPOT_VAULT_SEED,
};

#[event]
pub struct PublicSpotSwapEvent {
    pub payer: Pubkey,
    pub amount: u64,
    pub mint_in: Pubkey,
    pub mint_out: Pubkey,
}

/// Wallet pays mint A, receives mint B 1:1. Fee payer = token owner (`payer`).
#[derive(Accounts)]
pub struct PublicSpotSwap<'info> {
    /// Pays SOL fees and authorizes the mint-A transfer. Visible on the receipt.
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [SPOT_VAULT_SEED],
        bump = vault.load()?.bump,
    )]
    pub vault: AccountLoader<'info, VaultState>,

    #[account(
        mut,
        constraint = vault_token_a.owner == vault.key() @ ErrorCode::InvalidTokenAccount,
    )]
    pub vault_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_b.owner == vault.key() @ ErrorCode::InvalidTokenAccount,
    )]
    pub vault_token_b: Account<'info, TokenAccount>,

    /// Payer's mint-A ATA (debited).
    #[account(
        mut,
        constraint = user_token_a.owner == payer.key() @ ErrorCode::Unauthorized,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    /// Payer's mint-B ATA (credited).
    #[account(
        mut,
        constraint = user_token_b.owner == payer.key() @ ErrorCode::Unauthorized,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_public_spot_swap(ctx: Context<PublicSpotSwap>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroSwapAmount);
    require!(
        !ctx.accounts.global_config.pause_flag,
        ErrorCode::ProtocolPaused
    );

    let (vault_bump, mint_a, mint_b) = {
        let mut vault = ctx.accounts.vault.load_mut()?;
        let mint_a = vault.mint_a;
        let mint_b = vault.mint_b;
        // Spend mint A → receive A in reserves, pay B out.
        vault.apply_swap(amount, &mint_a)?;
        (vault.bump, mint_a, mint_b)
    };

    require_keys_eq!(
        ctx.accounts.vault_token_a.mint,
        mint_a,
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.vault_token_b.mint,
        mint_b,
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.user_token_a.mint,
        mint_a,
        ErrorCode::InvalidTokenAccount
    );
    require_keys_eq!(
        ctx.accounts.user_token_b.mint,
        mint_b,
        ErrorCode::InvalidTokenAccount
    );

    token::transfer(
        CpiContext::new(
            token::ID,
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.vault_token_a.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
    )?;

    let vault_seeds: &[&[u8]] = &[SPOT_VAULT_SEED, &[vault_bump]];
    token::transfer(
        CpiContext::new_with_signer(
            token::ID,
            Transfer {
                from: ctx.accounts.vault_token_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            &[vault_seeds],
        ),
        amount,
    )?;

    emit!(PublicSpotSwapEvent {
        payer: ctx.accounts.payer.key(),
        amount,
        mint_in: mint_a,
        mint_out: mint_b,
    });

    Ok(())
}
