use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;
use log::{debug, error, info};
use mongodb::bson::{oid::ObjectId, DateTime, Decimal128, doc};
use rust_decimal::Decimal;
use std::str::FromStr;

use common::domain::entity::{
    TokenBatch, TokenBatchStatus, TokenMarket, TokenHolding, TokenHoldingStatus,
    TokenTransaction, TokenTransactionType, TokenTransactionStatus,
    Enterprise, Invoice, User,
    CreateTokenBatchRequest, PurchaseTokenRequest, QueryTokenMarketRequest, QueryUserTokenHoldingsRequest,
    TokenBatchResponse, TokenMarketResponse, TokenHoldingResponse,
};
use crate::error::ServiceError;
use crate::repository::{
    TokenRepository, InvoiceRepository, EnterpriseRepository, UserRepository
};

pub struct TokenService {
    token_repository: Arc<TokenRepository>,
    invoice_repository: Arc<InvoiceRepository>,
    enterprise_repository: Arc<EnterpriseRepository>,
    user_repository: Arc<UserRepository>,
}

impl TokenService {
    pub fn new(
        token_repository: Arc<TokenRepository>,
        invoice_repository: Arc<InvoiceRepository>,
        enterprise_repository: Arc<EnterpriseRepository>,
        user_repository: Arc<UserRepository>,
    ) -> Self {
        Self {
            token_repository,
            invoice_repository,
            enterprise_repository,
            user_repository,
        }
    }

    // Token Batch Management

