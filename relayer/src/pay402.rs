//! HTTP 402 shielded payer (P1).
//!
//! Agent hits a paid resource → `402` quote → presents a settle **journal**
//! (amount + mint + nullifier). Unlock is keyed by **nullifier**, not a wallet.
//! Tests use committed journal fixtures — no `$PROVE`.
//!
//! Default quote is the localnet fixture (100 / mint `[3u8; 32]`). Override per
//! request: `GET /402/resource?amount=&mint=0x…`. Process defaults:
//! `QUOTE_AMOUNT`, `QUOTE_MINT` (hex) — use vault mint A on Devnet.

use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use zk_spot_shield::journal::JournalPublic;

/// Localnet happy-path fixture: spend 100 of mint `[3u8; 32]`.
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

/// Outstanding 402 price. GET updates it; POST must match.
#[derive(Clone)]
pub struct QuoteSlot {
    inner: Arc<Mutex<(u64, [u8; 32])>>,
}

impl Default for QuoteSlot {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new((QUOTE_AMOUNT, QUOTE_MINT))),
        }
    }
}

impl QuoteSlot {
    /// `QUOTE_AMOUNT` (u64) and `QUOTE_MINT` (0x-hex 32 bytes). Missing → fixture.
    pub fn from_env() -> Self {
        let amount = std::env::var("QUOTE_AMOUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|n: &u64| *n > 0)
            .unwrap_or(QUOTE_AMOUNT);
        let mint = std::env::var("QUOTE_MINT")
            .ok()
            .and_then(|s| hex_decode_32(&s))
            .unwrap_or(QUOTE_MINT);
        Self {
            inner: Arc::new(Mutex::new((amount, mint))),
        }
    }

    pub async fn get(&self) -> (u64, [u8; 32]) {
        *self.inner.lock().await
    }

    pub async fn set(&self, amount: u64, mint: [u8; 32]) {
        *self.inner.lock().await = (amount, mint);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentQuote {
    pub amount: u64,
    pub mint_hex: String,
    pub pay_via: &'static str,
    pub hint: &'static str,
    /// Demo: HTTP 402 is the product, not a wallet checkout.
    pub http_status: u16,
    pub unlock: &'static str,
    pub contrast: &'static str,
}

pub fn quote() -> PaymentQuote {
    quote_for(QUOTE_AMOUNT, QUOTE_MINT)
}

pub fn quote_for(amount: u64, mint: [u8; 32]) -> PaymentQuote {
    PaymentQuote {
        amount,
        mint_hex: hex_encode(&mint),
        pay_via: "zk_spot_shield.settle_shielded_spot",
        hint: "POST /402/pay with journal_base64 from a vault-only settle (nullifier path)",
        http_status: 402,
        unlock: "nullifier — not a wallet",
        contrast: "machines settle a 402 invoice without a wallet on the receipt — x402 / Stripe MPP show a payer pubkey; fee payer is not the customer",
    }
}

/// Stripe / 402 demo line. Fee payer signs; customer is the note.
pub const PUNCHLINE: &str = "machines settle a 402 invoice without a wallet on the receipt";

fn ok_miss(ok: bool) -> &'static str {
    if ok {
        "MATCH"
    } else {
        "MISS"
    }
}

/// First 10 hex chars after `0x` (demo logs).
pub fn short_hex(bytes: &[u8]) -> String {
    let full = hex_encode(bytes);
    if full.len() > 12 {
        format!("{}…", &full[..12])
    } else {
        full
    }
}

pub fn quote_banner(amount: u64, mint: &[u8; 32]) -> String {
    format!(
        "\n—— 402 PAYMENT REQUIRED ————————————————\n  \
         merchant  zk-spot-shield\n  \
         price     {amount}  mint {}\n  \
         unlock    settle journal (nullifier)\n  \
         customer  private note — not a wallet, not the fee payer\n  \
         punch     {PUNCHLINE}\n  \
         next      POST /402/pay after vault settle\n\
         ————————————————————————————————————————",
        short_hex(mint)
    )
}

pub fn pay_banner(amount: u64, nullifier: &[u8; 32]) -> String {
    format!(
        "\n—— 402 PAID —————————————————————————————\n  \
         receipt   nullifier {}\n  \
         full      {}\n  \
         amount    {amount}\n  \
         wallets   none in this receipt\n  \
         fee payer signed the Solana tx — not the customer\n  \
         customer  the private note, identified only by the nullifier\n  \
         punch     {PUNCHLINE}\n  \
         resource  GET /402/resource?nullifier={}\n\
         ————————————————————————————————————————",
        short_hex(nullifier),
        hex_encode(nullifier),
        hex_encode(nullifier)
    )
}

pub fn delivered_banner(nullifier: &[u8; 32]) -> String {
    format!(
        "\n—— 402 RESOURCE DELIVERED ———————————————\n  \
         paid with nullifier {}\n  \
         body      {PAID_BODY}\n\
         ————————————————————————————————————————",
        short_hex(nullifier)
    )
}

pub fn listen_banner(addr: &str) -> String {
    format!(
        "\n—— 402 MERCHANT —————————————————————————\n  \
         listen    {addr}\n  \
         GET  /402/resource  → HTTP 402 quote (nullifier, not a wallet)\n  \
         POST /402/pay       → journal in; match quote; unlock on nullifier\n  \
         punch     {PUNCHLINE}\n  \
         fee payer signs the Solana tx — not the customer\n  \
         customer  private note, identified only by the nullifier\n\
         ————————————————————————————————————————"
    )
}

pub fn receipt_lookup_banner(nullifier: &[u8; 32], paid: bool) -> String {
    if paid {
        return delivered_banner(nullifier);
    }
    format!(
        "\n—— 402 RECEIPT LOOKUP ———————————————————\n  \
         attempted {}\n  \
         ledger    not paid yet — still quoting\n  \
         wallets   not used for this lookup\n\
         ————————————————————————————————————————",
        hex_encode(nullifier)
    )
}

pub fn journal_decode_banner(quote_amount: u64, quote_mint: &[u8; 32], why: &str) -> String {
    format!(
        "\n—— 402 JOURNAL RECEIVED ————————————————\n  \
         quote     amount {quote_amount}  mint {}\n  \
         journal   decode failed ({why})\n  \
         nullifier (none — not a settle journal)\n  \
         result    REJECT\n\
         ————————————————————————————————————————",
        short_hex(quote_mint)
    )
}

pub fn journal_match_banner(
    quote_amount: u64,
    quote_mint: &[u8; 32],
    bytes_len: usize,
    got: Option<&JournalPublic>,
) -> String {
    let Some(p) = got else {
        return format!(
            "\n—— 402 JOURNAL RECEIVED ————————————————\n  \
             quote     amount {quote_amount}  mint {}\n  \
             journal   {bytes_len} bytes — not 104/144-byte settle journal\n  \
             nullifier (none)\n  \
             result    REJECT\n\
             ————————————————————————————————————————",
            short_hex(quote_mint)
        );
    };
    let amt_ok = p.requested_swap_amount == quote_amount;
    let mint_ok = p.asset_id_mint == *quote_mint;
    let result = if amt_ok && mint_ok {
        "UNLOCK — 402 invoice settled on nullifier"
    } else {
        "REJECT — journal does not match quote"
    };
    format!(
        "\n—— 402 JOURNAL RECEIVED ————————————————\n  \
         quote     amount {quote_amount}  mint {}\n  \
         journal   {bytes_len} bytes\n  \
         amount    {}  {}\n  \
         mint      {}  {}\n  \
         nullifier {}  ← customer (not a wallet)\n  \
         full      {}\n  \
         root      {}\n  \
         change    {}\n  \
         fee payer not on this receipt\n  \
         result    {result}\n\
         ————————————————————————————————————————",
        short_hex(quote_mint),
        p.requested_swap_amount,
        ok_miss(amt_ok),
        short_hex(&p.asset_id_mint),
        ok_miss(mint_ok),
        short_hex(&p.nullifier),
        hex_encode(&p.nullifier),
        short_hex(&p.merkle_root),
        p.change_amount
    )
}

/// Decode + match a pay body. Banner always names quote vs journal vs nullifier.
pub fn describe_pay_attempt(
    journal_base64: &str,
    quote_amount: u64,
    quote_mint: [u8; 32],
) -> (String, Result<JournalPublic, PayError>) {
    let bytes = match decode_journal_bytes(journal_base64) {
        Ok(b) => b,
        Err(e) => {
            let why = match &e {
                PayError::Decode(s) => s.as_str(),
                _ => "decode failed",
            };
            return (
                journal_decode_banner(quote_amount, &quote_mint, why),
                Err(e),
            );
        }
    };
    let parsed = JournalPublic::from_journal_bytes(&bytes);
    let banner = journal_match_banner(
        quote_amount,
        &quote_mint,
        bytes.len(),
        parsed.as_ref(),
    );
    (
        banner,
        accept_journal_quote(&bytes, quote_amount, quote_mint),
    )
}

#[derive(Deserialize)]
pub struct PayRequest {
    /// Base64 (or `0x` hex) of a 104- or 144-byte settle journal.
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
    accept_journal_quote(bytes, QUOTE_AMOUNT, QUOTE_MINT)
}

pub fn accept_journal_quote(
    bytes: &[u8],
    amount: u64,
    mint: [u8; 32],
) -> Result<JournalPublic, PayError> {
    let public = JournalPublic::from_journal_bytes(bytes).ok_or(PayError::Journal)?;
    if public.requested_swap_amount != amount {
        return Err(PayError::Amount);
    }
    if public.asset_id_mint != mint {
        return Err(PayError::Mint);
    }
    Ok(public)
}

pub fn parse_pay_body(journal_base64: &str) -> Result<JournalPublic, PayError> {
    parse_pay_body_quote(journal_base64, QUOTE_AMOUNT, QUOTE_MINT)
}

pub fn parse_pay_body_quote(
    journal_base64: &str,
    amount: u64,
    mint: [u8; 32],
) -> Result<JournalPublic, PayError> {
    let bytes = decode_journal_bytes(journal_base64)?;
    accept_journal_quote(&bytes, amount, mint)
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

    #[test]
    fn custom_amount_and_mint_unlock() {
        let mut j = happy_journal();
        j[0..8].copy_from_slice(&100_000u64.to_le_bytes());
        let mint = [0xab; 32];
        j[8..40].copy_from_slice(&mint);
        let public = accept_journal_quote(&j, 100_000, mint).expect("match");
        assert_eq!(public.requested_swap_amount, 100_000);
        assert_eq!(public.asset_id_mint, mint);
        assert!(accept_journal_quote(&j, 100, mint).is_err());
        assert!(accept_journal_quote(&j, 100_000, QUOTE_MINT).is_err());
    }

    #[test]
    fn demo_banners_are_402_not_wallet() {
        let mint = [0xab; 32];
        let nf = [0xcd; 32];
        let q = quote_banner(12, &mint);
        assert!(q.contains("402 PAYMENT REQUIRED"));
        assert!(q.contains("12"));
        assert!(q.contains("nullifier"));
        assert!(q.contains(PUNCHLINE));
        assert!(q.contains("not the fee payer"));
        let p = pay_banner(12, &nf);
        assert!(p.contains("402 PAID"));
        assert!(p.contains("none in this receipt"));
        assert!(p.contains(&hex_encode(&nf)));
        assert!(!p.to_lowercase().contains("keypair"));
        let listen = listen_banner("127.0.0.1:8787");
        assert!(listen.contains("402 MERCHANT"));
        assert!(listen.contains("8787"));
        assert!(listen.contains(PUNCHLINE));
        let d = delivered_banner(&nf);
        assert!(d.contains("RESOURCE DELIVERED"));
        assert!(d.contains(PAID_BODY));
        let quote = quote_for(12, mint);
        assert_eq!(quote.http_status, 402);
        assert!(quote.unlock.contains("nullifier"));
        assert!(quote.contrast.contains("x402"));
        assert!(quote.contrast.contains(PUNCHLINE));
        let miss = receipt_lookup_banner(&nf, false);
        assert!(miss.contains("not paid yet"));
        assert!(miss.contains(&hex_encode(&nf)));
    }

    #[test]
    fn journal_match_banner_shows_quote_vs_got() {
        let j = happy_journal();
        let public = JournalPublic::from_journal_bytes(&j).unwrap();
        let ok = journal_match_banner(100, &QUOTE_MINT, j.len(), Some(&public));
        assert!(ok.contains("402 JOURNAL RECEIVED"));
        assert!(ok.contains("MATCH"));
        assert!(ok.contains("UNLOCK"));
        assert!(ok.contains(&hex_encode(&public.nullifier)));
        assert!(ok.contains("customer (not a wallet)"));
        let miss = journal_match_banner(12, &QUOTE_MINT, j.len(), Some(&public));
        assert!(miss.contains("MISS"));
        assert!(miss.contains("REJECT"));
        let (banner, err) = describe_pay_attempt("not-journal", 100, QUOTE_MINT);
        assert!(err.is_err());
        assert!(banner.contains("decode failed") || banner.contains("REJECT"));
    }
}
