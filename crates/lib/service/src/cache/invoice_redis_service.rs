use anyhow::{Result, anyhow};
use redis::{AsyncCommands, Client, Commands, Connection, FromRedisValue, RedisError, ToRedisArgs};

use common::domain::dto::invoice_redis_dto::InvoiceRedisDto;

use serde_json;
use crate::error::ServiceError;

pub struct InvoiceRedisService {
    client: Client,
}

impl InvoiceRedisService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    // 获取Redis连接
    fn get_connection(&self) -> Result<Connection> {
        let conn = self.client.get_connection()?;
        Ok(conn)
    }

    // 获取所有可用票据
    pub fn get_available_invoices(&self) -> Result<Vec<InvoiceRedisDto>, ServiceError> {
        let mut conn = self.get_connection()?;

        // 获取所有票据的键
        let invoice_keys: Vec<String> = conn.keys("invoice:*")?;

        if invoice_keys.is_empty() {
            return Ok(Vec::new());
        }

        // 获取所有票据的JSON数据
        let mut invoices = Vec::new();
        for key in invoice_keys {
            let json_data: String = conn.get(&key)?;
            let invoice: InvoiceRedisDto = serde_json::from_str(&json_data)?;

            // 只返回可购买的票据
            if invoice.is_available_for_purchase() {
                invoices.push(invoice);
            }
        }

        Ok(invoices)
    }

    // 获取指定ID的票据
    pub fn get_invoice(&self, invoice_id: &str) -> Result<Option<InvoiceRedisDto>> {
        let mut conn = self.get_connection()?;

        let key = format!("invoice:{}", invoice_id);
        let exists: bool = conn.exists(&key)?;

        if !exists {
            return Ok(None);
        }

        let json_data: String = conn.get(&key)?;
        let invoice: InvoiceRedisDto = serde_json::from_str(&json_data)?;

        Ok(Some(invoice))
    }

    // 更新票据的可用份数
    pub fn update_invoice_shares(&self, invoice_id: &str, shares_to_remove: u64) -> Result<(), ServiceError> {
        let key = format!("invoice:{}", invoice_id);
        let mut conn = self.get_connection()?;

        // Use WATCH/MULTI/EXEC for atomic update if possible
        // Basic implementation (prone to race conditions):
        let current_json: String = conn.get(&key)?;
        let mut invoice: InvoiceRedisDto = serde_json::from_str(&current_json)
            .map_err(|e| ServiceError::SerializationError(format!("Failed to deserialize invoice from Redis: {}", e)))?;

        if invoice.available_shares < shares_to_remove {
            return Err(ServiceError::InvalidPurchaseShares(shares_to_remove, invoice.available_shares));
        }

        invoice.available_shares -= shares_to_remove;

        let updated_json = serde_json::to_string(&invoice)
            .map_err(|e| ServiceError::SerializationError(format!("Failed to serialize updated invoice: {}", e)))?;

        // Ignore the result of set
        let _: () = conn.set(&key, updated_json)?; 
        Ok(())
    }

    // 添加新票据到Redis
    pub fn add_invoice(&self, invoice: InvoiceRedisDto) -> Result<(), ServiceError> {
        let mut conn = self.get_connection()?;

        let key = format!("invoice:{}", invoice.invoice_id);
        let json_data = serde_json::to_string(&invoice)
            .map_err(|e| ServiceError::SerializationError(e.to_string()))?;

        let _: () = conn.set(&key, json_data)?;

        Ok(())
    }

    // 删除Redis中的票据
    pub fn delete_invoice(&self, invoice_id: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;

        let key = format!("invoice:{}", invoice_id);
        let deleted: i32 = conn.del(&key)?;

        // 返回是否成功删除（删除数量大于0）
        Ok(deleted > 0)
    }

    // Create or update an invoice in Redis
    pub fn set_invoice(&self, invoice: &InvoiceRedisDto) -> Result<(), ServiceError> {
        let key = format!("invoice:{}", invoice.invoice_id);
        let json_data = serde_json::to_string(invoice)
            .map_err(|e| ServiceError::SerializationError(format!("Failed to serialize invoice: {}", e)))?;
        let mut conn = self.get_connection()?;
        // Specify the return type for conn.set to () to satisfy FromRedisValue
        let _: () = conn.set(&key, json_data)?; 
        Ok(())
    }
}