    /// Create a new token batch for an invoice
    pub async fn create_token_batch(&self, request: CreateTokenBatchRequest) -> Result<String, ServiceError> {
        // Parse the request
        let invoice_id = ObjectId::parse_str(&request.invoice_id).map_err(|e| {
            ServiceError::InternalError(format!("Invalid invoice ID: {}", e))
        })?;

        let creditor_id = ObjectId::parse_str(&request.creditor_id).map_err(|e| {
            ServiceError::InternalError(format!("Invalid creditor ID: {}", e))
        })?;

        let debtor_id = ObjectId::parse_str(&request.debtor_id).map_err(|e| {
            ServiceError::InternalError(format!("Invalid debtor ID: {}", e))
        })?;

        // Validate the invoice exists
        let invoice = self.invoice_repository.find_by_id(invoice_id).await.map_err(|e| {
            ServiceError::NotFound(format!("Invoice not found: {}", e))
        })?.ok_or_else(|| ServiceError::NotFound("Invoice not found".to_string()))?;

        // Validate the creditor exists
        let creditor = self.enterprise_repository.find_by_id(creditor_id).await.map_err(|e| {
            ServiceError::NotFound(format!("Creditor not found: {}", e))
        })?.ok_or_else(|| ServiceError::NotFound("Creditor not found".to_string()))?;

        // Validate the debtor exists
        let debtor = self.enterprise_repository.find_by_id(debtor_id).await.map_err(|e| {
            ServiceError::NotFound(format!("Debtor not found: {}", e))
        })?.ok_or_else(|| ServiceError::NotFound("Debtor not found".to_string()))?;

        // Parse token supply and value
        let total_token_supply = Decimal::from_str_exact(&request.total_token_supply)
            .map_err(|e| ServiceError::InternalError(format!("Invalid token supply: {}", e)))?;
        let token_value = Decimal::from_str_exact(&request.token_value)
            .map_err(|e| ServiceError::InternalError(format!("Invalid token value: {}", e)))?;
        let interest_rate_apy = Decimal::from_str_exact(&request.interest_rate_apy)
            .map_err(|e| ServiceError::InternalError(format!("Invalid interest rate: {}", e)))?;

        // Parse maturity date
        let maturity_date = DateTime::parse_rfc3339_str(&request.maturity_date)
            .map_err(|e| ServiceError::InternalError(format!("Invalid maturity date: {}", e)))?;

        // Calculate total value
        let total_value = total_token_supply * token_value;

        // Convert decimal values to Decimal128 safely
        let total_token_supply_d128 = Decimal128::from_str(&total_token_supply.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert total token supply: {}", e)))?;
        let token_value_d128 = Decimal128::from_str(&token_value.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert token value: {}", e)))?;
        let total_value_d128 = Decimal128::from_str(&total_value.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert total value: {}", e)))?;
        let zero_d128 = Decimal128::from_str("0")
            .map_err(|e| ServiceError::InternalError(format!("Failed to create zero value: {}", e)))?;
        let interest_rate_d128 = Decimal128::from_str(&interest_rate_apy.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert interest rate: {}", e)))?;

        // Create token batch
        let token_batch = TokenBatch {
            id: None,
            batch_reference: request.batch_reference,
            invoice_id,
            creditor_id,
            debtor_id,
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
        let batch_id = self.token_repository.create_token_batch(token_batch.clone()).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token batch: {}", e)))?;

        // Create token market entry
        let token_market = TokenMarket {
            id: None,
            batch_id,
            batch_reference: token_batch.batch_reference.clone(),
            creditor_address: creditor.wallet_address.clone(),
            debtor_address: debtor.wallet_address.clone(),
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
        self.token_repository.create_token_market(token_market).await
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
            Some(id) => Some(ObjectId::parse_str(&id).map_err(|e| {
                ServiceError::InternalError(format!("Invalid creditor ID: {}", e))
            })?),
            None => None,
        };

        let token_batches = self.token_repository.list_token_batches(
            status,
            creditor_id,
            stablecoin_symbol,
            page,
            page_size,
        ).await.map_err(|e| ServiceError::MongoDbError(format!("Failed to list token batches: {}", e)))?;

        let mut responses = Vec::new();
        for batch in token_batches {
            let batch_id = batch.id.ok_or_else(|| 
                ServiceError::InternalError("Token batch is missing ID".to_string())
            )?;
            
            // Get creditor and debtor names
            let creditor = self.enterprise_repository.find_by_id(batch.creditor_id).await
                .map_err(|e| ServiceError::InternalError(format!("Failed to get creditor details: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound("Creditor not found".to_string()))?;
                
            let debtor = self.enterprise_repository.find_by_id(batch.debtor_id).await
                .map_err(|e| ServiceError::InternalError(format!("Failed to get debtor details: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound("Debtor not found".to_string()))?;

            let response = TokenBatchResponse {
                id: batch_id.to_hex(),
                batch_reference: batch.batch_reference,
                creditor_name: creditor.name,
                debtor_name: debtor.name,
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
    pub async fn list_token_markets(
        &self,
        request: QueryTokenMarketRequest,
    ) -> Result<Vec<TokenMarketResponse>, ServiceError> {
        let token_markets = self.token_repository.list_token_markets(
            request.stablecoin_symbol,
            request.page,
            request.page_size,
        ).await.map_err(|e| ServiceError::MongoDbError(format!("Failed to list token markets: {}", e)))?;

        let mut responses = Vec::new();
        for market in token_markets {
            let market_id = market.id.ok_or_else(|| 
                ServiceError::InternalError("Token market is missing ID".to_string())
            )?;
            
            let response = TokenMarketResponse {
                id: market_id.to_hex(),
                batch_reference: market.batch_reference,
                creditor_address: market.creditor_address,
                debtor_address: market.debtor_address,
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
    pub async fn purchase_tokens(
        &self,
        user_id: String,
        request: PurchaseTokenRequest,
    ) -> Result<String, ServiceError> {
        // Parse IDs and amounts
        let user_id_obj = ObjectId::parse_str(&user_id).map_err(|e| {
            ServiceError::InternalError(format!("Invalid user ID: {}", e))
        })?;

        let batch_id = ObjectId::parse_str(&request.batch_id).map_err(|e| {
            ServiceError::InternalError(format!("Invalid batch ID: {}", e))
        })?;

        let token_amount = Decimal::from_str_exact(&request.token_amount)
            .map_err(|e| ServiceError::InternalError(format!("Invalid token amount: {}", e)))?;

        // Verify user exists
        let user = self.user_repository.find_by_wallet_address(&user_id).await.map_err(|e| {
            ServiceError::NotFound(format!("User not found: {}", e))
        })?.ok_or_else(|| ServiceError::NotFound("User not found".to_string()))?;

        // Get token batch and market
        let token_batch = self.token_repository.get_token_batch_by_id(batch_id).await
            .map_err(|e| ServiceError::NotFound(format!("Token batch not found: {}", e)))?;
        
        // Validate availability
        let available_amount = Decimal::from_str(&token_batch.available_token_amount.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to parse available amount: {}", e)))?;
        
        if token_amount > available_amount {
            return Err(ServiceError::InternalError(format!(
                "Requested amount {} exceeds available amount {}", 
                token_amount, available_amount
            )));
        }

        // Calculate purchase value
        let token_value = Decimal::from_str(&token_batch.token_value.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to parse token value: {}", e)))?;
        let purchase_value = token_amount * token_value;

        // Convert to Decimal128 safely
        let token_amount_d128 = Decimal128::from_str(&token_amount.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert token amount: {}", e)))?;
        let purchase_value_d128 = Decimal128::from_str(&purchase_value.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert purchase value: {}", e)))?;
        
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

        let holding_id = self.token_repository.create_token_holding(token_holding).await
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

        self.token_repository.create_token_transaction(token_transaction).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to create token transaction: {}", e)))?;

        // Update token batch available amount
        let mut updated_batch = token_batch.clone();
        let new_available = available_amount - token_amount;
        let current_sold = Decimal::from_str(&token_batch.sold_token_amount.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to parse sold amount: {}", e)))?;
        let new_sold = current_sold + token_amount;
        
        // Convert to Decimal128 safely
        let new_available_d128 = Decimal128::from_str(&new_available.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert new available amount: {}", e)))?;
        let new_sold_d128 = Decimal128::from_str(&new_sold.to_string())
            .map_err(|e| ServiceError::InternalError(format!("Failed to convert new sold amount: {}", e)))?;
        
        updated_batch.available_token_amount = new_available_d128;
        updated_batch.sold_token_amount = new_sold_d128;
        
        self.token_repository.update_token_batch(batch_id, updated_batch).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to update token batch: {}", e)))?;

        // Update token market - get the market by batch_id first
        let token_market_result = self.token_repository.get_token_market_by_batch_id(batch_id).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to get token market: {}", e)))?;
        
        if let Some(mut token_market) = token_market_result {
            let market_id = token_market.id.ok_or_else(|| 
                ServiceError::InternalError("Token market is missing ID".to_string())
            )?;
            
            token_market.available_token_amount = new_available_d128;
            token_market.sold_token_amount = new_sold_d128;
            
            // Calculate remaining transaction amount
            let current_remaining = Decimal::from_str(&token_market.remaining_transaction_amount.to_string())
                .map_err(|e| ServiceError::InternalError(format!("Failed to parse remaining amount: {}", e)))?;
            let new_remaining = current_remaining - purchase_value;
            let new_remaining_d128 = Decimal128::from_str(&new_remaining.to_string())
                .map_err(|e| ServiceError::InternalError(format!("Failed to convert new remaining amount: {}", e)))?;
                
            token_market.remaining_transaction_amount = new_remaining_d128;
            token_market.updated_at = DateTime::now();
            
            self.token_repository.update_token_market(market_id, token_market).await
                .map_err(|e| ServiceError::MongoDbError(format!("Failed to update token market: {}", e)))?;
        } else {
            error!("Token market for batch {} not found", batch_id);
            // We continue despite market not being found, as it's not critical to the purchase
        }

        Ok(holding_id.to_hex())
    }

    /// Get token holdings for a user
    pub async fn get_user_token_holdings(
        &self,
        request: QueryUserTokenHoldingsRequest,
    ) -> Result<Vec<TokenHoldingResponse>, ServiceError> {
        let user_id = ObjectId::parse_str(&request.user_id).map_err(|e| {
            ServiceError::InternalError(format!("Invalid user ID: {}", e))
        })?;

        let holdings = self.token_repository.get_token_holdings_by_user_id(user_id).await
            .map_err(|e| ServiceError::MongoDbError(format!("Failed to get token holdings: {}", e)))?;

        let mut responses = Vec::new();
        for holding in holdings {
            let holding_id = holding.id.ok_or_else(|| 
                ServiceError::InternalError("Token holding is missing ID".to_string())
            )?;
            
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
} 