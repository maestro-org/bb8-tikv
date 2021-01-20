![Security audit](https://github.com/shenek/bb8-tikv/workflows/Security%20audit/badge.svg)
![Code Quality](https://github.com/shenek/bb8-tikv/workflows/Code%20Quality/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

# bb8 connection pool for TiKV client
[TiKV client](https://github.com/tikv/client-rust) support for the [bb8](https://github.com/khuey/bb8) connection pool.

## Usage
```rust
use bb8::Pool;
use bb8_tivk::TiKVRawConnectionManager;

async fn execute() {
    let pd_servers: Vec<String> = vec!["127.0.0.1:2379".into()];
    let manager = TiKVRawConnectionManager::new(pd_servers, None).unwrap();
    let pool = Pool::builder().max_size(10).build(manager).await.unwrap();

    let client = pool.get().await.unwrap();
    client
        .put("TEST".to_string(), b"111".to_vec())
        .await
        .unwrap();
}

```

For details how to use the client see [TiKV client](https://github.com/tikv/client-rust).
