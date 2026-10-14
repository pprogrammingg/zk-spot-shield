//! Days 41–43: deposit custody, leaf PDA, deposit-tip root → settle.

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
    zk_spot_shield::state::{DepositLeafAccount, VaultState},
};

const SETTLE_CU_LIMIT: u32 = 1_400_000;
const MINT_B_BYTES: [u8; 32] = [4u8; 32];

/// `happy.json` derived.leaf — Poseidon(secret=0x01.., addr=0x02.., balance=1000).
const HAPPY_LEAF: [u8; 32] = [
    0x26, 0x3f, 0xf1, 0x74, 0x37, 0xc6, 0x65, 0xe4, 0xa4, 0x78, 0x62, 0x85, 0x99, 0x42, 0xf8,
    0x35, 0xfc, 0xf6, 0x8c, 0x55, 0x64, 0x22, 0x38, 0xcc, 0x4a, 0x2d, 0x3f, 0x82, 0xd0, 0x9d,
    0x6a, 0x76,
];

fn load_program(svm: &mut LiteSVM, program_id: Pubkey) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let candidates = [
        root.join("target/deploy/zk_spot_shield.so"),
        root.join("target/sbpf-solana-solana/release/zk_spot_shield.so"),
    ];
    let bytes = candidates
        .iter()
        .find_map(|p| std::fs::read(p).ok())
        .unwrap_or_else(|| {
            panic!(
                "Failed to read program .so (tried {:?}). Run `anchor build` first.",
                candidates
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
        .map(|_| ())
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

fn deposit_ix(
    program_id: Pubkey,
    depositor: Pubkey,
    leaf: [u8; 32],
    amount: u64,
    vault_token: Pubkey,
    user_token: Pubkey,
) -> Instruction {
    let (vault, _) = VaultState::find_pda(&program_id);
    let (deposit_leaf, _) = DepositLeafAccount::find_pda(&program_id, &leaf);
    Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::DepositShielded { leaf, amount }.data(),
        zk_spot_shield::accounts::DepositShielded {
            depositor,
            vault,
            deposit_leaf,
            vault_token,
            user_token,
            token_program: TOKEN_PROGRAM_ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    )
}

#[test]
fn deposit_moves_tokens_and_records_leaf() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    init_config_and_vault(&mut svm, program_id, &payer);
    let (vta, _vtb, uta, _utb) = seed_vault_and_tokens(
        &mut svm,
        program_id,
        payer.pubkey(),
        [3u8; 32],
        0,
        0,
        1_000,
        0,
    );

    send_ix(
        &mut svm,
        &payer,
        deposit_ix(
            program_id,
            payer.pubkey(),
            HAPPY_LEAF,
            1_000,
            vta,
            uta,
        ),
    )
    .expect("deposit");

    let vault = read_vault(&svm, program_id);
    assert_eq!(vault.reserve_a, 1_000);
    assert_eq!(token_amount(&svm, uta), 0);
    assert_eq!(token_amount(&svm, vta), 1_000);

    let (leaf_pda, _) = DepositLeafAccount::find_pda(&program_id, &HAPPY_LEAF);
    let acc = svm.get_account(&leaf_pda).expect("deposit leaf PDA");
    assert_eq!(acc.owner, program_id);
    let stored: &DepositLeafAccount = bytemuck::from_bytes(&acc.data[8..]);
    assert_eq!(stored.leaf, HAPPY_LEAF);
    assert_eq!(stored.amount, 1_000);
    assert_eq!(stored.depositor, payer.pubkey());
}

#[test]
fn deposit_rejects_zero_amount() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    init_config_and_vault(&mut svm, program_id, &payer);
    let (vta, _vtb, uta, _utb) = seed_vault_and_tokens(
        &mut svm,
        program_id,
        payer.pubkey(),
        [3u8; 32],
        0,
        0,
        1_000,
        0,
    );

    let err = send_ix(
        &mut svm,
        &payer,
        deposit_ix(program_id, payer.pubkey(), HAPPY_LEAF, 0, vta, uta),
    )
    .unwrap_err();
    assert!(
        err.contains("ZeroDepositAmount"),
        "expected ZeroDepositAmount, got: {err}"
    );
}

#[test]
fn deposit_then_register_tip_then_settle() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let (proof, journal) = happy_fixture();
    let public = JournalPublic::from_journal_bytes(&journal).expect("journal");

    init_config_and_vault(&mut svm, program_id, &payer);
    // Liquidity for settle payout on B; user holds A to deposit + later pay settle.
    let (vta, vtb, uta, utb) = seed_vault_and_tokens(
        &mut svm,
        program_id,
        payer.pubkey(),
        public.asset_id_mint,
        0,
        1_000,
        2_000,
        0,
    );

    // Day 41–42: deposit happy leaf (tip of empty tree ⇒ fixture merkle_root).
    send_ix(
        &mut svm,
        &payer,
        deposit_ix(
            program_id,
            payer.pubkey(),
            HAPPY_LEAF,
            1_000,
            vta,
            uta,
        ),
    )
    .expect("deposit happy leaf");
    assert_eq!(read_vault(&svm, program_id).reserve_a, 1_000);

    // Day 43: register tip (= journal merkle_root for index-0 happy leaf).
    let (global_config, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );
    let (clean_funds_root, _) =
        zk_spot_shield::state::CleanFundsRoot::find_pda(&program_id, &public.merkle_root);
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::RegisterCleanFundsRoot {
                root: public.merkle_root,
            }
            .data(),
            zk_spot_shield::accounts::RegisterCleanFundsRoot {
                authority: payer.pubkey(),
                global_config,
                clean_funds_root,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    )
    .expect("register deposit tip");

    let (nullifier_account, _) =
        zk_spot_shield::state::NullifierAccount::find_pda(&program_id, &public.nullifier);
    let (vault, _) = VaultState::find_pda(&program_id);
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &zk_spot_shield::instruction::SettleShieldedSpot { proof, journal }.data(),
            zk_spot_shield::accounts::SettleShieldedSpot {
                payer: payer.pubkey(),
                global_config,
                vault,
                clean_funds_root,
                nullifier_account,
                vault_token_a: vta,
                vault_token_b: vtb,
                user_token_a: uta,
                user_token_b: utb,
                token_program: TOKEN_PROGRAM_ID,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
        ),
    )
    .expect("settle against deposit-tip root");

    let amount = public.requested_swap_amount;
    let vault_state = read_vault(&svm, program_id);
    // deposit 1000 + settle take 100 on A; pay 100 B from pre-seeded reserve.
    assert_eq!(vault_state.reserve_a, 1_000 + amount);
    assert_eq!(vault_state.reserve_b, 1_000 - amount);
    assert_eq!(token_amount(&svm, uta), 2_000 - 1_000 - amount);
    assert_eq!(token_amount(&svm, utb), amount);
}
