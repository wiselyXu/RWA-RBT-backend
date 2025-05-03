use std::str::FromStr;
use mongodb::{bson::{self, Decimal128, oid::ObjectId}, Database};
use anyhow::{Result, Context, anyhow};
use chrono::{NaiveDate, TimeZone, Utc};
use chrono::Datelike;
use std::sync::Arc;
use log::{info, error, warn};
use crate::error::ServiceError;

use crate::repository::{
    DailyInterestAccrualRepository,
    UserInvoiceHoldingRepository,
    InvoiceRepository,
    TransactionRepository
};
use common::domain::entity::{DailyInterestAccrual, Transaction, TransactionType};

pub struct InterestCalculationService {
    db: Arc<Database>,
    holding_repo: UserInvoiceHoldingRepository,
    accrual_repo: DailyInterestAccrualRepository,
    invoice_repo: InvoiceRepository,
    transaction_repo: TransactionRepository,
}

impl InterestCalculationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            holding_repo: UserInvoiceHoldingRepository::new(&db),
            accrual_repo: DailyInterestAccrualRepository::new(&db),
            invoice_repo: InvoiceRepository::new(&db),
            transaction_repo: TransactionRepository::new(&db),
            db,
        }
    }
    
    /// 计算并记录所有活跃持仓的每日利息
    pub async fn calculate_daily_interest(&self, date: NaiveDate) -> Result<u32> {
        info!("Starting daily interest calculation for date: {}", date);
        
        // 获取所有活跃持仓
        let active_holdings = self.holding_repo.find_active_holdings().await?;
        
        if active_holdings.is_empty() {
            info!("No active holdings found for interest calculation");
            return Ok(0);
        }
        
        info!("Found {} active holdings for interest calculation", active_holdings.len());
        
        // 当前年份是否是闰年
        let is_leap_year = date.year() % 4 == 0 && (date.year() % 100 != 0 || date.year() % 400 == 0);
        let days_in_year = if is_leap_year { 366.0 } else { 365.0 };
        
        // 将日期转换为MongoDB的DateTime
        let start_of_day_naive = date.and_hms_opt(0, 0, 0).unwrap();
        let start_of_day_utc = Utc.from_utc_datetime(&start_of_day_naive);
        let accrual_date = bson::DateTime::from_millis(start_of_day_utc.timestamp_millis());
        
        let mut success_count = 0;
        
        // 为每个持仓计算利息
        for holding in active_holdings {
            let holding_id = holding.holding_id.clone();
            
            // 检查是否已经计算过这一天的利息
            if self.accrual_repo.has_accrual(&holding_id, accrual_date).await? {
                warn!("Daily interest already calculated for holding {} on {}", holding_id, date);
                continue;
            }
            
            // 获取票据信息，用于获取年利率
            let invoice = match self.invoice_repo.find_by_id(holding.invoice_id).await? {
                Some(invoice) => invoice,
                None => {
                    error!("Invoice {} not found for holding {}", holding.invoice_id, holding_id);
                    continue;
                }
            };
            
            // 计算日利率：年利率 / 当年天数
            let annual_rate = 0.0;
            let daily_rate = annual_rate / days_in_year / 100.0; // 年利率是百分比，需要除以100
            
            // 计算当日利息：本金 * 日利率
            let principal = holding.current_balance.to_string().parse::<f64>().unwrap_or(0.0);
            let daily_interest = principal * daily_rate;
            
            // 转换为Decimal128
            let daily_interest_decimal = Decimal128::from_str(&daily_interest.to_string())
                .map_err(|e| ServiceError::DecimalConversionError(format!("Failed to parse daily interest '{}': {}", daily_interest, e)))?;
            
            // 创建日利息记录
            let accrual = DailyInterestAccrual::new(
                holding.user_id.clone(),
                holding.invoice_id,
                holding_id.clone(),
                accrual_date,
                daily_interest_decimal.clone(),
            );
            
            // 保存到数据库
            self.accrual_repo.create(accrual).await?;
            
            // 更新持仓的累计利息和最后计息日期
            self.holding_repo.update_accrued_interest(
                &holding_id,
                daily_interest_decimal.clone(),
                accrual_date,
            ).await?;
            
            // 创建利息计入的交易记录
            let transaction = Transaction::new(
                holding.user_id.clone(),
                holding.invoice_id,
                holding_id.clone(),
                TransactionType::InterestAccrual,
                daily_interest_decimal,
            );
            
            self.transaction_repo.create(transaction).await?;
            
            success_count += 1;
        }
        
        info!("Successfully calculated daily interest for {} holdings on {}", success_count, date);
        Ok(success_count)
    }
    
    /// 处理到期票据的兑付
    pub async fn process_maturity_payments(&self, date: NaiveDate) -> Result<u32> {
        info!("Processing maturity payments for date: {}", date);
        
        // 获取当天到期的持仓
        let maturing_holdings = self.holding_repo.find_maturing_holdings(date).await?;
        
        if maturing_holdings.is_empty() {
            info!("No maturing holdings found for date: {}", date);
            return Ok(0);
        }
        
        info!("Found {} holdings maturing on {}", maturing_holdings.len(), date);
        
        let mut success_count = 0;
        
        // 处理每个到期的持仓
        for holding in maturing_holdings {
            let holding_id = holding.holding_id.clone();
            
            // 计算到期兑付金额（本金 + 累计利息）
            let principal = holding.purchase_amount.clone();
            let accrued_interest = holding.total_accrued_interest.clone();
            
            // 计算总兑付金额
            let maturity_amount = {
                let p_str = principal.to_string();
                let i_str = accrued_interest.to_string();
                
                let p = p_str.parse::<f64>().unwrap_or(0.0);
                let i = i_str.parse::<f64>().unwrap_or(0.0);
                
                let total = p + i;
                Decimal128::from_str(&total.to_string())
                    .map_err(|e| ServiceError::DecimalConversionError(format!("Failed to parse maturity amount '{}': {}", total, e)))?
            };
            
            // 创建到期兑付的交易记录
            let transaction = Transaction::new_maturity_payment(
                holding.user_id.clone(),
                holding.invoice_id,
                holding_id.clone(),
                maturity_amount,
            );
            
            self.transaction_repo.create(transaction).await?;
            
            // 更新持仓状态为已到期
            self.holding_repo.update_holding_status(
                &holding_id,
                common::domain::entity::HoldingStatus::Matured,
            ).await?;
            
            success_count += 1;
        }
        
        info!("Successfully processed {} maturity payments for date: {}", success_count, date);
        Ok(success_count)
    }
}
