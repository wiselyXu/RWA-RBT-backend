use anyhow::Result;
use chrono::Utc;
use log::{debug, error, info};
use mongodb::bson::{DateTime, Decimal128, doc, oid::ObjectId};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;

use crate::error::ServiceError;
use crate::repository::{EnterpriseRepository, InvoiceBatchRepository, InvoiceRepository, TokenRepository, UserRepository};
use common::domain::entity::token::CreateTokenBatchFromInvoiceBatchRequest;
use common::domain::entity::{
    CreateTokenBatchRequest, Enterprise, Invoice, PurchaseTokenRequest, QueryTokenMarketRequest, QueryUserTokenHoldingsRequest, TokenBatch, TokenBatchResponse, TokenBatchStatus,
    TokenHolding, TokenHoldingResponse, TokenHoldingStatus, TokenMarket, TokenMarketResponse, TokenTransaction, TokenTransactionStatus, TokenTransactionType, User,
    invoice_batch::InvoiceBatchStatus,
};
use mongodb::Database;

pub struct TokenService {
    token_repository: Arc<TokenRepository>,
    invoice_repository: Arc<InvoiceRepository>,
    enterprise_repository: Arc<EnterpriseRepository>,
    user_repository: Arc<UserRepository>,
    database: Arc<Database>,
}

