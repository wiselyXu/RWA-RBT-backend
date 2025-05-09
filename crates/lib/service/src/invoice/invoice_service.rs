use std::str::FromStr;
use mongodb::{Database, bson::{doc, Decimal128, oid::ObjectId}};
use anyhow::{Result, anyhow};
use chrono::{Utc, NaiveDate, Duration, Datelike, TimeZone};
use crate::{
   
    repository::{
        UserRepository,
        UserInvoiceHoldingRepository,
        DailyInterestAccrualRepository,
        TransactionRepository,
        InvoiceRepository,
    },
};
use common::domain::{
    entity::{
        UserInvoiceHolding,
        Transaction,
        DailyInterestAccrual,
        HoldingStatus,
        TransactionType,
        Invoice,
        invoice_status::InvoiceStatus,
    },
    dto::{

        purchase_invoice_dto::PurchaseInvoiceDto,
        holding_dto:: HoldingDto,
        interest_detail_dto::InterestDetailDto,
    },
};
use redis::Client as RedisClient;
use common::domain::dto::invoice_redis_dto::InvoiceRedisDto;
use crate::cache::InvoiceRedisService;
use futures::TryStreamExt;
use log::{error, info, warn};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use mongodb::{ClientSession, Collection};
use std::collections::HashMap;
use std::sync::Arc;
use mongodb::bson::DateTime;
use crate::error::ServiceError;
use mongodb::error::Error as MongoError;
use futures::FutureExt;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal_macros::dec;

pub struct InvoiceService {
    db: Database,
    invoice_redis_service: InvoiceRedisService,
    user_repo: UserRepository,
    user_holding_repo: UserInvoiceHoldingRepository,
    interest_accrual_repo: DailyInterestAccrualRepository,
    transaction_repo: TransactionRepository,
    invoice_repository: InvoiceRepository,
}

impl InvoiceService {
    pub fn new(db: Database, redis_client: RedisClient) -> Self {
        Self {
            user_repo: UserRepository::new(&db),
            user_holding_repo: UserInvoiceHoldingRepository::new(&db),
            interest_accrual_repo: DailyInterestAccrualRepository::new(&db),
            transaction_repo: TransactionRepository::new(&db),
            invoice_repository: InvoiceRepository::new(&db),
            invoice_redis_service: InvoiceRedisService::new(redis_client),
            db,
        }
    }
    
    // 获取所有可购买的票据
    pub async fn get_available_invoices(&self) -> Result<Vec<InvoiceRedisDto>, ServiceError> {
        self.invoice_redis_service.get_available_invoices()
    }
    
    // 上链功能(将票据状态从Pending更新为Verified)
    pub async fn verify_invoice(&self, invoice_id: &str) -> Result<Invoice, ServiceError> {
        // 将invoice_id从字符串转换为ObjectId
        let obj_id = ObjectId::from_str(invoice_id)
            .map_err(|_| ServiceError::InternalError(format!("Invalid invoice id: {}", invoice_id)))?;
            
        // 获取票据
        let invoice = self.invoice_repository.find_by_id(obj_id).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to find invoice: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound(format!("Invoice not found: {}", invoice_id)))?;
            
        // 检查票据状态是否为Pending
        if invoice.status != InvoiceStatus::Pending {
            return Err(ServiceError::InternalError(format!(
                "Invoice cannot be verified: status is {:?}, expected Pending", 
                invoice.status
            )));
        }
        
        // 更新票据状态为Verified
        self.invoice_repository.update_status(obj_id, InvoiceStatus::Verified).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to update invoice status: {}", e)))?;
            
