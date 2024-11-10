use std::str::FromStr;

use crate::{
    store::{JobStore, QueueItem},
    JobquError, JobquResult,
};

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: std::sync::Arc<tokio::sync::Mutex<sqlx::sqlite::SqlitePool>>,
}

impl SqliteStore {
    pub fn builder() -> SqliteStoreBuilder {
        SqliteStoreBuilder::new()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct JobRecord {
    id: i64,
    job_type: String,
    payload: String,
    created_at: i64,
}

impl From<JobRecord> for QueueItem {
    fn from(record: JobRecord) -> Self {
        Self {
            id: record.id as u64,
            job_type: record.job_type,
            payload: record.payload,
            created_at: record.created_at as u64,
        }
    }
}

pub struct SqliteStoreBuilder {
    pub url: String,
    pub min_connections: u32,
    pub max_connections: u32,
}

impl SqliteStoreBuilder {
    fn new() -> Self {
        Self {
            url: "".to_string(),
            min_connections: 0,
            max_connections: 10,
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    pub fn min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    pub fn max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub async fn connect(&self) -> JobquResult<SqliteStore> {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(&self.url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .min_connections(self.min_connections)
            .max_connections(self.max_connections)
            .connect_with(options)
            .await
            .map_err(JobquError::SqlxFailedToConnect)?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY,
                job_type TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(JobquError::SqlxFailedToInitialize)?;

        // Ok(SqliteStore { pool })
        Ok(SqliteStore {
            pool: std::sync::Arc::new(tokio::sync::Mutex::new(pool)),
        })
    }
}

#[async_trait::async_trait]
impl JobStore for SqliteStore {
    async fn enqueue(&self, job_type: &str, payload: &str) -> JobquResult<()> {
        let tx = self.pool.lock().await;

        sqlx::query(
            r#"
            INSERT INTO jobs (job_type, payload, created_at) VALUES (?, ?, ?)
            "#,
        )
        .bind(job_type)
        .bind(payload)
        .bind(chrono::Utc::now().timestamp())
        .execute(&*tx)
        .await?;

        Ok(())
    }

    async fn dequeue(&self) -> JobquResult<Option<QueueItem>> {
        let tx = self.pool.lock().await;

        let record = sqlx::query_as::<sqlx::Sqlite, JobRecord>(
            r#"
            SELECT
                id,
                job_type,
                payload,
                created_at
            FROM
                jobs
            WHERE
                1 = 1
            ORDER BY
                created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(&*tx)
        .await?;

        if let Some(record) = &record {
            sqlx::query(
                r#"
            DELETE FROM jobs WHERE id = ?
            "#,
            )
            .bind(record.id)
            .execute(&*tx)
            .await?;
        }

        Ok(record.map(|r| r.into()))
    }
}
