use anyhow::{Result, anyhow};
use redis::{Client, Commands, Connection, AsyncCommands};

use serde_json;
use std::sync::Arc;
use common::domain::dto::invoice_redis_dto::InvoiceRedisDto;
use common::domain::entity::InvoiceStatus;

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
    pub fn get_available_invoices(&self) -> Result<Vec<InvoiceRedisDto>> {
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
    pub fn update_invoice_shares(&self, invoice_id: &str, purchased_shares: u64) -> Result<()> {
        let mut conn = self.get_connection()?;
        
        let key = format!("invoice:{}", invoice_id);
        let exists: bool = conn.exists(&key)?;
        
        if !exists {
            return Err(anyhow!("票据不存在"));
        }
        
        let json_data: String = conn.get(&key)?;
        let mut invoice: InvoiceRedisDto = serde_json::from_str(&json_data)?;
        
        if invoice.available_shares < purchased_shares {
            return Err(anyhow!("可用份数不足"));
        }
        
        // 更新可用份数
        invoice.available_shares -= purchased_shares;
        
        // 如果可用份数为0，更新状态
        if invoice.available_shares == 0 {
            invoice.status = InvoiceStatus::SoldOut;
        }
        
        // 保存更新后的票据数据
        let updated_json = serde_json::to_string(&invoice)?;
        conn.set(&key, updated_json)?;
        
        Ok(())
    }
    
    // 添加新票据到Redis
    pub fn add_invoice(&self, invoice: InvoiceRedisDto) -> Result<()> {
        let mut conn = self.get_connection()?;
        
        let key = format!("invoice:{}", invoice.invoice_id);
        let json_data = serde_json::to_string(&invoice)?;
        
        conn.set(&key, json_data)?;
        
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
}
