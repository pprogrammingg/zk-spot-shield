use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction, std::path::PathBuf,
};

#[test]
fn test_initialize() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let counter =
        Pubkey::find_program_address(&[zk_spot_shield::constants::COUNTER_SEED], &program_id).0;

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/zk_spot_shield.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::Initialize {}.data(),
        zk_spot_shield::accounts::Initialize {
            payer: payer.pubkey(),
            counter,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());

    let counter_account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &counter_account.data;
    let counter_state = zk_spot_shield::state::Counter::try_deserialize(&mut data).unwrap();
    assert_eq!(counter_state.count, 0);
    assert_eq!(counter_state.authority, payer.pubkey());

    let instruction = Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::Increment {}.data(),
        zk_spot_shield::accounts::Increment {
            counter,
            authority: payer.pubkey(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());

    let counter_account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &counter_account.data;
    let counter_state = zk_spot_shield::state::Counter::try_deserialize(&mut data).unwrap();
    assert_eq!(counter_state.count, 1);
    assert_eq!(counter_state.authority, payer.pubkey());
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