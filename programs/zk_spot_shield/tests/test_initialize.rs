//! Day 21: LiteSVM inits create `GlobalConfig` + zero-copy `VaultState` at expected sizes.

use {
    anchor_lang::{
        prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize,
        InstructionData, Space, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    std::path::PathBuf,
};

fn load_program(svm: &mut LiteSVM, program_id: Pubkey) {
    let mut program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    program_path.push("../../target/deploy/zk_spot_shield.so");

    let bytes = std::fs::read(&program_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read program binary at {:?}. Did you run `anchor build` first?",
            program_path
        )
    });

    svm.add_program(program_id, &bytes).unwrap();
}

fn send_ix(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx)
        .unwrap_or_else(|err| panic!("Transaction failed with error: {err:?}"));
}

#[test]
fn test_global_config_initialize() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();

    let (global_config, _bump) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::InitializeGlobalConfig {}.data(),
            zk_spot_shield::accounts::InitializeGlobalConfig {
                payer: payer.pubkey(),
                global_config,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let global_config_account = svm
        .get_account(&global_config)
        .expect("GlobalConfig account not found after transaction");

    assert_eq!(
        global_config_account.data.len(),
        8 + zk_spot_shield::state::GlobalConfig::INIT_SPACE,
        "GlobalConfig rent space"
    );

    let mut data: &[u8] = &global_config_account.data;
    let config = zk_spot_shield::state::GlobalConfig::try_deserialize(&mut data)
        .expect("Failed to deserialize GlobalConfig state");

    assert_eq!(config.authority, payer.pubkey());
    assert_eq!(config.vkey_hash, zk_spot_shield::constants::VKEY_HASH);
    assert!(!config.pause_flag);
}

#[test]
fn test_vault_initialize() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();

    let (vault_pda, expected_bump) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::SPOT_VAULT_SEED],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::InitializeVault {}.data(),
            zk_spot_shield::accounts::InitializeVault {
                payer: payer.pubkey(),
                vault: vault_pda,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let vault_account = svm
        .get_account(&vault_pda)
        .expect("VaultState account not found after transaction");

    assert_eq!(
        vault_account.data.len(),
        8 + std::mem::size_of::<zk_spot_shield::state::VaultState>(),
        "VaultState rent space (disc + zero-copy)"
    );

    let state_bytes = &vault_account.data[8..];
    let vault_state: &zk_spot_shield::state::VaultState = bytemuck::from_bytes(state_bytes);

    assert_eq!(vault_state.authority, payer.pubkey());
    assert_eq!(vault_state.mint_a, Pubkey::default());
    assert_eq!(vault_state.mint_b, Pubkey::default());
    assert_eq!(vault_state.reserve_a, 0);
    assert_eq!(vault_state.reserve_b, 0);
    assert_eq!(vault_state.bump, expected_bump);
    assert_eq!(vault_state._padding, [0u8; 7]);
}

/// Day 21 Exit: both PDAs exist after init with correct sizes and admin fields.
#[test]
fn test_day21_init_global_config_and_vault() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();

    let (global_config, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::SPOT_VAULT_SEED],
        &program_id,
    );

    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::InitializeGlobalConfig {}.data(),
            zk_spot_shield::accounts::InitializeGlobalConfig {
                payer: payer.pubkey(),
                global_config,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    );
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::InitializeVault {}.data(),
            zk_spot_shield::accounts::InitializeVault {
                payer: payer.pubkey(),
                vault: vault_pda,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    let config_acc = svm.get_account(&global_config).expect("GlobalConfig");
    assert_eq!(
        config_acc.data.len(),
        8 + zk_spot_shield::state::GlobalConfig::INIT_SPACE
    );
    let mut data: &[u8] = &config_acc.data;
    let config = zk_spot_shield::state::GlobalConfig::try_deserialize(&mut data).unwrap();
    assert_eq!(config.authority, payer.pubkey());
    assert_eq!(config.vkey_hash, zk_spot_shield::constants::VKEY_HASH);
    assert!(!config.pause_flag);

    let vault_acc = svm.get_account(&vault_pda).expect("VaultState");
    assert_eq!(
        vault_acc.data.len(),
        8 + std::mem::size_of::<zk_spot_shield::state::VaultState>()
    );
    let vault: &zk_spot_shield::state::VaultState = bytemuck::from_bytes(&vault_acc.data[8..]);
    assert_eq!(vault.authority, payer.pubkey());
    assert_eq!(vault.bump, vault_bump);
    assert_eq!(vault.reserve_a, 0);
    assert_eq!(vault.reserve_b, 0);
}
