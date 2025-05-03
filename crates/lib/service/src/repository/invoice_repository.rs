use crate::error::ServiceError;
use futures::stream::TryStreamExt;
use mongodb::{
    ClientSession, Collection, Database,
    bson::{self, DateTime, Decimal128, doc, oid::ObjectId},
    options::{FindOneOptions, FindOptions, UpdateOptions},
    results::{DeleteResult, UpdateResult},
};
use serde::Serialize;

use common::domain::entity::Invoice;

use chrono;
use common::domain::dto::invoice_dto::{CreateInvoiceDto};
use common::domain::entity::invoice_status::InvoiceStatus;



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
        let mut cursor = self.collection.find(filter).with_options(find_options).await?;
        let mut results = Vec::new();
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }
        log::warn!("Finding all items:{:?}", results.clone());
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
        self.collection
            .find_one(filter)
            .session(session)
            .await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))
    }

    // Find invoices by user_address
    pub async fn find_by_user(&self, user_address: &str) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! { "payee": user_address };
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();

        let cursor = self.collection.find(filter).await?;
        let invoices = cursor.try_collect().await?;

        Ok(invoices)
    }

    // Create new invoice from data provided by frontend/API (not directly from blockchain event)
    pub async fn create_from_blockchain(&self, data: &CreateInvoiceDto) -> Result<Invoice, ServiceError> {
        // Create a new invoice instance
        let mut invoice = Invoice::new(data);

        // Default status for new invoices created via API
        invoice.status = InvoiceStatus::Pending;

        // Insert the invoice and get its ID
        let result = self
            .collection
            .insert_one(&invoice)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to insert invoice: {}", e)))?;

        let mut created_invoice = invoice;
        created_invoice.id = result.inserted_id.as_object_id();

        Ok(created_invoice)
    }

    // Create new invoice (standard method)
    pub async fn create(&self, data: &CreateInvoiceDto) -> Result<Invoice, mongodb::error::Error> {
        let invoice = Invoice::new(data);
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
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();

        let cursor = self.collection.find(filter).await?;
        let invoices = cursor.try_collect().await?;

        Ok(invoices)
    }

    // Find invoices by debtor
    pub async fn find_by_debtor(&self, debtor_id: ObjectId) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! { "debtor_id": debtor_id };
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();

        let cursor = self.collection.find(filter).await?;
        let invoices = cursor.try_collect().await?;

        Ok(invoices)
    }

    // Update invoice status
    pub async fn update_status(&self, id: ObjectId, status: InvoiceStatus) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        // Ensure status is serialized correctly to BSON
        let status_bson = bson::to_bson(&status).map_err(|e| mongodb::error::Error::custom(format!("Failed to serialize status: {}", e)))?;
        let update = doc! { "$set": { "status": status_bson, "updated_at": now } };

        self.collection.update_one(filter, update).await
    }

    // Add invoice to batch
    pub async fn add_to_batch(&self, id: ObjectId, batch_id: ObjectId) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        // Ensure status is serialized correctly to BSON
        let status_bson = bson::to_bson(&InvoiceStatus::Packaged).map_err(|e| mongodb::error::Error::custom(format!("Failed to serialize status: {}", e)))?;
        let update = doc! {
            "$set": {
                "batch_id": batch_id,
                "status": status_bson,
                "updated_at": now
            }
        };

        self.collection.update_one(filter, update).await
    }

    // 查找属于特定批次的所有发票
    pub async fn find_by_batch_id(&self, batch_id: ObjectId) -> Result<Vec<Invoice>, mongodb::error::Error> {
        let filter = doc! { "batch_id": batch_id };
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.collection.find(filter).with_options(find_options).await?;
        cursor.try_collect().await
    }
}
