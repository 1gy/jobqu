use crate::{core::JobquResult, JobquError};

#[async_trait::async_trait]
pub trait Job {
    fn job_type() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }

    async fn handle(&self) -> JobquResult<()>;
}

type JobFactory = fn(String) -> serde_json::Result<Box<dyn Job + Send + Sync>>;

#[derive(Debug, Clone)]
pub(crate) struct JobRegistry {
    factories: std::collections::HashMap<String, JobFactory>,
}

impl JobRegistry {
    pub(crate) fn new() -> Self {
        Self {
            factories: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn register<T>(&mut self)
    where
        T: Job + serde::Serialize + serde::de::DeserializeOwned + 'static + Send + Sync,
    {
        let name = T::job_type();
        self.factories.insert(name.to_string(), |payload| {
            let job = serde_json::from_str::<T>(&payload)?;
            Ok(Box::new(job))
        });
    }

    pub(crate) fn create(
        &self,
        job_type: &str,
        payload: impl Into<String>,
    ) -> JobquResult<Box<dyn Job + Send + Sync>> {
        if let Some(factory) = self.factories.get(job_type) {
            Ok(factory(payload.into())?)
        } else {
            Err(JobquError::JobNotRegistered {
                job_type: job_type.to_string(),
            })
        }
    }
}
