//! Public A→B swap (no proof / nullifier).

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
    zk_spot_shield::state::VaultState,
};

const MINT_B_BYTES: [u8; 32] = [4u8; 32];

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

fn send_ix(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) -> Result<(), String> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(200_000),
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
    let (vault_pda, _) =
        Pubkey::find_program_address(&[zk_spot_shield::constants::SPOT_VAULT_SEED], &program_id);

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

fn seed_vault_and_tokens(
    svm: &mut LiteSVM,
    program_id: Pubkey,
    payer: Pubkey,
    reserve_a: u64,
    reserve_b: u64,
    user_a: u64,
    user_b: u64,
) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let mint_a = Pubkey::new_from_array([3u8; 32]);
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

fn public_swap_ix(
    program_id: Pubkey,
    payer: Pubkey,
    amount: u64,
    vault_token_a: Pubkey,
    vault_token_b: Pubkey,
    user_token_a: Pubkey,
    user_token_b: Pubkey,
) -> Instruction {
    let (global_config, _) = Pubkey::find_program_address(
        &[zk_spot_shield::constants::GLOBAL_CONFIG_SEED],
        &program_id,
    );
    let (vault, _) = VaultState::find_pda(&program_id);
    Instruction::new_with_bytes(
        program_id,
        &zk_spot_shield::instruction::PublicSpotSwap { amount }.data(),
        zk_spot_shield::accounts::PublicSpotSwap {
            payer,
            global_config,
            vault,
            vault_token_a,
            vault_token_b,
            user_token_a,
            user_token_b,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

#[test]
fn public_spot_swap_moves_a_for_b() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    init_config_and_vault(&mut svm, program_id, &payer);
    let (vta, vtb, uta, utb) =
        seed_vault_and_tokens(&mut svm, program_id, payer.pubkey(), 0, 500, 100, 0);

    send_ix(
        &mut svm,
        &payer,
        public_swap_ix(program_id, payer.pubkey(), 40, vta, vtb, uta, utb),
    )
    .expect("public swap");

    let vault = read_vault(&svm, program_id);
    assert_eq!(vault.reserve_a, 40);
    assert_eq!(vault.reserve_b, 460);
    assert_eq!(token_amount(&svm, uta), 60);
    assert_eq!(token_amount(&svm, utb), 40);
    assert_eq!(token_amount(&svm, vta), 40);
    assert_eq!(token_amount(&svm, vtb), 460);
}

#[test]
fn public_spot_swap_rejects_zero_and_underflow() {
    let program_id = zk_spot_shield::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    load_program(&mut svm, program_id);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    init_config_and_vault(&mut svm, program_id, &payer);
    let (vta, vtb, uta, utb) =
        seed_vault_and_tokens(&mut svm, program_id, payer.pubkey(), 0, 10, 100, 0);

    let err = send_ix(
        &mut svm,
        &payer,
        public_swap_ix(program_id, payer.pubkey(), 0, vta, vtb, uta, utb),
    )
    .unwrap_err();
    assert!(!err.is_empty());

    let err = send_ix(
        &mut svm,
        &payer,
        public_swap_ix(program_id, payer.pubkey(), 50, vta, vtb, uta, utb),
    )
    .unwrap_err();
    assert!(!err.is_empty());
    assert_eq!(read_vault(&svm, program_id).reserve_b, 10);
}
