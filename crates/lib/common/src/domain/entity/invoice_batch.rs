use std::net::ToSocketAddrs;
use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceBatch {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub payee: String, // Reference to Enterprise
    pub payer: String,   // Reference to Enterprise
    pub rbt_token_address: Option<String>, // Blockchain address of the RBT token
    pub accepted_currency: String, // e.g., "USDC"
    pub token_batch_id: Option<ObjectId>, // Reference to TokenBatch after creation
    pub status: InvoiceBatchStatus,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize,PartialEq)]
pub enum InvoiceBatchStatus {
    Packaging, // Invoices being added
    Issued,    // Invoices packaged, ready for token creation
    Trading,   // Associated with TokenBatch and trading
    Repaying,  // Debtor is making repayments
    Settled,   // Fully repaid
    Defaulted, // Failed to repay
}

// Helper methods
impl InvoiceBatch {
    pub fn new(
        payee: &str,
        payer: &str,
        accepted_currency: String,
    ) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            payee:payee.to_string(),
            payer:payer.to_string(),
            rbt_token_address: None,
            accepted_currency,
            token_batch_id: None,
            status: InvoiceBatchStatus::Packaging,
            created_at: now,
            updated_at: now,
        }
    }
} 