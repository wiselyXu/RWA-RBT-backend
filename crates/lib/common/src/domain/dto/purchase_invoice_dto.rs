use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseInvoiceDto {
    pub invoice_id: String,
    pub shares: u64,
    // 如果使用金额代替份数，则使用下面的字段
    // pub amount: f64,
}