        // 获取并返回更新后的票据
        self.invoice_repository.find_by_id(obj_id).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to find updated invoice: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound(format!("Updated invoice not found: {}", invoice_id)))
    }
    
    // 批量发行票据到市场(将票据状态从Verified更新为OnSale)
    pub async fn issue_invoices(&self, invoice_ids: &[String]) -> Result<usize, ServiceError> {
        if invoice_ids.is_empty() {
            return Err(ServiceError::InvoiceNotIssue("No invoices selected for issuance".to_string()));
        }

        // 首先收集所有有效的票据进行一致性验证
        let mut valid_invoices = Vec::new();
        
        for invoice_id in invoice_ids {            
            // 获取票据
            let invoice = match self.invoice_repository.find_by_id(invoice_id.parse().unwrap()).await {
                Ok(Some(invoice)) => invoice,
                Ok(None) => {
                    warn!("Invoice not found: {}", invoice_id);
                    // Consider if you want to continue processing other valid invoices or stop entirely.
                    // For now, we'll break as per original logic.
                    return Err(ServiceError::InvoiceNotFound(invoice_id.clone()));
                },
                Err(e) => {
                    error!("Failed to find invoice {}: {}", invoice_id, e);
                    // Propagate the error or handle as appropriate
                    return Err(ServiceError::MongoDbError(format!("Failed to find invoice {}: {}", invoice_id, e)));
                }
            };
            
            // 检查票据状态是否为Verified
            if invoice.status != InvoiceStatus::Verified {
                warn!(
                    "Invoice {} cannot be issued: status is {:?}, expected Verified", 
                    invoice_id, invoice.status
                );
                // Return an error if an invoice is not in the correct state
                return Err(ServiceError::InvoiceNotIssue(format!(
                    "Invoice {} cannot be issued: status is {:?}, expected Verified", 
                    invoice_id, invoice.status
                )));
            }
            valid_invoices.push(invoice);
        }
        
        // 如果没有有效票据，返回错误
        if valid_invoices.is_empty() {
            return Err(ServiceError::InvoiceNotIssue("No valid verified invoices found for issuance".to_string()));
        }
        
        // 验证所有票据的债权人、债务人和货币种类是否一致 (This validation block is kept as per previous logic)
        let first_invoice = &valid_invoices[0];
        let expected_payee = &first_invoice.payee;
        let expected_payer = &first_invoice.payer;
        let expected_currency = &first_invoice.currency;
        
        for invoice in &valid_invoices {
            if invoice.payee != *expected_payee || 
               invoice.payer != *expected_payer || 
               invoice.currency != *expected_currency {
                return Err(ServiceError::InvoiceNotIssue(
                    format!("All invoices must have the same payee, payer and currency. Invoice {} has different values.", 
                    invoice.invoice_number)
                ));
            }
        }
        
        // Transactions are removed. Operations will be performed individually.
        
        let mut success_count = 0;
        
        // Create InvoiceBatch instance
        let invoice_batch_data = common::domain::entity::invoice_batch::InvoiceBatch::new(
            expected_payee,
            expected_payer,
            expected_currency.clone()
        );
        
        let batch_repo = crate::repository::InvoiceBatchRepository::new(&self.db);
        
        // Save the batch to the database (assuming a 'create' method exists or will be created that doesn't require a session)
        let saved_batch = match batch_repo.create(&invoice_batch_data).await {
            Ok(batch) => batch,
            Err(e) => {
                error!("Failed to create invoice batch: {}", e);
                return Err(ServiceError::from(e)); // Propagate the error
            }
        };
        
        // Update all valid invoices to be Packaged and associate them with the batch
        for invoice in &valid_invoices {
            if let Some(id) = invoice.id {
                if let Some(batch_obj_id) = saved_batch.id {
                    match self.invoice_repository.add_to_batch(id, batch_obj_id).await {
                        Ok(_) => {
                            success_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to add invoice {} to batch {}: {}", id, batch_obj_id, e);
                            // Decide how to handle partial failures. For now, log and continue,
                            // but the batch is created. Consider if this is the desired behavior.
                            // Alternatively, you could try to "undo" or mark the batch as problematic.
                        }
                    }
                } else {
                    // This should ideally not happen if batch creation was successful and returned an ID.
                    error!("Batch ID is missing after creation, cannot associate invoice {}", id);
                    // Consider returning an error here as it indicates a problem with batch creation logic.
                    return Err(ServiceError::InternalError("Batch ID is missing after successful creation".to_string()));
                }
            }
        }
        
        if success_count == 0 && !valid_invoices.is_empty() {
            // If a batch was created but no invoices could be added to it.
            // This might leave an empty batch. Consider if this is acceptable or if the batch should be deleted/marked.
            error!("Batch {} created, but failed to add any invoices to it.", saved_batch.id.map_or_else(|| "UnknownID".to_string(), |id| id.to_string()));
            return Err(ServiceError::InternalError("Batch created, but failed to associate any invoices.".to_string()));
        } else if success_count < valid_invoices.len() {
             warn!("Batch {} created. Successfully added {} out of {} invoices.",
                saved_batch.id.map_or_else(|| "UnknownID".to_string(), |id| id.to_string()),
                success_count,
                valid_invoices.len()
            );
            // Partial success, proceed to update batch status
        }


        // Update the batch status to Issued
        if let Some(batch_obj_id) = saved_batch.id {
            match batch_repo.update_status(batch_obj_id, common::domain::entity::invoice_batch::InvoiceBatchStatus::Issued).await {
                Ok(_) => {
                    info!("Successfully issued {} invoices into batch {}. Batch status updated to Issued.", success_count, batch_obj_id);
                    Ok(success_count)
                }
                Err(e) => {
                    error!("Failed to update batch {} status to Issued: {}. {} invoices were associated.", batch_obj_id, e, success_count);
                    // Even if batch status update fails, invoices were associated.
                    // Return the count of associated invoices but log the status update error.
                    // The caller needs to be aware that the batch might not be in 'Issued' state.
                    // Consider if a more specific error should be returned to indicate this.
                    Err(ServiceError::MongoDbError(format!("Failed to update batch status to Issued for batch {}: {}", batch_obj_id, e)))
                }
            }
        } else {
            // This case should ideally be caught earlier
            error!("Cannot update status for a batch with no ID. {} invoices processed.", success_count);
            Err(ServiceError::InternalError("Cannot update status for a batch with no ID.".to_string()))
        }
    }
    
    // 购买票据
    pub async fn purchase_invoice(&self, user_id: &str, purchase_req: PurchaseInvoiceDto) -> Result<String> {
        // 验证票据是否可购买
        let invoice_id = &purchase_req.invoice_id;
        let purchase_amount_f64 = purchase_req.purchase_amount;
        
        let invoice = self.invoice_redis_service.get_invoice(invoice_id)?
            .ok_or_else(|| anyhow!("票据不存在"))?;
            
        if !invoice.is_available_for_purchase() {
            return Err(anyhow!("票据当前不可购买"));
        }
        
        // Calculate shares and validate amount/shares (using rust_decimal)
        let purchase_amount_dec = Decimal::from_f64(purchase_amount_f64)
            .ok_or_else(|| anyhow!("无效的购买金额格式"))?;
        let share_price_dec = Decimal::from_f64(invoice.share_price)
             .ok_or_else(|| anyhow!("无效的 Redis 份额价格格式"))?;
        let zero_dec = dec!(0.0);
        let one_dec = dec!(1.0);

        if purchase_amount_dec <= zero_dec || share_price_dec <= zero_dec {
             return Err(anyhow!("购买金额和份额价格必须为正"));
        }

        let calculated_shares_dec = (purchase_amount_dec / share_price_dec).round();
        let calculated_shares = calculated_shares_dec.to_u64()
             .ok_or_else(|| anyhow!("计算出的份额无效"))?;

        let actual_purchase_amount_dec = calculated_shares_dec * share_price_dec;
        let actual_purchase_decimal128 = Decimal128::from_str(&actual_purchase_amount_dec.to_string())
            .map_err(|e| anyhow!("无法转换最终购买金额: {}", e))?;

        if calculated_shares == 0 {
            return Err(anyhow!("购买金额太小，无法购买任何份额"));
        }

        if calculated_shares > invoice.available_shares {
            return Err(anyhow!("购买份数 ({}) 超过可用份数 ({})", calculated_shares, invoice.available_shares));
        }
        
        // 开始事务
        let mut session = self.db.client().start_session().await?;
        session.start_transaction().await?;
        
        let obj_invoice_id = ObjectId::parse_str(invoice_id)?;
        
        // 1. 创建用户持仓记录
        let holding = UserInvoiceHolding::new(
            user_id.to_string(),
            obj_invoice_id,
            actual_purchase_decimal128.clone()
        );
        
        let holding = self.user_holding_repo.create_session(holding, &mut session).await?;
        
        // 2. 记录交易
        let transaction = Transaction::new_purchase(
            user_id.to_string(),
            obj_invoice_id,
            holding.holding_id.clone(),
            actual_purchase_decimal128.clone()
        );
        
        self.transaction_repo.create_session(transaction, &mut session).await?;
        
        // 3. 更新Redis中的票据可用份数
        self.invoice_redis_service.update_invoice_shares(invoice_id, calculated_shares)?;
        
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
                    user_id: holding.user_id.clone(),
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
    
    // Renamed from process_daily_interest_accrual to match original intent
    pub async fn calculate_daily_interest_for_date(&self, accrual_date: NaiveDate) -> Result<u32, ServiceError> {
        info!("Processing daily interest accrual for date: {}", accrual_date);
        
        // Convert NaiveDate to BSON DateTime
        let start_of_day_naive = accrual_date.and_hms_opt(0, 0, 0).unwrap();
        let start_of_day_utc = Utc.from_utc_datetime(&start_of_day_naive);
        let accrual_datetime = DateTime::from_millis(start_of_day_utc.timestamp_millis());

        // Find all active holdings
        let active_holdings = self.user_holding_repo.find_active_holdings().await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?; 

        let mut success_count = 0; 
        // Loop through each active holding
        for holding in active_holdings {
            let holding_id_str = holding.holding_id.clone(); 

            // --- Idempotency Check (outside transaction) --- 
            let already_accrued = self.interest_accrual_repo.has_accrual(&holding_id_str, accrual_datetime).await
                 .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;
            if already_accrued {
                warn!("Interest already accrued for holding {} on {}. Skipping.", holding_id_str, accrual_date);
                continue;
            }
            
            // Get invoice details (needed for rate calculation, can be outside transaction)
            // Using Redis cache for efficiency
             let invoice = match self.invoice_redis_service.get_invoice(&holding.invoice_id.to_hex()) {
                 Ok(Some(inv)) => inv,
                 Ok(None) => {
                     error!("Invoice info not found in Redis for holding {}. Cannot calculate interest.", holding_id_str);
                     continue; // Skip this holding
                 }
                 Err(e) => {
                    error!("Failed to get invoice {} from Redis for holding {}: {}. Skipping.", holding.invoice_id.to_hex(), holding_id_str, e);
                    continue; // Skip this holding
                 }
             };

            // --- Date Range Check --- 
            // Convert BSON accrual_datetime back to NaiveDate for comparison
            // Need a utility function or manual conversion here
            let accrual_naive_date = { 
                let millis = accrual_datetime.timestamp_millis();
                let naive = chrono::NaiveDateTime::from_timestamp_millis(millis).unwrap();
                 Utc.from_utc_datetime(&naive).date_naive()
             }; // Get NaiveDate

            if accrual_naive_date < invoice.issue_date || accrual_naive_date >= invoice.maturity_date {
                warn!("Skipping holding {} as accrual date {} is outside interest period ({}-{}).", 
                       holding_id_str, accrual_naive_date, invoice.issue_date, invoice.maturity_date);
                continue;
            }

            // --- Calculate Interest (outside transaction) --- 
            let is_leap_year = accrual_naive_date.year() % 4 == 0 && (accrual_naive_date.year() % 100 != 0 || accrual_naive_date.year() % 400 == 0);
            let days_in_year = if is_leap_year { 366.0 } else { 365.0 };
            let annual_rate = invoice.annual_rate; 
            let daily_rate = annual_rate / days_in_year / 100.0; 
            let principal = holding.current_balance; 
            let principal_f64 = principal.to_string().parse::<f64>().unwrap_or(0.0); // Handle potential parse error better?
            let daily_interest_f64 = principal_f64 * daily_rate;
            let daily_interest_decimal = match Decimal128::from_str(&format!("{:.8}", daily_interest_f64)) {
                Ok(d) => d,
                Err(e) => {
                    error!("Failed to parse daily interest '{}' for holding {}: {}. Skipping.", daily_interest_f64, holding_id_str, e);
                    continue;
                }
            };
            
            // --- Start Transaction per Holding --- 
            let mut session = match self.db.client().start_session().await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to start session for holding {}: {}. Skipping.", holding_id_str, e);
                    continue;
                }
            };
            let transaction_result = session.start_transaction()
                .and_run(
                    (self, &holding, accrual_datetime, daily_interest_decimal.clone()), // Clone interest decimal
                    |session, (service, h, date, interest)| {
                        async move { 
                            // a. Create DailyInterestAccrual record
                            let accrual = DailyInterestAccrual::new(
                                h.user_id.clone(),
                                h.invoice_id,
                                h.holding_id.clone(),
                                date.clone(), 
                                interest.clone(),
                            );
                            service.interest_accrual_repo.create_session(accrual, session).await?; 

                            // b. Update holding's total_accrued_interest and last_accrual_date
                            service.user_holding_repo.update_accrued_interest_session(
                                &h.holding_id,
                                interest.clone(),
                                date.clone(), 
                                session
                            ).await?;

                            // c. Create InterestAccrual transaction record
                            let transaction = Transaction::new( 
                                h.user_id.clone(),
                                h.invoice_id,
                                h.holding_id.clone(),
                                TransactionType::InterestAccrual, 
                                interest.clone(),
                            );
                            service.transaction_repo.create_session(transaction, session).await?;
                            
                            Ok(())
                        }
                        .map(|res| res.map_err(|service_err: ServiceError| {
                            MongoError::custom(Box::new(service_err))
                        }))
                       .boxed()
                    }
                ).await;

            match transaction_result {
                Ok(_) => {
                    success_count += 1;
                    info!("Successfully accrued interest for holding {}", holding_id_str);
                }
                Err(e) => {
                    error!("Failed transaction for interest accrual on holding {}: {:?}", holding_id_str, e);
                    // Optionally try to decode the custom error if needed for specific logging
                    if let MongoError { kind: ref error_kind, .. } = e {
                        if let mongodb::error::ErrorKind::Custom(inner_error) = &**error_kind {
                            if let Some(service_error) = inner_error.downcast_ref::<ServiceError>() {
                                error!("(ServiceError details: {:?})", service_error);
                            }
                        }
                    }
                    // Continue with the next holding
                }
            }
            // --- End Transaction per Holding ---
        }

        info!("Finished daily interest accrual process for date {}. Accrued for {} holdings.", accrual_date, success_count);
        Ok(success_count)
    }

    // Renamed from process_maturity_payments
    pub async fn process_maturity_payments_for_date(&self, payment_date: NaiveDate) -> Result<u32, ServiceError> {
        info!("Processing maturity payments for date: {}", payment_date);

        // Find holdings maturing on this date
        let maturing_holdings = self.user_holding_repo.find_maturing_holdings(payment_date).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;

        if maturing_holdings.is_empty() {
            info!("No maturing holdings found for date: {}", payment_date);
            return Ok(0);
        }

        info!("Found {} holdings maturing on {}", maturing_holdings.len(), payment_date);
        let mut success_count = 0;

        for holding in maturing_holdings {
             let holding_id_str = holding.holding_id.clone();

             // --- Calculate Payment Amount (outside transaction) --- 
             let principal = holding.purchase_amount.clone();
             let accrued_interest = holding.total_accrued_interest.clone();
             // TODO: Implement robust Decimal128 addition if necessary
             let principal_f64 = principal.to_string().parse::<f64>().unwrap_or(0.0); // Handle parse error?
             let interest_f64 = accrued_interest.to_string().parse::<f64>().unwrap_or(0.0); // Handle parse error?
             let total_payment_f64 = principal_f64 + interest_f64;
             let total_payment_decimal = match Decimal128::from_str(&format!("{:.8}", total_payment_f64)) {
                 Ok(d) => d,
                 Err(e) => {
                     error!("Failed to parse total payment '{}' for holding {}: {}. Skipping.", total_payment_f64, holding_id_str, e);
                     continue;
                 }
             };

            // --- Start Transaction per Holding --- 
            let mut session = match self.db.client().start_session().await {
                 Ok(s) => s,
                 Err(e) => {
                     error!("Failed to start session for maturity payment on holding {}: {}. Skipping.", holding_id_str, e);
                     continue;
                 }
            };
            let transaction_result = session.start_transaction()
                .and_run(
                    (self, &holding, total_payment_decimal.clone()), // Clone payment decimal
                    |session, (service, h, payment_amount)| {
                        async move { 
                            // a. Create MaturityPayment transaction record
                            let transaction = Transaction::new_maturity_payment(
                                h.user_id.clone(),
                                h.invoice_id,
                                h.holding_id.clone(),
                                payment_amount.clone(),
                            );
                            service.transaction_repo.create_session(transaction, session).await?;

                            // b. Update holding status to Matured
                            service.user_holding_repo.update_holding_status_session(
                                &h.holding_id,
                                HoldingStatus::Matured, 
                                session,
                            ).await?;
                            
                            // c. Simulate crediting user balance
                            let user_addr = &h.user_id;
                            let update_successful = service.user_repo.update_balance_session(user_addr, payment_amount.clone(), session).await?;
                            if !update_successful {
                                error!("Failed to credit balance for user {} during maturity payment.", user_addr);
                                // Note: Returning error here rolls back the whole transaction
                                return Err(ServiceError::BalanceUpdateFailed(user_addr.to_string()));
                            }
                            info!("Credited maturity payment {} to user {} within transaction", payment_amount, user_addr);

                            Ok(())
                        }
                        .map(|res| res.map_err(|service_err: ServiceError| {
                            MongoError::custom(Box::new(service_err))
                        }))
                       .boxed()
                    }
                ).await;

             match transaction_result {
                Ok(_) => {
                    success_count += 1;
                    info!("Successfully processed maturity for holding {}", holding_id_str);
                    // TODO: Trigger actual off-chain payout here AFTER successful commit?
                    // Or should payout be triggered by listening to events/DB changes?
                }
                Err(e) => {
                    error!("Failed transaction for maturity payment on holding {}: {:?}", holding_id_str, e);
                     if let MongoError { kind: ref error_kind, .. } = e {
                        if let mongodb::error::ErrorKind::Custom(inner_error) = &**error_kind {
                            if let Some(service_error) = inner_error.downcast_ref::<ServiceError>() {
                                error!("(ServiceError details: {:?})", service_error);
                            }
                        }
                    }
                    // Continue with the next holding
                }
            }
            // --- End Transaction per Holding --- 
        }

        info!("Successfully processed {} maturity payments for date: {}", success_count, payment_date);
        Ok(success_count)
    }
}
