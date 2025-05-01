
use serde::{Deserialize, Serialize};
use mongodb::bson::{self, oid::ObjectId, DateTime, Document, Decimal128};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub invoice_id: ObjectId,
    pub holding_id: String,
    pub transaction_type: TransactionType,
    pub amount: Decimal128,
    pub transaction_date: DateTime,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Document>,
}

impl Transaction {
    pub fn new(
        user_id: String,
        invoice_id: ObjectId,
        holding_id: String,
        transaction_type: TransactionType,
        amount: Decimal128,
    ) -> Self {
        let now = bson::DateTime::now();
        
        Self {
            id: None,
            user_id,
            invoice_id,
            holding_id,
            transaction_type,
            amount,
            transaction_date: now,
            status: "completed".to_string(),
            created_at: now,
            updated_at: now,
            metadata: None,
        }
    }
    
    pub fn new_purchase(
        user_id: String,
        invoice_id: ObjectId,
        holding_id: String,
        amount: Decimal128,
    ) -> Self {
        Self::new(
            user_id,
            invoice_id, 
            holding_id,
            TransactionType::Purchase,
            amount
        )
    }
    
    pub fn new_maturity_payment(
        user_id: String,
        invoice_id: ObjectId,
        holding_id: String,
        amount: Decimal128,
    ) -> Self {
        Self::new(
            user_id,
            invoice_id, 
            holding_id,
            TransactionType::MaturityPayment,
            amount
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Purchase,
    InterestAccrual,
    MaturityPayment,
    Withdrawal,
}
