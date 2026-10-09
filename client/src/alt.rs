//! Day 32: Address Lookup Table sizing helpers for settle txs.
//!
//! Groth16 proof + journal stay in **instruction data**. ALTs only compress
//! account **pubkeys** so the packet stays under the 1232-byte MTU.

/// Solana packet data size limit (legacy / v0 message body budget).
pub const PACKET_DATA_SIZE: usize = 1232;

/// Anchor settle ix data: 8-byte disc + Borsh `Vec` proof (356) + journal (104).
pub const SETTLE_IX_DATA_LEN: usize = 8 + 4 + 356 + 4 + 104; // 476

/// Accounts in `SettleShieldedSpot` (payer … system_program).
pub const SETTLE_ACCOUNT_KEYS: usize = 11;

/// Static keys that belong in a settle ALT (reuse across many settles).
/// Per-settle keys (nullifier PDA, user ATAs, clean-root) stay as static message keys
/// or a short-lived ALT extend — see `notes/budgets.md`.
pub const SETTLE_ALT_STATIC_SLOT_LABELS: &[&str] = &[
    "token_program",
    "system_program",
    "global_config",
    "vault",
    "vault_token_a",
    "vault_token_b",
];

/// How many of the 11 settle keys we model as ALT-indexed (static set above).
pub const SETTLE_ALT_INDEXED_KEYS: usize = 6;

/// Rough legacy message byte cost for a settle tx (1 signer, CU-limit ix + settle ix).
///
/// Not a wire-perfect serializer — enough to decide when an ALT is required.
pub fn estimate_legacy_settle_message_bytes(settle_ix_data_len: usize) -> usize {
    const SIG_LEN: usize = 64;
    const HEADER: usize = 3;
    const BLOCKHASH: usize = 32;
    // Keys: payer + program + compute-budget program + 11 settle accounts
    // (some overlap in reality; treat as upper bound).
    let num_keys = 1 + 1 + 1 + SETTLE_ACCOUNT_KEYS;
    let keys = num_keys * 32;
    // CU limit ix: program idx + empty accounts + ~5 data bytes
    let cu_ix = 1 + 1 + 1 + 5;
    // Settle ix: program idx + 11 account idxs + data
    let settle_ix = 1 + 1 + SETTLE_ACCOUNT_KEYS + settle_ix_data_len;
    SIG_LEN + HEADER + keys + BLOCKHASH + cu_ix + settle_ix
}

/// Same shape with `SETTLE_ALT_INDEXED_KEYS` replaced by 1-byte ALT indexes.
pub fn estimate_v0_settle_message_bytes_with_alt(settle_ix_data_len: usize) -> usize {
    const SIG_LEN: usize = 64;
    const HEADER: usize = 3;
    const BLOCKHASH: usize = 32;
    // Remaining static keys: payer, program, compute-budget, + (11 - 6) settle keys
    let static_keys = 1 + 1 + 1 + (SETTLE_ACCOUNT_KEYS - SETTLE_ALT_INDEXED_KEYS);
    let keys = static_keys * 32;
    // One lookup table meta: table key (32) + writable/readonly index lists (~1 + n)
    let alt_meta = 32 + 1 + SETTLE_ALT_INDEXED_KEYS + 1;
    let cu_ix = 1 + 1 + 1 + 5;
    let settle_ix = 1 + 1 + SETTLE_ACCOUNT_KEYS + settle_ix_data_len;
    SIG_LEN + HEADER + keys + BLOCKHASH + alt_meta + cu_ix + settle_ix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settle_ix_data_matches_budget_note() {
        assert_eq!(SETTLE_IX_DATA_LEN, 476);
    }

    #[test]
    fn legacy_settle_pressures_packet_budget() {
        let legacy = estimate_legacy_settle_message_bytes(SETTLE_IX_DATA_LEN);
        // Upper-bound estimate should show why Day 32 ALTs exist.
        assert!(
            legacy > 900,
            "legacy settle estimate unexpectedly small: {legacy}"
        );
    }

    #[test]
    fn alt_settle_fits_under_packet_mtu() {
        let with_alt = estimate_v0_settle_message_bytes_with_alt(SETTLE_IX_DATA_LEN);
        assert!(
            with_alt < PACKET_DATA_SIZE,
            "ALT settle estimate {with_alt} must fit under {PACKET_DATA_SIZE}"
        );
        let legacy = estimate_legacy_settle_message_bytes(SETTLE_IX_DATA_LEN);
        assert!(
            with_alt < legacy,
            "ALT path ({with_alt}) should beat legacy ({legacy})"
        );
    }

    #[test]
    fn alt_static_slot_count_matches_constant() {
        assert_eq!(
            SETTLE_ALT_STATIC_SLOT_LABELS.len(),
            SETTLE_ALT_INDEXED_KEYS
        );
    }
}
