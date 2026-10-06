use std::path::{Path, PathBuf};

use zk_circuit_io::{compute_leaf, index0_empty_path, verify_merkle_path, PrivateInputs};

pub fn corrupt_merkle_path_inputs() -> PrivateInputs {
    let mut inputs = happy_path_inputs();
    inputs.merkle_path[0].0 = [4u8; 32];
    // leave expected_root alone
    inputs
}

pub fn happy_path_inputs() -> PrivateInputs {
    let secret = [1u8; 32];
    let user_address = [2u8; 32];
    let asset_id_mint = [3u8; 32];
    let balance = 1_000;
    let requested_swap_amount = 100; // <= balance
    let leaf = compute_leaf(&secret, &user_address, balance);
    let merkle_path = index0_empty_path();
    let expected_root = verify_merkle_path(leaf, &merkle_path);
    PrivateInputs {
        secret,
        user_address,
        merkle_path,
        balance,
        requested_swap_amount,
        asset_id_mint,
        expected_root,
    }
}

/// `zk-circuit/fixtures/happy/` — Groth16 + journal for [`happy_path_inputs`] only.
pub fn happy_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/happy")
}

pub fn use_network() -> bool {
    matches!(std::env::var("SP1_USE_NETWORK").ok().as_deref(), Some("1"))
}

/// Repo root `.env` (gitignored). Safe to call if the file is missing.
pub fn load_root_dotenv() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env");
    let _ = dotenvy::from_path(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use sp1_sdk::{Elf, Prover, ProverClient, SP1Stdin};
    use zk_circuit_io::PublicOutputs;

    /// Guest ELF from `cargo prove build` / host prove path. Needed for execute.
    const GUEST_ELF: Elf = Elf::Static(include_bytes!("../../../target/elf/guest"));

    #[test]
    fn happy_fixture_journal_matches_happy_path_inputs() {
        let inputs = happy_path_inputs();
        let dir = happy_fixture_dir();
        let journal = fs::read(dir.join("journal.bin"))
            .expect("commit zk-circuit/fixtures/happy/journal.bin");
        let public = PublicOutputs::from_journal_bytes(&journal).expect("journal layout");
        assert_eq!(public.requested_swap_amount, inputs.requested_swap_amount);
        assert_eq!(public.asset_id_mint, inputs.asset_id_mint);
        assert_eq!(public.merkle_root, inputs.expected_root);
        assert_ne!(public.nullifier, [0u8; 32]);
        let groth16 = fs::read(dir.join("groth16.bin")).expect("commit groth16.bin");
        assert_eq!(groth16.len(), 356);
    }

    #[test]
    fn corrupt_merkle_path_mismatches_expected_root() {
        let inputs = corrupt_merkle_path_inputs();
        let leaf = compute_leaf(&inputs.secret, &inputs.user_address, inputs.balance);
        let computed = verify_merkle_path(leaf, &inputs.merkle_path);
        assert_ne!(
            computed, inputs.expected_root,
            "fixture must disagree on root (do not recompute expected_root after corrupt)"
        );
    }

    #[tokio::test]
    async fn corrupt_merkle_path_execute_fails() {
        let inputs = corrupt_merkle_path_inputs();
        let mut stdin = SP1Stdin::new();
        stdin.write(&inputs);
        let client = ProverClient::from_env().await;
        // Guest panics on root mismatch. SP1 6.5 may still `Ok` with an empty
        // journal (commit never ran) instead of `Err` — both are fail-closed.
        let rejected = match client.execute(GUEST_ELF, stdin).await {
            Err(_) => true,
            Ok((pv, _)) => pv.as_slice().is_empty(),
        };
        assert!(
            rejected,
            "guest must reject bad inclusion (Err or empty journal after panic)"
        );
    }
}
