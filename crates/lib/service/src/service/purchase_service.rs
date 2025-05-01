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
    pub async fn purchase_invoice(&self, user_address: &str, purchase_data: PurchaseInvoiceDto) -> Result<UserInvoiceHolding, ServiceError> {
        info!("Processing invoice purchase for user: {}, invoice: {}", user_address, purchase_data.invoice_id);
        
        // 1. 从Redis获取票据信息 (Do this outside transaction as it doesn't involve MongoDB writes)
        let invoice_redis = self.redis_service.get_invoice(&purchase_data.invoice_id)?
            .ok_or(ServiceError::InvoiceNotFound(purchase_data.invoice_id.clone()))?;
            
        // 2. 验证票据是否可购买
        if !invoice_redis.is_available_for_purchase() {
            return Err(ServiceError::InvoiceNotAvailable(purchase_data.invoice_id.clone()));
        }
        
        // 3. 验证购买份数
        if purchase_data.shares == 0 || purchase_data.shares > invoice_redis.available_shares {
            return Err(ServiceError::InvalidPurchaseShares(purchase_data.shares, invoice_redis.available_shares));
        }
        
        // 4. 计算购买金额 (Use Decimal128 directly if possible or ensure precision)
        // TODO: Consider using rust_decimal for intermediate calculation if precision is critical
        let purchase_amount_f64 = purchase_data.shares as f64 * invoice_redis.share_price;
        let purchase_decimal = Decimal128::from_str_exact(&purchase_amount_f64.to_string())
            .map_err(|e| ServiceError::DecimalConversionError(e.to_string()))?;
        
        // 5. Start MongoDB Transaction
        let mut session = self.client.start_session(None).await
            .map_err(|e| ServiceError::MongoDbError(e.into()))?;

        info!("Starting transaction for purchase by user {}", user_address);

        let transaction_result = session.start_transaction(None)
            .and_run(
                // Pass necessary data into the closure
                (self, user_address, &purchase_data, &invoice_redis, &purchase_decimal),
                |session, (service, user_addr, p_data, i_redis, p_decimal)| {
                    async move {
                        // --- Transaction Operations ---

                        // a. 检查用户并扣除余额
                        let user = service.user_repo.find_by_wallet_address_session(user_addr, session).await?
                             .ok_or_else(|| ServiceError::UserNotFound(user_addr.to_string()))?;

                        // Check balance (Ensure sufficient funds)
                        // TODO: Handle potential precision issues if using direct comparison
                        if user.balance < *p_decimal {
                            error!("Insufficient funds for user {}. Required: {}, Available: {}", user_addr, p_decimal, user.balance);
                            // Use specific error for insufficient funds
                            return Err(ServiceError::InsufficientFunds(user_addr.to_string(), p_decimal.to_string(), user.balance.to_string()));
                        }

                        // Deduct balance (using negative amount)
                        // Need to create negative Decimal128. For now, assuming direct negation works or use appropriate method.
                        // Let's assume Decimal128 supports negation for simplicity. If not, construct manually.
                        let amount_to_deduct = p_decimal.clone(); // Need a way to negate Decimal128
                        // A simple way might be multiplying by -1 represented as Decimal128, but need to confirm API
                        // For now, let's structure as if negation exists ` -amount_to_deduct ` 
                        // Actual implementation might need `Decimal128::from_str_exact(&format!("-{}", amount_to_deduct.to_string()))?`
                        
                        // Placeholder for actual negation:
                        let negative_purchase_amount = Decimal128::from_str_exact(&format!("-{}", amount_to_deduct.to_string()))
                                                        .map_err(|_| ServiceError::InternalError("Failed to negate purchase amount".to_string()))?;

                        let update_successful = service.user_repo.update_balance_session(user_addr, negative_purchase_amount, session).await?;

                        if !update_successful {
                            error!("Failed to update balance for user {} during purchase, update returned false.", user_addr);
                            // This might happen if the user document was somehow modified between the find and update, or the filter condition failed
                            return Err(ServiceError::BalanceUpdateFailed(user_addr.to_string()));
                        }
                        info!("Deducted {} from user {} balance", p_decimal, user_addr);

                        // b. 查找数据库中的票据记录
                        let invoice_mongo = service.invoice_repo.find_by_number_session(&i_redis.invoice_number, session).await?
                            .ok_or_else(|| ServiceError::InvoiceNotFound(i_redis.invoice_number.clone()))?; // Should exist if Redis had it, but good practice to check

                        // c. 创建持仓记录
                        let holding = UserInvoiceHolding::new(
                            user_addr.to_string(),
                            invoice_mongo.id.unwrap(), // Assuming ID is always present after find
                            p_decimal.clone(),
                        );
                        let created_holding = service.holding_repo.create_session(holding, session).await?;
                        info!("Created holding record within transaction for user {}", user_addr);

                        // d. 创建购买交易记录
                        let transaction_record = Transaction::new_purchase(
                            user_addr.to_string(),
                            invoice_mongo.id.unwrap(),
                            created_holding.holding_id.clone(),
                            p_decimal.clone(),
                        );
                        service.transaction_repo.create_session(transaction_record, session).await?;
                        info!("Created transaction record within transaction for user {}", user_addr);

                        // --- End Transaction Operations ---

                        Ok(created_holding) // Return the created holding if successful
                    }.boxed() // Box the future
                },
            )
            .await; // Execute the transaction

        // Handle Transaction Outcome
        let created_holding = match transaction_result {
            Ok(holding) => {
                info!("Transaction committed successfully for user {}", user_address);
                holding
            }
            Err(e) => {
                error!("Transaction failed for user {}: {:?}", user_address, e);
                // Attempt to map MongoDB transaction errors to ServiceError if needed
                 match e {
                     mongodb::error::Error::Transaction { error } => {
                         // Check if it's a custom error returned from the closure
                         if let Some(service_error) = error.downcast_ref::<ServiceError>() {
                             return Err(service_error.clone()); // Propagate the specific ServiceError
                         } else {
                             // Otherwise, return a generic transaction error, including the inner error message
                            return Err(ServiceError::MongoDbTransactionError(error.to_string()));
                         }
                     }
                     _ => return Err(ServiceError::MongoDbError(e.into())), // General MongoDB error
                 }
            }
        };

        // 6. 更新Redis中的票据可用份数 (Do this *after* successful commit)
        // If this fails, the DB state is consistent, but Redis is stale.
        // Needs a reconciliation strategy or retry mechanism for robustness.
        match self.redis_service.update_invoice_shares(&purchase_data.invoice_id, purchase_data.shares).await {
            Ok(_) => info!("Successfully updated Redis shares for invoice {}", purchase_data.invoice_id),
            Err(e) => {
                error!("Failed to update Redis shares for invoice {} after successful DB transaction: {}. Data may be inconsistent.", purchase_data.invoice_id, e);
                // Potentially schedule a background job to retry Redis update
                // For now, just log the error. The core purchase is recorded.
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
        
        let invoices = self.redis_service.get_available_invoices()?;
        
        info!("Found {} available invoices", invoices.len());
        Ok(invoices)
    }
}
