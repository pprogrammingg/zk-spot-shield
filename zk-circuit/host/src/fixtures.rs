//! Day 17: single happy-path pack for host execute, prove, and (later) TS client.
//!
//! - **Runtime source:** [`happy_path_inputs`] (built with `zk-circuit-io` hashes).
//! - **Portable dump:** `zk-circuit/fixtures/happy.json` (hex fields; must match this pack).
//! - **Proof artifacts:** `zk-circuit/fixtures/happy/{journal,groth16,vkey}` for that pack only.
//!
//! Do not invent a second guest fixture. Corrupt-path tests clone this pack and mutate one field.

use std::path::{Path, PathBuf};

use zk_circuit_io::{
    compute_leaf, compute_nullifier, index0_empty_path, verify_merkle_path, PrivateInputs,
    PublicOutputs,
};

pub fn corrupt_merkle_path_inputs() -> PrivateInputs {
    let mut inputs = happy_path_inputs();
    inputs.merkle_path[0].0 = [4u8; 32];
    // leave expected_root alone
    inputs
}

/// Canonical happy-path private inputs (obvious test bytes, not real secrets).
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

/// Journal the guest must commit for [`happy_path_inputs`] (same `zk-circuit-io` hashes).
pub fn expected_public_outputs(inputs: &PrivateInputs) -> PublicOutputs {
    let leaf = compute_leaf(&inputs.secret, &inputs.user_address, inputs.balance);
    PublicOutputs {
        requested_swap_amount: inputs.requested_swap_amount,
        asset_id_mint: inputs.asset_id_mint,
        nullifier: compute_nullifier(&inputs.secret, &leaf, &inputs.asset_id_mint),
        merkle_root: inputs.expected_root,
    }
}

/// `zk-circuit/fixtures/happy/` — Groth16 + journal for [`happy_path_inputs`] only.
pub fn happy_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/happy")
}

/// Portable Day 17 dump of [`happy_path_inputs`] (hex). Kept in sync by unit test.
pub fn happy_json_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/happy.json")
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

    /// Guest ELF from `cargo prove build` / host prove path. Needed for execute.
    const GUEST_ELF: Elf = Elf::Static(include_bytes!("../../../target/elf/guest"));

    fn hex32(b: &[u8; 32]) -> String {
        format!("0x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())
    }

    fn json_string_field(raw: &str, key: &str) -> String {
        let needle = format!("\"{key}\"");
        let i = raw.find(&needle).unwrap_or_else(|| panic!("missing key {key}"));
        let after = &raw[i + needle.len()..];
        let colon = after.find(':').expect(":");
        let rest = after[colon + 1..].trim_start();
        let start = rest.find('"').expect("string value");
        let rest = &rest[start + 1..];
        let end = rest.find('"').expect("closing quote");
        rest[..end].to_string()
    }

    fn json_u64_field(raw: &str, key: &str) -> u64 {
        let needle = format!("\"{key}\"");
        let i = raw.find(&needle).unwrap_or_else(|| panic!("missing key {key}"));
        let after = &raw[i + needle.len()..];
        let colon = after.find(':').expect(":");
        let rest = after[colon + 1..].trim_start();
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        digits.parse().unwrap_or_else(|_| panic!("u64 for {key}"))
    }

    #[test]
    fn happy_fixture_journal_matches_happy_path_inputs() {
        let inputs = happy_path_inputs();
        let expected = expected_public_outputs(&inputs);
        let dir = happy_fixture_dir();
        let journal = fs::read(dir.join("journal.bin"))
            .expect("commit zk-circuit/fixtures/happy/journal.bin");
        let public = PublicOutputs::from_journal_bytes(&journal).expect("journal layout");
        assert_eq!(public.requested_swap_amount, expected.requested_swap_amount);
        assert_eq!(public.asset_id_mint, expected.asset_id_mint);
        assert_eq!(public.merkle_root, expected.merkle_root);
        assert_eq!(public.nullifier, expected.nullifier);
        let groth16 = fs::read(dir.join("groth16.bin")).expect("commit groth16.bin");
        assert_eq!(groth16.len(), 356);
    }

    #[test]
    fn happy_json_matches_happy_path_inputs() {
        let raw = fs::read_to_string(happy_json_path())
            .expect("commit zk-circuit/fixtures/happy.json (Day 17 portable pack)");
        let inputs = happy_path_inputs();
        let expected = expected_public_outputs(&inputs);

        assert_eq!(json_string_field(&raw, "secret"), hex32(&inputs.secret));
        assert_eq!(
            json_string_field(&raw, "user_address"),
            hex32(&inputs.user_address)
        );
        assert_eq!(json_u64_field(&raw, "balance"), inputs.balance);
        assert_eq!(
            json_u64_field(&raw, "requested_swap_amount"),
            inputs.requested_swap_amount
        );
        assert_eq!(
            json_string_field(&raw, "asset_id_mint"),
            hex32(&inputs.asset_id_mint)
        );
        assert_eq!(
            json_string_field(&raw, "expected_root"),
            hex32(&inputs.expected_root)
        );
        assert_eq!(json_string_field(&raw, "leaf"), {
            let leaf = compute_leaf(&inputs.secret, &inputs.user_address, inputs.balance);
            hex32(&leaf)
        });
        assert_eq!(
            json_string_field(&raw, "nullifier"),
            hex32(&expected.nullifier)
        );

        for (level, (sibling, is_right)) in inputs.merkle_path.iter().enumerate() {
            let sib_hex = hex32(sibling);
            assert!(
                raw.contains(&sib_hex),
                "happy.json missing merkle_path[{level}] sibling {sib_hex}"
            );
            assert!(
                raw.contains(&format!("\"level\": {level}")),
                "happy.json missing merkle_path level {level}"
            );
            assert!(is_right, "index0 empty path siblings are always on the right");
        }
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
