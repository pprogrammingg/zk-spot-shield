//! In-memory prove job queue (Day 45). Worker is a stub until Day 46 tx build.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex, Notify};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    /// Stub success — no Groth16 yet (Day 46+).
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveJob {
    pub id: Uuid,
    pub status: JobStatus,
    /// Opaque private intent bytes (client-packed; not logged in full).
    #[serde(skip_serializing)]
    pub intent: Vec<u8>,
    pub intent_len: usize,
    pub error: Option<String>,
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

    pub async fn set_status(&self, id: Uuid, status: JobStatus, error: Option<String>) {
        let mut map = self.inner.lock().await;
        if let Some(job) = map.get_mut(&id) {
            job.status = status;
            job.error = error;
        }
    }
}

impl Default for ProveQueue {
    fn default() -> Self {
        Self::new().0
    }
}

/// Background worker: mark jobs Running → Done (prove stub).
pub async fn run_worker(queue: ProveQueue, mut rx: mpsc::UnboundedReceiver<Uuid>) {
    while let Some(id) = rx.recv().await {
        queue.set_status(id, JobStatus::Running, None).await;
        // Day 46 will call host prove + build tx here.
        tokio::task::yield_now().await;
        queue.set_status(id, JobStatus::Done, None).await;
        tracing::info!(%id, "prove job stub complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enqueue_and_worker_marks_done() {
        let (queue, rx) = ProveQueue::new();
        let q = queue.clone();
        let worker = tokio::spawn(async move { run_worker(q, rx).await });

        let job = queue.enqueue(vec![1, 2, 3]).await;
        assert_eq!(job.status, JobStatus::Queued);
        assert_eq!(job.intent_len, 3);

        // Poll until Done (worker is cooperative).
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
