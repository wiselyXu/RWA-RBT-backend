use std::str::FromStr;
use anyhow::{Result, Context, anyhow};
use mongodb::{Database, bson::{self, Decimal128, doc}, Client};
use std::sync::Arc;
use log::{info, error, warn};
use common::domain::dto::invoice_redis_dto::InvoiceRedisDto;
use common::domain::dto::purchase_invoice_dto::PurchaseInvoiceDto;
use common::domain::entity::{UserInvoiceHolding, Transaction, TransactionType, User};
use crate::repository::{UserRepository, InvoiceRepository, UserInvoiceHoldingRepository, TransactionRepository};
use crate::cache::InvoiceRedisService;
use crate::error::ServiceError;
use futures::future::FutureExt;
use mongodb::error::Error as MongoError;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal_macros::dec;

pub struct PurchaseService {
    client: Arc<Client>,
    redis_service: Arc<InvoiceRedisService>,
    user_repo: UserRepository,
    invoice_repo: InvoiceRepository,
    holding_repo: UserInvoiceHoldingRepository,
    transaction_repo: TransactionRepository,
}

impl PurchaseService {
    pub fn new(client: Arc<Client>, redis_service: Arc<InvoiceRedisService>) -> Self {
        let db = Arc::new(client.database("rwa-db"));
        Self {
            user_repo: UserRepository::new(&db),
            invoice_repo: InvoiceRepository::new(&db),
            holding_repo: UserInvoiceHoldingRepository::new(&db),
            transaction_repo: TransactionRepository::new(&db),
            client,
            redis_service,
        }
    }
    
