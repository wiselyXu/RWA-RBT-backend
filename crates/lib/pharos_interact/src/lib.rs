#![allow(warnings)]
use anyhow::{anyhow, Context, Result};
use dotenv::dotenv;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, U256};
use std::convert::TryFrom;
use std::env;
use std::sync::Arc; // Import anyhow Result, Context, and anyhow!

use log::error;
use salvo_oapi::ToSchema;
use common::domain::dto::invoice_dto::InvoiceDataDto;
use common::domain::dto::query_invoice_dto::QueryParamsDto;

// Regenerate bindings using the updated ABI
abigen!(
    InvoiceContractABI,   // Name of the generated module
    "./invoice_abi.json", // Path to the ABI file
    event_derives(serde::Deserialize, serde::Serialize)
);

impl TryFrom<InvoiceDataDto> for InvoiceData {
    type Error = anyhow::Error; // Change associated error type to anyhow::Error

    fn try_from(val: InvoiceDataDto) -> Result<Self, Self::Error> {
        Ok(InvoiceData {
            invoice_number: val.invoice_number,
            payee: val.payee.parse::<Address>().context("Invalid payee address format")?,
            payer: val.payer.parse::<Address>().context("Invalid payer address format")?,
            amount: U256::from_dec_str(&val.amount).context("Invalid amount format")?,
            ipfs_hash: val.ipfs_hash,
            contract_hash: val.contract_hash,
            timestamp: U256::from_dec_str(&val.timestamp).context("Invalid timestamp format")?,
            due_date: U256::from_dec_str(&val.due_date).context("Invalid due date format")?,
            token_batch: val.token_batch,
            is_cleared: val.is_cleared,
            is_valid: val.is_valid,
        })
    }
}

impl From<InvoiceData> for InvoiceDataDto {
    fn from(val: InvoiceData) -> Self {
        InvoiceDataDto {
            invoice_number: val.invoice_number,
            payee: format!("{:?}", val.payee), // Convert Address to hex string
            payer: format!("{:?}", val.payer), // Convert Address to hex string
            amount: val.amount.to_string(),    // Convert U256 to string
            ipfs_hash: val.ipfs_hash,
            contract_hash: val.contract_hash,
            timestamp: val.timestamp.to_string(), // Convert U256 to string
            due_date: val.due_date.to_string(),   // Convert U256 to string
            token_batch: val.token_batch,
            is_cleared: val.is_cleared,
            is_valid: val.is_valid,
        }
    }
}

// --- Contract Interaction Traits ---
// These traits define the capabilities of contract interaction

/// Trait for contract query operations (read-only)
#[async_trait::async_trait]
pub trait ContractQuerier {
    /// Query invoices based on filter parameters
    async fn query_invoices(&self, params: QueryParamsDto) -> Result<Vec<InvoiceDataDto>>;
}

/// Trait for contract write operations that modify blockchain state
#[async_trait::async_trait]
pub trait ContractWriter: ContractQuerier {
    /// Create multiple invoices in a batch
    async fn batch_create_invoices(&self, invoices: Vec<InvoiceDataDto>) -> Result<Option<TransactionReceipt>>;

    /// Create a token batch from invoices
    async fn create_token_batch(
        &self,
        batch_id: String,
        invoice_numbers: Vec<String>,
        stable_token_address: String,
        min_term_str: String,
        max_term_str: String,
        interest_rate_str: String,
    ) -> Result<Option<TransactionReceipt>>;

    /// Confirm a token batch issue
    async fn confirm_token_batch_issue(&self, batch_id: String) -> Result<Option<TransactionReceipt>>;

    /// Purchase shares from a token batch
    async fn purchase_shares(&self, batch_id: String, amount_str: String) -> Result<Option<TransactionReceipt>>;
}

// --- Contract Interaction Logic ---

pub struct InvoiceContract<M: Middleware> {
    contract: InvoiceContractABI<M>,
    client: Arc<M>, // Keep client if needed for direct calls, otherwise remove
}

