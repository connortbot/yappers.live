use redis::{Client, AsyncCommands, RedisResult, ToRedisArgs};
use redis::aio::{MultiplexedConnection, PubSub};
use crate::error::{ErrorResponse, ErrorCode};
use std::collections::{HashMap, HashSet};

pub trait RedisKey: ToRedisArgs + Send + Sync {}
pub trait RedisValue: ToRedisArgs + Send + Sync {}
pub trait RedisField: ToRedisArgs + Send + Sync {}

impl<T: ToRedisArgs + Send + Sync> RedisKey for T {}
impl<T: ToRedisArgs + Send + Sync> RedisValue for T {}
impl<T: ToRedisArgs + Send + Sync> RedisField for T {}

#[derive(Debug, Clone)]
pub struct PubSubMessage {
    pub channel: String,
    pub payload: String,
}

#[derive(Clone)]
pub struct RedisClient {
    connection: MultiplexedConnection,
    client: Client,
}

impl RedisClient {
    pub async fn new(redis_url: String) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let connection = client.get_multiplexed_tokio_connection().await?;
        
        Ok(RedisClient { 
            connection,
            client,
        })
    }

    pub async fn ping(&self) -> RedisResult<String> {
        let mut conn = self.connection.clone();
        conn.ping().await
    }

    pub async fn set(&self, key: impl RedisKey, value: impl RedisValue) -> RedisResult<()> {
        let mut conn = self.connection.clone();
        conn.set(key, value).await
    }

    pub async fn get(&self, key: impl RedisKey) -> RedisResult<Option<String>> {
        let mut conn = self.connection.clone();
        conn.get(key).await
    }

    pub async fn del(&self, key: impl RedisKey) -> RedisResult<usize> {
        let mut conn = self.connection.clone();
        conn.del(key).await
    }

    pub async fn exists(&self, key: impl RedisKey) -> RedisResult<bool> {
        let mut conn = self.connection.clone();
        conn.exists(key).await
    }

    pub async fn lpush(&self, key: impl RedisKey, value: impl RedisValue) -> RedisResult<usize> {
        let mut conn = self.connection.clone();
        conn.lpush(key, value).await
    }

    pub async fn lrange(&self, key: impl RedisKey, start: isize, stop: isize) -> RedisResult<Vec<String>> {
        let mut conn = self.connection.clone();
        conn.lrange(key, start, stop).await
    }

    pub async fn lrem(&self, key: impl RedisKey, count: isize, value: impl RedisValue) -> RedisResult<usize> {
        let mut conn = self.connection.clone();
        conn.lrem(key, count, value).await
    }

    pub async fn incr(&self, key: impl RedisKey, delta: i64) -> RedisResult<i64> {
        let mut conn = self.connection.clone();
        conn.incr(key, delta).await
    }

    pub async fn decr(&self, key: impl RedisKey, delta: i64) -> RedisResult<i64> {
        let mut conn = self.connection.clone();
        conn.decr(key, delta).await
    }

    pub async fn hset(&self, key: impl RedisKey, field: impl RedisField, value: impl RedisValue) -> RedisResult<bool> {
        let mut conn = self.connection.clone();
        conn.hset(key, field, value).await
    }

    pub async fn hget(&self, key: impl RedisKey, field: impl RedisField) -> RedisResult<Option<String>> {
        let mut conn = self.connection.clone();
        conn.hget(key, field).await
    }

    pub async fn hgetall(&self, key: impl RedisKey) -> RedisResult<HashMap<String, String>> {
        let mut conn = self.connection.clone();
        conn.hgetall(key).await
    }

    pub async fn sadd(&self, key: impl RedisKey, member: impl RedisValue) -> RedisResult<usize> {
        let mut conn = self.connection.clone();
        conn.sadd(key, member).await
    }

    pub async fn srem(&self, key: impl RedisKey, member: impl RedisValue) -> RedisResult<usize> {
        let mut conn = self.connection.clone();
        conn.srem(key, member).await
    }

    pub async fn smembers(&self, key: impl RedisKey) -> RedisResult<HashSet<String>> {
        let mut conn = self.connection.clone();
        conn.smembers(key).await
    }

    pub async fn multi(&self) -> RedisResult<redis::Pipeline> {
        Ok(redis::pipe().atomic().clone())
    }

    pub async fn exec_pipeline(&self, pipeline: redis::Pipeline) -> RedisResult<Vec<redis::Value>> {
        let mut conn = self.connection.clone();
        pipeline.query_async(&mut conn).await
    }

    pub async fn watch(&self, key: impl RedisKey) -> RedisResult<()> {
        let mut conn = self.connection.clone();
        redis::cmd("WATCH").arg(key).query_async(&mut conn).await
    }

    // HELPERS
    pub async fn get_required(&self, key: impl RedisKey, error_resp: ErrorResponse) -> Result<String, ErrorResponse> {
        match self.get(key).await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(error_resp),
            Err(e) => Err(ErrorResponse {
                error: ErrorCode::InternalServerError,
                message: format!("Redis error: {}", e),
            }),
        }
    }

    // PUB/SUB METHODS
    // game_channel::{game_id}
    pub async fn publish(&self, channel: impl RedisKey, message: impl RedisValue) -> RedisResult<usize> {
        let mut conn = self.connection.clone(); // Cheap clone - same TCP connection
        conn.publish(channel, message).await
    }

    pub async fn get_pubsub(&self) -> RedisResult<PubSub> {
        self.client.get_async_pubsub().await
    }

    pub async fn create_shared_pubsub(&self, patterns: Vec<String>) -> RedisResult<PubSub> {
        let mut pubsub = self.get_pubsub().await?;
        
        for pattern in patterns {
            pubsub.psubscribe(pattern).await?;
        }
        
        Ok(pubsub)
    }
}