use mongodb::{Database, bson::{doc, Decimal128, oid::ObjectId}};
use anyhow::{Result, anyhow};
use chrono::{Utc, NaiveDate, Duration};
use crate::{
    cache::InvoiceRedisService,
    repository::{
        UserInvoiceHoldingRepository,
        DailyInterestAccrualRepository,
        TransactionRepository,
    },
};
use common::domain::{
    entity::{
        UserInvoiceHolding,
        Transaction,
        DailyInterestAccrual,
        HoldingStatus,
    },
    dto::{
        invoice_dto::InvoiceRedisDto,
        purchase_invoice_dto::PurchaseInvoiceDto,
        holding_dto:: HoldingDto,
        interest_detail_dto::InterestDetailDto,
    },
};
use redis::Client as RedisClient;

pub struct InvoiceService {
    db: Database,
    invoice_redis_service: InvoiceRedisService,
    user_holding_repo: UserInvoiceHoldingRepository,
    interest_accrual_repo: DailyInterestAccrualRepository,
    transaction_repo: TransactionRepository,
}

impl InvoiceService {
    pub fn new(db: Database, redis_client: RedisClient) -> Self {
        Self {
            user_holding_repo: UserInvoiceHoldingRepository::new(&db),
            interest_accrual_repo: DailyInterestAccrualRepository::new(&db),
            transaction_repo: TransactionRepository::new(&db),
            invoice_redis_service: InvoiceRedisService::new(redis_client),
            db,
        }
    }
    
    // 获取所有可购买的票据
    pub async fn get_available_invoices(&self) -> Result<Vec<InvoiceRedisDto>> {
        self.invoice_redis_service.get_available_invoices()
    }
    
    // 购买票据
    pub async fn purchase_invoice(&self, user_id: &str, purchase_req: PurchaseInvoiceDto) -> Result<String> {
        // 验证票据是否可购买
        let invoice_id = &purchase_req.invoice_id;
        let shares = purchase_req.shares;
        
        let invoice = self.invoice_redis_service.get_invoice(invoice_id)?
            .ok_or_else(|| anyhow!("票据不存在"))?;
            
        if !invoice.is_available_for_purchase() {
            return Err(anyhow!("票据当前不可购买"));
        }
        
        if shares == 0 {
            return Err(anyhow!("购买份数不能为0"));
        }
        
        if shares > invoice.available_shares {
            return Err(anyhow!("购买份数超过可用份数"));
        }
        
        // 计算购买金额
        let purchase_amount = Decimal128::from_string(&format!("{}", shares as f64 * invoice.share_price))?;
        
        // 开始事务
        let mut session = self.db.client().start_session(None).await?;
        session.start_transaction(None).await?;
        
        let obj_invoice_id = ObjectId::parse_str(invoice_id)?;
        
        // 1. 创建用户持仓记录
        let holding = UserInvoiceHolding::new(
            user_id.to_string(),
            obj_invoice_id,
            purchase_amount
        );
        
        let holding = self.user_holding_repo.create(holding).await?;
        
        // 2. 记录交易
        let transaction = Transaction::new_purchase(
            user_id.to_string(),
            obj_invoice_id,
            holding.holding_id.clone(),
            purchase_amount
        );
        
        self.transaction_repo.create(transaction).await?;
        
        // 3. 更新Redis中的票据可用份数
        self.invoice_redis_service.update_invoice_shares(invoice_id, shares)?;
        
        // 提交事务
        session.commit_transaction().await?;
        
        Ok(holding.holding_id)
    }
    
