use bb8::Pool;
use bb8_tivk::TiKVRawConnectionManager;

#[tokio::main]
async fn main() {
    let pd_servers: Vec<String> = vec!["127.0.0.1:2379".into()];
    let manager = TiKVRawConnectionManager::new(pd_servers, None).unwrap();
    let pool = Pool::builder().max_size(10).build(manager).await.unwrap();

    let client = pool.get().await.unwrap();
    client
        .put("TEST".to_string(), b"111".to_vec())
        .await
        .unwrap();

    let value = client.get("TEST".to_string()).await.unwrap();
    println!("{}", String::from_utf8(value.unwrap()).unwrap());
}
