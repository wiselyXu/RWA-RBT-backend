use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceBatch {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub creditor_id: ObjectId, // Reference to Enterprise
    pub debtor_id: ObjectId,   // Reference to Enterprise
    pub rbt_token_address: Option<String>, // Blockchain address of the RBT token
    pub total_amount: Decimal128,
    pub accepted_currency: String, // e.g., "USDC"
    pub interest_rate_apy: Decimal128,
    pub default_interest_rate_apy: Decimal128,
    pub issuance_date: DateTime,
    pub maturity_date: DateTime,
    pub status: InvoiceBatchStatus,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvoiceBatchStatus {
    Packaging, // Invoices being added
    Issued,    // RBT minted, ready for market
    Trading,   // RBT being bought/sold
    Repaying,  // Debtor is making repayments
    Settled,   // Fully repaid
    Defaulted, // Failed to repay
}

// Helper methods
impl InvoiceBatch {
    pub fn new(
        creditor_id: ObjectId,
        debtor_id: ObjectId,
        total_amount: Decimal128,
        accepted_currency: String,
        interest_rate_apy: Decimal128,
        default_interest_rate_apy: Decimal128,
        maturity_date: DateTime,
    ) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            creditor_id,
            debtor_id,
            rbt_token_address: None,
            total_amount,
            accepted_currency,
            interest_rate_apy,
            default_interest_rate_apy,
            issuance_date: now,
            maturity_date,
            status: InvoiceBatchStatus::Packaging,
            created_at: now,
            updated_at: now,
        }
    }
} 