// Implement ContractQuerier for InvoiceContract
#[async_trait::async_trait]
impl<M: Middleware + Send + Sync + 'static> ContractQuerier for InvoiceContract<M> {
    async fn query_invoices(&self, params_dto: QueryParamsDto) -> Result<Vec<InvoiceDataDto>> {
        // Convert QueryParamsDto to internal QueryParams
        let params = QueryParams {
            batch_id: "".to_string(), // Consider adding batch_id to QueryParamsDto if needed for filtering
            payee: match params_dto.payee {
                Some(addr_str) => addr_str.parse::<Address>().context("Invalid payee address in query")?,
                None => Address::zero(),
            },
            invoice_number: params_dto.invoice_number.unwrap_or_else(|| "".to_string()),
            payer: match params_dto.payer {
                Some(addr_str) => addr_str.parse::<Address>().context("Invalid payer address in query")?,
                None => Address::zero(),
            },
            // Assuming ABI might have changed or the booleans are handled differently.
            // Adapt based on the actual `QueryParams` struct definition from `abigen!`
            // is_cleared: params_dto.is_cleared.unwrap_or(false),
            // is_valid: params_dto.is_valid.unwrap_or(true),
            check_valid: params_dto.is_valid.unwrap_or(false), // Assuming check_valid corresponds to is_valid query?
        };

        log::info!("Query_Invoices parameters: {:?}", params.clone());
        // Call the contract
        let result: QueryResult = self.contract.query_invoices(params).call().await.map_err(|e| {
            error!("Error calling queryInvoices: {}", e);
            anyhow!("Contract query failed: {}", e)
        })?;

        // Convert internal Vec<InvoiceData> to Vec<InvoiceDataDto>
        let result_dto: Vec<InvoiceDataDto> = result.invoices.into_iter().map(InvoiceDataDto::from).collect();

        Ok(result_dto)
    }
}

