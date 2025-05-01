pub mod invoice_redis_service;

pub use invoice_redis_service::InvoiceRedisService;

use anyhow::{Result, Context};
use redis::Client;
use configs::redis::Redis as RedisConfig;

// 初始化Redis客户端
pub fn init_redis_client(config: &RedisConfig) -> Result<Client> {
    let redis_url = format!(
        "redis://{}:{}@{}:{}/{}",
        config.username, 
        config.password, 
        config.host, 
        config.port, 
        config.db
    );
    
    let client = Client::open(redis_url)
        .context("Failed to create Redis client")?;
        
    // 测试连接
    let mut conn = client.get_connection()
        .context("Failed to connect to Redis")?;
        
    redis::cmd("PING")
        .query(&mut conn)
        .context("Failed to ping Redis server")?;
        
    Ok(client)
}
