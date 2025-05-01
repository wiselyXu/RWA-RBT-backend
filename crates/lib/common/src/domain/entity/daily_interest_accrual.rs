
use serde::{Deserialize, Serialize};
use mongodb::bson::{self, oid::ObjectId, DateTime, Document, Decimal128};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyInterestAccrual {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub invoice_id: ObjectId,
    pub holding_id: String,
    pub accrual_date: DateTime,
    pub daily_interest_amount: Decimal128,
    pub calculated_at: DateTime,
    pub created_at: DateTime,
}

impl DailyInterestAccrual {
    pub fn new(
        user_id: String,
        invoice_id: ObjectId,
        holding_id: String,
        accrual_date: DateTime,
        daily_interest_amount: Decimal128,
    ) -> Self {
        let now = DateTime::now();
        
        Self {
            id: None,
            user_id,
            invoice_id,
            holding_id,
            accrual_date,
            daily_interest_amount,
            calculated_at: now,
            created_at: now,
        }
    }
}