impl TokenService {
    pub fn new(
        token_repository: Arc<TokenRepository>,
        invoice_repository: Arc<InvoiceRepository>,
        enterprise_repository: Arc<EnterpriseRepository>,
        user_repository: Arc<UserRepository>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            token_repository,
            invoice_repository,
            enterprise_repository,
            user_repository,
            database,
        }
    }

    // Token Batch Management

    /// Create a new token batch for an invoice
    pub async fn create_token_batch(&self, request: CreateTokenBatchRequest) -> Result<String, ServiceError> {
        // Parse the request
        let invoice_id = ObjectId::parse_str(&request.invoice_id).map_err(|e| ServiceError::InternalError(format!("Invalid invoice ID: {}", e)))?;

        let creditor_id = ObjectId::parse_str(&request.creditor_id).map_err(|e| ServiceError::InternalError(format!("Invalid creditor ID: {}", e)))?;

        let debtor_id = ObjectId::parse_str(&request.debtor_id).map_err(|e| ServiceError::InternalError(format!("Invalid debtor ID: {}", e)))?;

        // Validate the invoice exists
        let invoice = self
            .invoice_repository
            .find_by_id(invoice_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("Invoice not found: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound("Invoice not found".to_string()))?;

        // Validate the creditor exists
        let payee = self
            .enterprise_repository
            .find_by_id(creditor_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("Creditor not found: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound("Creditor not found".to_string()))?;

        // Validate the debtor exists
        let payer = self
            .enterprise_repository
            .find_by_id(debtor_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("Debtor not found: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound("Debtor not found".to_string()))?;

        // Parse token supply and value
        let total_token_supply = Decimal::from_str_exact(&request.total_token_supply).map_err(|e| ServiceError::InternalError(format!("Invalid token supply: {}", e)))?;
        let token_value = Decimal::from_str_exact(&request.token_value).map_err(|e| ServiceError::InternalError(format!("Invalid token value: {}", e)))?;
        let interest_rate_apy = Decimal::from_str_exact(&request.interest_rate_apy).map_err(|e| ServiceError::InternalError(format!("Invalid interest rate: {}", e)))?;

        // Parse maturity date
        let maturity_date = DateTime::parse_rfc3339_str(&request.maturity_date).map_err(|e| ServiceError::InternalError(format!("Invalid maturity date: {}", e)))?;

        // Calculate total value
        let total_value = total_token_supply * token_value;

        // Convert decimal values to Decimal128 safely
        let total_token_supply_d128 =
            Decimal128::from_str(&total_token_supply.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert total token supply: {}", e)))?;
        let token_value_d128 = Decimal128::from_str(&token_value.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert token value: {}", e)))?;
        let total_value_d128 = Decimal128::from_str(&total_value.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert total value: {}", e)))?;
        let zero_d128 = Decimal128::from_str("0").map_err(|e| ServiceError::InternalError(format!("Failed to create zero value: {}", e)))?;
        let interest_rate_d128 =
            Decimal128::from_str(&interest_rate_apy.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert interest rate: {}", e)))?;

        // Create token batch
        let token_batch = TokenBatch {
            id: None,
            batch_reference: request.batch_reference,
            invoice_id,
            payee: payee.wallet_address.clone(),
            payer: payer.wallet_address.clone(),
            stablecoin_symbol: request.stablecoin_symbol,
            total_token_supply: total_token_supply_d128,
            token_value: token_value_d128,
            total_value: total_value_d128,
            contract_address: None,
            sold_token_amount: zero_d128,
            available_token_amount: total_token_supply_d128,
            status: TokenBatchStatus::Available,
            interest_rate_apy: interest_rate_d128,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            maturity_date,
        };

        // Save the token batch
        let batch_id = self
            .token_repository
            .create_token_batch(token_batch.clone())
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token batch: {}", e)))?;

        // Create token market entry
        let token_market = TokenMarket {
            id: None,
            batch_id,
            batch_reference: token_batch.batch_reference.clone(),
            payee: payee.wallet_address.clone(),
            payer: payer.wallet_address.clone(),
            stablecoin_symbol: token_batch.stablecoin_symbol.clone(),
            total_token_amount: token_batch.total_token_supply,
            sold_token_amount: zero_d128,
            available_token_amount: token_batch.available_token_amount,
            purchased_token_amount: zero_d128,
            token_value_per_unit: token_batch.token_value,
            remaining_transaction_amount: token_batch.total_value,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        // Save the token market
        self.token_repository
            .create_token_market(token_market)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token market: {}", e)))?;

        Ok(batch_id.to_hex())
    }

    /// Get token batches with optional filtering
    pub async fn list_token_batches(
        &self,
        status: Option<TokenBatchStatus>,
        creditor_id: Option<String>,
        stablecoin_symbol: Option<String>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<TokenBatchResponse>, ServiceError> {
        let creditor_id = match creditor_id {
            Some(id) => Some(ObjectId::parse_str(&id).map_err(|e| ServiceError::InternalError(format!("Invalid creditor ID: {}", e)))?),
            None => None,
        };

        let token_batches = self
            .token_repository
            .list_token_batches(status, creditor_id, stablecoin_symbol, page, page_size)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to list token batches: {}", e)))?;

        let mut responses = Vec::new();
        for batch in token_batches {
            let batch_id = batch.id.ok_or_else(|| ServiceError::InternalError("Token batch is missing ID".to_string()))?;

            let response = TokenBatchResponse {
                id: batch_id.to_hex(),
                batch_reference: batch.batch_reference,
                payee: batch.payee,
                payer: batch.payer,
                stablecoin_symbol: batch.stablecoin_symbol,
                total_token_supply: batch.total_token_supply.to_string(),
                token_value: batch.token_value.to_string(),
                total_value: batch.total_value.to_string(),
                sold_token_amount: batch.sold_token_amount.to_string(),
                available_token_amount: batch.available_token_amount.to_string(),
                status: format!("{:?}", batch.status),
                interest_rate_apy: batch.interest_rate_apy.to_string(),
                maturity_date: batch.maturity_date.to_string(),
            };

            responses.push(response);
        }

        Ok(responses)
    }

    // Token Market Operations

    /// Get token market listings with optional filtering
    pub async fn list_token_markets(&self, request: QueryTokenMarketRequest) -> Result<Vec<TokenMarketResponse>, ServiceError> {
        let token_markets = self
            .token_repository
            .list_token_markets(request.stablecoin_symbol, request.page, request.page_size)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to list token markets: {}", e)))?;

        let mut responses = Vec::new();
        for market in token_markets {
            let market_id = market.id.ok_or_else(|| ServiceError::InternalError("Token market is missing ID".to_string()))?;

            let response = TokenMarketResponse {
                id: market_id.to_hex(),
                batch_reference: market.batch_reference,
                payee: market.payee,
                payer: market.payer,
                stablecoin_symbol: market.stablecoin_symbol,
                total_token_amount: market.total_token_amount.to_string(),
                sold_token_amount: market.sold_token_amount.to_string(),
                available_token_amount: market.available_token_amount.to_string(),
                token_value_per_unit: market.token_value_per_unit.to_string(),
                remaining_transaction_amount: market.remaining_transaction_amount.to_string(),
            };

            responses.push(response);
        }

        Ok(responses)
    }

    // Token Purchase and Holding Operations

    /// Purchase tokens from a batch
    pub async fn purchase_tokens(&self, user_id: String, request: PurchaseTokenRequest) -> Result<String, ServiceError> {
        // Parse IDs and amounts
        let user_id_obj = ObjectId::parse_str(&user_id).map_err(|e| ServiceError::InternalError(format!("Invalid user ID: {}", e)))?;

        let batch_id = ObjectId::parse_str(&request.batch_id).map_err(|e| ServiceError::InternalError(format!("Invalid batch ID: {}", e)))?;

        let token_amount = Decimal::from_str_exact(&request.token_amount).map_err(|e| ServiceError::InternalError(format!("Invalid token amount: {}", e)))?;

        // Verify user exists
        let user = self
            .user_repository
            .find_by_wallet_address(&user_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("User not found: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        // Get token batch and market
        let token_batch = self
            .token_repository
            .get_token_batch_by_id(batch_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("Token batch not found: {}", e)))?;

        // Validate availability
        let available_amount =
            Decimal::from_str(&token_batch.available_token_amount.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to parse available amount: {}", e)))?;

        if token_amount > available_amount {
            return Err(ServiceError::InternalError(format!(
                "Requested amount {} exceeds available amount {}",
                token_amount, available_amount
            )));
        }

        // Calculate purchase value
        let token_value = Decimal::from_str(&token_batch.token_value.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to parse token value: {}", e)))?;
        let purchase_value = token_amount * token_value;

        // Convert to Decimal128 safely
        let token_amount_d128 = Decimal128::from_str(&token_amount.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert token amount: {}", e)))?;
        let purchase_value_d128 = Decimal128::from_str(&purchase_value.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert purchase value: {}", e)))?;

        // Create token holding
        let token_holding = TokenHolding {
            id: None,
            user_id: user_id_obj,
            batch_id,
            batch_reference: token_batch.batch_reference.clone(),
            token_amount: token_amount_d128,
            purchase_value: purchase_value_d128,
            current_value: purchase_value_d128, // Initially same as purchase value
            purchase_date: DateTime::now(),
            status: TokenHoldingStatus::Active,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        let holding_id = self
            .token_repository
            .create_token_holding(token_holding)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token holding: {}", e)))?;

        // Create transaction record
        let token_transaction = TokenTransaction {
            id: None,
            batch_id,
            batch_reference: token_batch.batch_reference.clone(),
            user_id: user_id_obj,
            transaction_type: TokenTransactionType::Purchase,
            token_amount: token_amount_d128,
            transaction_value: purchase_value_d128,
            stablecoin_symbol: token_batch.stablecoin_symbol.clone(),
            transaction_hash: None, // Would be set after blockchain confirmation
            status: TokenTransactionStatus::Completed,
            transaction_date: DateTime::now(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        };

        self.token_repository
            .create_token_transaction(token_transaction)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token transaction: {}", e)))?;

        // Update token batch available amount
        let mut updated_batch = token_batch.clone();
        let new_available = available_amount - token_amount;
        let current_sold = Decimal::from_str(&token_batch.sold_token_amount.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to parse sold amount: {}", e)))?;
        let new_sold = current_sold + token_amount;

        // Convert to Decimal128 safely
        let new_available_d128 =
            Decimal128::from_str(&new_available.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert new available amount: {}", e)))?;
        let new_sold_d128 = Decimal128::from_str(&new_sold.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert new sold amount: {}", e)))?;

        updated_batch.available_token_amount = new_available_d128;
        updated_batch.sold_token_amount = new_sold_d128;

        self.token_repository
            .update_token_batch(batch_id, updated_batch)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to update token batch: {}", e)))?;

        // Update token market - get the market by batch_id first
        let token_market_result = self
            .token_repository
            .get_token_market_by_batch_id(batch_id)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to get token market: {}", e)))?;

        if let Some(mut token_market) = token_market_result {
            let market_id = token_market.id.ok_or_else(|| ServiceError::InternalError("Token market is missing ID".to_string()))?;

            token_market.available_token_amount = new_available_d128;
            token_market.sold_token_amount = new_sold_d128;

            // Calculate remaining transaction amount
            let current_remaining = Decimal::from_str(&token_market.remaining_transaction_amount.to_string())
                .map_err(|e| ServiceError::InternalError(format!("Failed to parse remaining amount: {}", e)))?;
            let new_remaining = current_remaining - purchase_value;
            let new_remaining_d128 =
                Decimal128::from_str(&new_remaining.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert new remaining amount: {}", e)))?;

            token_market.remaining_transaction_amount = new_remaining_d128;
            token_market.updated_at = DateTime::now();

            self.token_repository
                .update_token_market(market_id, token_market)
                .await
                .map_err(|e| ServiceError::MongoDbError(format!("Failed to update token market: {}", e)))?;
        } else {
            error!("Token market for batch {} not found", batch_id);
            // We continue despite market not being found, as it's not critical to the purchase
        }

        Ok(holding_id.to_hex())
    }

    /// Get token holdings for a user
    pub async fn get_user_token_holdings(&self, request: QueryUserTokenHoldingsRequest) -> Result<Vec<TokenHoldingResponse>, ServiceError> {
        let user_id = ObjectId::parse_str(&request.user_id).map_err(|e| ServiceError::InternalError(format!("Invalid user ID: {}", e)))?;

        let holdings = self
            .token_repository
            .get_token_holdings_by_user_id(user_id)
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to get token holdings: {}", e)))?;

        let mut responses = Vec::new();
        for holding in holdings {
            let holding_id = holding.id.ok_or_else(|| ServiceError::InternalError("Token holding is missing ID".to_string()))?;

            let response = TokenHoldingResponse {
                id: holding_id.to_hex(),
                batch_reference: holding.batch_reference,
                token_amount: holding.token_amount.to_string(),
                purchase_value: holding.purchase_value.to_string(),
                current_value: holding.current_value.to_string(),
                purchase_date: holding.purchase_date.to_string(),
                status: format!("{:?}", holding.status),
            };

            responses.push(response);
        }

        Ok(responses)
    }

    /// 从发票批次创建token批次
    pub async fn create_token_batch_from_invoice_batch(
        &self,
        invoice_batch_id: &str,
        token_batch_request: CreateTokenBatchFromInvoiceBatchRequest,
    ) -> Result<String, ServiceError> {
        // 解析发票批次ID
        let batch_id = ObjectId::parse_str(invoice_batch_id).map_err(|e| ServiceError::InternalError(format!("Invalid invoice batch ID: {}", e)))?;

        // 使用已注入的数据库实例创建仓库
        let invoice_batch_repo = InvoiceBatchRepository::new(&self.database);

        let invoice_batch = invoice_batch_repo
            .find_by_id(batch_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("Invoice batch not found: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound("Invoice batch not found".to_string()))?;

        // 确保批次状态为Issued
        if invoice_batch.status != InvoiceBatchStatus::Issued {
            return Err(ServiceError::InternalError(format!(
                "Invoice batch is not in Issued status, current status: {:?}",
                invoice_batch.status
            )));
        }

        // 获取批次中的所有发票
        let invoices = self
            .invoice_repository
            .find_by_batch_id(batch_id)
            .await
            .map_err(|e| ServiceError::NotFound(format!("Failed to fetch invoices: {}", e)))?;

        if invoices.is_empty() {
            return Err(ServiceError::NotFound("No invoices found in this batch".to_string()));
        }

        // 计算批次总金额
        let mut total_amount: u64 = 0;
        let mut earliest_due_date = i64::MAX;

        for invoice in &invoices {
            total_amount += invoice.amount;
            if invoice.due_date < earliest_due_date {
                earliest_due_date = invoice.due_date;
            }
        }

        // 将总金额转换为Decimal128用于Token批次创建
        let total_amount_d128 = Decimal128::from_str(&total_amount.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert amount: {}", e)))?;

        // 设置零值用于初始化
        let zero_d128 = Decimal128::from_str("0").map_err(|e| ServiceError::InternalError(format!("Failed to create zero value: {}", e)))?;

        // 处理令牌值和总供应量
        let token_value_d128 = Decimal128::from_str(&token_batch_request.token_value).map_err(|e| ServiceError::InternalError(format!("Failed to convert token value: {}", e)))?;

        // 计算令牌总供应量
        let total_value = Decimal::from_str(&total_amount.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert total amount: {}", e)))?;
        let token_value = Decimal::from_str(&token_batch_request.token_value).map_err(|e| ServiceError::InternalError(format!("Failed to convert token value: {}", e)))?;
        let total_supply = (total_value / token_value).round();

        let total_token_supply_d128 = Decimal128::from_str(&total_supply.to_string()).map_err(|e| ServiceError::InternalError(format!("Failed to convert token supply: {}", e)))?;

        // 处理利率
        let interest_rate_d128 =
            Decimal128::from_str(&token_batch_request.interest_rate_apy).map_err(|e| ServiceError::InternalError(format!("Failed to convert interest rate: {}", e)))?;

        // 处理到期日
        let maturity_date = if let Some(custom_maturity_date) = token_batch_request.maturity_date {
            // 如果提供了自定义到期日，则使用它
            match DateTime::parse_rfc3339_str(&custom_maturity_date) {
                Ok(date) => date,
                Err(_) => {
                    // 如果解析失败，则使用发票中的最早到期日
                    DateTime::from_millis(earliest_due_date)
                }
            }
        } else {
            // 使用发票中的最早到期日期
            DateTime::from_millis(earliest_due_date)
        };

        // 创建token批次
        let token_batch = TokenBatch {
            id: None,
            batch_reference: token_batch_request.batch_reference,
            invoice_id: batch_id, // 使用invoice_batch的ID，而不是单个发票
            payee: invoice_batch.payee,
            payer: invoice_batch.payer,
            stablecoin_symbol: token_batch_request.stablecoin_symbol,
            total_token_supply: total_token_supply_d128,
            token_value: token_value_d128,
            total_value: total_amount_d128,
            contract_address: None,
            sold_token_amount: zero_d128,
            available_token_amount: total_token_supply_d128,
            status: TokenBatchStatus::Available,
            interest_rate_apy: interest_rate_d128,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            maturity_date,
        };

        // 启动MongoDB事务
        let mut session = self
            .database
            .client()
            .start_session()
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to start MongoDB session: {}", e)))?;

        session
            .start_transaction()
            .await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to start transaction: {}", e)))?;

        match async {
            // 保存token批次
            let token_batch_id = self
                .token_repository
                .create_token_batch_with_session(token_batch.clone(), &mut session)
                .await
                .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token batch: {}", e)))?;

            // 创建token市场条目
            let token_market = TokenMarket {
                id: None,
                batch_id: token_batch_id,
                batch_reference: token_batch.batch_reference.clone(),
                payee: token_batch.payee.clone(),
                payer: token_batch.payer.clone(),
                stablecoin_symbol: token_batch.stablecoin_symbol.clone(),
                total_token_amount: token_batch.total_token_supply,
                sold_token_amount: zero_d128,
                available_token_amount: token_batch.available_token_amount,
                purchased_token_amount: zero_d128,
                token_value_per_unit: token_batch.token_value,
                remaining_transaction_amount: token_batch.total_value,
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };

            // 保存token市场
            self.token_repository
                .create_token_market_with_session(token_market, &mut session)
                .await
                .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token market: {}", e)))?;

            // 更新发票批次的token_batch_id和状态
            invoice_batch_repo
                .update_token_batch_id_with_session(batch_id, token_batch_id, &mut session)
                .await
                .map_err(|e| ServiceError::MongoDbError(format!("Failed to update invoice batch: {}", e)))?;

            // 更新发票批次状态为Trading
            invoice_batch_repo
                .update_status_with_session(batch_id, InvoiceBatchStatus::Trading, &mut session)
                .await
                .map_err(|e| ServiceError::MongoDbError(format!("Failed to update invoice batch status: {}", e)))?;

            Result::<_, ServiceError>::Ok(token_batch_id)
        }
        .await
        {
            Ok(id) => {
                // 提交事务
                session.commit_transaction().await.map_err(|e| {
                    error!("Failed to commit transaction: {}", e);
                    ServiceError::MongoDbError(format!("Failed to commit transaction: {}", e))
                })?;

                Ok(id.to_hex())
            }
            Err(e) => {
                // 出错时回滚事务
                if let Err(abort_err) = session.abort_transaction().await {
                    error!("Failed to abort transaction: {}", abort_err);
                }
                Err(e)
            }
        }
    }
}
