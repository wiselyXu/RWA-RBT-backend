use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use salvo::oapi::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InterestDetailDto {
    pub accrual_date: NaiveDate,
    pub daily_interest_amount: String,
    pub invoice_title: String,
    pub invoice_number: String,
}
