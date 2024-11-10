use std::collections::VecDeque;

use crate::{store::QueueItem, JobStore, JobquResult};

#[derive(Debug, Clone)]
pub struct MemoryStore {
    queue: std::sync::Arc<tokio::sync::Mutex<VecDeque<QueueItem>>>,
}

impl MemoryStore {
    pub fn builder() -> MemoryStoreBuilder {
        MemoryStoreBuilder::new()
    }
}

pub struct MemoryStoreBuilder {}

impl MemoryStoreBuilder {
    fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> MemoryStore {
        MemoryStore {
            queue: std::sync::Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
        }
    }
}

#[async_trait::async_trait]
impl JobStore for MemoryStore {
    async fn enqueue(&self, job_type: &str, payload: &str) -> JobquResult<()> {
        let mut queue = self.queue.lock().await;
        let created_at = chrono::Utc::now().timestamp() as u64;
        queue.push_back(QueueItem {
            id: 0,
            job_type: job_type.to_string(),
            payload: payload.to_string(),
            created_at,
        });

        Ok(())
    }

    async fn dequeue(&self) -> JobquResult<Option<QueueItem>> {
        let mut queue = self.queue.lock().await;
        Ok(queue.pop_front())
    }
}