    // 获取用户持仓列表
    pub async fn get_user_holdings(&self, user_id: &str) -> Result<Vec<HoldingDto>> {
        let holdings = self.user_holding_repo.find_by_user_id(user_id).await?;
        
        let mut holding_dtos = Vec::new();
        for holding in holdings {
            if let Some(invoice) = self.invoice_redis_service.get_invoice(&holding.invoice_id.to_hex())? {
                let holding_dto = HoldingDto {
                    holding_id: holding.holding_id.clone(),
                    invoice_id: holding.invoice_id.to_hex(),
                    invoice_number: invoice.invoice_number,
                    title: invoice.title,
                    purchase_date: holding.purchase_date,
                    current_balance: holding.current_balance.to_string(),
                    total_accrued_interest: holding.total_accrued_interest.to_string(),
                    annual_rate: invoice.annual_rate,
                    maturity_date: invoice.maturity_date,
                    status: holding.holding_status,
                };
                holding_dtos.push(holding_dto);
            }
        }
        
        Ok(holding_dtos)
    }
    
    // 获取持仓利息明细
    pub async fn get_holding_interest_details(&self, user_id: &str, holding_id: &str) -> Result<Vec<InterestDetailDto>> {
        // 验证持仓归属
        let holding = self.user_holding_repo.find_by_user_id_and_holding_id(user_id, holding_id).await?
            .ok_or_else(|| anyhow!("持仓记录不存在或不属于当前用户"))?;
            
        let accruals = self.interest_accrual_repo.find_by_user_id_and_holding_id(user_id, holding_id).await?;
        
        let invoice_opt = self.invoice_redis_service.get_invoice(&holding.invoice_id.to_hex())?;
        
        let mut details = Vec::new();
        for accrual in accruals {
            let detail = InterestDetailDto {
                accrual_date: NaiveDate::from_ymd_opt(
                    accrual.accrual_date.timestamp_millis() as i32 / 1000 / 86400 + 1970, 
                    1, 
                    1
                ).unwrap() + Duration::days(
                    accrual.accrual_date.timestamp_millis() as i64 / 1000 / 86400 - (1970 * 365)
                ),
                daily_interest_amount: accrual.daily_interest_amount.to_string(),
                invoice_title: invoice_opt.as_ref().map_or("未知票据".to_string(), |i| i.title.clone()),
                invoice_number: invoice_opt.as_ref().map_or("未知编号".to_string(), |i| i.invoice_number.clone()),
            };
            details.push(detail);
        }
        
        Ok(details)
    }
    
    // 计算指定日期的所有持仓利息（定时任务使用）
    pub async fn calculate_daily_interest(&self, accrual_date: NaiveDate) -> Result<(usize, usize, Vec<String>)> {
        let active_holdings = self.user_holding_repo.find_active_holdings().await?;
        
        let mut success_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();
        
        let accrual_datetime = mongodb::bson::DateTime::from_chrono(
            accrual_date.and_hms_opt(0, 0, 0).unwrap().naive_utc()
        );
        
        for holding in active_holdings {
            match self.calculate_holding_interest(&holding, accrual_datetime).await {
                Ok(true) => success_count += 1,
                Ok(false) => skipped_count += 1,
                Err(e) => {
                    let error_msg = format!(
                        "计算利息失败: 持仓ID={}, 用户ID={}, 错误={}", 
                        holding.holding_id, holding.user_id, e
                    );
                    errors.push(error_msg);
                }
            }
        }
        
        Ok((success_count, skipped_count, errors))
    }
    
