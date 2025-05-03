use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::NaiveDate;
use crate::domain::entity::Invoice;

/// Data Transfer Object for Invoice data. Used for both input and output.
#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
#[salvo(schema(example = json!({
    "payee": "0xabc1234567890abcdef1234567890abcdef123456", // 匹配 payee, 示例为地址字符串
    "payer": "0xdef4567890abcdef1234567890abcdef1234567890", // 匹配 payer, 示例为地址字符串
    "amount": "1000000000000000000", // 匹配 amount, 示例为 U256 字符串 (例如 1 Ether)
    "invoice_ipfs_hash": "Qmabcdef1234567890abcdef1234567890abcdef12345678", // 匹配 ipfs_hash
    "contract_ipfs_hash": "0x1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b", // 匹配 contract_hash, 示例为合约/交易哈希字符串
    "due_date": "1704067200", // 匹配 due_date, 示例为 U256 字符串 (Unix timestamp)
    "currency": "CNY" // Added currency
})))]
pub struct InvoiceDataDto {
    pub payee: String,  // Use String for address representation
    pub payer: String,  // Use String for address representation
    pub amount: u64, // Use String for U256 representation
    pub invoice_ipfs_hash: String,
    pub contract_ipfs_hash: String,
    pub due_date: i64,  // Use String for U256 representation
    pub currency: String, // Added currency field
    pub invoice_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
#[salvo(schema(example = json!({
    "payee": "0xabc1234567890abcdef1234567890abcdef123456", // 匹配 payee, 示例为地址字符串
    "payer": "0xdef4567890abcdef1234567890abcdef1234567890", // 匹配 payer, 示例为地址字符串
    "amount": "1000000000000000000", // 匹配 amount, 示例为 U256 字符串 (例如 1 Ether)
    "invoice_ipfs_hash": "Qmabcdef1234567890abcdef1234567890abcdef12345678", // 匹配 ipfs_hash
    "contract_ipfs_hash": "0x1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b", // 匹配 contract_hash, 示例为合约/交易哈希字符串
    "due_date": "1704067200", // 匹配 due_date, 示例为 U256 字符串 (Unix timestamp)
    "currency": "CNY" // Added currency
})))]
pub struct CreateInvoiceDto {
    pub payee: String,  // Use String for address representation
    pub payer: String,  // Use String for address representation
    pub amount: u64, // Use String for U256 representation
    pub invoice_ipfs_hash: String,
    pub contract_ipfs_hash: String,
    pub due_date: i64,  // 到期时间戳
    pub currency: String, // Added currency field
}