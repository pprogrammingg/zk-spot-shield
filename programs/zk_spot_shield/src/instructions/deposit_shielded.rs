//! SPL deposit into vault + record public leaf commitment PDA.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode,
    state::{DepositLeafAccount, VaultState},
    DEPOSIT_LEAF_SEED_PREFIX, SPOT_VAULT_SEED,
};

#[event]
pub struct DepositEvent {
    pub leaf: [u8; 32],
    pub amount: u64,
    pub mint: Pubkey,
    pub depositor: Pubkey,
}

/// Accounts for one deposit tx: SPL custody into the vault + durable leaf PDA.
///
/// Ix data carries `leaf` (32-byte Poseidon commitment) and `amount`. The leaf is
/// written into `deposit_leaf` account data (and mirrored in `DepositEvent` logs) —
/// not only logs. Secret / full note never appear on-chain.
#[derive(Accounts)]
#[instruction(leaf: [u8; 32], amount: u64)]
pub struct DepositShielded<'info> {
    /// Wallet that signs, pays rent for the new leaf PDA, and authorizes the SPL transfer.
    #[account(mut)]
    pub depositor: Signer<'info>,

    /// Escrow state PDA (`["spot_vault"]`): mints + reserves; `reserve_*` bumped by `amount`.
    #[account(
        mut,
        seeds = [SPOT_VAULT_SEED],
        bump = vault.load()?.bump,
    )]
    pub vault: AccountLoader<'info, VaultState>,

    /// New PDA `["deposit_leaf", leaf]` holding public leaf + amount + mint + depositor.
    /// `init` fails if this leaf was already deposited.
    #[account(
        init,
        payer = depositor,
        space = 8 + std::mem::size_of::<DepositLeafAccount>(),
        seeds = [DEPOSIT_LEAF_SEED_PREFIX, leaf.as_ref()],
        bump,
    )]
    pub deposit_leaf: AccountLoader<'info, DepositLeafAccount>,

    /// Vault’s SPL token account (owner = vault PDA) that receives the deposited tokens.
    #[account(
        mut,
        constraint = vault_token.owner == vault.key() @ ErrorCode::InvalidTokenAccount,
        constraint = user_token.mint == vault_token.mint @ ErrorCode::InvalidTokenAccount,
    )]
    pub vault_token: Account<'info, TokenAccount>,

    /// Depositor’s SPL token account (same mint); balance decreases by `amount`.
    #[account(
        mut,
        constraint = user_token.owner == depositor.key() @ ErrorCode::Unauthorized,
    )]
    pub user_token: Account<'info, TokenAccount>,

    /// SPL Token program — CPI target for the transfer.
    pub token_program: Program<'info, Token>,
    /// System program — creates the `deposit_leaf` account (`init`).
    pub system_program: Program<'info, System>,
}

pub fn handle_deposit_shielded(
    ctx: Context<DepositShielded>,
    leaf: [u8; 32],
    amount: u64,
) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroDepositAmount);
    require!(leaf != [0u8; 32], ErrorCode::ZeroDepositLeaf);

    let mint = ctx.accounts.user_token.mint;
    require_keys_eq!(
        ctx.accounts.vault_token.mint,
        mint,
        ErrorCode::InvalidTokenAccount
    );

    {
        let mut vault = ctx.accounts.vault.load_mut()?;
        vault.apply_deposit(amount, &mint)?;
    }

    token::transfer(
        CpiContext::new(
            token::ID,
            Transfer {
                from: ctx.accounts.user_token.to_account_info(),
                to: ctx.accounts.vault_token.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount,
    )?;

    {
        let mut acc = ctx.accounts.deposit_leaf.load_init()?;
        acc.leaf = leaf;
        acc.amount = amount;
        acc.mint = mint;
        acc.depositor = ctx.accounts.depositor.key();
    }

    emit!(DepositEvent {
        leaf,
        amount,
        mint,
        depositor: ctx.accounts.depositor.key(),
    });

    Ok(())
}
