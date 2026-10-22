//! Parse settle intent, build+sign as fee payer, send with RPC retry.
//!
//! Intent JSON (UTF-8) — **pre-proven** bytes only (fixtures / offline prove).
//! Never calls Succinct / `$PROVE`.

use anchor_lang::prelude::Pubkey;
use serde::{Deserialize, Serialize};
use solana_hash::Hash;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

use crate::fees::{compute_relayer_fee, FeeConfig};
use crate::rpc_retry::{with_exponential_backoff, BackoffConfig};
use crate::tx_build::{build_settle_ix, sign_settle_as_fee_payer, SettleAccountKeys};

/// Client-submitted settle payload (proof already available).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleIntent {
    /// Base64 Groth16 proof (356 bytes decoded for happy fixture).
    pub proof_base64: String,
    /// Base64 journal (104 bytes for happy fixture).
    pub journal_base64: String,
    pub global_config: String,
    pub vault: String,
    pub clean_funds_root: String,
    pub nullifier_account: String,
    pub vault_token_a: String,
    pub vault_token_b: String,
    pub user_token_a: String,
    pub user_token_b: String,
    /// Journal amount (for fee stub); optional — defaults to 0 fee if omitted.
    #[serde(default)]
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitSettleResult {
    pub fee_payer: String,
    pub relayer_fee: u64,
    pub signature: Option<String>,
    pub dry_run: bool,
}

fn b64_decode(s: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|e| format!("base64: {e}"))
}

fn parse_pubkey(s: &str) -> Result<Pubkey, String> {
    s.parse()
        .map_err(|_| format!("invalid pubkey: {s}"))
}

impl SettleIntent {
    pub fn from_intent_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("settle intent json: {e}"))
    }

    pub fn account_keys(&self, fee_payer: Pubkey) -> Result<SettleAccountKeys, String> {
        Ok(SettleAccountKeys {
            fee_payer,
            global_config: parse_pubkey(&self.global_config)?,
            vault: parse_pubkey(&self.vault)?,
            clean_funds_root: parse_pubkey(&self.clean_funds_root)?,
            nullifier_account: parse_pubkey(&self.nullifier_account)?,
            vault_token_a: parse_pubkey(&self.vault_token_a)?,
            vault_token_b: parse_pubkey(&self.vault_token_b)?,
            user_token_a: parse_pubkey(&self.user_token_a)?,
            user_token_b: parse_pubkey(&self.user_token_b)?,
        })
    }

    pub fn proof_and_journal(&self) -> Result<(Vec<u8>, Vec<u8>), String> {
        Ok((b64_decode(&self.proof_base64)?, b64_decode(&self.journal_base64)?))
    }
}

/// Build and sign a settle tx as `fee_payer` (no RPC).
pub fn build_signed_settle(
    fee_payer: &Keypair,
    intent: &SettleIntent,
    recent_blockhash: Hash,
    fee_cfg: FeeConfig,
) -> Result<(VersionedTransaction, u64), String> {
    let keys = intent.account_keys(fee_payer.pubkey())?;
    let (proof, journal) = intent.proof_and_journal()?;
    let fee = compute_relayer_fee(intent.amount, fee_cfg);
    let tx = sign_settle_as_fee_payer(fee_payer, &keys, proof, journal, recent_blockhash)?;
    Ok((tx, fee))
}

/// Trait so unit tests can mock RPC without a live cluster.
pub trait RpcSender: Send + Sync {
    fn get_latest_blockhash(&self) -> Result<Hash, String>;
    fn send_transaction(&self, tx: &VersionedTransaction) -> Result<String, String>;
}

