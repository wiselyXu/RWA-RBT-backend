use mongodb::{
    bson::{doc, DateTime, Decimal128, oid::ObjectId},
    Collection, Database, ClientSession,
    options::FindOptions,
    results::InsertOneResult,
};
use anyhow::{Result, anyhow};
use common::domain::entity::DailyInterestAccrual;
use futures::TryStreamExt;
use chrono::NaiveDate;
use crate::error::ServiceError;

pub struct DailyInterestAccrualRepository {
    collection: Collection<DailyInterestAccrual>,
}

impl DailyInterestAccrualRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<DailyInterestAccrual>("daily_interest_accruals"),
        }
    }
    
    // 创建每日利息记录
    pub async fn create(&self, accrual: DailyInterestAccrual) -> Result<DailyInterestAccrual> {
        let mut accrual = accrual;
        let result = self.collection.insert_one(&accrual).await?;
        accrual.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(accrual)
    }
    
    // 创建每日利息记录 within a transaction session
    pub async fn create_session(&self, accrual: DailyInterestAccrual, session: &mut ClientSession) -> Result<DailyInterestAccrual, ServiceError> {
        let mut accrual = accrual;
        let result: InsertOneResult = self.collection.insert_one(&accrual).session(session).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;
        accrual.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(accrual)
    }
    
    // 查询用户的利息明细
    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<DailyInterestAccrual>> {
        let filter = doc! { "user_id": user_id };
        let options = FindOptions::builder()
            .sort(doc! { "accrual_date": -1 }) // 按计息日期倒序
            .build();
        
        let cursor = self.collection.find(filter).await?;
        let accruals = cursor.try_collect().await?;
        Ok(accruals)
    }
    
    // 查询用户特定持仓的利息明细
    pub async fn find_by_user_id_and_holding_id(
        &self,
        user_id: &str,
        holding_id: &str,
    ) -> Result<Vec<DailyInterestAccrual>> {
        let filter = doc! {
            "user_id": user_id,
            "holding_id": holding_id,
        };
        let options = FindOptions::builder()
            .sort(doc! { "accrual_date": -1 }) // 按计息日期倒序
            .build();
        
        let cursor = self.collection.find(filter).await?;
        let accruals = cursor.try_collect().await?;
        Ok(accruals)
    }
    
    // 检查指定日期的利息是否已计算
    pub async fn has_accrual(
        &self,
        holding_id: &str,
        accrual_date: DateTime,
    ) -> Result<bool> {
        let start_of_day = DateTime::from_millis(
            accrual_date.timestamp_millis() - (accrual_date.timestamp_millis() % 86400000) // 取当天0点
        );
        let end_of_day = DateTime::from_millis(start_of_day.timestamp_millis() + 86399999); // 当天23:59:59
        
        let filter = doc! {
            "holding_id": holding_id,
            "accrual_date": {
                "$gte": start_of_day,
                "$lte": end_of_day
            }
        };
        
        let count = self.collection.count_documents(filter).await?;
        Ok(count > 0)
    }
}