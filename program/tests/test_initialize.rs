use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction, std::path::PathBuf,
};

#[test]
fn test_global_config_initialize() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();

    let (global_config, _bump) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );

    let mut svm = LiteSVM::new();

    let mut program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    program_path.push("../target/deploy/zk_spot_shield.so");

    let bytes = std::fs::read(&program_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read program binary at {:?}. Did you run `anchor build` first?",
            program_path
        )
    });

    svm.add_program(program_id, &bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let vkey_hash = [0u8; 32];

    let instruction = Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::InitializeGlobalConfig {  }.data(),
        zk_spot_shield::accounts::InitializeGlobalConfig {
            payer: payer.pubkey(),
            global_config,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();

    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &blockhash,
    );

    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&payer],
    )
    .unwrap();

    let _meta = match svm.send_transaction(tx) {
        Ok(meta) => meta,
        Err(err) => {
            panic!("Transaction failed with error: {err:?}");
        }
    };

    // Read GlobalConfig back from the chain
    let global_config_account = svm
        .get_account(&global_config)
        .expect("GlobalConfig account not found after transaction");

    let mut data: &[u8] = &global_config_account.data;

    let config = zk_spot_shield::state::GlobalConfig::try_deserialize(&mut data)
        .expect("Failed to deserialize GlobalConfig state");

    // Verify initialization
    assert_eq!(config.authority, payer.pubkey());
    assert_eq!(config.vkey_hash, vkey_hash);
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

    let mut program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    program_path.push("../target/deploy/zk_spot_shield.so");

    let bytes = std::fs::read(&program_path).unwrap_or_else(|_| {
        panic!(
            "Failed to read program binary at {:?}. Did you run `anchor build` first?",
            program_path
        )
    });

    svm.add_program(program_id, &bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::InitializeVault {}.data(),
        zk_spot_shield::accounts::InitializeVault {
            payer: payer.pubkey(),
            vault: vault_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();

    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &blockhash,
    );

    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&payer],
    )
    .unwrap();

    let _meta = match svm.send_transaction(tx) {
        Ok(meta) => meta,
        Err(err) => {
            panic!("Transaction failed with error: {err:?}");
        }
    };

    // Read VaultState back from LiteSVM
    let vault_account = svm
        .get_account(&vault_pda)
        .expect("VaultState account not found after transaction");

    // Account data length must equal 8 bytes (discriminator) + 120 bytes (VaultState)
    assert_eq!(vault_account.data.len(), 8 + std::mem::size_of::<zk_spot_shield::state::VaultState>());

    // Skip the 8-byte Anchor discriminator and cast the remaining zero-copy data via bytemuck
    let state_bytes = &vault_account.data[8..];
    let vault_state: &zk_spot_shield::state::VaultState = bytemuck::from_bytes(state_bytes);

    // Assert initial state field values
    assert_eq!(vault_state.authority, payer.pubkey());
    assert_eq!(vault_state.mint_a, Pubkey::default());
    assert_eq!(vault_state.mint_b, Pubkey::default());
    assert_eq!(vault_state.reserve_a, 0);
    assert_eq!(vault_state.reserve_b, 0);
    assert_eq!(vault_state.bump, expected_bump);
    assert_eq!(vault_state._padding, [0u8; 7]);
}