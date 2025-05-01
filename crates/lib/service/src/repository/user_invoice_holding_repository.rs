use mongodb::{
    bson::{doc, DateTime, Decimal128, oid::ObjectId},
    Collection, Database,
    options::FindOptions,
};
use anyhow::{Result, anyhow};
use common::domain::entity::{UserInvoiceHolding, HoldingStatus};
use futures::TryStreamExt;
use chrono::{Utc, NaiveDate};

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
                "holding_status": status,
                "updated_at": DateTime::now(),
            }
        };
        
        self.collection.update_one(filter, update).await?;
        Ok(())
    }
    
    // 查询到期日为指定日期的活跃持仓
    pub async fn find_maturing_holdings(&self, maturity_date: NaiveDate) -> Result<Vec<UserInvoiceHolding>> {
        // 需要与Invoice集合做关联查询，这里使用聚合管道
        let start_of_day = DateTime::from_chrono(maturity_date.and_hms_opt(0, 0, 0).unwrap().naive_utc());
        let end_of_day = DateTime::from_chrono(maturity_date.and_hms_opt(23, 59, 59).unwrap().naive_utc());
        
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
        let results = cursor.try_collect().await?;
        
        Ok(results)
    }
}