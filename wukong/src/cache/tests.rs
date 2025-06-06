use super::redis_client::RedisClient;

async fn setup_client() -> RedisClient {
    RedisClient::new("redis://127.0.0.1:6379".to_string())
        .await
        .expect("Failed to connect to Redis")
}

#[tokio::test]
async fn test_connection() {
    let client = setup_client().await;
    let pong = client.ping().await.expect("Failed to ping");
    assert_eq!(pong, "PONG");
}

#[tokio::test]
async fn test_basic_operations() {
    let client = setup_client().await;
    let key = "test:basic";

    // Set and get
    client.set(key, "value").await.expect("Failed to set");
    let result: Option<String> = client.get(key).await.expect("Failed to get");
    assert_eq!(result, Some("value".to_string()));

    // Delete
    let deleted = client.del(key).await.expect("Failed to delete");
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn test_counters() {
    let client = setup_client().await;
    let key = "test:counter";
    
    let _ = client.del(key).await; // cleanup
    
    let count = client.incr(key, 5).await.expect("Failed to increment");
    assert_eq!(count, 5);
    
    let count = client.decr(key, 2).await.expect("Failed to decrement");
    assert_eq!(count, 3);
    
    client.del(key).await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_lists() {
    let client = setup_client().await;
    let key = "test:list";
    
    let _ = client.del(key).await; // cleanup
    
    client.lpush(key, "item1").await.expect("Failed to push");
    client.lpush(key, "item2").await.expect("Failed to push");
    
    let items: Vec<String> = client.lrange(key, 0, -1).await.expect("Failed to range");
    assert_eq!(items, vec!["item2", "item1"]); // lpush adds to front
    
    client.del(key).await.expect("Failed to cleanup");
}

#[tokio::test]
async fn test_concurrent_access() {
    let client = setup_client().await;
    
    // Test that &self allows concurrent operations
    let (r1, r2, r3) = tokio::join!(
        client.set("key1", "val1"),
        client.set("key2", "val2"),
        client.incr("counter", 1)
    );
    
    r1.expect("Failed concurrent set 1");
    r2.expect("Failed concurrent set 2");
    r3.expect("Failed concurrent incr");
    
    // Cleanup
    let _ = tokio::join!(
        client.del("key1"),
        client.del("key2"),
        client.del("counter")
    );
} 