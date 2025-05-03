use std::fmt::Display;
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceStatus {
    Pending,    // 未上链
    Verified,   // 已上链
    Packaged,   // 已包含在发票批次中
    Repaid,     // 已清算
    Overdue,    // 已逾期
    Defaulted,  // 已违约
    OnSale,     // 在售
}

impl Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            InvoiceStatus::Pending => "未上链".to_string(),
            InvoiceStatus::Verified => "已上链".to_string(),
            InvoiceStatus::Packaged => "已包含在发票批次中".to_string(),
            InvoiceStatus::Repaid => "已清算".to_string(),
            InvoiceStatus::Overdue => "已逾期".to_string(),
            InvoiceStatus::Defaulted => "已违约".to_string(),
            InvoiceStatus::OnSale => "在售".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Default for InvoiceStatus {
    fn default() -> Self {
        Self::Pending
    }
}