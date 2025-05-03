// server/crates/lib/service/src/entity/invoice.rs
use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::domain::dto::invoice_dto::CreateInvoiceDto;
use crate::domain::entity::invoice_status::InvoiceStatus;
use crate::utils::serde_format;
use crate::utils::snowflake_util::SnowflakeUtil;

/// Represents an invoice stored in the database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub invoice_number: String, // Should be unique per creditor/context

    // --- Core Invoice Data (partially mirrored from blockchain DTO) ---
    // --- Blockchain Specific Data (mirrored from InvoiceDataDto) ---
    pub payee: String,        // Blockchain payee address
    pub payer: String,        // Blockchain payer address
    pub amount: u64,             // Stored as u64, parsed from blockchain string
    pub currency: String,           // e.g., "USD", "USDC"
    pub due_date: i64,         // Stored as DateTime
    pub invoice_ipfs_hash: Option<String>, // Link to invoice document on IPFS (from DTO or internal)
    pub contract_ipfs_hash: Option<String>, // Hash of the contract (from DTO)
    pub status: InvoiceStatus,

    
    pub blockchain_timestamp: Option<String>, // Blockchain event timestamp (from DTO timestamp field)
    pub token_batch: Option<String>,  // Token batch identifier (from DTO)
    pub is_cleared: Option<bool>,     // Blockchain clearance status
    pub is_valid: Option<bool>,       // Blockchain validity status
    
    // --- Timestamps ---
    pub created_at: DateTime,
    pub updated_at: DateTime,
}


// Helper methods
impl Invoice {
    /// Creates a new Invoice instance, typically before saving to the database.
    pub fn new(
        data:&CreateInvoiceDto
    ) -> Self {
        let now = DateTime::now();
        let invoice_number = format!("INV-{}", SnowflakeUtil::get_id().unwrap_or_default());
        Self {
            id: None,
            invoice_number,
            amount:data.amount,
            currency:data.currency.clone(),
            due_date:   data.due_date,
            invoice_ipfs_hash: Some(data.invoice_ipfs_hash.clone()),
            contract_ipfs_hash: Some(data.contract_ipfs_hash.clone()),
            status: InvoiceStatus::Pending, // Default status
            payee:data.payee.clone(),
            payer: data.payer.clone(),
            blockchain_timestamp: None,
            token_batch: None,
            is_cleared: None,
            is_valid: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize,ToSchema)]
pub struct InvoiceDto {

    /// 票据Id (Database ObjectId)
    pub id: String,
    
    /// 票据编号
    pub invoice_number: String, // Should be unique per creditor/context

    // --- Core Invoice Data (partially mirrored from blockchain DTO) ---
    // --- Blockchain Specific Data (mirrored from InvoiceDataDto) ---
    /// 债权人企业address
    pub payee: String,        // Blockchain payee address
    /// 债务人企业address
    pub payer: String,        // Blockchain payer address
    /// 金额
    pub amount: u64,             // Stored as u64, parsed from blockchain string
    /// 货币种类
    pub currency: String,           // e.g., "USD", "USDC"
    /// 到期日时间戳
    pub due_date: i64,         // Stored as DateTime
    /// 票据 ipfs 地址
    pub invoice_ipfs_hash: Option<String>, // Link to invoice document on IPFS (from DTO or internal)
    /// 合同 ipfs 地址
    pub contract_ipfs_hash: Option<String>, // Hash of the contract (from DTO)
    /// 票据状态
    pub status: InvoiceStatus,
    /// 上链时间
    pub blockchain_timestamp: Option<String>, // Blockchain event timestamp (from DTO timestamp field)
    /// token 批次
    pub token_batch: Option<String>,  // Token batch identifier (from DTO)
    pub is_cleared: Option<bool>,     // Blockchain clearance status
    pub is_valid: Option<bool>,       // Blockchain validity status

    // --- Timestamps ---
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// 实现 from 方法
impl InvoiceDto {
    pub fn from(data: &Invoice) -> Self {
        Self {
            id: data.id.unwrap().to_string(),
            invoice_number: data.invoice_number.clone(),
            payee: data.payee.clone(),
            payer: data.payer.clone(),
            amount: data.amount,
            currency: data.currency.clone(),
            due_date: data.due_date,
            invoice_ipfs_hash: data.invoice_ipfs_hash.clone(),
            contract_ipfs_hash: data.contract_ipfs_hash.clone(),
            status: data.status,
            blockchain_timestamp: data.blockchain_timestamp.clone(),
            token_batch: data.token_batch.clone(),
            is_cleared: data.is_cleared,
            is_valid: data.is_valid,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }

    }
}