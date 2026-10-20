//! Day 44–47: `cargo run -p zk-spot-shield-relayer`
//!
//! Env:
//! - `RELAYER_BIND` default `127.0.0.1:8787`
//! - `RELAYER_FEE_PAYER` path to JSON keypair (optional; dry-run settle when set)
//! - `RELAYER_FEE_BPS` / `RELAYER_FEE_FLAT` (Day 47 fee stub)

use std::net::SocketAddr;
use std::sync::Arc;

use solana_keypair::Keypair;
use tracing_subscriber::EnvFilter;
use zk_spot_shield_relayer::{
    app_router, run_worker, AppState, FeeConfig, ProveQueue,
};

fn load_fee_payer() -> Option<Arc<Keypair>> {
    let path = std::env::var("RELAYER_FEE_PAYER").ok()?;
    let bytes = std::fs::read_to_string(&path).ok()?;
    let secret: Vec<u8> = serde_json::from_str(&bytes).ok()?;
    // Solana CLI JSON keypair is 64 bytes (secret || pubkey).
    Keypair::try_from(secret.as_slice()).ok().map(Arc::new)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let fee_cfg = FeeConfig::from_env();
    let fee_payer = load_fee_payer();
    if fee_payer.is_some() {
        tracing::info!("fee payer key loaded (settle dry-run enabled)");
    } else {
        tracing::warn!("RELAYER_FEE_PAYER unset — opaque intents only");
    }

    let (queue, rx) = ProveQueue::new();
    let worker_q = queue.clone();
    let worker_kp = fee_payer.clone();
    tokio::spawn(async move { run_worker(worker_q, rx, worker_kp, fee_cfg).await });

    let app = app_router(AppState {
        queue,
        pay: zk_spot_shield_relayer::PayLedger::default(),
    });
    let bind = std::env::var("RELAYER_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let addr: SocketAddr = bind.parse().expect("RELAYER_BIND host:port");
    tracing::info!(%addr, "zk-spot-shield-relayer listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
