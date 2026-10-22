//! In-memory prove/settle job queue.
//!
//! Worker dry-runs settle tx build with the configured fee-payer key (no `$PROVE`).

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use solana_keypair::Keypair;
use tokio::sync::{mpsc, Mutex, Notify};
use uuid::Uuid;

use crate::fees::FeeConfig;
use crate::submit::process_intent_dry_run;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    /// Settle tx built+signed (dry-run) or send stub success.
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveJob {
    pub id: Uuid,
    pub status: JobStatus,
    /// Opaque private intent bytes (JSON SettleIntent or raw witness).
    #[serde(skip_serializing)]
    pub intent: Vec<u8>,
    pub intent_len: usize,
    pub error: Option<String>,
    pub relayer_fee: Option<u64>,
}

impl ProveJob {
    pub fn new(intent: Vec<u8>) -> Self {
        let intent_len = intent.len();
        Self {
            id: Uuid::new_v4(),
            status: JobStatus::Queued,
            intent,
            intent_len,
            error: None,
            relayer_fee: None,
        }
    }
}

#[derive(Clone)]
pub struct ProveQueue {
    inner: Arc<Mutex<HashMap<Uuid, ProveJob>>>,
    tx: mpsc::UnboundedSender<Uuid>,
    notify: Arc<Notify>,
}

impl ProveQueue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Uuid>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                inner: Arc::new(Mutex::new(HashMap::new())),
                tx,
                notify: Arc::new(Notify::new()),
            },
            rx,
        )
    }

    pub async fn enqueue(&self, intent: Vec<u8>) -> ProveJob {
        let job = ProveJob::new(intent);
        let id = job.id;
        {
            let mut map = self.inner.lock().await;
            map.insert(id, job.clone());
        }
        let _ = self.tx.send(id);
        self.notify.notify_one();
        job
    }

    pub async fn get(&self, id: Uuid) -> Option<ProveJob> {
        self.inner.lock().await.get(&id).cloned()
    }

    pub async fn set_status(
        &self,
        id: Uuid,
        status: JobStatus,
        error: Option<String>,
        relayer_fee: Option<u64>,
    ) {
        let mut map = self.inner.lock().await;
        if let Some(job) = map.get_mut(&id) {
            job.status = status;
            job.error = error;
            if relayer_fee.is_some() {
                job.relayer_fee = relayer_fee;
            }
        }
    }
}

impl Default for ProveQueue {
    fn default() -> Self {
        Self::new().0
    }
}

/// Background worker: dry-run settle as fee payer when intent is JSON; else mark Done.
pub async fn run_worker(
    queue: ProveQueue,
    mut rx: mpsc::UnboundedReceiver<Uuid>,
    fee_payer: Option<Arc<Keypair>>,
    fee_cfg: FeeConfig,
) {
    while let Some(id) = rx.recv().await {
        queue
            .set_status(id, JobStatus::Running, None, None)
            .await;
        let intent = match queue.get(id).await {
            Some(j) => j.intent,
            None => continue,
        };

        let result = if let Some(ref kp) = fee_payer {
            process_intent_dry_run(kp, &intent, fee_cfg)
        } else {
            // No key configured — accept enqueue only.
            Ok(crate::submit::SubmitSettleResult {
                fee_payer: "unconfigured".into(),
                relayer_fee: crate::fees::compute_relayer_fee(0, fee_cfg),
                signature: None,
                dry_run: true,
            })
        };

        match result {
            Ok(r) => {
                queue
                    .set_status(id, JobStatus::Done, None, Some(r.relayer_fee))
                    .await;
                tracing::info!(%id, fee = r.relayer_fee, "settle intent dry-run ok");
            }
            Err(e) => {
                // Non-JSON intents stay Done without fee (witness-only until prove wired).
                if intent.first() == Some(&b'{') {
                    tracing::warn!(%id, error = %e, "settle intent failed");
                    queue
                        .set_status(id, JobStatus::Failed, Some(e), None)
                        .await;
                } else {
                    queue
                        .set_status(id, JobStatus::Done, None, None)
                        .await;
                    tracing::info!(%id, "opaque intent queued (prove stub)");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enqueue_and_worker_marks_done_opaque() {
        let (queue, rx) = ProveQueue::new();
        let q = queue.clone();
        let worker = tokio::spawn(async move {
            run_worker(q, rx, None, FeeConfig::default()).await
        });

        let job = queue.enqueue(vec![1, 2, 3]).await;
        assert_eq!(job.status, JobStatus::Queued);

        let mut seen = JobStatus::Queued;
        for _ in 0..50 {
            tokio::task::yield_now().await;
            if let Some(j) = queue.get(job.id).await {
                seen = j.status.clone();
                if seen == JobStatus::Done {
                    break;
                }
            }
        }
        assert_eq!(seen, JobStatus::Done);
        worker.abort();
    }
}