/// Dry-run / offline path: build+sign, return fee, no send.
pub fn process_intent_dry_run(
    fee_payer: &Keypair,
    intent_bytes: &[u8],
    fee_cfg: FeeConfig,
) -> Result<SubmitSettleResult, String> {
    let intent = SettleIntent::from_intent_bytes(intent_bytes)?;
    let keys = intent.account_keys(fee_payer.pubkey())?;
    let (proof, journal) = intent.proof_and_journal()?;
    // Validate ix builds (accounts + data).
    let _ix = build_settle_ix(&keys, proof.clone(), journal.clone());
    let fee = compute_relayer_fee(intent.amount, fee_cfg);
    let _tx = sign_settle_as_fee_payer(
        fee_payer,
        &keys,
        proof,
        journal,
        Hash::new_unique(),
    )?;
    Ok(SubmitSettleResult {
        fee_payer: fee_payer.pubkey().to_string(),
        relayer_fee: fee,
        signature: None,
        dry_run: true,
    })
}

/// Send with exponential backoff.
pub async fn process_intent_send<R: RpcSender>(
    fee_payer: &Keypair,
    intent_bytes: &[u8],
    fee_cfg: FeeConfig,
    backoff: BackoffConfig,
    rpc: &R,
) -> Result<SubmitSettleResult, String> {
    let intent = SettleIntent::from_intent_bytes(intent_bytes)?;
    let fee = compute_relayer_fee(intent.amount, fee_cfg);

    let sig = with_exponential_backoff(backoff, |_attempt| {
        let intent = &intent;
        async move {
            let blockhash = rpc.get_latest_blockhash()?;
            let (tx, _) = build_signed_settle(fee_payer, intent, blockhash, fee_cfg)?;
            rpc.send_transaction(&tx)
        }
    })
    .await?;

    Ok(SubmitSettleResult {
        fee_payer: fee_payer.pubkey().to_string(),
        relayer_fee: fee,
        signature: Some(sig),
        dry_run: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;

    #[test]
    fn dry_run_builds_with_fixture_shaped_bytes() {
        let kp = Keypair::new();
        let intent = SettleIntent {
            proof_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                vec![1u8; 356],
            ),
            journal_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                vec![2u8; 104],
            ),
            global_config: Pubkey::new_unique().to_string(),
            vault: Pubkey::new_unique().to_string(),
            clean_funds_root: Pubkey::new_unique().to_string(),
            nullifier_account: Pubkey::new_unique().to_string(),
            vault_token_a: Pubkey::new_unique().to_string(),
            vault_token_b: Pubkey::new_unique().to_string(),
            user_token_a: Pubkey::new_unique().to_string(),
            user_token_b: Pubkey::new_unique().to_string(),
            amount: 100,
        };
        let bytes = serde_json::to_vec(&intent).unwrap();
        let res = process_intent_dry_run(&kp, &bytes, FeeConfig { bps: 30, flat: 0 }).unwrap();
        assert!(res.dry_run);
        assert_eq!(res.fee_payer, kp.pubkey().to_string());
        assert_eq!(res.relayer_fee, 0); // 100 * 30 / 10000 = 0
        assert!(res.signature.is_none());
    }

    #[test]
    fn latency_fixture_to_relayer_sign() {
        use std::time::Instant;
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../zk-circuit/fixtures/happy");
        let t0 = Instant::now();
        let proof = std::fs::read(dir.join("groth16.bin")).unwrap();
        let journal = std::fs::read(dir.join("journal.bin")).unwrap();
        let load = t0.elapsed();

        let t1 = Instant::now();
        let _ = crate::pay402::accept_journal(&journal).unwrap();
        let pay402 = t1.elapsed();

        let kp = Keypair::new();
        let intent = SettleIntent {
            proof_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &proof,
            ),
            journal_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &journal,
            ),
            global_config: Pubkey::new_unique().to_string(),
            vault: Pubkey::new_unique().to_string(),
            clean_funds_root: Pubkey::new_unique().to_string(),
            nullifier_account: Pubkey::new_unique().to_string(),
            vault_token_a: Pubkey::new_unique().to_string(),
            vault_token_b: Pubkey::new_unique().to_string(),
            user_token_a: Pubkey::new_unique().to_string(),
            user_token_b: Pubkey::new_unique().to_string(),
            amount: 100,
        };
        let bytes = serde_json::to_vec(&intent).unwrap();
        let t2 = Instant::now();
        process_intent_dry_run(&kp, &bytes, FeeConfig::default()).unwrap();
        let sign = t2.elapsed();

        eprintln!(
            "latency fixture_load={load:?} pay402_accept={pay402:?} relayer_sign={sign:?}"
        );
        assert!(load.as_millis() < 500);
        assert!(sign.as_millis() < 2_000);
    }

    struct FlakyRpc {
        fails: AtomicU32,
        sent: Mutex<u32>,
    }

    impl RpcSender for FlakyRpc {
        fn get_latest_blockhash(&self) -> Result<Hash, String> {
            Ok(Hash::new_unique())
        }
        fn send_transaction(&self, _tx: &VersionedTransaction) -> Result<String, String> {
            let n = self.fails.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                return Err("rpc timeout".into());
            }
            *self.sent.lock().unwrap() += 1;
            Ok("5".repeat(64 / 5) + "abcd") // fake sig-shaped
        }
    }

    #[tokio::test]
    async fn send_retries_then_ok() {
        let kp = Keypair::new();
        let intent = SettleIntent {
            proof_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                vec![1u8; 8],
            ),
            journal_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                vec![2u8; 8],
            ),
            global_config: Pubkey::new_unique().to_string(),
            vault: Pubkey::new_unique().to_string(),
            clean_funds_root: Pubkey::new_unique().to_string(),
            nullifier_account: Pubkey::new_unique().to_string(),
            vault_token_a: Pubkey::new_unique().to_string(),
            vault_token_b: Pubkey::new_unique().to_string(),
            user_token_a: Pubkey::new_unique().to_string(),
            user_token_b: Pubkey::new_unique().to_string(),
            amount: 10_000,
        };
        let bytes = serde_json::to_vec(&intent).unwrap();
        let rpc = FlakyRpc {
            fails: AtomicU32::new(0),
            sent: Mutex::new(0),
        };
        let backoff = BackoffConfig {
            base: std::time::Duration::from_millis(1),
            max: std::time::Duration::from_millis(2),
            max_attempts: 5,
        };
        let res = process_intent_send(&kp, &bytes, FeeConfig::default(), backoff, &rpc)
            .await
            .unwrap();
        assert!(!res.dry_run);
        assert!(res.signature.is_some());
        assert_eq!(res.relayer_fee, 30);
        assert_eq!(*rpc.sent.lock().unwrap(), 1);
    }

    fn sample_intent() -> SettleIntent {
        SettleIntent {
            proof_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                vec![1u8; 356],
            ),
            journal_base64: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                vec![2u8; 104],
            ),
            global_config: Pubkey::new_unique().to_string(),
            vault: Pubkey::new_unique().to_string(),
            clean_funds_root: Pubkey::new_unique().to_string(),
            nullifier_account: Pubkey::new_unique().to_string(),
            vault_token_a: Pubkey::new_unique().to_string(),
            vault_token_b: Pubkey::new_unique().to_string(),
            user_token_a: Pubkey::new_unique().to_string(),
            user_token_b: Pubkey::new_unique().to_string(),
            amount: 100,
        }
    }

    #[tokio::test]
    async fn concurrent_dry_run_sixteen() {
        let bytes = serde_json::to_vec(&sample_intent()).unwrap();
        let t0 = std::time::Instant::now();
        let mut handles = Vec::new();
        for _ in 0..16 {
            let bytes = bytes.clone();
            handles.push(tokio::spawn(async move {
                let kp = Keypair::new();
                process_intent_dry_run(&kp, &bytes, FeeConfig::default())
            }));
        }
        for h in handles {
            let res = h.await.unwrap().unwrap();
            assert!(res.dry_run);
            assert!(res.signature.is_none());
        }
        let elapsed = t0.elapsed();
        eprintln!("concurrent 16 dry-run wall={elapsed:?}");
        assert!(elapsed.as_secs() < 5);
    }
}
