use anyhow::Result;
use common::domain::entity::{Transaction, TransactionType};
use futures::TryStreamExt;
use mongodb::{Collection, Database, bson::{DateTime, Decimal128, doc, oid::ObjectId}, options::FindOptions, ClientSession, results::InsertOneResult};
use crate::error::ServiceError;

pub struct TransactionRepository {
    collection: Collection<Transaction>,
}

impl TransactionRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("transactions"),
        }
    }

    // 创建交易记录
    pub async fn create(&self, transaction: Transaction) -> Result<Transaction> {
        let result = self.collection.insert_one(transaction).await?;
        let id = result.inserted_id.as_object_id().unwrap();

        let created_transaction = self.find_by_id(id).await?;
        Ok(created_transaction.unwrap())
    }

    // 创建交易记录 within a transaction session
    pub async fn create_session(&self, transaction: Transaction, session: &mut ClientSession) -> Result<Transaction, ServiceError> {
        let mut transaction = transaction;
        let result: InsertOneResult = self.collection.insert_one(&transaction).session(session).await
             .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;
        transaction.id = Some(result.inserted_id.as_object_id().unwrap()); // Assign the MongoDB generated _id
        // We return the input struct updated with the ID, as insert_one_with_session doesn't return the doc.
        Ok(transaction)
    }

    // 根据ID查找交易记录
    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<Transaction>> {
        let filter = doc! { "_id": id };
        let result = self.collection.find_one(filter).await?;
        Ok(result)
    }

    // 根据用户ID查找交易记录
    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Transaction>> {
        let filter = doc! { "user_id": user_id };

        let cursor = self.collection.find(filter).await?;
        let transactions = cursor.try_collect().await?;

        Ok(transactions)
    }

    // 根据持仓ID查找交易记录
    pub async fn find_by_holding_id(&self, holding_id: &str) -> Result<Vec<Transaction>> {
        let filter = doc! { "holding_id": holding_id };

        let cursor = self.collection.find(filter).await?;
        let transactions = cursor.try_collect().await?;

        Ok(transactions)
    }

    // 根据用户ID和交易类型查找交易记录
    pub async fn find_by_user_id_and_type(&self, user_id: &str, transaction_type: &str) -> Result<Vec<Transaction>> {
        let filter = doc! {
            "user_id": user_id,
            "transaction_type": transaction_type
        };

        let cursor = self.collection.find(filter).await?;
        let transactions = cursor.try_collect().await?;

        Ok(transactions)
    }
}
