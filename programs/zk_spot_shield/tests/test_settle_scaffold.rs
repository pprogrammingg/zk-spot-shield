//! Day 22–32: settle path, CU profile, compute budget, zero-copy load, ALT sizing.

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, program_pack::Pack},
        InstructionData, ToAccountMetas,
    },
    anchor_spl::token::{
        spl_token::state::{Account as SplTokenAccount, AccountState, Mint},
        ID as TOKEN_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_account::Account,
    solana_compute_budget_interface::ComputeBudgetInstruction,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    std::path::PathBuf,
    zk_spot_shield::journal::JournalPublic,
    zk_spot_shield::state::VaultState,
};

/// Match `client::SETTLE_CU_LIMIT` (Day 30). Default LiteSVM budget is 200k.
const SETTLE_CU_LIMIT: u32 = 1_400_000;
const MINT_B_BYTES: [u8; 32] = [4u8; 32];

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

fn happy_fixture() -> (Vec<u8>, Vec<u8>) {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("../../zk-circuit/fixtures/happy");
    let proof = std::fs::read(dir.join("groth16.bin")).expect("commit groth16.bin");
    let journal = std::fs::read(dir.join("journal.bin")).expect("commit journal.bin");
    (proof, journal)
}

fn send_ix(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> Result<(), String> {
    send_ix_cu(svm, payer, instruction).map(|_| ())
}

/// Day 29: return compute units consumed on success.
fn send_ix_cu(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> Result<u64, String> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(SETTLE_CU_LIMIT),
            instruction,
        ],
        Some(&payer.pubkey()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx)
        .map(|meta| meta.compute_units_consumed)
        .map_err(|e| format!("{e:?}"))
}

fn pack_mint(supply: u64) -> Vec<u8> {
    let mint = Mint {
        mint_authority: Default::default(),
        supply,
        decimals: 0,
        is_initialized: true,
        freeze_authority: Default::default(),
    };
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();
    data
}

fn pack_token(mint: Pubkey, owner: Pubkey, amount: u64) -> Vec<u8> {
    let acc = SplTokenAccount {
        mint,
        owner,
        amount,
        delegate: Default::default(),
        state: AccountState::Initialized,
        is_native: Default::default(),
        delegated_amount: 0,
        close_authority: Default::default(),
    };
    let mut data = vec![0u8; SplTokenAccount::LEN];
    SplTokenAccount::pack(acc, &mut data).unwrap();
    data
}

