use serde::{Deserialize, Serialize};
use uuid::Uuid;
use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use std::fmt;
use salvo_oapi::ToSchema;

// Represents the status of a token batch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenBatchStatus {
    Pending,    // Waiting for approval or finalization
    Available,  // Available for purchase in the market
    Funding,    // Currently being funded
    Funded,     // Fully funded
    Cancelled,  // Cancelled before funding
    Completed,  // Invoice settled, tokens might be redeemable/finished
    Expired,    // Funding period expired
}

// Implement Display for TokenBatchStatus
impl fmt::Display for TokenBatchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenBatchStatus::Pending => write!(f, "Pending"),
            TokenBatchStatus::Available => write!(f, "Available"),
            TokenBatchStatus::Funding => write!(f, "Funding"),
            TokenBatchStatus::Funded => write!(f, "Funded"),
            TokenBatchStatus::Cancelled => write!(f, "Cancelled"),
            TokenBatchStatus::Completed => write!(f, "Completed"),
            TokenBatchStatus::Expired => write!(f, "Expired"),
        }
    }
}

// Represents a batch of invoice-backed tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBatch {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,            // Unique identifier for the batch
    pub batch_reference: String,         // User-friendly reference (e.g., BATCH-001)
    pub invoice_id: ObjectId,            // Foreign key to the associated invoice
    pub creditor_id: ObjectId,           // Foreign key to the creditor user/entity
    pub debtor_id: ObjectId,             // Foreign key to the debtor entity
    
    pub stablecoin_symbol: String,       // Symbol of the stablecoin used (e.g., USDT, HKDC, MNT)
    pub total_token_supply: Decimal128,  // Total number of tokens issued in this batch
    pub token_value: Decimal128,         // Value of each token in the batch (shown in screenshots as "交付份额")
    pub total_value: Decimal128,         // Total stablecoin value of the batch

    pub contract_address: Option<String>, // Address of the token contract for this batch
    
    pub sold_token_amount: Decimal128,   // Amount of tokens that have been sold
    pub available_token_amount: Decimal128, // Amount of tokens available for purchase
    
    pub status: TokenBatchStatus,        // Current status of the batch
    pub interest_rate_apy: Decimal128,   // Annual interest rate
    
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub maturity_date: DateTime,         // When the invoice is due to be paid
}

// Token Market - represents the marketplace status of tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMarket {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub batch_id: ObjectId,              // Reference to the TokenBatch
    pub batch_reference: String,         // For easier querying (e.g., BATCH-1)
    pub creditor_address: String,        // Blockchain address of the creditor
    pub debtor_address: String,          // Blockchain address of the debtor
    pub stablecoin_symbol: String,       // MNT, HKDC, USDT etc.
    pub total_token_amount: Decimal128,  // Total tokens in this batch
    pub sold_token_amount: Decimal128,   // Tokens that have been sold
    pub available_token_amount: Decimal128, // Tokens available for purchase
    pub purchased_token_amount: Decimal128, // Tokens purchased in current session
    pub token_value_per_unit: Decimal128,   // Value per token
    pub remaining_transaction_amount: Decimal128, // Remaining value to be traded
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// Token Holding - represents an investor's token holdings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolding {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,               // Reference to the User/Investor
    pub batch_id: ObjectId,              // Reference to the TokenBatch
    pub batch_reference: String,         // For easier querying (e.g., BATCH-1)
    pub token_amount: Decimal128,        // Amount of tokens held
    pub purchase_value: Decimal128,      // Original purchase value
    pub current_value: Decimal128,       // Current value (including interest)
    pub purchase_date: DateTime,         // When the tokens were purchased
    pub status: TokenHoldingStatus,      // Current status of the holding
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// Status of a token holding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenHoldingStatus {
    Active,     // Currently held and active
    Redeemed,   // Redeemed after maturity
    Transferred, // Transferred to another investor
    Defaulted,  // The underlying invoice defaulted
}

// Token Transaction - represents a purchase/sale of tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub batch_id: ObjectId,              // Reference to the TokenBatch
    pub batch_reference: String,         // For easier querying (e.g., BATCH-1)
    pub user_id: ObjectId,               // Reference to the User/Investor
    pub transaction_type: TokenTransactionType,
    pub token_amount: Decimal128,        // Amount of tokens transacted
    pub transaction_value: Decimal128,   // Value of the transaction
    pub stablecoin_symbol: String,       // Symbol of the stablecoin used
    pub transaction_hash: Option<String>, // Blockchain transaction hash if applicable
    pub status: TokenTransactionStatus,
    pub transaction_date: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

// Type of token transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenTransactionType {
    Purchase,   // Investor buying tokens
    Sale,       // Investor selling tokens
    Redemption, // Redeeming tokens after maturity
    Interest,   // Interest payment
}

// Status of a token transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenTransactionStatus {
    Pending,    // Transaction initiated but not confirmed
    Completed,  // Transaction completed successfully
    Failed,     // Transaction failed
    Cancelled,  // Transaction cancelled
}

// Request DTOs

// Create Token Batch Request
#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
pub struct CreateTokenBatchRequest {
    pub batch_reference: String,
    pub invoice_id: String,
    pub creditor_id: String,
    pub debtor_id: String,
    pub stablecoin_symbol: String,
    pub total_token_supply: String,
    pub token_value: String,
    pub interest_rate_apy: String,
    pub maturity_date: String,
}

// Purchase Token Request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PurchaseTokenRequest {
    pub batch_id: String,
    pub token_amount: String,
}

// Query Token Market Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTokenMarketRequest {
    pub stablecoin_symbol: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// Query User Token Holdings Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUserTokenHoldingsRequest {
    pub user_id: String,
}

// 从发票批次创建Token批次的请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTokenBatchFromInvoiceBatchRequest {
    pub batch_reference: String,
    pub stablecoin_symbol: String,
    pub token_value: String,
    pub interest_rate_apy: String,
    pub maturity_date: Option<String>,  // 可选，如不提供则使用发票中最早的到期日
}

// Response DTOs

// Token Batch Response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenBatchResponse {
    pub id: String,
    pub batch_reference: String,
    pub creditor_name: String,
    pub debtor_name: String,
    pub stablecoin_symbol: String,
    pub total_token_supply: String,
    pub token_value: String,
    pub total_value: String,
    pub sold_token_amount: String,
    pub available_token_amount: String,
    pub status: String,
    pub interest_rate_apy: String,
    pub maturity_date: String,
}

// Token Market Response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenMarketResponse {
    pub id: String,
    pub batch_reference: String,
    pub creditor_address: String,
    pub debtor_address: String,
    pub stablecoin_symbol: String,
    pub total_token_amount: String,
    pub sold_token_amount: String,
    pub available_token_amount: String,
    pub token_value_per_unit: String,
    pub remaining_transaction_amount: String,
}

// Token Holding Response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TokenHoldingResponse {
    pub id: String,
    pub batch_reference: String,
    pub token_amount: String,
    pub purchase_value: String,
    pub current_value: String,
    pub purchase_date: String,
    pub status: String,
} 