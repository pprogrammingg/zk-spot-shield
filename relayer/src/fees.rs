//! Relayer fee stub (bps of settle amount) for treasury refill.

/// Flat + bps fee model. Token units match journal `requested_swap_amount`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeConfig {
    /// Basis points of settle amount (10_000 = 100%).
    pub bps: u16,
    /// Flat fee in the same units as amount (often 0).
    pub flat: u64,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            bps: 30, // 0.30%
            flat: 0,
        }
    }
}

impl FeeConfig {
    /// From env: `RELAYER_FEE_BPS` (default 30), `RELAYER_FEE_FLAT` (default 0).
    pub fn from_env() -> Self {
        let bps = std::env::var("RELAYER_FEE_BPS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        let flat = std::env::var("RELAYER_FEE_FLAT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Self { bps, flat }
    }
}

/// Compute user fee charged against payout / deposit to refill relayer SOL + prove credits.
/// Saturates at `amount` (never returns more than the settle amount).
pub fn compute_relayer_fee(amount: u64, cfg: FeeConfig) -> u64 {
    if amount == 0 {
        return 0;
    }
    let bps_part = (amount as u128)
        .saturating_mul(cfg.bps as u128)
        .saturating_div(10_000);
    let total = bps_part.saturating_add(cfg.flat as u128);
    total.min(amount as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_bps_and_flat() {
        assert_eq!(
            compute_relayer_fee(10_000, FeeConfig { bps: 30, flat: 0 }),
            30
        );
        assert_eq!(
            compute_relayer_fee(100, FeeConfig { bps: 0, flat: 5 }),
            5
        );
        assert_eq!(
            compute_relayer_fee(100, FeeConfig { bps: 10_000, flat: 0 }),
            100
        );
        assert_eq!(compute_relayer_fee(0, FeeConfig::default()), 0);
    }

    #[test]
    fn fee_never_exceeds_amount() {
        assert_eq!(
            compute_relayer_fee(10, FeeConfig { bps: 0, flat: 999 }),
            10
        );
    }
}