// Implement ContractWriter for InvoiceContract
#[async_trait::async_trait]
impl<M: Middleware + Send + Sync + 'static> ContractWriter for InvoiceContract<M> {
    async fn batch_create_invoices(&self, invoices: Vec<InvoiceDataDto>) -> Result<Option<TransactionReceipt>> {
        let invoice_data_vec: Result<Vec<InvoiceData>, _> = invoices.into_iter().map(InvoiceData::try_from).collect();

        let invoice_data_vec = invoice_data_vec.context("Failed to parse one or more invoice data DTOs")?;

        // Clone data needed for estimation *before* moving it to the final send call
        let data_for_estimation = invoice_data_vec.clone();
        let data_for_send = invoice_data_vec; // Original vec will be moved here

        // 1. Estimate required gas
        let gas_estimate = self.contract.batch_create_invoices(data_for_estimation)
            .estimate_gas()
            .await
            .map_err(|e| {
                error!("Error estimating gas for batchCreateInvoices: {}", e);
                anyhow!("Failed to estimate gas (potential revert): {}", e)
            })?;

        // 2. Log estimate for debugging
        log::warn!("Estimated gas for batchCreateInvoices: {}", gas_estimate);

        // 3. Set gas limit with a buffer
        let gas_limit = gas_estimate * 1; // Add a 25% buffer to the estimate

        

        // 4. Prepare the contract call object and hold it in a variable
        // This ensures the ContractCall object lives long enough.
        let call = self
            .contract
            .batch_create_invoices(data_for_send) // Move original data here
            .gas(gas_limit); // Set the calculated gas limit

        // 5. Send the transaction using the prepared call object
        let pending_tx_result = call.send().await; // Call send().await separately

        // 6. Handle potential errors during the send phase
        let pending_tx = pending_tx_result.map_err(|e| {
            error!("Error sending batchCreateInvoices transaction: {}", e);
            anyhow!("Failed to send transaction: {}", e)
        })?;

    
        // 5. Wait for the transaction receipt
        // The '.await' on the PendingTransaction resolves to Result<Option<TransactionReceipt>, ProviderError>
        let receipt_result = pending_tx.await;

        // Handle the result of waiting for the receipt
        match receipt_result {
            Ok(Some(receipt)) => {
                // Transaction confirmed successfully
                // Check receipt status
                if receipt.status == Some(1.into()) {
                    log::info!(
                        "batchCreateInvoices transaction successful! Hash: {:?}, Block: {:?}",
                        receipt.transaction_hash,
                        receipt.block_number.unwrap_or_default()
                    );
                    Ok(Some(receipt)) // Return the successful receipt
                } else {
                    // Transaction confirmed but reverted (status 0)
                    error!(
                        "batchCreateInvoices transaction confirmed but reverted (status 0). Hash: {:?}, Block: {:?}. Check contract logic and parameters.",
                        receipt.transaction_hash,
                        receipt.block_number.unwrap_or_default()
                    );
                    // Return an error indicating the revert, including the receipt for context
                    Err(anyhow!(
                        "Transaction reverted (status 0). Receipt: {:?}", // Consider a more structured error
                        receipt
                    ))
                }
            }
            Ok(None) => {
                // Transaction was dropped from the mempool (should be rare with sufficient gas)
                error!("batchCreateInvoices transaction was dropped from mempool and not confirmed.");
                Err(anyhow!("Transaction dropped from mempool"))
            }
            Err(e) => {
                // Error while waiting for confirmation (e.g., RPC provider issue)
                error!("Error waiting for batchCreateInvoices transaction receipt: {}", e);
                Err(anyhow!("Failed to get transaction receipt: {}", e)) // Wrap ProviderError into anyhow::Error
            }
        }
    }

    async fn create_token_batch(
        &self,
        batch_id: String,
        invoice_numbers: Vec<String>,
        stable_token_address: String,
        min_term_str: String,
        max_term_str: String,
        interest_rate_str: String,
    ) -> Result<Option<TransactionReceipt>> {
        // Parse inputs
        let stable_token = stable_token_address.parse::<Address>().context("Invalid stable token address")?;
        let min_term = U256::from_dec_str(&min_term_str).context("Invalid min term format")?;
        let max_term = U256::from_dec_str(&max_term_str).context("Invalid max term format")?;
        let interest_rate = U256::from_dec_str(&interest_rate_str).context("Invalid interest rate format")?;

        let tx = self.contract.create_token_batch(
            batch_id.clone(), // Clone batch_id for potential logging
            invoice_numbers,
            stable_token,
            min_term,
            max_term,
            interest_rate,
        );
        let pending_tx = tx.send().await.map_err(|e| {
            error!("Error sending createTokenBatch transaction for batch '{}': {}", batch_id, e);
            anyhow!("Failed to send createTokenBatch transaction: {}", e)
        })?;
        pending_tx.await.map_err(|e| {
            error!("Error waiting for createTokenBatch transaction receipt for batch '{}': {}", batch_id, e);
            anyhow!("Failed to get createTokenBatch transaction receipt: {}", e)
        })
    }

    async fn confirm_token_batch_issue(&self, batch_id: String) -> Result<Option<TransactionReceipt>> {
        let tx = self.contract.confirm_token_batch_issue(batch_id.clone());
        let pending_tx = tx.send().await.map_err(|e| {
            error!("Error sending confirmTokenBatchIssue transaction for batch '{}': {}", batch_id, e);
            anyhow!("Failed to send confirmTokenBatchIssue transaction: {}", e)
        })?;
        pending_tx.await.map_err(|e| {
            error!("Error waiting for confirmTokenBatchIssue transaction receipt for batch '{}': {}", batch_id, e);
            anyhow!("Failed to get confirmTokenBatchIssue transaction receipt: {}", e)
        })
    }

    async fn purchase_shares(&self, batch_id: String, amount_str: String) -> Result<Option<TransactionReceipt>> {
        // Parse amount
        let amount = U256::from_dec_str(&amount_str).context("Invalid amount format")?;

        let tx = self.contract.purchase_shares(batch_id.clone(), amount);
        let pending_tx = tx.send().await.map_err(|e| {
            error!("Error sending purchaseShares transaction for batch '{}' amount '{}': {}", batch_id, amount_str, e);
            anyhow!("Failed to send purchaseShares transaction: {}", e)
        })?;
        pending_tx.await.map_err(|e| {
            error!("Error waiting for purchaseShares transaction receipt for batch '{}': {}", batch_id, e);
            anyhow!("Failed to get purchaseShares transaction receipt: {}", e)
        })
    }
}

impl<M: Middleware + Send + Sync + 'static> InvoiceContract<M> {
    /// Creates a new instance of the InvoiceContract wrapper.
    pub fn new(address: Address, client: Arc<M>) -> Self {
        let contract = InvoiceContractABI::new(address, client.clone());
        Self { contract, client }
    }

    // No method implementations here - they're all in the trait impls
}

// --- Initialization ---

