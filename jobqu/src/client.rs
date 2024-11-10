use crate::{Job, JobStore, JobquResult};

#[derive(Debug, Clone)]
pub struct Client<S: JobStore> {
    store: S,
}

impl<S> Client<S>
where
    S: JobStore,
{
    fn new(store: S) -> Self {
        Self { store }
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn enqueue<J>(&self, job: J) -> JobquResult<()>
    where
        J: Job + serde::Serialize,
    {
        let job_type = J::job_type();
        let payload = serde_json::to_string(&job)?;
        self.store.enqueue(job_type, &payload).await
    }
}

pub struct ClientBuilder {
    // TODO: add options
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn connect<S: JobStore>(&self, store: S) -> Client<S> {
        Client::new(store)
    }
}
