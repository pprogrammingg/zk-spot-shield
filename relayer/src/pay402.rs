//! Day 50: HTTP 402 shielded payer (P1).
//!
//! Agent hits a paid resource → `402` quote → presents a settle **journal**
//! (amount + mint + nullifier). Unlock is keyed by **nullifier**, not a wallet.
//! Tests use committed journal fixtures — no `$PROVE`.

use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use zk_spot_shield::journal::{JournalPublic, JOURNAL_BYTE_LEN};

/// Happy-path quote: fixture spend of 100 of mint `[3u8; 32]`.
pub const QUOTE_AMOUNT: u64 = 100;
pub const QUOTE_MINT: [u8; 32] = [3u8; 32];
pub const PAID_BODY: &str = "shielded-402-ok";

#[derive(Clone, Default)]
pub struct PayLedger {
    inner: Arc<Mutex<HashSet<[u8; 32]>>>,
}

impl PayLedger {
    pub async fn is_paid(&self, nullifier: &[u8; 32]) -> bool {
        self.inner.lock().await.contains(nullifier)
    }

    pub async fn mark_paid(&self, nullifier: [u8; 32]) {
        self.inner.lock().await.insert(nullifier);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentQuote {
    pub amount: u64,
    pub mint_hex: String,
    pub pay_via: &'static str,
    pub hint: &'static str,
}

pub fn quote() -> PaymentQuote {
    PaymentQuote {
        amount: QUOTE_AMOUNT,
        mint_hex: hex_encode(&QUOTE_MINT),
        pay_via: "zk_spot_shield.settle_shielded_spot",
        hint: "POST /402/pay with journal_base64 from a vault-only settle (nullifier path)",
    }
}

#[derive(Deserialize)]
pub struct PayRequest {
    /// Base64 (or `0x` hex) of the 104-byte settle journal.
    pub journal_base64: String,
}

#[derive(Serialize)]
pub struct PayResponse {
    pub paid: bool,
    pub nullifier_hex: String,
    pub amount: u64,
}

#[derive(Debug)]
pub enum PayError {
    Decode(String),
    Journal,
    Amount,
    Mint,
}

impl PayError {
    pub fn status_msg(&self) -> (axum::http::StatusCode, String) {
        match self {
            PayError::Decode(s) => (axum::http::StatusCode::BAD_REQUEST, s.clone()),
            PayError::Journal => (
                axum::http::StatusCode::BAD_REQUEST,
                "invalid journal".into(),
            ),
            PayError::Amount | PayError::Mint => (
                axum::http::StatusCode::PAYMENT_REQUIRED,
                "journal does not match quote".into(),
            ),
        }
    }
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::from("0x");
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub fn hex_decode_32(s: &str) -> Option<[u8; 32]> {
    let t = s.trim();
    let t = t
        .strip_prefix("0x")
        .or_else(|| t.strip_prefix("0X"))
        .unwrap_or(t);
    if t.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&t[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn decode_journal_bytes(s: &str) -> Result<Vec<u8>, PayError> {
    let t = s.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        if !hex.len().is_multiple_of(2) {
            return Err(PayError::Decode("hex length must be even".into()));
        }
        let mut out = Vec::with_capacity(hex.len() / 2);
        for i in (0..hex.len()).step_by(2) {
            let b = u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| PayError::Decode("invalid hex".into()))?;
            out.push(b);
        }
        return Ok(out);
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(t)
        .map_err(|e| PayError::Decode(format!("base64: {e}")))
}

/// Accept a settle journal as payment. Does **not** look at any wallet pubkey.
pub fn accept_journal(bytes: &[u8]) -> Result<JournalPublic, PayError> {
    if bytes.len() != JOURNAL_BYTE_LEN {
        return Err(PayError::Journal);
    }
    let public = JournalPublic::from_journal_bytes(bytes).ok_or(PayError::Journal)?;
    if public.requested_swap_amount != QUOTE_AMOUNT {
        return Err(PayError::Amount);
    }
    if public.asset_id_mint != QUOTE_MINT {
        return Err(PayError::Mint);
    }
    Ok(public)
}

pub fn parse_pay_body(journal_base64: &str) -> Result<JournalPublic, PayError> {
    let bytes = decode_journal_bytes(journal_base64)?;
    accept_journal(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn happy_journal() -> Vec<u8> {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../zk-circuit/fixtures/happy/journal.bin");
        std::fs::read(p).expect("journal.bin")
    }

    #[test]
    fn happy_journal_unlocks_quote() {
        let j = happy_journal();
        let public = accept_journal(&j).expect("fixture must match quote");
        assert_eq!(public.requested_swap_amount, 100);
        assert_eq!(public.asset_id_mint, QUOTE_MINT);
        assert_ne!(public.nullifier, [0u8; 32]);
    }

    #[test]
    fn wrong_amount_rejected() {
        let mut j = happy_journal();
        j[0..8].copy_from_slice(&50u64.to_le_bytes());
        assert!(matches!(accept_journal(&j), Err(PayError::Amount)));
    }

    #[test]
    fn garbage_rejected() {
        assert!(accept_journal(&[1, 2, 3]).is_err());
    }
}
