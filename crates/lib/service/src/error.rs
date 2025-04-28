use thiserror::Error;

use redis::RedisError;
use mongodb::error::Error as MongoError;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Cache error: {0}")]
    CacheError(#[from] RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    // Add other service-specific errors as needed
}

// Optional: Implement conversion to Salvo error if this crate needs to interact directly with HTTP responses
// impl From<ServiceError> for salvo::Error { ... } 