fn set_token_account(svm: &mut LiteSVM, key: Pubkey, mint: Pubkey, owner: Pubkey, amount: u64) {
    svm.set_account(
        key,
        Account {
            lamports: 1_000_000_000,
            data: pack_token(mint, owner, amount),
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

fn token_amount(svm: &LiteSVM, key: Pubkey) -> u64 {
    let acc = svm.get_account(&key).expect("token account");
    SplTokenAccount::unpack(&acc.data).unwrap().amount
}

fn init_config_and_vault(svm: &mut LiteSVM, program_id: Pubkey, payer: &Keypair) {
    let (global_config, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );
    let (vault_pda, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::SPOT_VAULT_SEED],
        &program_id,
    );

    send_ix(
        svm,
        payer,
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
    )
    .expect("init global_config");

    send_ix(
        svm,
        payer,
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
    )
    .expect("init vault");
}

/// Patch vault mints/reserves and create SPL mint + vault/user token accounts.
#[allow(clippy::too_many_arguments)]
fn seed_vault_and_tokens(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    payer: Pubkey,
    asset_mint: [u8; 32],
    reserve_a: u64,
    reserve_b: u64,
    user_a: u64,
    user_b: u64,
) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let mint_a = Pubkey::new_from_array(asset_mint);
    let mint_b = Pubkey::new_from_array(MINT_B_BYTES);
    let (vault_pda, _) = VaultState::find_pda(&program_id);

    let mut vault_acc = svm.get_account(&vault_pda).expect("vault");
    {
        let state: &mut VaultState = bytemuck::from_bytes_mut(&mut vault_acc.data[8..]);
        state.mint_a = mint_a;
        state.mint_b = mint_b;
        state.reserve_a = reserve_a;
        state.reserve_b = reserve_b;
    }
    svm.set_account(vault_pda, vault_acc).unwrap();

    svm.set_account(
        mint_a,
        Account {
            lamports: 1_000_000_000,
            data: pack_mint(reserve_a + user_a),
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    svm.set_account(
        mint_b,
        Account {
            lamports: 1_000_000_000,
            data: pack_mint(reserve_b + user_b),
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let vault_token_a = Pubkey::new_unique();
    let vault_token_b = Pubkey::new_unique();
    let user_token_a = Pubkey::new_unique();
    let user_token_b = Pubkey::new_unique();

    set_token_account(svm, vault_token_a, mint_a, vault_pda, reserve_a);
    set_token_account(svm, vault_token_b, mint_b, vault_pda, reserve_b);
    set_token_account(svm, user_token_a, mint_a, payer, user_a);
    set_token_account(svm, user_token_b, mint_b, payer, user_b);

    (vault_token_a, vault_token_b, user_token_a, user_token_b)
}

fn read_vault(svm: &LiteSVM, program_id: Pubkey) -> VaultState {
    let (vault_pda, _) = VaultState::find_pda(&program_id);
    let acc = svm.get_account(&vault_pda).expect("vault");
    *bytemuck::from_bytes(&acc.data[8..])
}

fn register_root(svm: &mut LiteSVM, program_id: Pubkey, authority: &Keypair, root: [u8; 32]) {
    let (global_config, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );
    let (clean_funds_root, _) =
        zk_spot_shield::state::CleanFundsRoot::find_pda(&program_id, &root);

    send_ix(
        svm,
        authority,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::RegisterCleanFundsRoot { root }.data(),
            zk_spot_shield::accounts::RegisterCleanFundsRoot {
                authority: authority.pubkey(),
                global_config,
                clean_funds_root,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    )
    .expect("register_clean_funds_root");
}

#[allow(clippy::too_many_arguments)]
fn settle_accounts(
    program_id: Pubkey,
    payer: Pubkey,
    clean_funds_root: Pubkey,
    nullifier: &[u8; 32],
    vault_token_a: Pubkey,
    vault_token_b: Pubkey,
    user_token_a: Pubkey,
    user_token_b: Pubkey,
) -> zk_spot_shield::accounts::SettleShieldedSpot {
    let (global_config, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );
    let (vault, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::SPOT_VAULT_SEED],
        &program_id,
    );
    let (nullifier_account, _) =
        zk_spot_shield::state::NullifierAccount::find_pda(&program_id, nullifier);
    zk_spot_shield::accounts::SettleShieldedSpot {
        payer,
        global_config,
        vault,
        clean_funds_root,
        nullifier_account,
        vault_token_a,
        vault_token_b,
        user_token_a,
        user_token_b,
        token_program: TOKEN_PROGRAM_ID,
        system_program: anchor_lang::system_program::ID,
    }
}

fn settle_ix(
    program_id: Pubkey,
    accounts: &zk_spot_shield::accounts::SettleShieldedSpot,
    proof: Vec<u8>,
    journal: Vec<u8>,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::SettleShieldedSpot { proof, journal }.data(),
        accounts.to_account_metas(None),
    )
}

fn prepare_happy_settle(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    payer: &Keypair,
    reserve_b: u64,
) -> (
    Vec<u8>,
    Vec<u8>,
    JournalPublic,
    zk_spot_shield::accounts::SettleShieldedSpot,
) {
    init_config_and_vault(svm, program_id, payer);
    let (proof, journal) = happy_fixture();
    let public = JournalPublic::from_journal_bytes(&journal).expect("journal");
    let (vta, vtb, uta, utb) = seed_vault_and_tokens(
        svm,
        program_id,
        payer.pubkey(),
        public.asset_id_mint,
        1_000,
        reserve_b,
        1_000,
        0,
    );
    register_root(svm, program_id, payer, public.merkle_root);
    let (clean_pda, _) =
        zk_spot_shield::state::CleanFundsRoot::find_pda(&program_id, &public.merkle_root);
    let accounts = settle_accounts(
        program_id,
        payer.pubkey(),
        clean_pda,
        &public.nullifier,
        vta,
        vtb,
        uta,
        utb,
    );
    (proof, journal, public, accounts)
}

#[test]
fn settle_cu_profile_within_budget() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    let cu = send_ix_cu(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect("settle for CU profile");

    // Day 29: Groth16 dominates; full settle should land near ~280–400k, well under 1.4M.
    // Exact number is recorded in `notes/budgets.md` after this test is green.
    assert!(
        cu >= 200_000,
        "settle CU {cu} unexpectedly low (verify should dominate)"
    );
    assert!(
        cu <= 800_000,
        "settle CU {cu} above Day 29 hotspot band; update notes/budgets.md"
    );
    // Keep the measured value visible in test output / CI logs.
    eprintln!("Day 29 settle compute_units_consumed={cu}");
}

#[test]
fn settle_verifies_happy_fixture_with_registered_root() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect("valid fixture must settle");

    let nullifier_acc = svm
        .get_account(&accounts.nullifier_account)
        .expect("nullifier PDA created");
    assert_eq!(nullifier_acc.owner, program_id);
    assert!(nullifier_acc.data.len() >= 8 + 32);
}

#[test]
fn settle_moves_vault_reserves_and_token_balances() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);
    let amount = public.requested_swap_amount;

    send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect("settle must move reserves + SPL");

    let vault = read_vault(&svm, program_id);
    assert_eq!(vault.reserve_a, 1_000 + amount);
    assert_eq!(vault.reserve_b, 1_000 - amount);

    assert_eq!(token_amount(&svm, accounts.user_token_a), 1_000 - amount);
    assert_eq!(token_amount(&svm, accounts.vault_token_a), 1_000 + amount);
    assert_eq!(token_amount(&svm, accounts.vault_token_b), 1_000 - amount);
    assert_eq!(token_amount(&svm, accounts.user_token_b), amount);
}

#[test]
fn settle_rejects_vault_reserve_underflow() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 50);

    let err = send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect_err("insufficient reserve_b must fail");
    assert!(
        err.contains("VaultReserveUnderflow") || err.contains("custom program error"),
        "expected VaultReserveUnderflow, got: {err}"
    );

    let vault = read_vault(&svm, program_id);
    assert_eq!(vault.reserve_a, 1_000);
    assert_eq!(vault.reserve_b, 50);
}

