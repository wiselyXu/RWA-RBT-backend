use thiserror::Error;
use redis::RedisError;
use mongodb::error::Error as MongoError;
use anyhow::Error as AnyhowError;
use serde_json;

#[derive(Error, Debug, Clone)]
pub enum ServiceError {
    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("MongoDB error: {0}")]
    MongoDbError(String),

    #[error("MongoDB transaction error: {0}")]
    MongoDbTransactionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Initialization error: {0}")]
    InitializationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invoice not found: {0}")]
    InvoiceNotFound(String),

    #[error("Invoice not issue: {0}")]
    InvoiceNotIssue(String),

    #[error("Invoice not available for purchase: {0}")]
    InvoiceNotAvailable(String),

    #[error("Invalid purchase shares. Requested: {0}, Available: {1}")]
    InvalidPurchaseShares(u64, u64),

    #[error("Invalid purchase amount: {0}")]
    InvalidPurchaseAmount(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("User {0} has insufficient funds. Required: {1}, Available: {2}")]
    InsufficientFunds(String, String, String),

    #[error("Failed to update balance for user: {0}")]
    BalanceUpdateFailed(String),

    #[error("Decimal conversion error: {0}")]
    DecimalConversionError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(String),

    #[error("Holding not found: {0}")]
    HoldingNotFound(String),

    #[error("Interest already accrued for holding {0} on date {1}")]
    InterestAlreadyAccrued(String, String),
}

impl From<RedisError> for ServiceError {
    fn from(err: RedisError) -> Self {
        ServiceError::CacheError(err.to_string())
    }
}

impl From<MongoError> for ServiceError {
    fn from(err: MongoError) -> Self {
        ServiceError::MongoDbError(err.to_string())
    }
}

impl From<AnyhowError> for ServiceError {
    fn from(err: AnyhowError) -> Self {
        ServiceError::AnyhowError(err.to_string())
    }
}

// Implement From<serde_json::Error>
impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> Self {
        ServiceError::SerializationError(err.to_string())
    }
}

// Optional: Implement conversion to Salvo error if this crate needs to interact directly with HTTP responses
// impl From<ServiceError> for salvo::Error { ... } 