use std::cmp::PartialEq;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

use mongodb::bson::{oid::ObjectId, DateTime};
use salvo::oapi::ToSchema;
use crate::domain::entity::invoice_status::InvoiceStatus;

#[derive(Debug, Clone,Serialize,PartialEq, Deserialize, ToSchema)]
pub struct InvoiceRedisDto {
    pub invoice_id: String,
    pub invoice_number: String,
    pub title: String,
    pub description: Option<String>,
    pub annual_rate: f64,
    pub total_shares: u64,
    pub available_shares: u64,
    pub share_price: f64,
    pub issue_date: NaiveDate,
    pub maturity_date: NaiveDate,
    pub status: InvoiceStatus,
}


impl InvoiceRedisDto {
    pub fn calculate_daily_rate(&self, is_leap_year: bool) -> f64 {
        let days_in_year = if is_leap_year { 366.0 } else { 365.0 };
        self.annual_rate / 100.0 / days_in_year
    }
    
    pub fn is_available_for_purchase(&self) -> bool {
        self.status == InvoiceStatus::Packaged && self.available_shares > 0
    }
    
    pub fn update_available_shares(&mut self, purchased_shares: u64) -> Result<(), String> {
        if purchased_shares > self.available_shares {
            return Err(format!(
                "购买份数({})超过可用份数({})", 
                purchased_shares, self.available_shares
            ));
        }
        
        self.available_shares -= purchased_shares;
        
        // 如果可用份数为0，则将状态更新为售罄
        if self.available_shares == 0 {
            self.status = InvoiceStatus::Repaid;
        }
        Ok(())
    }
}