/// Initializes a connection to the blockchain and creates an InvoiceContract instance.
///
/// Reads configuration (RPC URL, contract address, private key) from environment variables.
///
/// # Best Practices
/// For server applications, it's recommended to call this function **once** during startup
/// and store the resulting `InvoiceContract` instance in a shared state (e.g., `lazy_static`,
/// `once_cell`, or application state in a web framework) to reuse the connection and signer.
///
/// # Errors
/// Returns an error if environment variables are missing or invalid, or if connection
/// to the blockchain fails.
pub async fn initialize_contract_from_env() -> Result<InvoiceContract<SignerMiddleware<Provider<Http>, LocalWallet>>> {
    dotenv().ok(); // Load .env file
    let rpc_url = env::var("PHAROS_RPC_URL").context("Failed to read PHAROS_RPC_URL from environment")?;
    let contract_address_str = env::var("INVOICE_CONTRACT_ADDRESS").context("Failed to read INVOICE_CONTRACT_ADDRESS from environment")?;
    let private_key_str = env::var("SIGNER_PRIVATE_KEY").context("Failed to read SIGNER_PRIVATE_KEY from environment")?;

    let provider = Provider::<Http>::try_from(rpc_url).context("Failed to create HTTP provider from RPC URL")?;
    let chain_id = provider.get_chainid().await.context("Failed to get chain ID from provider")?.as_u64();
    let wallet = private_key_str.parse::<LocalWallet>().context("Failed to parse private key")?.with_chain_id(chain_id);
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    let contract_address = contract_address_str.parse::<Address>().context("Failed to parse contract address")?;

    Ok(InvoiceContract::new(contract_address, client))
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::domain::dto::invoice_dto::InvoiceDataDto;
    use common::domain::dto::query_invoice_dto::QueryParamsDto;

    // Test function for querying invoices
    async fn test_query_invoices(contract: &impl ContractQuerier) -> Result<()> {
        println!("\n--- Testing queryInvoices ---");

        let query_params = QueryParamsDto {
            payer: None, // Query any payer
            is_cleared: None,
            payee: None, // Query any payee
            invoice_number: None, // Query for the specific invoice number
            is_valid: None,
        };

        let query_result = contract.query_invoices(query_params.clone()).await?;
        println!("Query successful. Total matching invoices found: {}", query_result.len());
        println!("Fetched Invoices:");

        if query_result.is_empty() {
            println!("  No matching invoices found.");
        } else {
            for invoice_data in query_result.iter() {
                // Print details of each found invoice
                println!(
                    "  - Num: {}, Payee: {}, Payer: {}, Amount: {}, Valid: {}, Cleared: {}, Batch: {}",
                    invoice_data.invoice_number, invoice_data.payee, invoice_data.payer, invoice_data.amount, 
                    invoice_data.is_valid, invoice_data.is_cleared, invoice_data.token_batch
                );
            }
        }

        Ok(())
    }

    // Test function for creating invoices
    async fn test_batch_create_invoices(contract: &impl ContractWriter) -> Result<()> {
        println!("\n--- Testing batchCreateInvoices ---");
        println!("WARNING: This will create REAL invoices on the blockchain");

        // Example payee/payer addresses - REPLACE WITH REAL ADDRESSES FOR TESTING
        let payee_address = "0x95459aed5538bfa47a194d3a0bbbe7a472b5dcd0";
        let payer_address = "0x360a0E35B3e3b678069E3E84c20889A9399A3fF7";

        // Create sample invoices - adjust as needed
        let invoices = vec![InvoiceDataDto {
            invoice_number: format!("INV{}", chrono::Utc::now().timestamp()), // Make unique
            payee: payee_address.to_string(),
            payer: payer_address.to_string(),
            amount: "1000000000000000000".to_string(), // 1 ETH in wei
            ipfs_hash: "QmExample1".to_string(),
            contract_hash: "0x1234567890abcdef".to_string(),
            timestamp: chrono::Utc::now().timestamp().to_string(), // Will be set by contract
            due_date: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp().to_string(),
            token_batch: "".to_string(), // Empty for now
            is_cleared: false,
            is_valid: false,
        }];

        println!("Attempting to create invoice(s): {:?}", invoices);

        match contract.batch_create_invoices(invoices).await {
            Ok(receipt) => {
                println!("Invoices created successfully!");
                println!("Transaction receipt: {:?}", receipt);
            }
            Err(e) => {
                println!("Failed to create invoices: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    // Test function for creating a token batch
    async fn test_create_token_batch(contract: &impl ContractWriter) -> Result<()> {
        println!("\n--- Testing createTokenBatch ---");
        println!("WARNING: This will create a REAL token batch on the blockchain");

        // Create a unique batch ID
        let batch_id = format!("BATCH{}", chrono::Utc::now().timestamp());

        // Example invoice numbers - MUST BE EXISTING INVOICE NUMBERS on the blockchain
        let invoice_numbers = vec!["INV001".to_string(), "INV002".to_string()];

        // Example stable token address - REPLACE WITH REAL TOKEN ADDRESS
        let stable_token = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"; // Example: WETH address

        // Terms
        let min_term = "2592000".to_string(); // 30 days in seconds
        let max_term = "7776000".to_string(); // 90 days in seconds
        let interest_rate = "500".to_string(); // 5% represented as 500 (contract may use basis points)

        println!("Creating token batch '{}' with {} invoices...", batch_id, invoice_numbers.len());

        match contract
            .create_token_batch(batch_id.clone(), invoice_numbers, stable_token.to_string(), min_term, max_term, interest_rate)
            .await
        {
            Ok(receipt) => {
                println!("Token batch created successfully!");
                println!("Transaction receipt: {:?}", receipt);
            }
            Err(e) => {
                println!("Failed to create token batch: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    // Test function for confirming a token batch
    async fn test_confirm_token_batch(contract: &impl ContractWriter) -> Result<()> {
        println!("\n--- Testing confirmTokenBatchIssue ---");
        println!("WARNING: This will confirm a REAL token batch on the blockchain");

        // Use the latest batch ID from create_token_batch test or specify one
        // In production you would likely store and retrieve from a database
        let batch_id = "BATCH1234567890"; // REPLACE WITH AN EXISTING BATCH ID

        println!("Confirming token batch '{}'...", batch_id);

        match contract.confirm_token_batch_issue(batch_id.to_string()).await {
            Ok(receipt) => {
                println!("Token batch confirmed successfully!");
                println!("Transaction receipt: {:?}", receipt);
            }
            Err(e) => {
                println!("Failed to confirm token batch: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    // Test function for purchasing shares
    async fn test_purchase_shares(contract: &impl ContractWriter) -> Result<()> {
        println!("\n--- Testing purchaseShares ---");
        println!("WARNING: This will purchase REAL shares on the blockchain");

        // Use a batch ID from a confirmed batch
        let batch_id = "BATCH1234567890"; // REPLACE WITH AN EXISTING CONFIRMED BATCH ID

        // Amount to purchase
        let amount = "1000000000000000000".to_string(); // 1 token unit (adjust as needed)

        println!("Purchasing {} shares from batch '{}'...", amount, batch_id);

        match contract.purchase_shares(batch_id.to_string(), amount).await {
            Ok(receipt) => {
                println!("Shares purchased successfully!");
                println!("Transaction receipt: {:?}", receipt);
            }
            Err(e) => {
                println!("Failed to purchase shares: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_query() -> Result<()> {
        // 注意：这个测试需要在有真实网络连接和环境配置的情况下运行
        // 可以通过 cargo test -- --ignored 命令来运行
        
        dotenv().ok(); // 确保加载 .env 文件
        if env::var("PHAROS_RPC_URL").is_err() {
            println!("跳过测试：环境变量未配置");
            return Ok(());
        }

        let contract = initialize_contract_from_env().await?;
        test_query_invoices(&contract).await
    }

    #[tokio::test]
    #[ignore] // 由于会创建真实的链上交易，默认忽略此测试
    async fn test_create() -> Result<()> {
        let contract = initialize_contract_from_env().await?;
        test_batch_create_invoices(&contract).await
    }

    #[tokio::test]
    #[ignore] // 由于会创建真实的链上交易，默认忽略此测试
    async fn test_batch() -> Result<()> {
        let contract = initialize_contract_from_env().await?;
        test_create_token_batch(&contract).await
    }

    #[tokio::test]
    #[ignore] // 由于会创建真实的链上交易，默认忽略此测试
    async fn test_confirm() -> Result<()> {
        let contract = initialize_contract_from_env().await?;
        test_confirm_token_batch(&contract).await
    }

    #[tokio::test]
    #[ignore] // 由于会创建真实的链上交易，默认忽略此测试
    async fn test_purchase() -> Result<()> {
        let contract = initialize_contract_from_env().await?;
        test_purchase_shares(&contract).await
    }
}

