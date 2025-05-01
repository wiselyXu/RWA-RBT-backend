pub mod invoice_redis_service;

pub use invoice_redis_service::InvoiceRedisService;

use anyhow::{Result, Context};
use log::info;
use redis::{Client, RedisError};
use configs::cfgs::Redis;

// 初始化Redis客户端
pub fn init_redis_client(redis_config: &Redis) -> std::result::Result<Client, RedisError> {
    info!("Initializing Redis client for URL: {}", redis_config.url);
    let client = Client::open(redis_config.url.as_str())?;
    // Note: Connection is established lazily when a command is executed.
    // You could optionally ping here to ensure connectivity immediately.
    // client.get_connection()?.ping()?;
    info!("Redis client initialized successfully.");
    Ok(client)
}