    // 计算单个持仓的利息
    async fn calculate_holding_interest(&self, holding: &UserInvoiceHolding, accrual_date: mongodb::bson::DateTime) -> Result<bool> {
        // 检查持仓状态
        if !matches!(holding.holding_status, HoldingStatus::Active) {
            return Ok(false); // 跳过非活跃持仓
        }
        
        // 获取票据信息
        let invoice = self.invoice_redis_service.get_invoice(&holding.invoice_id.to_hex())?
            .ok_or_else(|| anyhow!("票据信息不存在"))?;
            
        // 检查当前计算日期是否在计息区间内
        let accrual_naive_date = NaiveDate::from_ymd_opt(
            accrual_date.timestamp_millis() as i32 / 1000 / 86400 + 1970, 
            1, 
            1
        ).unwrap() + Duration::days(
            accrual_date.timestamp_millis() as i64 / 1000 / 86400 - (1970 * 365)
        );
        
        if accrual_naive_date < invoice.issue_date || accrual_naive_date >= invoice.maturity_date {
            return Ok(false); // 不在计息区间内
        }
        
        // 检查是否已经计算过该日期的利息（幂等性）
        if accrual_date.timestamp_millis() <= holding.last_accrual_date.timestamp_millis() {
            return Ok(false); // 已计算过
        }
        
        // 检查是否已存在该日期的利息记录
        let already_has_accrual = self.interest_accrual_repo.has_accrual(&holding.holding_id, accrual_date).await?;
        if already_has_accrual {
            return Ok(false); // 已有记录
        }
        
        // 计算日利息
        let is_leap_year = accrual_naive_date.year() % 4 == 0 && 
                          (accrual_naive_date.year() % 100 != 0 || accrual_naive_date.year() % 400 == 0);
        
        let daily_rate = invoice.calculate_daily_rate(is_leap_year);
        let current_balance_f64 = holding.current_balance.to_string().parse::<f64>()?;
        let daily_interest = current_balance_f64 * daily_rate;
        
        let daily_interest_decimal = Decimal128::from_string(&format!("{:.8}", daily_interest))?;
        
        // 创建利息记录
        let accrual = DailyInterestAccrual::new(
            holding.user_id.clone(),
            holding.invoice_id,
            holding.holding_id.clone(),
            accrual_date,
            daily_interest_decimal
        );
        
        self.interest_accrual_repo.create(accrual).await?;
        
        // 更新持仓记录中的累计利息和最后计息日期
        self.user_holding_repo.update_accrued_interest(
            &holding.holding_id,
            daily_interest_decimal,
            accrual_date
        ).await?;
        
        Ok(true)
    }
    
    // 处理到期票据
    pub async fn process_maturing_invoices(&self, maturity_date: NaiveDate) -> Result<(usize, Vec<String>)> {
        let maturity_holdings = self.user_holding_repo.find_maturing_holdings(maturity_date).await?;
        
        let mut success_count = 0;
        let mut errors = Vec::new();
        
        for holding in maturity_holdings {
            match self.process_maturity_payment(&holding).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    let error_msg = format!(
                        "处理到期兑付失败: 持仓ID={}, 用户ID={}, 错误={}", 
                        holding.holding_id, holding.user_id, e
                    );
                    errors.push(error_msg);
                }
            }
        }
        
        Ok((success_count, errors))
    }
    
    // 处理单个到期票据的兑付
    async fn process_maturity_payment(&self, holding: &UserInvoiceHolding) -> Result<()> {
        // 计算应付总额：本金 + 累计利息
        let principal = holding.current_balance.to_string().parse::<f64>()?;
        let interest = holding.total_accrued_interest.to_string().parse::<f64>()?;
        let total_payment = principal + interest;
        
        let total_payment_decimal = Decimal128::from_string(&format!("{:.8}", total_payment))?;
        
        // 开始事务
        let mut session = self.db.client().start_session(None).await?;
        session.start_transaction(None).await?;
        
        // 1. 记录兑付交易
        let transaction = Transaction::new_maturity_payment(
            holding.user_id.clone(),
            holding.invoice_id,
            holding.holding_id.clone(),
            total_payment_decimal
        );
        
        self.transaction_repo.create(transaction).await?;
        
        // 2. 更新持仓状态为已到期
        self.user_holding_repo.update_holding_status(
            &holding.holding_id,
            HoldingStatus::Matured
        ).await?;
        
        // 3. 在此处执行实际的资金兑付操作（例如调用支付系统）
        // TODO: 实现实际的资金兑付逻辑
        
        // 提交事务
        session.commit_transaction().await?;
        
        Ok(())
    }
}
