use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use super::invoice_batch; // Import related entity

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettlementNft {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub batch_id: ObjectId,         // Reference to InvoiceBatch (should be unique)
    pub nft_contract_address: String,
    pub token_id: String,          // NFT Token ID on the contract
    pub creditor_owner_address: Option<String>, // Wallet that received creditor's NFT
    pub debtor_owner_address: Option<String>,   // Wallet that received debtor's NFT
    pub issuance_timestamp: DateTime,          // When the NFT was minted/recorded
}

// Helper methods
impl SettlementNft {
    pub fn new(
        batch_id: ObjectId,
        nft_contract_address: String,
        token_id: String,
        issuance_timestamp: DateTime,
    ) -> Self {
        Self {
            id: None,
            batch_id,
            nft_contract_address,
            token_id,
            creditor_owner_address: None,
            debtor_owner_address: None,
            issuance_timestamp,
        }
    }
}
