use std::str::FromStr;
use serde::{Deserialize, Serialize};
use mongodb::bson::{self, oid::ObjectId, DateTime, Document, Decimal128};
use salvo::oapi::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum HoldingStatus {
    Active,
    Matured,
    Sold,
}

impl Default for HoldingStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInvoiceHolding {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub invoice_id: ObjectId,
    pub holding_id: String,
    pub purchase_date: DateTime,
    pub purchase_amount: Decimal128,
    pub current_balance: Decimal128,
    pub total_accrued_interest: Decimal128,
    pub last_accrual_date: DateTime,
    pub holding_status: HoldingStatus,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Document>,
}

impl UserInvoiceHolding {
    pub fn new(user_id: String, invoice_id: ObjectId, purchase_amount: Decimal128) -> Self {
        let now = bson::DateTime::now();
        let holding_id = uuid::Uuid::new_v4().to_string();
        
        Self {
            id: None,
            user_id,
            invoice_id,
            holding_id,
            purchase_date: now,
            purchase_amount: purchase_amount.clone(),
            current_balance: purchase_amount,
            total_accrued_interest: Decimal128::from_str("0").unwrap(),
            last_accrual_date: now,
            holding_status: HoldingStatus::Active,
            created_at: now,
            updated_at: now,
            metadata: None,
        }
    }
}
