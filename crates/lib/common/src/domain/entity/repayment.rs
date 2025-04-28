use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use serde::{Deserialize, Serialize};
use super::{enterprise, invoice_batch}; // Import related entities

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Repayment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub batch_id: ObjectId,       // Reference to InvoiceBatch
    pub debtor_id: ObjectId,      // Reference to Enterprise (who paid)
    pub amount: Decimal128,       // Amount repaid
    pub currency: String,         // Currency used for repayment
    pub transaction_hash: String, // On-chain transaction hash (should be unique)
    pub repayment_timestamp: DateTime, // Timestamp from blockchain event or tx confirmation
    pub created_at: DateTime,     // When the record was created in the DB
}

// Helper methods
impl Repayment {
    pub fn new(
        batch_id: ObjectId,
        debtor_id: ObjectId,
        amount: Decimal128,
        currency: String,
        transaction_hash: String,
        repayment_timestamp: DateTime,
    ) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            batch_id,
            debtor_id,
            amount,
            currency,
            transaction_hash,
            repayment_timestamp,
            created_at: now,
        }
    }
}