//! Axum `/health` + `/submit-intent` queue, settle tx as fee payer,
//! fee stub + RPC backoff, HTTP 402 payer (fixtures — no `$PROVE`).

pub mod fees;
pub mod pay402;
pub mod queue;
pub mod routes;
pub mod rpc_retry;
pub mod submit;
pub mod tx_build;

pub use fees::{compute_relayer_fee, FeeConfig};
pub use pay402::{quote, PayLedger, QUOTE_AMOUNT};
pub use queue::{run_worker, JobStatus, ProveJob, ProveQueue};
pub use routes::{app_router, AppState};
pub use rpc_retry::{backoff_delay, with_exponential_backoff, BackoffConfig};
pub use submit::{
    build_signed_settle, process_intent_dry_run, process_intent_send, SettleIntent,
    SubmitSettleResult,
};
pub use tx_build::{build_settle_ix, sign_settle_as_fee_payer, SettleAccountKeys};
