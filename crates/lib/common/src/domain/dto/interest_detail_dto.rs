use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestDetailDto {
    pub accrual_date: NaiveDate,
    pub daily_interest_amount: String,
    pub invoice_title: String,
    pub invoice_number: String,
}
