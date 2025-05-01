use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    NotForSale,    // 未发售
    OnSale,        // 在售
    SoldOut,       // 售罄
    Matured,       // 已到期
    Terminated,    // 已终止
}

impl ToString for InvoiceStatus {
    fn to_string(&self) -> String {
        match self {
            InvoiceStatus::NotForSale => "NOT_FOR_SALE".to_string(),
            InvoiceStatus::OnSale => "ON_SALE".to_string(),
            InvoiceStatus::SoldOut => "SOLD_OUT".to_string(),
            InvoiceStatus::Matured => "MATURED".to_string(),
            InvoiceStatus::Terminated => "TERMINATED".to_string(),
        }
    }
}

impl Default for InvoiceStatus {
    fn default() -> Self {
        Self::NotForSale
    }
}
