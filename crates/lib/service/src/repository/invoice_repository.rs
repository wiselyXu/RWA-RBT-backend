use futures::stream::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime, Decimal128},
    options::{FindOptions, UpdateOptions, FindOneOptions},
    results::{DeleteResult, UpdateResult},
    Collection, Database, ClientSession,
};
use serde::Serialize;
use crate::error::ServiceError;

use common::domain::entity::{Invoice, InvoiceStatus};
use std::str::FromStr;
use common::domain::dto::invoice_dto::InvoiceDataDto;

pub struct InvoiceRepository {
    collection: Collection<Invoice>,
}

/// Fields that can be updated for an Invoice.
#[derive(Serialize, Default, Debug)]
pub struct UpdateInvoiceData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<InvoiceStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<ObjectId>,
    // Blockchain related fields are generally not updated directly via this struct
    // They are typically set when creating from blockchain or via specific sync logic.
    // Add them here only if direct update through a general update endpoint is needed.
    // pub payee: Option<String>,
    // pub payer: Option<String>,
    // pub contract_hash: Option<String>,
    // pub blockchain_timestamp: Option<String>,
    // pub token_batch: Option<String>,
    // pub is_cleared: Option<bool>,
    // pub is_valid: Option<bool>,
}

impl InvoiceRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Invoice>("invoices"),
        }
    }

    // Find invoice by ID
    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<Invoice>, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        self.collection.find_one(filter).await
    }

    // Find all invoices (consider adding pagination/filtering later)
    pub async fn find_all(&self) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! {};
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let mut cursor = self.collection.find(filter).await?;
        let mut results = Vec::new();
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }
        Ok(results)
    }

    // Find invoice by invoice_number
    pub async fn find_by_invoice_number(&self, invoice_number: &str) -> Result<Option<Invoice>, mongodb::error::Error> {
        let filter = doc! { "invoice_number": invoice_number };
        self.collection.find_one(filter).await
    }

    // Find invoice by invoice_number within a transaction session
    pub async fn find_by_number_session(&self, invoice_number: &str, session: &mut ClientSession) -> Result<Option<Invoice>, ServiceError> {
        let filter = doc! { "invoice_number": invoice_number };
        // Use find_one() with session argument
        self.collection.find_one(filter).session(session).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))
    }

    // Find invoices by user_address
    pub async fn find_by_user(&self, user_address: &str) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! { "payee": user_address };
        let find_options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();

        let cursor = self.collection.find(filter).await?;
        let invoices = cursor.try_collect().await?;

        Ok(invoices)
    }

    // Create new invoice from blockchain data
    pub async fn create_from_blockchain(
        &self,
        data: &InvoiceDataDto
    ) -> Result<Invoice, mongodb::error::Error> {
        // Parse the amount from String to u64
        let amount:u64 = data.amount.parse().unwrap();
        
        // Attempt to parse due_date from string (assuming it's a Unix timestamp string)
        let due_date_timestamp = i64::from_str(&data.due_date).unwrap_or_else(|_| chrono::Utc::now().timestamp());
        let due_date = DateTime::from_millis(due_date_timestamp * 1000); // Convert seconds to milliseconds

        // Attempt to parse blockchain_timestamp from string (assuming it's a Unix timestamp string)
        let blockchain_timestamp_val = i64::from_str(&data.timestamp).unwrap_or_else(|_| chrono::Utc::now().timestamp());
        let blockchain_datetime = DateTime::from_millis(blockchain_timestamp_val * 1000);
        
        // Create default ObjectIds for creditor/debtor for now
        // TODO: Implement lookup or creation logic based on payee/payer addresses
        let creditor_id = ObjectId::new(); 
        let debtor_id = ObjectId::new();
        
        // Create a new invoice instance
        let mut invoice = Invoice::new(
            data.invoice_number.clone(),
            creditor_id,
            debtor_id,
            amount,
            "USD".to_string(), // Default currency, consider making dynamic
            due_date,
        );
        
        // Map fields from DTO to the Invoice entity
        invoice.payee = Some(data.payee.clone());
        invoice.payer = Some(data.payer.clone());
        invoice.ipfs_hash = Some(data.ipfs_hash.clone());
        invoice.contract_hash = Some(data.contract_hash.clone());
        invoice.blockchain_timestamp = Some(blockchain_datetime.to_string()); // Store parsed timestamp as string or DateTime?
        invoice.token_batch = Some(data.token_batch.clone());
        invoice.is_cleared = Some(data.is_cleared);
        invoice.is_valid = Some(data.is_valid);
        
        // Optionally update status based on blockchain data
        if data.is_valid {
            invoice.status = InvoiceStatus::Verified;
        }
        if data.is_cleared {
            invoice.status = InvoiceStatus::Repaid;
        }

        // Insert the invoice and get its ID
        let result = self.collection.insert_one(&invoice).await?;
        
        let mut created_invoice = invoice;
        created_invoice.id = result.inserted_id.as_object_id();
        
        Ok(created_invoice)
    }

    // Create new invoice (standard method)
    pub async fn create(
        &self,
        invoice_number: String,
        creditor_id: ObjectId,
        debtor_id: ObjectId,
        amount: u64,
        currency: String,
        due_date: DateTime,
        // Add other relevant fields as needed, e.g., ipfs_hash
    ) -> Result<Invoice, mongodb::error::Error> {
        let invoice = Invoice::new(
            invoice_number,
            creditor_id,
            debtor_id,
            amount,
            currency,
            due_date,
        );
        // Set other fields if provided

        let result = self.collection.insert_one(&invoice).await?;
        
        let mut created_invoice = invoice;
        created_invoice.id = result.inserted_id.as_object_id();
        
        Ok(created_invoice)
    }


    // Generic Update invoice data
    pub async fn update(&self, id: ObjectId, data: UpdateInvoiceData) -> Result<UpdateResult, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        
        // Convert the UpdateInvoiceData struct to a BSON document
        // This automatically skips None values due to `skip_serializing_if`
        let mut update_doc = bson::to_document(&data).map_err(|e| {
            // Provide more context for serialization errors
            mongodb::error::Error::custom(format!("Failed to serialize update data: {}", e))
        })?;
        
        // Always update the 'updated_at' timestamp
        update_doc.insert("updated_at", DateTime::now());

        // Create the $set update operation
        let update = doc! { "$set": update_doc };
        
        // Perform the update operation
        self.collection.update_one(filter, update).await
    }
    
    // Delete invoice by ID
    pub async fn delete(&self, id: ObjectId) -> Result<DeleteResult, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter).await
    }

    // Find invoices by creditor
    pub async fn find_by_creditor(&self, creditor_id: ObjectId) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! { "creditor_id": creditor_id };
        let find_options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        
        let cursor = self.collection.find(filter).await?;
        let invoices = cursor.try_collect().await?;
        
        Ok(invoices)
    }

    // Find invoices by debtor
    pub async fn find_by_debtor(&self, debtor_id: ObjectId) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! { "debtor_id": debtor_id };
        let find_options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        
        let cursor = self.collection.find(filter).await?;
        let invoices = cursor.try_collect().await?;
        
        Ok(invoices)
    }

    // Update invoice status
    pub async fn update_status(&self, id: ObjectId, status: InvoiceStatus) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        // Ensure status is serialized correctly to BSON
        let status_bson = bson::to_bson(&status).map_err(|e| 
            mongodb::error::Error::custom(format!("Failed to serialize status: {}", e))
        )?;
        let update = doc! { "$set": { "status": status_bson, "updated_at": now } };
        
        self.collection.update_one(filter, update).await
    }

    // Add invoice to batch
    pub async fn add_to_batch(&self, id: ObjectId, batch_id: ObjectId) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        // Ensure status is serialized correctly to BSON
        let status_bson = bson::to_bson(&InvoiceStatus::Packaged).map_err(|e| 
            mongodb::error::Error::custom(format!("Failed to serialize status: {}", e))
        )?;
        let update = doc! { 
            "$set": { 
                "batch_id": batch_id, 
                "status": status_bson, 
                "updated_at": now 
            } 
        };
        
        self.collection.update_one(filter, update).await
    }
} 