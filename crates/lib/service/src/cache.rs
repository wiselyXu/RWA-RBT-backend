use redis::{Client, RedisError};
use log::info;
use configs::cfgs::Redis;

/// Initializes the Redis client.
/// Note: The redis crate handles connection pooling internally.
pub fn init_redis_client(redis_config: &Redis) -> Result<Client, RedisError> {
    info!("Initializing Redis client for URL: {}", redis_config.url);
    let client = Client::open(redis_config.url.as_str())?;
    // Note: Connection is established lazily when a command is executed.
    // You could optionally ping here to ensure connectivity immediately.
    // client.get_connection()?.ping()?; 
    info!("Redis client initialized successfully.");
    Ok(client)
}

// Example usage for caching a token (to be used in auth controller):
/*
use redis::AsyncCommands;

const TOKEN_EXPIRATION_SECONDS: usize = 3600; // 1 hour

pub async fn cache_token(redis_client: &Client, user_id: i64, token: &str) -> Result<(), RedisError> {
    let mut conn = redis_client.get_async_connection().await?;
    let key = format!("user_token:{}", user_id);
    conn.set_ex(key, token, TOKEN_EXPIRATION_SECONDS).await
}

pub async fn get_cached_token(redis_client: &Client, user_id: i64) -> Result<Option<String>, RedisError> {
    let mut conn = redis_client.get_async_connection().await?;
    let key = format!("user_token:{}", user_id);
    conn.get(key).await
}

pub async fn delete_cached_token(redis_client: &Client, user_id: i64) -> Result<(), RedisError> {
    let mut conn = redis_client.get_async_connection().await?;
    let key = format!("user_token:{}", user_id);
    conn.del(key).await
}
*/ 