use mongodb::{bson::{doc, DateTime, Decimal128, oid::ObjectId, Bson}, Collection, Database, ClientSession, options::{FindOptions, UpdateOptions}, bson};
use anyhow::{Result, anyhow};
use common::domain::entity::{UserInvoiceHolding, HoldingStatus};
use futures::stream::{StreamExt, TryStreamExt};
use chrono::{Utc, NaiveDate, TimeZone};
use crate::error::ServiceError;

pub struct UserInvoiceHoldingRepository {
    collection: Collection<UserInvoiceHolding>,
}

impl UserInvoiceHoldingRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<UserInvoiceHolding>("user_invoice_holdings"),
        }
    }
    
    // 创建新的持仓记录
    pub async fn create(&self, holding: UserInvoiceHolding) -> Result<UserInvoiceHolding> {
        let mut holding = holding;
        let result = self.collection.insert_one(&holding).await?;
        holding.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(holding)
    }
    
    // 创建新的持仓记录 within a transaction session
    pub async fn create_session(&self, holding: UserInvoiceHolding, session: &mut ClientSession) -> Result<UserInvoiceHolding, ServiceError> {
        let mut holding = holding;
        // Use .insert_one().session()
        let result = self.collection.insert_one(&holding).session(session).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;
        holding.id = Some(result.inserted_id.as_object_id().unwrap()); // Assign the MongoDB generated _id
        Ok(holding) // Return the original holding struct, now with the db id
    }
    
    // 根据用户ID查询持仓列表
    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<UserInvoiceHolding>> {
        let filter = doc! { "user_id": user_id };
        let options = FindOptions::builder().sort(doc! { "created_at": -1 }).build();
        let cursor = self.collection.find(filter).await?;
        let holdings = cursor.try_collect().await?;
        Ok(holdings)
    }
    
    // 根据用户ID和持仓ID查询具体持仓
    pub async fn find_by_user_id_and_holding_id(&self, user_id: &str, holding_id: &str) -> Result<Option<UserInvoiceHolding>> {
        let filter = doc! {
            "user_id": user_id,
            "holding_id": holding_id,
        };
        self.collection.find_one(filter).await.map_err(|e| anyhow!("查询失败: {}", e))
    }
    
    // 查询所有活跃的持仓
    pub async fn find_active_holdings(&self) -> Result<Vec<UserInvoiceHolding>> {
        let filter = doc! { "holding_status": "Active" };
        let cursor = self.collection.find(filter).await?;
        let holdings = cursor.try_collect().await?;
        Ok(holdings)
    }
    
    // 更新累计利息和最后计息日期
    pub async fn update_accrued_interest(
        &self,
        holding_id: &str,
        additional_interest: Decimal128,
        accrual_date: DateTime,
    ) -> Result<()> {
        let filter = doc! { "holding_id": holding_id };
        let update = doc! {
            "$inc": { "total_accrued_interest": additional_interest },
            "$set": { 
                "last_accrual_date": accrual_date,
                "updated_at": DateTime::now(),
            }
        };
        
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
    
    // 更新持仓状态（例如到期）
    pub async fn update_holding_status(
        &self,
        holding_id: &str,
        status: HoldingStatus,
    ) -> Result<()> {
        let filter = doc! { "holding_id": holding_id };
        let update = doc! {
            "$set": { 
                "holding_status": bson::to_bson(&status).map_err(|e| ServiceError::SerializationError(e.to_string()))?,
                "updated_at": DateTime::now(),
            }
        };
        
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
    
    // 更新累计利息和最后计息日期 within a transaction session
    pub async fn update_accrued_interest_session(
        &self,
        holding_id: &str,
        additional_interest: Decimal128,
        accrual_date: DateTime,
        session: &mut ClientSession,
    ) -> Result<(), ServiceError> {
        let filter = doc! { "holding_id": holding_id };
        let update = doc! {
            "$inc": { "total_accrued_interest": additional_interest },
            "$set": { 
                "last_accrual_date": accrual_date,
                "updated_at": DateTime::now(),
            }
        };
        
        self.collection.update_one(filter, update).session(session).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;
        Ok(())
    }
    
    // 更新持仓状态（例如到期）within a transaction session
    pub async fn update_holding_status_session(
        &self,
        holding_id: &str,
        status: HoldingStatus,
        session: &mut ClientSession,
    ) -> Result<(), ServiceError> {
        let filter = doc! { "holding_id": holding_id };
        let update = doc! {
            "$set": { 
                "holding_status": bson::to_bson(&status).map_err(|e| ServiceError::SerializationError(e.to_string()))?,
                "updated_at": DateTime::now(),
            }
        };
        
        self.collection.update_one(filter, update).session(session).await
             .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;
        Ok(())
    }
    
    // 查询到期日为指定日期的活跃持仓
    pub async fn find_maturing_holdings(&self, maturity_date: NaiveDate) -> Result<Vec<UserInvoiceHolding>> {
        // 需要与Invoice集合做关联查询，这里使用聚合管道
        // Convert NaiveDate to bson::DateTime via chrono::DateTime<Utc> and millis
        let start_of_day_naive = maturity_date.and_hms_opt(0, 0, 0).unwrap();
        let end_of_day_naive = maturity_date.and_hms_opt(23, 59, 59).unwrap();

        let start_of_day_utc = Utc.from_utc_datetime(&start_of_day_naive);
        let end_of_day_utc = Utc.from_utc_datetime(&end_of_day_naive);

        let start_of_day = DateTime::from_millis(start_of_day_utc.timestamp_millis());
        let end_of_day = DateTime::from_millis(end_of_day_utc.timestamp_millis());
        
        let pipeline = vec![
            doc! {
                "$lookup": {
                    "from": "invoices",
                    "localField": "invoice_id",
                    "foreignField": "_id",
                    "as": "invoice"
                }
            },
            doc! {
                "$match": {
                    "holding_status": "Active",
                    "invoice.maturity_date": {
                        "$gte": start_of_day,
                        "$lte": end_of_day
                    }
                }
            },
            doc! {
                "$project": {
                    "invoice": 0
                }
            }
        ];
        
        let cursor = self.collection.aggregate(pipeline).await?;
        // Collect documents first, then deserialize
        let docs: Vec<bson::Document> = cursor.try_collect().await?;
        let mut results = Vec::new();
        for doc in docs {
            match bson::from_document(doc) {
                Ok(holding) => results.push(holding),
                Err(e) => {
                    // Log or handle deserialization error for individual documents
                    eprintln!("Failed to deserialize holding from aggregation: {}", e); 
                    // Depending on requirements, you might return an error or just skip the doc
                    // return Err(anyhow!("Failed to deserialize holding: {}", e));
                }
            }
        }
        
        Ok(results)
    }
}