use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::NaiveDate;
/// Data Transfer Object for Invoice data. Used for both input and output.
#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
#[serde(rename_all = "camelCase")]
#[salvo(schema(example = json!({
    "invoiceNumber": "INV-12345", // 匹配 invoice_number
    "payee": "0xabc1234567890abcdef1234567890abcdef123456", // 匹配 payee, 示例为地址字符串
    "payer": "0xdef4567890abcdef1234567890abcdef1234567890", // 匹配 payer, 示例为地址字符串
    "amount": "1000000000000000000", // 匹配 amount, 示例为 U256 字符串 (例如 1 Ether)
    "ipfsHash": "Qmabcdef1234567890abcdef1234567890abcdef12345678", // 匹配 ipfs_hash
    "contractHash": "0x1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b", // 匹配 contract_hash, 示例为合约/交易哈希字符串
    "timestamp": "1678886400", // 匹配 timestamp, 示例为 U256 字符串 (Unix timestamp)
    "dueDate": "1704067200", // 匹配 due_date, 示例为 U256 字符串 (Unix timestamp)
    "tokenBatch": "BatchID-XYZ", // 匹配 token_batch
    "isCleared": false, // 匹配 is_cleared
    "isValid": true // 匹配 is_valid
})))]
pub struct InvoiceDataDto {
    pub invoice_number: String,
    pub payee: String,  // Use String for address representation
    pub payer: String,  // Use String for address representation
    pub amount: String, // Use String for U256 representation
    pub ipfs_hash: String,
    pub contract_hash: String,
    pub timestamp: String, // Use String for U256 representation
    pub due_date: String,  // Use String for U256 representation
    pub token_batch: String,
    pub is_cleared: bool,
    pub is_valid: bool,
}