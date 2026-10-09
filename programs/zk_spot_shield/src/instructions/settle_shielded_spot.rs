//! Settle hot path (Days 25–31): verify → root → vault → SPL → nullifier.
//!
//! Day 31 load path: `VaultState` / `CleanFundsRoot` stay `AccountLoader` (zero-copy).
//! Mint checks run once after a single `load_mut` — constraints only pin vault bump /
//! token owners so we avoid repeated `vault.load()` during account validation.

use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, CreateAccount};
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode,
    journal::{JournalPublic, JOURNAL_BYTE_LEN},
    state::{CleanFundsRoot, GlobalConfig, NullifierAccount, VaultState},
    verify_sp1::{verify_sp1_groth16_v6, SP1_V6_PROOF_BYTE_LEN},
    GLOBAL_CONFIG_SEED, NULLIFIER_SEED_PREFIX, SPOT_VAULT_SEED,
};

/// Happy-path fixture sizes (`notes/budgets.md` / Day 12 wrap).
pub const SETTLE_PROOF_BYTE_LEN: usize = SP1_V6_PROOF_BYTE_LEN;
pub const SETTLE_JOURNAL_BYTE_LEN: usize = JOURNAL_BYTE_LEN;

/// Journal layout: amount(8) | mint(32) | nullifier(32) | merkle_root(32).
const JOURNAL_NULLIFIER_OFFSET: usize = 8 + 32;

/// Account order (Day 31): signers/config → zero-copy loaders → tokens → programs.
#[derive(Accounts)]
#[instruction(proof: Vec<u8>, journal: Vec<u8>)]
pub struct SettleShieldedSpot<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
        constraint = journal.len() >= JOURNAL_NULLIFIER_OFFSET + 32 @ ErrorCode::InvalidJournal
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [SPOT_VAULT_SEED],
        bump = vault.load()?.bump,
    )]
    pub vault: AccountLoader<'info, VaultState>,

    /// Approved root PDA for journal `merkle_root` (seeds checked in handler).
    pub clean_funds_root: AccountLoader<'info, CleanFundsRoot>,

    /// One-time spend tag PDA. Must be empty; created after success (Day 28).
    /// CHECK: empty ⇒ unused; non-empty ⇒ `NullifierAlreadyUsed`.
    #[account(
        mut,
        seeds = [NULLIFIER_SEED_PREFIX, &journal[JOURNAL_NULLIFIER_OFFSET..JOURNAL_NULLIFIER_OFFSET + 32]],
        bump,
    )]
    pub nullifier_account: UncheckedAccount<'info>,

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

    #[account(
        mut,
        constraint = user_token_a.owner == payer.key() @ ErrorCode::Unauthorized,
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.owner == payer.key() @ ErrorCode::Unauthorized,
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_settle_shielded_spot(
    ctx: Context<SettleShieldedSpot>,
    proof: Vec<u8>,
    journal: Vec<u8>,
) -> Result<()> {
    require!(!proof.is_empty(), ErrorCode::EmptyProof);
    require!(!journal.is_empty(), ErrorCode::EmptyJournal);
    require!(
        journal.len() >= JOURNAL_NULLIFIER_OFFSET + 32,
        ErrorCode::InvalidJournal
    );
    require!(
        !ctx.accounts.global_config.pause_flag,
        ErrorCode::ProtocolPaused
    );
    require!(
        ctx.accounts.nullifier_account.data_is_empty(),
        ErrorCode::NullifierAlreadyUsed
    );

    let vkey_hash = ctx.accounts.global_config.vkey_hash;

    verify_sp1_groth16_v6(&proof, &journal, &vkey_hash).map_err(|e| {
        msg!("sp1 groth16 verify failed: {:?}", e);
        error!(ErrorCode::InvalidProof)
    })?;

    let public = JournalPublic::from_journal_bytes(&journal).ok_or_else(|| {
        msg!("journal parse failed (expected {} bytes)", JOURNAL_BYTE_LEN);
        error!(ErrorCode::InvalidJournal)
    })?;

    let (expected_root_pda, _) =
        CleanFundsRoot::find_pda(ctx.program_id, &public.merkle_root);
    require_keys_eq!(
        ctx.accounts.clean_funds_root.key(),
        expected_root_pda,
        ErrorCode::MerkleRootNotFound
    );

    {
        let root_acc = ctx.accounts.clean_funds_root.load()?;
        require!(
            root_acc.root == public.merkle_root,
            ErrorCode::MerkleRootNotFound
        );
    }

    let amount = public.requested_swap_amount;
    let asset_mint = Pubkey::new_from_array(public.asset_id_mint);

    // Single zero-copy mut load: mutate reserves + copy mints/bump onto the stack.
    let (vault_bump, mint_a, mint_b) = {
        let mut vault = ctx.accounts.vault.load_mut()?;
        vault.apply_swap(amount, &asset_mint)?;
        (vault.bump, vault.mint_a, vault.mint_b)
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

    // Day 27: 1:1 SPL — user pays journal mint; vault PDA pays the other mint.
    let vault_seeds: &[&[u8]] = &[SPOT_VAULT_SEED, &[vault_bump]];
    if asset_mint == mint_a {
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
    } else if asset_mint == mint_b {
        token::transfer(
            CpiContext::new(
                token::ID,
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.vault_token_b.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            amount,
        )?;
        token::transfer(
            CpiContext::new_with_signer(
                token::ID,
                Transfer {
                    from: ctx.accounts.vault_token_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                &[vault_seeds],
            ),
            amount,
        )?;
    } else {
        return err!(ErrorCode::UnknownVaultMint);
    }

    // Day 28: create + write nullifier PDA only after verify/vault/SPL succeed.
    let space = 8 + std::mem::size_of::<NullifierAccount>();
    let lamports = Rent::get()?.minimum_balance(space);
    let nf_bump = ctx.bumps.nullifier_account;
    let nf_seeds: &[&[u8]] = &[
        NULLIFIER_SEED_PREFIX,
        public.nullifier.as_ref(),
        &[nf_bump],
    ];
    system_program::create_account(
        CpiContext::new_with_signer(
            system_program::ID,
            CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.nullifier_account.to_account_info(),
            },
            &[nf_seeds],
        ),
        lamports,
        space as u64,
        ctx.program_id,
    )?;

    {
        let nf_info = ctx.accounts.nullifier_account.to_account_info();
        let mut data = nf_info.try_borrow_mut_data()?;
        let disc = NullifierAccount::DISCRIMINATOR;
        data[..8].copy_from_slice(disc);
        data[8..40].copy_from_slice(&public.nullifier);
    }

    msg!(
        "settle_shielded_spot: ok amount={}",
        public.requested_swap_amount
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::InstructionData;

    #[test]
    fn settle_shielded_spot_ix_data_includes_proof_and_journal() {
        let proof = vec![1u8, 2, 3];
        let journal = vec![4u8, 5];
        let data = crate::instruction::SettleShieldedSpot {
            proof: proof.clone(),
            journal: journal.clone(),
        }
        .data();

        assert!(data.len() >= 8 + 4 + proof.len() + 4 + journal.len());
        assert_eq!(&data[8 + 4..8 + 4 + proof.len()], proof.as_slice());
        let journal_start = 8 + 4 + proof.len() + 4;
        assert_eq!(
            &data[journal_start..journal_start + journal.len()],
            journal.as_slice()
        );
    }

    #[test]
    fn settle_fixture_byte_lens_match_budget_note() {
        assert_eq!(SETTLE_PROOF_BYTE_LEN, 356);
        assert_eq!(SETTLE_JOURNAL_BYTE_LEN, 104);
    }
}
