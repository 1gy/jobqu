#[derive(Debug)]
pub enum JobquError {
    #[cfg(any(feature = "sqlite"))]
    SqlxError(sqlx::Error),
    #[cfg(any(feature = "sqlite"))]
    SqlxFailedToConnect(sqlx::Error),
    #[cfg(any(feature = "sqlite"))]
    SqlxFailedToInitialize(sqlx::Error),

    SerdeError(serde_json::Error),

    JobNotRegistered {
        job_type: String,
    },
}

#[cfg(any(feature = "sqlite"))]
impl From<sqlx::Error> for JobquError {
    fn from(err: sqlx::Error) -> Self {
        JobquError::SqlxError(err)
    }
}

impl From<serde_json::Error> for JobquError {
    fn from(err: serde_json::Error) -> Self {
        JobquError::SerdeError(err)
    }
}

impl std::fmt::Display for JobquError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(any(feature = "sqlite"))]
            JobquError::SqlxError(err) => write!(f, "Sqlx error: {}", err),
            #[cfg(any(feature = "sqlite"))]
            JobquError::SqlxFailedToConnect(err) => {
                write!(f, "Failed to connect to database: {}", err)
            }
            #[cfg(any(feature = "sqlite"))]
            JobquError::SqlxFailedToInitialize(err) => {
                write!(f, "Failed to initialize database: {}", err)
            }

            JobquError::SerdeError(err) => write!(f, "Serde error: {}", err),
            JobquError::JobNotRegistered { job_type } => {
                write!(f, "Job not registered: {}", job_type)
            }
        }
    }
}

pub type JobquResult<T> = Result<T, JobquError>;
