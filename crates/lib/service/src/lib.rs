#![allow(warnings)]
pub mod db;
pub mod repository;
pub mod invoice;
pub mod service;
pub mod error;
pub mod cache;

use ::redis::{Client, RedisError};
use log::info;
use configs::cfgs::Redis;
// Re-export key items for easier access from other crates
pub use db::{create_indexes, init_mongodb};
pub use error::ServiceError;
pub use repository::{EnterpriseRepository, InvoiceRepository, UserRepository};

// Optional: Define a struct to hold initialized clients/pools
// pub struct ServiceContext {
//     pub db_pool: sea_orm::DatabaseConnection,
//     pub redis_client: redis::Client,
// }
// 
// impl ServiceContext {
//     pub async fn new(db_config: &common::config::config::DatabaseConfig, redis_config: &common::config::config::RedisConfig) -> Result<Self, ServiceError> {
//         let db_pool = init_db_pool(db_config).await?;
//         let redis_client = init_redis_client(redis_config)?;
//         Ok(Self { db_pool, redis_client })
//     }
// }

