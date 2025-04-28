use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use serde::{Deserialize, Serialize};
use super::{user, invoice_batch}; // Import related entities

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RbtHolding {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,     // Reference to User
    pub batch_id: ObjectId,    // Reference to InvoiceBatch
    pub amount: Decimal128,    // Amount of RBT held
    pub updated_at: DateTime,  // Record when the holding was last updated
}

// Helper methods
impl RbtHolding {
    pub fn new(user_id: ObjectId, batch_id: ObjectId, amount: Decimal128) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            user_id,
            batch_id,
            amount,
            updated_at: now,
        }
    }
}