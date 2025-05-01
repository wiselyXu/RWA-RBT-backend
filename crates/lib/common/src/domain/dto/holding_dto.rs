use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use mongodb::bson::{DateTime, Decimal128, oid::ObjectId};
use crate::domain::entity::HoldingStatus;
use salvo::oapi::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HoldingDto {
    pub holding_id: String,
    pub user_id: String,
    pub invoice_id: String,
    pub invoice_number: String,
    pub title: String,
    pub purchase_date: DateTime,
    pub current_balance: String,
    pub total_accrued_interest: String,
    pub annual_rate: f64,
    pub maturity_date: NaiveDate,
    pub status: HoldingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestDetailDto {
    pub accrual_date: NaiveDate,
    pub daily_interest_amount: String,
    pub invoice_title: String,
    pub invoice_number: String,
}