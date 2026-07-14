use crate::cache::redis::RedisPool;
use crate::jobs::status::JobStatus;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueuedJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub branch: String,
    pub status: JobStatus,
    pub config: serde_json::Value,
}

#[derive(Clone)]
pub struct JobQueue {
    redis: RedisPool,
    queue_key: String,
}

impl JobQueue {
    pub fn new(redis: RedisPool) -> Self {
        Self {
            redis,
            queue_key: "queue:scans".to_string(),
        }
    }

    pub async fn enqueue(&self, job: QueuedJob) -> Result<(), String> {
        let payload = serde_json::to_string(&job).map_err(|e| e.to_string())?;
        let mut con = self.redis.con.clone();
        redis::cmd("LPUSH")
            .arg(&self.queue_key)
            .arg(&payload)
            .query_async::<()>(&mut con)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn dequeue(&self) -> Option<QueuedJob> {
        let mut con = self.redis.con.clone();
        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(&self.queue_key)
            .arg(5)
            .query_async(&mut con)
            .await
            .ok()?;
        result.and_then(|(_key, payload)| serde_json::from_str(&payload).ok())
    }

    pub async fn len(&self) -> usize {
        let mut con = self.redis.con.clone();
        redis::cmd("LLEN")
            .arg(&self.queue_key)
            .query_async(&mut con)
            .await
            .unwrap_or(0)
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}
