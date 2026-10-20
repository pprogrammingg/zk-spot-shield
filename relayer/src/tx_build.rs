//! Day 46: build `settle_shielded_spot` with relayer as fee payer.
//!
//! Fee payer ≠ note owner: `payer` pays SOL / nullifier rent. Recipient ATAs are
//! separate. Journal still binds amount / mint / nullifier / merkle_root.
//! Callers supply fixture or pre-proven bytes — no `$PROVE`.

use anchor_lang::{InstructionData, ToAccountMetas};
use solana_hash::Hash;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_pubkey::Pubkey as SolPubkey;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

use client::SETTLE_CU_LIMIT;
use zk_spot_shield::accounts::SettleShieldedSpot as SettleAccounts;
use zk_spot_shield::instruction::SettleShieldedSpot as SettleIx;

fn to_sol_pubkey(pk: anchor_lang::prelude::Pubkey) -> SolPubkey {
    SolPubkey::new_from_array(pk.to_bytes())
}

/// Account metas for a settle (fee payer first).
#[derive(Debug, Clone)]
pub struct SettleAccountKeys {
    pub fee_payer: anchor_lang::prelude::Pubkey,
    pub global_config: anchor_lang::prelude::Pubkey,
    pub vault: anchor_lang::prelude::Pubkey,
    pub clean_funds_root: anchor_lang::prelude::Pubkey,
    pub nullifier_account: anchor_lang::prelude::Pubkey,
    pub vault_token_a: anchor_lang::prelude::Pubkey,
    pub vault_token_b: anchor_lang::prelude::Pubkey,
    pub user_token_a: anchor_lang::prelude::Pubkey,
    pub user_token_b: anchor_lang::prelude::Pubkey,
}

impl SettleAccountKeys {
    pub fn to_anchor_accounts(&self) -> SettleAccounts {
        SettleAccounts {
            payer: self.fee_payer,
            global_config: self.global_config,
            vault: self.vault,
            clean_funds_root: self.clean_funds_root,
            nullifier_account: self.nullifier_account,
            vault_token_a: self.vault_token_a,
            vault_token_b: self.vault_token_b,
            user_token_a: self.user_token_a,
            user_token_b: self.user_token_b,
            token_program: anchor_spl::token::ID,
            system_program: anchor_lang::system_program::ID,
        }
    }
}

fn cu_limit_ix() -> Instruction {
    // Same layout as `solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit`.
    use solana_compute_budget_interface::ComputeBudgetInstruction;
    let ix = ComputeBudgetInstruction::set_compute_unit_limit(SETTLE_CU_LIMIT);
    Instruction {
        program_id: to_sol_pubkey(anchor_lang::prelude::Pubkey::new_from_array(
            ix.program_id.to_bytes(),
        )),
        accounts: ix
            .accounts
            .into_iter()
            .map(|m| AccountMeta {
                pubkey: SolPubkey::new_from_array(m.pubkey.to_bytes()),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: ix.data,
    }
}

/// Build the settle instruction (program id from `zk_spot_shield::ID`).
pub fn build_settle_ix(
    accounts: &SettleAccountKeys,
    proof: Vec<u8>,
    journal: Vec<u8>,
) -> Instruction {
    let metas = accounts.to_anchor_accounts().to_account_metas(None);
    Instruction {
        program_id: to_sol_pubkey(zk_spot_shield::ID),
        accounts: metas
            .into_iter()
            .map(|m| AccountMeta {
                pubkey: to_sol_pubkey(m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: SettleIx { proof, journal }.data(),
    }
}

/// CU budget + settle with `fee_payer` as message fee payer.
pub fn build_settle_message(
    accounts: &SettleAccountKeys,
    proof: Vec<u8>,
    journal: Vec<u8>,
    recent_blockhash: Hash,
) -> Message {
    let ixs = vec![
        cu_limit_ix(),
        build_settle_ix(accounts, proof, journal),
    ];
    Message::new_with_blockhash(&ixs, Some(&to_sol_pubkey(accounts.fee_payer)), &recent_blockhash)
}

/// Sign a settle message with the relayer fee-payer keypair only.
pub fn sign_settle_as_fee_payer(
    fee_payer: &Keypair,
    accounts: &SettleAccountKeys,
    proof: Vec<u8>,
    journal: Vec<u8>,
    recent_blockhash: Hash,
) -> Result<VersionedTransaction, String> {
    let fp = to_sol_pubkey(accounts.fee_payer);
    assert_eq!(
        fee_payer.pubkey(),
        fp,
        "fee_payer keypair must match accounts.fee_payer"
    );
    let msg = build_settle_message(accounts, proof, journal, recent_blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[fee_payer])
        .map_err(|e| format!("sign settle: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_keys(fee_payer: anchor_lang::prelude::Pubkey) -> SettleAccountKeys {
        SettleAccountKeys {
            fee_payer,
            global_config: anchor_lang::prelude::Pubkey::new_unique(),
            vault: anchor_lang::prelude::Pubkey::new_unique(),
            clean_funds_root: anchor_lang::prelude::Pubkey::new_unique(),
            nullifier_account: anchor_lang::prelude::Pubkey::new_unique(),
            vault_token_a: anchor_lang::prelude::Pubkey::new_unique(),
            vault_token_b: anchor_lang::prelude::Pubkey::new_unique(),
            user_token_a: anchor_lang::prelude::Pubkey::new_unique(),
            user_token_b: anchor_lang::prelude::Pubkey::new_unique(),
        }
    }

    #[test]
    fn settle_ix_marks_fee_payer_signer() {
        let kp = Keypair::new();
        let keys = sample_keys(anchor_lang::prelude::Pubkey::new_from_array(
            kp.pubkey().to_bytes(),
        ));
        let ix = build_settle_ix(&keys, vec![1; 356], vec![2; 104]);
        assert_eq!(ix.program_id, to_sol_pubkey(zk_spot_shield::ID));
        assert_eq!(ix.accounts[0].pubkey, kp.pubkey());
        assert!(ix.accounts[0].is_signer);
        assert!(ix.accounts[0].is_writable);
        let user_a = ix
            .accounts
            .iter()
            .find(|a| a.pubkey == to_sol_pubkey(keys.user_token_a));
        assert!(user_a.is_some_and(|a| !a.is_signer));
    }

    #[test]
    fn signed_tx_fee_payer_is_relayer() {
        let kp = Keypair::new();
        let keys = sample_keys(anchor_lang::prelude::Pubkey::new_from_array(
            kp.pubkey().to_bytes(),
        ));
        let tx = sign_settle_as_fee_payer(
            &kp,
            &keys,
            vec![0u8; 356],
            vec![0u8; 104],
            Hash::new_unique(),
        )
        .expect("sign");
        assert_eq!(tx.message.header().num_required_signatures, 1);
        let msg_keys = tx.message.static_account_keys();
        assert_eq!(msg_keys[0], kp.pubkey());
        assert_eq!(SETTLE_CU_LIMIT, 1_400_000);
    }

    #[test]
    fn fee_payer_differs_from_user_ata_ok() {
        let relayer = Keypair::new();
        let mut keys = sample_keys(anchor_lang::prelude::Pubkey::new_from_array(
            relayer.pubkey().to_bytes(),
        ));
        keys.user_token_a = anchor_lang::prelude::Pubkey::new_unique();
        assert_ne!(keys.fee_payer, keys.user_token_a);
        let ix = build_settle_ix(&keys, vec![9; 10], vec![8; 10]);
        assert_eq!(ix.accounts[0].pubkey, relayer.pubkey());
    }
}
