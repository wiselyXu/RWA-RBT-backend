use crate::error::ServiceError;
use futures::stream::TryStreamExt;
use mongodb::{
    ClientSession, Collection, Database,
    bson::{self, DateTime, Decimal128, doc, oid::ObjectId},
    options::{FindOneOptions, FindOptions, UpdateOptions},
    results::{DeleteResult, UpdateResult, InsertOneResult},
};
use serde::Serialize;

use common::domain::entity::invoice_batch::{InvoiceBatch, InvoiceBatchStatus};

pub struct InvoiceBatchRepository {
    collection: Collection<InvoiceBatch>,
}

impl InvoiceBatchRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<InvoiceBatch>("invoice_batches"),
        }
    }

    // Find batch by ID
    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<InvoiceBatch>, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        self.collection.find_one(filter).await
    }

    // Find all batches
    pub async fn find_all(&self) -> Result<Vec<InvoiceBatch>, mongodb::error::Error> {
        let filter = doc! {};
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.collection.find(filter).with_options(find_options).await?;
        cursor.try_collect().await
    }

    // Find batches by creditor_id
    pub async fn find_by_creditor(&self, creditor_id: ObjectId) -> Result<Vec<InvoiceBatch>, mongodb::error::Error> {
        let filter = doc! { "creditor_id": creditor_id };
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.collection.find(filter).with_options(find_options).await?;
        cursor.try_collect().await
    }

    // Find batches by debtor_id
    pub async fn find_by_debtor(&self, debtor_id: ObjectId) -> Result<Vec<InvoiceBatch>, mongodb::error::Error> {
        let filter = doc! { "debtor_id": debtor_id };
        let find_options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.collection.find(filter).with_options(find_options).await?;
        cursor.try_collect().await
    }

    // Create a new batch
    pub async fn create(&self, batch: &InvoiceBatch) -> Result<InvoiceBatch, mongodb::error::Error> {
        let result = self.collection.insert_one(batch).await?;
        let mut created_batch = batch.clone();
        created_batch.id = result.inserted_id.as_object_id();
        Ok(created_batch)
    }

    // Create a new batch within a session
    pub async fn create_with_session(&self, batch: &InvoiceBatch, session: &mut ClientSession) -> Result<InvoiceBatch, ServiceError> {
        let result = self.collection
            .insert_one(batch)
            .session(session)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create batch: {}", e)))?;
            
        let mut created_batch = batch.clone();
        created_batch.id = result.inserted_id.as_object_id();
        Ok(created_batch)
    }

    // Update batch status
    pub async fn update_status(&self, id: ObjectId, status: InvoiceBatchStatus) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        
        // Ensure status is serialized correctly to BSON
        let status_bson = bson::to_bson(&status)
            .map_err(|e| mongodb::error::Error::custom(format!("Failed to serialize status: {}", e)))?;
            
        let update = doc! { "$set": { "status": status_bson, "updated_at": now } };

        self.collection.update_one(filter, update).await
    }

    // Update batch token address
    pub async fn update_token_address(&self, id: ObjectId, token_address: &str) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        let update = doc! { 
            "$set": { 
                "rbt_token_address": token_address, 
                "status": bson::to_bson(&InvoiceBatchStatus::Issued)?, 
                "updated_at": now 
            } 
        };

        self.collection.update_one(filter, update).await
    }

    // Delete batch
    pub async fn delete(&self, id: ObjectId) -> Result<DeleteResult, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter).await
    }

    // Update batch token batch reference
    pub async fn update_token_batch_id_with_session(
        &self, 
        id: ObjectId, 
        token_batch_id: ObjectId,
        session: &mut ClientSession
    ) -> Result<UpdateResult, ServiceError> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        let update = doc! { 
            "$set": { 
                "token_batch_id": token_batch_id, 
                "updated_at": now 
            } 
        };

        self.collection.update_one(filter, update)
            .session(session)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to update token batch id: {}", e)))
    }
    
    // Update batch status with session
    pub async fn update_status_with_session(
        &self, 
        id: ObjectId, 
        status: InvoiceBatchStatus,
        session: &mut ClientSession
    ) -> Result<UpdateResult, ServiceError> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        
        // Ensure status is serialized correctly to BSON
        let status_bson = bson::to_bson(&status)
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to serialize status: {}", e)))?;
            
        let update = doc! { "$set": { "status": status_bson, "updated_at": now } };

        self.collection.update_one(filter, update)
            .session(session)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to update status: {}", e)))
    }
} 