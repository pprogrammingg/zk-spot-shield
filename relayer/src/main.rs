//! Day 44+: `cargo run -p zk-spot-shield-relayer`
//!
//! Env: `RELAYER_BIND` default `127.0.0.1:8787`

use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;
use zk_spot_shield_relayer::{app_router, run_worker, AppState, ProveQueue};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let (queue, rx) = ProveQueue::new();
    let worker_q = queue.clone();
    tokio::spawn(async move { run_worker(worker_q, rx).await });

    let app = app_router(AppState { queue });
    let bind = std::env::var("RELAYER_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let addr: SocketAddr = bind.parse().expect("RELAYER_BIND host:port");
    tracing::info!(%addr, "zk-spot-shield-relayer listening");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
