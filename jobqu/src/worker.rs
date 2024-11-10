use crate::{job::JobRegistry, store::JobStore, Job, JobquResult};

#[derive(Debug, Clone)]
pub struct Worker<S: JobStore + 'static> {
    store: S,
    registry: std::sync::Arc<tokio::sync::Mutex<JobRegistry>>,
}

impl<S> Worker<S>
where
    S: JobStore + Send + 'static + Sync,
{
    pub fn builder() -> WorkerBuilder<S> {
        WorkerBuilder::new()
    }

    async fn process(&self) -> JobquResult<()> {
        if let Some(record) = self.store.dequeue().await? {
            let job = self
                .registry
                .lock()
                .await
                .create(&record.job_type, record.payload)?;
            job.handle().await?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct WorkerBuilder<S: JobStore> {
    worker_count: u64,
    registry: JobRegistry,
    polling_interval: u64,

    _store_type: std::marker::PhantomData<S>,
}

impl<S> WorkerBuilder<S>
where
    S: JobStore + Send + 'static + Sync,
    Worker<S>: Send,
{
    pub fn new() -> Self {
        Self {
            worker_count: 1,
            registry: JobRegistry::new(),
            polling_interval: 1000,
            _store_type: std::marker::PhantomData,
        }
    }

    pub fn worker_count(mut self, worker_count: u64) -> Self {
        self.worker_count = worker_count;
        self
    }

    pub fn polling_interval(mut self, polling_interval: u64) -> Self {
        self.polling_interval = polling_interval;
        self
    }

    pub fn register<T>(mut self) -> Self
    where
        T: Job + serde::Serialize + serde::de::DeserializeOwned + 'static + Send + Sync,
    {
        self.registry.register::<T>();
        self
    }

    pub async fn start(&self, store: S) -> JobquResult<()> {
        let cancellation_token = tokio_util::sync::CancellationToken::new();

        let mut worker_handles = vec![];

        for _ in 0..self.worker_count {
            let worker = Worker {
                store: store.clone(),
                registry: std::sync::Arc::new(tokio::sync::Mutex::new(self.registry.clone())),
            };
            let cancellation_token = cancellation_token.clone();
            let polling_interval = self.polling_interval;

            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(polling_interval)) => {
                            if cancellation_token.is_cancelled() {
                                break;
                            }
                            worker.process().await.unwrap();
                        }
                        _ = cancellation_token.cancelled() => {
                            break;
                        }
                    }
                }
            });
            worker_handles.push(handle);
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                dbg!("Ctrl-C received");
                cancellation_token.cancel();
            }
        }

        while worker_handles.iter().any(|h| !h.is_finished()) {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(())
    }
}