#[test]
fn settle_rejects_replay_same_nullifier() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof.clone(), journal.clone()),
    )
    .expect("first settle must succeed");

    svm.expire_blockhash();

    // Re-fund user A so SPL is not the failure mode on replay.
    set_token_account(
        &mut svm,
        accounts.user_token_a,
        Pubkey::new_from_array([3u8; 32]),
        payer.pubkey(),
        1_000,
    );
    set_token_account(
        &mut svm,
        accounts.vault_token_b,
        Pubkey::new_from_array(MINT_B_BYTES),
        accounts.vault,
        1_000,
    );
    {
        let mut vault_acc = svm.get_account(&accounts.vault).unwrap();
        let state: &mut VaultState = bytemuck::from_bytes_mut(&mut vault_acc.data[8..]);
        state.reserve_a = 1_000;
        state.reserve_b = 1_000;
        svm.set_account(accounts.vault, vault_acc).unwrap();
    }

    let err = send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect_err("second settle with same nullifier must fail");
    assert!(
        err.contains("NullifierAlreadyUsed") || err.contains("custom program error"),
        "expected NullifierAlreadyUsed, got: {err}"
    );
}

#[test]
fn settle_rejects_when_paused() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::Pause {}.data(),
            zk_spot_shield::accounts::SetPause {
                authority: payer.pubkey(),
                global_config: accounts.global_config,
            }
            .to_account_metas(None),
        ),
    )
    .expect("pause");

    let err = send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect_err("paused settle must fail");
    assert!(
        err.contains("ProtocolPaused") || err.contains("custom program error"),
        "expected ProtocolPaused, got: {err}"
    );
}

#[test]
fn unpause_allows_settle_again() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::Pause {}.data(),
            zk_spot_shield::accounts::SetPause {
                authority: payer.pubkey(),
                global_config: accounts.global_config,
            }
            .to_account_metas(None),
        ),
    )
    .expect("pause");
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::Unpause {}.data(),
            zk_spot_shield::accounts::SetPause {
                authority: payer.pubkey(),
                global_config: accounts.global_config,
            }
            .to_account_metas(None),
        ),
    )
    .expect("unpause");

    send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect("settle after unpause");
}

#[test]
fn settle_rejects_unregistered_merkle_root() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    init_config_and_vault(&mut svm, program_id, &payer);
    let (proof, journal) = happy_fixture();
    let public = JournalPublic::from_journal_bytes(&journal).expect("journal");
    let (vta, vtb, uta, utb) = seed_vault_and_tokens(
        &mut svm,
        program_id,
        payer.pubkey(),
        public.asset_id_mint,
        1_000,
        1_000,
        1_000,
        0,
    );

    let other_root = [0xABu8; 32];
    register_root(&mut svm, program_id, &payer, other_root);
    let (wrong_pda, _) =
        zk_spot_shield::state::CleanFundsRoot::find_pda(&program_id, &other_root);

    let accounts = settle_accounts(
        program_id,
        payer.pubkey(),
        wrong_pda,
        &public.nullifier,
        vta,
        vtb,
        uta,
        utb,
    );

    let err = send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect_err("wrong CleanFundsRoot PDA must fail");
    assert!(
        err.contains("MerkleRootNotFound") || err.contains("custom program error"),
        "expected MerkleRootNotFound, got: {err}"
    );
}

#[test]
fn settle_rejects_garbage_proof() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (mut proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);
    proof[120] ^= 0xff;

    let err = send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, proof, journal),
    )
    .expect_err("garbage proof must fail");
    assert!(!err.is_empty());
}

#[test]
fn settle_scaffold_fails_when_account_missing() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    let mut metas = accounts.to_account_metas(None);
    let system_prog = metas.pop().expect("system_program");
    let token_prog = metas.pop().expect("token_program");
    let _removed = metas.pop().expect("user_token_b");
    metas.push(token_prog);
    metas.push(system_prog);

    let err = send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::SettleShieldedSpot { proof, journal }.data(),
            metas,
        ),
    )
    .expect_err("missing account must fail");
    assert!(!err.is_empty());
}

#[test]
fn settle_scaffold_fails_on_empty_proof() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (_proof, journal, _public, accounts) =
        prepare_happy_settle(&mut svm, program_id, &payer, 1_000);

    let err = send_ix(
        &mut svm,
        &payer,
        settle_ix(program_id, &accounts, vec![], journal),
    )
    .expect_err("empty proof must fail");
    assert!(!err.is_empty());
}
