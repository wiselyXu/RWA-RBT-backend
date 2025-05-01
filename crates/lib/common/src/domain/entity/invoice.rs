// server/crates/lib/service/src/entity/invoice.rs
use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
// use crate::domain::entity::enterprise::EnterpriseDto; // Not used directly here
// use super::{enterprise, invoice_batch, Enterprise}; // Not used directly here

/// Represents an invoice stored in the database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub invoice_number: String, // Should be unique per creditor/context
    
    // --- Database specific relationships ---
    pub creditor_id: ObjectId,  // Reference to internal Enterprise entity
    pub debtor_id: ObjectId,    // Reference to internal Enterprise entity
    pub batch_id: Option<ObjectId>, // Reference to InvoiceBatch (nullable)

    // --- Core Invoice Data (partially mirrored from blockchain DTO) ---
    pub amount: u64,             // Stored as u64, parsed from blockchain string
    pub currency: String,           // e.g., "USD", "USDC"
    pub due_date: DateTime,         // Stored as DateTime
    pub ipfs_hash: Option<String>, // Link to invoice document on IPFS (from DTO or internal)
    pub status: InvoiceStatus,

    // --- Blockchain Specific Data (mirrored from InvoiceDataDto) ---
    pub payee: Option<String>,        // Blockchain payee address
    pub payer: Option<String>,        // Blockchain payer address
    pub contract_hash: Option<String>, // Hash of the contract (from DTO)
    pub blockchain_timestamp: Option<String>, // Blockchain event timestamp (from DTO timestamp field)
    pub token_batch: Option<String>,  // Token batch identifier (from DTO)
    pub is_cleared: Option<bool>,     // Blockchain clearance status
    pub is_valid: Option<bool>,       // Blockchain validity status
    
    // --- RWA Specific Data (for interest calculation) ---
    pub annual_interest_rate: f64, // Annual interest rate percentage
    
    // --- Timestamps ---
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
pub enum InvoiceStatus {
    Pending,    // 已创建在数据库中，可能尚未上链
    Verified,   // 可能对应于链上的有效性验证
    Packaged,   // 已包含在发票批次中
    Repaid,     // 对应于链上的清算状态
    Overdue,    // 已逾期
    Defaulted,  // 已违约

    OnSale,
    SoldOut,
}

// Helper methods
impl Invoice {
    /// Creates a new Invoice instance, typically before saving to the database.
    pub fn new(
        invoice_number: String,
        creditor_id: ObjectId,
        debtor_id: ObjectId,
        amount: u64,
        currency: String,
        due_date: DateTime,
    ) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            invoice_number,
            creditor_id,
            debtor_id,
            amount,
            currency,
            due_date,
            status: InvoiceStatus::Pending, // Default status
            ipfs_hash: None,
            batch_id: None,
            payee: None,
            payer: None,
            contract_hash: None,
            blockchain_timestamp: None,
            token_batch: None,
            is_cleared: None,
            is_valid: None,
            annual_interest_rate: 0.0, // Default rate, should be set explicitly
            created_at: now,
            updated_at: now,
        }
    }
}

/// Data Transfer Object for sending Invoice data out via API.
#[derive(Clone, Debug, Serialize, Deserialize,ToSchema)]
pub struct InvoiceDto {
    /// 票据Id (Database ObjectId)
    pub id: String,
    /// 票据编号
    pub invoice_number: String,
    /// 债权人企业ID (Database ObjectId)
    pub creditor_id: String,
    /// 债务人企业ID (Database ObjectId)
    pub debtor_id: String,
    /// 金额
    pub amount: u64,
    /// 货币
    pub currency: String,
    /// 到期日期
    pub due_date: DateTime,
    /// 状态
    pub status: InvoiceStatus,
    /// IPFS Hash (Invoice Document)
    pub ipfs_hash: Option<String>,
    /// 批次ID (Database ObjectId)
    pub batch_id: Option<String>,
    /// 创建时间
    pub created_at: DateTime,
    /// 更新时间
    pub updated_at: DateTime,
    /// 合约地址 (Payee Address from Blockchain)
    pub payee: Option<String>,
    /// 合约地址 (Payer Address from Blockchain)
    pub payer: Option<String>,
    /// 合约哈希 (from Blockchain)
    pub contract_hash: Option<String>,
    /// 区块链时间戳 (from Blockchain)
    pub blockchain_timestamp: Option<String>,
    /// Token批次 (from Blockchain)
    pub token_batch: Option<String>,
    /// 是否已清算 (from Blockchain)
    pub is_cleared: Option<bool>,
    /// 是否有效 (from Blockchain)
    pub is_valid: Option<bool>,
    /// 年化利率
    pub annual_interest_rate: f64,
}

impl From<&Invoice> for InvoiceDto {
    fn from(data: &Invoice) -> InvoiceDto {
        InvoiceDto {
            id: data.id.map(|id| id.to_string()).unwrap_or_default(), // Handle potential None id
            invoice_number: data.invoice_number.clone(),
            creditor_id: data.creditor_id.to_string(),
            debtor_id: data.debtor_id.to_string(),
            amount: data.amount,
            currency: data.currency.clone(),
            due_date: data.due_date,
            status: data.status.clone(),
            ipfs_hash: data.ipfs_hash.clone(),
            batch_id: data.batch_id.map(|id| id.to_string()),
            created_at: data.created_at,
            updated_at: data.updated_at,
            payee: data.payee.clone(),
            payer: data.payer.clone(),
            contract_hash: data.contract_hash.clone(),
            blockchain_timestamp: data.blockchain_timestamp.clone(),
            token_batch: data.token_batch.clone(),
            is_cleared: data.is_cleared,
            is_valid: data.is_valid,
            annual_interest_rate: data.annual_interest_rate,
        }
    }
}