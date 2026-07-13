use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;
use crate::jobs::status::JobStatus;

#[derive(Debug, Clone)]
pub struct QueuedJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub branch: String,
    pub status: JobStatus,
    pub config: serde_json::Value,
}

#[derive(Clone)]
pub struct JobQueue {
    inner: Arc<Mutex<JobQueueInner>>,
    notifier: mpsc::Sender<Uuid>,
}

struct JobQueueInner {
    jobs: VecDeque<QueuedJob>,
}

impl JobQueue {
    pub fn new() -> (Self, mpsc::Receiver<Uuid>) {
        let (tx, rx) = mpsc::channel(256);
        let queue = Self {
            inner: Arc::new(Mutex::new(JobQueueInner {
                jobs: VecDeque::new(),
            })),
            notifier: tx,
        };
        (queue, rx)
    }

    pub async fn enqueue(&self, job: QueuedJob) -> Result<(), ()> {
        let mut inner = self.inner.lock().await;
        inner.jobs.push_back(job);
        let _ = self.notifier.try_send(Uuid::new_v4());
        Ok(())
    }

    pub async fn dequeue(&self) -> Option<QueuedJob> {
        let mut inner = self.inner.lock().await;
        inner.jobs.pop_front()
    }

    pub async fn len(&self) -> usize {
        self.inner.lock().await.jobs.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}
