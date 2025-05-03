use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime, Decimal128, Document, from_document, to_document},
    Collection, Database,
    options::FindOptions,
};
use std::sync::Arc;
use log::{debug, error, info};

use common::domain::entity::{
    TokenBatch, TokenBatchStatus, TokenMarket, TokenHolding, TokenHoldingStatus,
    TokenTransaction, TokenTransactionType, TokenTransactionStatus,
};

use crate::error::ServiceError;

pub struct TokenRepository {
    db: Arc<Database>,
    token_batch_collection: Collection<Document>,
    token_market_collection: Collection<Document>,
    token_holding_collection: Collection<Document>,
    token_transaction_collection: Collection<Document>,
}

impl TokenRepository {
    pub fn new(db: Arc<Database>) -> Self {
        TokenRepository {
            token_batch_collection: db.collection("token_batches"),
            token_market_collection: db.collection("token_markets"),
            token_holding_collection: db.collection("token_holdings"),
            token_transaction_collection: db.collection("token_transactions"),
            db,
        }
    }

    // Token Batch CRUD operations
    pub async fn create_token_batch(&self, token_batch: TokenBatch) -> Result<ObjectId> {
        let mut token_batch = token_batch;
        let now = DateTime::now();
        token_batch.created_at = now;
        token_batch.updated_at = now;
        
        let doc = to_document(&token_batch)?;
        
        let result = self.token_batch_collection.insert_one(doc).await?;
        
        let id = result.inserted_id.as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted id"))?;
        
        info!("Created token batch with id: {}", id);
        Ok(id)
    }

    pub async fn update_token_batch(&self, id: ObjectId, token_batch: TokenBatch) -> Result<()> {
        let mut token_batch = token_batch;
        token_batch.updated_at = DateTime::now();
        
        let doc = to_document(&token_batch)?;
        
        let filter = doc! { "_id": id };
        
        let result = self.token_batch_collection.replace_one(filter, doc).await?;
        
        if result.modified_count == 0 {
            return Err(anyhow::anyhow!("Token batch not found"));
        }
        
        info!("Updated token batch with id: {}", id);
        Ok(())
    }

    pub async fn get_token_batch_by_id(&self, id: ObjectId) -> Result<TokenBatch> {
        let filter = doc! { "_id": id };
        
        let result = self.token_batch_collection.find_one(filter).await?;
        
        match result {
            Some(doc) => {
                let token_batch = from_document(doc)?;
                Ok(token_batch)
            },
            None => Err(anyhow::anyhow!("Token batch not found")),
        }
    }

    pub async fn list_token_batches(
        &self, 
        status: Option<TokenBatchStatus>,
        creditor_id: Option<ObjectId>,
        stablecoin_symbol: Option<String>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<TokenBatch>> {
        let mut filter = Document::new();
        
        if let Some(status) = status {
            filter.insert("status", status.to_string());
        }
        
        if let Some(creditor_id) = creditor_id {
            filter.insert("creditor_id", creditor_id);
        }
        
        if let Some(stablecoin_symbol) = stablecoin_symbol {
            filter.insert("stablecoin_symbol", stablecoin_symbol);
        }
        
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let skip = (page - 1) * page_size;
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(page_size as i64)
            .sort(doc! { "created_at": -1 })
            .build();
        
        let mut cursor = self.token_batch_collection.find(filter).await?;
        
        let mut token_batches = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            match from_document(document) {
                Ok(token_batch) => token_batches.push(token_batch),
                Err(e) => {
                    error!("Failed to convert document to token batch: {}", e);
                    // Continue with other documents
                }
            }
        }
        
        Ok(token_batches)
    }

    // Token Batch CRUD operations with session support
    pub async fn create_token_batch_with_session(&self, token_batch: TokenBatch, session: &mut mongodb::ClientSession) -> Result<ObjectId> {
        let mut token_batch = token_batch;
        let now = DateTime::now();
        token_batch.created_at = now;
        token_batch.updated_at = now;
        
        let doc = to_document(&token_batch)?;
        
        let result = self.token_batch_collection.insert_one(doc)
            .session(session)
            .await?;
        
        let id = result.inserted_id.as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted id"))?;
        
        info!("Created token batch with id: {}", id);
        Ok(id)
    }

    // Token Market CRUD operations
    pub async fn create_token_market(&self, token_market: TokenMarket) -> Result<ObjectId> {
        let mut token_market = token_market;
        let now = DateTime::now();
        token_market.created_at = now;
        token_market.updated_at = now;
        
        let doc = to_document(&token_market)?;
        
        let result = self.token_market_collection.insert_one(doc).await?;
        
        let id = result.inserted_id.as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted id"))?;
        
        info!("Created token market with id: {}", id);
        Ok(id)
    }

