use crate::JobquResult;

#[derive(Debug)]
pub struct QueueItem {
    pub id: u64,
    pub job_type: String,
    pub payload: String,
    pub created_at: u64,
}

#[async_trait::async_trait]
pub trait JobStore: Sized + Clone + Send {
    async fn enqueue(&self, job_type: &str, payload: &str) -> JobquResult<()>;
    async fn dequeue(&self) -> JobquResult<Option<QueueItem>>;
}
