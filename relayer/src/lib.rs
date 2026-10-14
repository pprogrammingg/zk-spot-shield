//! Days 44–45: Axum health + `/submit-proof` job queue (prove/tx wiring is Day 46+).

pub mod queue;
pub mod routes;

pub use queue::{run_worker, JobStatus, ProveJob, ProveQueue};
pub use routes::{app_router, AppState};