    pub async fn update_token_market(&self, id: ObjectId, token_market: TokenMarket) -> Result<()> {
        let mut token_market = token_market;
        token_market.updated_at = DateTime::now();
        
        let doc = to_document(&token_market)?;
        
        let filter = doc! { "_id": id };
        
        let result = self.token_market_collection.replace_one(filter, doc).await?;
        
        if result.modified_count == 0 {
            return Err(anyhow::anyhow!("Token market not found"));
        }
        
        info!("Updated token market with id: {}", id);
        Ok(())
    }

    pub async fn get_token_market_by_id(&self, id: ObjectId) -> Result<TokenMarket> {
        let filter = doc! { "_id": id };
        
        let result = self.token_market_collection.find_one(filter).await?;
        
        match result {
            Some(doc) => {
                let token_market = from_document(doc)?;
                Ok(token_market)
            },
            None => Err(anyhow::anyhow!("Token market not found")),
        }
    }

    pub async fn get_token_market_by_batch_id(&self, batch_id: ObjectId) -> Result<Option<TokenMarket>> {
        let filter = doc! { "batch_id": batch_id };
        
        let result = self.token_market_collection.find_one(filter).await?;
        
        match result {
            Some(doc) => {
                let token_market = from_document(doc)?;
                Ok(Some(token_market))
            },
            None => Ok(None),
        }
    }

    pub async fn list_token_markets(
        &self,
        stablecoin_symbol: Option<String>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<TokenMarket>> {
        let mut filter = Document::new();
        
        if let Some(stablecoin_symbol) = stablecoin_symbol {
            filter.insert("stablecoin_symbol", stablecoin_symbol);
        }
        
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let skip = (page - 1) * page_size;
        
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(page_size as i64)
            .sort(doc! { "created_at": -1 })
            .build();
        
        let mut cursor = self.token_market_collection.find(filter).await?;
        
        let mut token_markets = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            match from_document(document) {
                Ok(token_market) => token_markets.push(token_market),
                Err(e) => {
                    error!("Failed to convert document to token market: {}", e);
                    // Continue with other documents
                }
            }
        }
        
        Ok(token_markets)
    }

    // Token Market CRUD operations with session support
    pub async fn create_token_market_with_session(&self, token_market: TokenMarket, session: &mut mongodb::ClientSession) -> Result<ObjectId> {
        let mut token_market = token_market;
        let now = DateTime::now();
        token_market.created_at = now;
        token_market.updated_at = now;
        
        let doc = to_document(&token_market)?;
        
        let result = self.token_market_collection.insert_one(doc)
            .session(session)
            .await?;
        
        let id = result.inserted_id.as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted id"))?;
        
        info!("Created token market with id: {}", id);
        Ok(id)
    }

    // Token Holding CRUD operations
    pub async fn create_token_holding(&self, token_holding: TokenHolding) -> Result<ObjectId> {
        let mut token_holding = token_holding;
        let now = DateTime::now();
        token_holding.created_at = now;
        token_holding.updated_at = now;
        
        let doc = to_document(&token_holding)?;
        
        let result = self.token_holding_collection.insert_one(doc).await?;
        
        let id = result.inserted_id.as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted id"))?;
        
        info!("Created token holding with id: {}", id);
        Ok(id)
    }

    pub async fn get_token_holdings_by_user_id(&self, user_id: ObjectId) -> Result<Vec<TokenHolding>> {
        let filter = doc! { "user_id": user_id };
        
        let find_options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .build();
        
        let mut cursor = self.token_holding_collection.find(filter).await?;
        
        let mut token_holdings = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            match from_document(document) {
                Ok(token_holding) => token_holdings.push(token_holding),
                Err(e) => {
                    error!("Failed to convert document to token holding: {}", e);
                    // Continue with other documents
                }
            }
        }
        
        Ok(token_holdings)
    }

    // Token Transaction CRUD operations
    pub async fn create_token_transaction(&self, token_transaction: TokenTransaction) -> Result<ObjectId> {
        let mut token_transaction = token_transaction;
        let now = DateTime::now();
        token_transaction.created_at = now;
        token_transaction.updated_at = now;
        
        let doc = to_document(&token_transaction)?;
        
        let result = self.token_transaction_collection.insert_one(doc).await?;
        
        let id = result.inserted_id.as_object_id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get inserted id"))?;
        
        info!("Created token transaction with id: {}", id);
        Ok(id)
    }

    pub async fn get_token_transactions_by_user_id(&self, user_id: ObjectId) -> Result<Vec<TokenTransaction>> {
        let filter = doc! { "user_id": user_id };
        
        let find_options = FindOptions::builder()
            .sort(doc! { "transaction_date": -1 })
            .build();
        
        let mut cursor = self.token_transaction_collection.find(filter).await?;
        
        let mut token_transactions = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            match from_document(document) {
                Ok(token_transaction) => token_transactions.push(token_transaction),
                Err(e) => {
                    error!("Failed to convert document to token transaction: {}", e);
                    // Continue with other documents
                }
            }
        }
        
        Ok(token_transactions)
    }
} 