    /// 用户购买票据 (使用事务)
    pub async fn purchase_invoice(&self, user_address: &str, purchase_data: &PurchaseInvoiceDto) -> Result<UserInvoiceHolding, ServiceError> {
        info!("Processing invoice purchase for user: {}, invoice: {}, amount: {}", 
              user_address, purchase_data.invoice_id, purchase_data.purchase_amount);
        
        // 1. 从Redis获取票据信息
        let invoice_redis = self.redis_service.get_invoice(&purchase_data.invoice_id)?
            .ok_or_else(|| ServiceError::InvoiceNotFound(purchase_data.invoice_id.clone()))?;
            
        // 2. 验证票据是否可购买
        if !invoice_redis.is_available_for_purchase() {
            return Err(ServiceError::InvoiceNotAvailable(purchase_data.invoice_id.clone()));
        }

        // 3. 验证和计算购买信息 (Using rust_decimal for precision)
        let purchase_amount_dec = Decimal::from_f64(purchase_data.purchase_amount)
            .ok_or_else(|| ServiceError::DecimalConversionError("Invalid purchase amount format".to_string()))?;
        let share_price_dec = Decimal::from_f64(invoice_redis.share_price)
             .ok_or_else(|| ServiceError::DecimalConversionError("Invalid share price format in Redis".to_string()))?;
        let zero_dec = dec!(0.0);
        let one_dec = dec!(1.0);
        
        if purchase_amount_dec <= zero_dec || share_price_dec <= zero_dec {
             return Err(ServiceError::InvalidPurchaseAmount("Purchase amount and share price must be positive".to_string()));
        }

        // Calculate shares. Round to nearest integer. Handle potential division by zero already checked.
        // Ensure we round correctly to avoid small amounts buying 0 shares or large amounts losing a share.
        let calculated_shares_dec = (purchase_amount_dec / share_price_dec).round();
        let calculated_shares = calculated_shares_dec.to_u64()
            .ok_or_else(|| ServiceError::InvalidPurchaseAmount("Calculated shares resulted in an invalid number".to_string()))?;
        
        // Recalculate the actual purchase amount based on whole shares to ensure consistency
        let actual_purchase_amount_dec = calculated_shares_dec * share_price_dec;
        let actual_purchase_decimal128 = Decimal128::from_str(&actual_purchase_amount_dec.to_string())
            .map_err(|e| ServiceError::DecimalConversionError(format!("Failed to convert final purchase amount: {}", e)))?;

        if calculated_shares == 0 {
             return Err(ServiceError::InvalidPurchaseAmount("Purchase amount too small to buy any shares".to_string()));
        }

        if calculated_shares > invoice_redis.available_shares {
            return Err(ServiceError::InvalidPurchaseShares(calculated_shares, invoice_redis.available_shares));
        }

        // 5. Start MongoDB Transaction (using actual_purchase_decimal128)
        let mut session = self.client.start_session().await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;

        info!("Starting transaction for purchase by user {}: calculated shares: {}, actual amount: {}", 
              user_address, calculated_shares, actual_purchase_decimal128);

        let transaction_result = session.start_transaction()
            .and_run(
                (self, user_address, &purchase_data, &invoice_redis, &actual_purchase_decimal128, calculated_shares),
                |session, (service, user_addr, p_data, i_redis, actual_p_decimal, shares_to_purchase)| {
                    async move {
                        // --- Transaction Operations ---
                        // a. 检查用户并扣除余额 (using actual_p_decimal)
                        let user = service.user_repo.find_by_wallet_address_session(user_addr, session).await?
                             .ok_or_else(|| ServiceError::UserNotFound(user_addr.to_string()))?;

                        // Check balance
                        let balance_f64 = user.balance.to_string().parse::<f64>()
                            .map_err(|_| ServiceError::InternalError(format!("Failed to parse user balance for comparison: {}", user.balance)))?;
                        let purchase_f64 = actual_p_decimal.to_string().parse::<f64>()
                            .map_err(|_| ServiceError::InternalError(format!("Failed to parse purchase amount for comparison: {}", actual_p_decimal)))?;
                        
                        if balance_f64 < purchase_f64 { 
                            error!("Insufficient funds for user {}. Required: {}, Available: {}", user_addr, actual_p_decimal, user.balance);
                            return Err(ServiceError::InsufficientFunds(user_addr.to_string(), actual_p_decimal.to_string(), user.balance.to_string()));
                        }
                        
                        // Deduct balance
                        let negative_purchase_amount = Decimal128::from_str(&format!("-{}", actual_p_decimal.to_string()))
                                                        .map_err(|_| ServiceError::InternalError("Failed to negate purchase amount".to_string()))?;
                        let update_successful = service.user_repo.update_balance_session(user_addr, negative_purchase_amount, session).await?;
                        if !update_successful {
                            error!("Failed to update balance for user {} during purchase, update returned false.", user_addr);
                            return Err(ServiceError::BalanceUpdateFailed(user_addr.to_string()));
                        }
                        info!("Deducted {} from user {} balance", actual_p_decimal, user_addr);

                        // b. 查找数据库中的票据记录
                        let invoice_mongo = service.invoice_repo.find_by_number_session(&i_redis.invoice_number, session).await?
                            .ok_or_else(|| ServiceError::InvoiceNotFound(i_redis.invoice_number.clone()))?;

                        // c. 创建持仓记录 (using actual_p_decimal)
                        let holding = UserInvoiceHolding::new(
                            user_addr.to_string(),
                            invoice_mongo.id.unwrap(), 
                            actual_p_decimal.clone(),
                        );
                        let created_holding = service.holding_repo.create_session(holding, session).await?;
                        info!("Created holding record within transaction for user {}", user_addr);

                        // d. 创建购买交易记录 (using actual_p_decimal)
                        let transaction_record = Transaction::new_purchase(
                            user_addr.to_string(),
                            invoice_mongo.id.unwrap(),
                            created_holding.holding_id.clone(),
                            actual_p_decimal.clone(),
                        );
                        service.transaction_repo.create_session(transaction_record, session).await?;
                        info!("Created transaction record within transaction for user {}", user_addr);

                        // Clone u64 just in case (though it's Copy)
                        Ok((created_holding, shares_to_purchase.clone()))
                    }
                    .map(|res| res.map_err(|service_err: ServiceError| { 
                         MongoError::custom(Box::new(service_err))
                    }))
                   .boxed() 
                },
            )
            .await;

        // Destructure result outside the transaction closure
        let (created_holding, purchased_shares) = match transaction_result {
            Ok(result_tuple) => {
                info!("Transaction committed successfully for user {}", user_address);
                Ok(result_tuple) 
            }
            Err(e) => {
                error!("Transaction failed for user {}: {:?}", user_address, e);
                 // Check the error kind
                 match &*e.kind { // Dereference the Box<ErrorKind>
                     mongodb::error::ErrorKind::Custom(inner_error) => { 
                        // Try downcasting the inner Box<dyn Error + Send + Sync>
                        if let Some(service_error) = inner_error.downcast_ref::<ServiceError>() {
                             Err(service_error.clone())
                        } else {
                            // Custom error wasn't our ServiceError
                            Err(ServiceError::MongoDbTransactionError(format!("Unknown custom transaction error: {}", e)))
                        }
                     },
                     _ if e.contains_label("TransientTransactionError") => { 
                         Err(ServiceError::MongoDbTransactionError(format!("Transient transaction error (retry possible): {}", e)))
                     },
                     _ if e.contains_label("UnknownTransactionCommitResult") => { 
                         Err(ServiceError::MongoDbTransactionError(format!("Unknown commit result (needs check): {}", e)))
                     },
                     _ => { // Fallback for other MongoDB errors
                         Err(ServiceError::MongoDbError(e.to_string()))
                     }
                 }
            }
        }?; 

        // 6. Update Redis (using purchased_shares which is u64)
        match self.redis_service.update_invoice_shares(&purchase_data.invoice_id, purchased_shares) {
            Ok(_) => info!("Successfully updated Redis shares ({}) for invoice {}", purchased_shares, purchase_data.invoice_id),
            Err(e) => {
                error!("Failed to update Redis shares ({}) for invoice {} after successful DB transaction: {}. Data may be inconsistent.", purchased_shares, purchase_data.invoice_id, e);
            }
        }

        info!("Successfully completed invoice purchase process for user {}", user_address);
        Ok(created_holding) 
    }
    
    /// 获取用户的所有票据持仓
    pub async fn get_user_holdings(&self, user_address: &str) -> Result<Vec<UserInvoiceHolding>> {
        info!("Fetching holdings for user: {}", user_address);
        
        let holdings = self.holding_repo.find_by_user_id(user_address).await?;
        
        info!("Found {} holdings for user {}", holdings.len(), user_address);
        Ok(holdings)
    }
    
    /// 获取可购买的票据列表
    pub async fn get_available_invoices(&self) -> Result<Vec<InvoiceRedisDto>> {
        info!("Fetching available invoices for purchase");
        
        let invoices = self.redis_service.get_available_invoices()
            .map_err(|e| anyhow!(e.to_string()))?;
        
        info!("Found {} available invoices", invoices.len());
        Ok(invoices)
    }
}
