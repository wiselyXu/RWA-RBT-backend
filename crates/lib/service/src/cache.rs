use redis::{Client, RedisError};
use log::info;
use configs::cfgs::Redis;

pub fn init_redis_client(redis_config: &Redis) -> Result<Client, RedisError> {
    info!("Initializing Redis client for URL: {}", redis_config.url);
    let client = Client::open(redis_config.url.as_str())?;
    // Note: Connection is established lazily when a command is executed.
    // You could optionally ping here to ensure connectivity immediately.
    // client.get_connection()?.ping()?; 
    info!("Redis client initialized successfully.");
    Ok(client)
}
