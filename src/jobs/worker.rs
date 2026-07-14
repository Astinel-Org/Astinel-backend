use crate::jobs::queue::JobQueue;
use crate::jobs::scan_job::ScanJobExecutor;
use std::sync::Arc;
use tracing::{error, info, instrument};

pub struct WorkerPool {
    queue: JobQueue,
    executor: Arc<ScanJobExecutor>,
    worker_count: usize,
}

impl WorkerPool {
    pub fn new(queue: JobQueue, executor: Arc<ScanJobExecutor>, worker_count: usize) -> Self {
        Self {
            queue,
            executor,
            worker_count,
        }
    }

    pub async fn start(&self) {
        for i in 0..self.worker_count {
            let queue = self.queue.clone();
            let executor = self.executor.clone();
            tokio::spawn(async move {
                Worker::new(i, queue, executor).run().await;
            });
        }
        info!("Started {} workers", self.worker_count);
    }
}

struct Worker {
    id: usize,
    queue: JobQueue,
    executor: Arc<ScanJobExecutor>,
}

impl Worker {
    fn new(id: usize, queue: JobQueue, executor: Arc<ScanJobExecutor>) -> Self {
        Self {
            id,
            queue,
            executor,
        }
    }

    #[instrument(skip(self), fields(worker_id = self.id))]
    async fn run(&self) {
        loop {
            if let Some(job) = self.queue.dequeue().await {
                info!("Worker {} processing job {}", self.id, job.id);
                if let Err(e) = self.executor.execute(job).await {
                    error!("Worker {} job failed: {}", self.id, e);
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}
