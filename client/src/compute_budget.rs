//! Day 30: compute-budget ix helpers for settle-shaped transactions.

use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_instruction::Instruction;

/// Happy-path settle needs Groth16 verify (~280–350k CU) plus vault/SPL/nullifier.
/// Leave headroom under the ~1.4M tx cap (`notes/budgets.md`).
pub const SETTLE_CU_LIMIT: u32 = 1_400_000;

/// Optional priority fee (micro-lamports per CU). `0` = no price ix.
pub const SETTLE_CU_PRICE_MICRO_LAMPORTS: u64 = 0;

/// Recommended CU limit for a settle tx (compute-budget ix only).
pub fn settle_cu_limit_ix() -> Instruction {
    ComputeBudgetInstruction::set_compute_unit_limit(SETTLE_CU_LIMIT)
}

/// Optional CU price ix when `SETTLE_CU_PRICE_MICRO_LAMPORTS > 0`.
pub fn settle_cu_price_ix() -> Option<Instruction> {
    if SETTLE_CU_PRICE_MICRO_LAMPORTS == 0 {
        None
    } else {
        Some(ComputeBudgetInstruction::set_compute_unit_price(
            SETTLE_CU_PRICE_MICRO_LAMPORTS,
        ))
    }
}

/// Prefixed compute-budget ixs for a settle transaction (limit, then optional price).
pub fn settle_compute_budget_ixs() -> Vec<Instruction> {
    let mut ixs = vec![settle_cu_limit_ix()];
    if let Some(price) = settle_cu_price_ix() {
        ixs.push(price);
    }
    ixs
}

/// Build a settle-shaped ix list: compute budget first, then the settle ix.
pub fn with_settle_compute_budget(settle_ix: Instruction) -> Vec<Instruction> {
    let mut ixs = settle_compute_budget_ixs();
    ixs.push(settle_ix);
    ixs
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_pubkey::Pubkey;

    #[test]
    fn settle_budget_includes_limit() {
        let ixs = settle_compute_budget_ixs();
        assert_eq!(ixs.len(), 1);
        assert_eq!(ixs[0], settle_cu_limit_ix());
        assert_eq!(SETTLE_CU_LIMIT, 1_400_000);
    }

    #[test]
    fn with_settle_compute_budget_prefixes_settle() {
        let settle = Instruction {
            program_id: Pubkey::new_from_array([7u8; 32]),
            accounts: vec![],
            data: vec![9, 9],
        };
        let ixs = with_settle_compute_budget(settle.clone());
        assert_eq!(ixs.len(), 2);
        assert_eq!(ixs[0], settle_cu_limit_ix());
        assert_eq!(ixs[1].data, settle.data);
        assert_eq!(ixs[1].program_id, settle.program_id);
    }
}
