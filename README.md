# jobqu

## 概要
- アプリケーションに組み込み可能なジョブキューの実験プロジェクトです
- バックエンドにSQLiteを利用します

## TODO
- [ ] エラーが発生したジョブをリトライできるようにする
- [ ] キューの優先度を実装する
- [ ] ログ出力を強化する

## 使い方

### ジョブの作成
```rust
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
```

### ジョブの登録、実行
```rust
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
```
