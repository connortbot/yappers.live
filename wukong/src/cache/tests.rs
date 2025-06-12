use super::redis_client::RedisClient;
use super::key_builder::key;

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

// the schema is super deep so may as well test it here
#[test]
fn test_team_draft_key_schema() {
    let key1 = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("yapper_id").unwrap()
        .get_key().unwrap();
    assert_eq!(key1, "team_draft::game123::yapper_id");

    let key2 = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("phase").unwrap()
        .get_key().unwrap();
    assert_eq!(key2, "team_draft::game123::phase");

    let key3 = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("round").unwrap()
        .field("pool").unwrap()
        .get_key().unwrap();
    assert_eq!(key3, "team_draft::game123::round::pool");

    let key4 = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("round").unwrap()
        .field("competition").unwrap()
        .get_key().unwrap();
    assert_eq!(key4, "team_draft::game123::round::competition");

    let key5 = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("round").unwrap()
        .field("starting_drafter_id").unwrap()
        .get_key().unwrap();
    assert_eq!(key5, "team_draft::game123::round::starting_drafter_id");

    // This should disambiguate from the other "round" paths
    let key6 = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("round").unwrap()
        .field("player_to_picks").unwrap()
        .field("player456").unwrap()
        .get_key().unwrap();
    assert_eq!(key6, "team_draft::game123::round::player_to_picks::player456");

    let round_fields = vec![
        "round", "pool", "competition", "team_size", 
        "starting_drafter_id", "current_drafter_id"
    ];
    
    for field in round_fields {
        let key = key("team_draft").unwrap()
            .field("game123").unwrap()
            .field("round").unwrap()
            .field(field).unwrap()
            .get_key().unwrap();
        assert_eq!(key, format!("team_draft::game123::round::{}", field));
    }

    // Test invalid field should still fail
    let invalid_key = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("round").unwrap()
        .field("invalid_field");
    assert!(invalid_key.is_err());

    let incomplete_key = key("team_draft").unwrap()
        .field("game123").unwrap()
        .field("round").unwrap()
        .field("player_to_picks").unwrap()
        .get_key();
    assert!(incomplete_key.is_err(), "Should require player_id after player_to_picks");
} 