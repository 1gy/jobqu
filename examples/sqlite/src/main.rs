use jobqu::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "jobqu::serde")]
pub struct GreetJob {
    pub name: String,
}

#[jobqu::async_trait]
impl jobqu::Job for GreetJob {
    async fn handle(&self) -> jobqu::JobquResult<()> {
        println!("Hello, {}", self.name);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> jobqu::JobquResult<()> {
    let store = jobqu::SqliteStore::builder()
        .url("sqlite://greeting.db")
        .connect()
        .await?;

    let client = jobqu::Client::<jobqu::SqliteStore>::builder().connect(store.clone());
    for i in 0..10 {
        let job = GreetJob {
            name: format!("Alice {}", i),
        };
        client.enqueue(job).await?;
    }

    jobqu::Worker::<jobqu::SqliteStore>::builder()
        .worker_count(2)
        .register::<GreetJob>()
        .start(store.clone())
        .await?;

    Ok(())
}
