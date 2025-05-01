use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRedisDto {
    pub invoice_id: String,
    pub invoice_number: String,
    pub title: String,
    pub payee: String,
    pub payer: String,
    pub issue_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub face_value: String,
    pub annual_rate: f64,
    pub total_shares: u32,
    pub available_shares: u32,
    pub share_price: f64,
    pub status: String,
}

impl InvoiceRedisDto {
    pub fn is_available_for_purchase(&self) -> bool {
        self.status == "ACTIVE" && self.available_shares > 0
    }
    
    pub fn calculate_daily_rate(&self, is_leap_year: bool) -> f64 {
        // 计算日利率：年利率 / 当年天数
        let days_in_year = if is_leap_year { 366.0 } else { 365.0 };
        self.annual_rate / days_in_year / 100.0 // 年利率是百分比，需要除以100
    }
}
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