use anyhow::{Context, Result};
use ethers::types::{Address, U256};
use pharos_interact::initialize_contract_from_env;
use std::env;
use common::domain::dto::invoice_dto::InvoiceDataDto;
use common::domain::dto::query_invoice_dto::QueryParamsDto;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up simple logging
    env_logger::init();

    // Parse command line args to determine which test to run
    let args: Vec<String> = env::args().collect();
    let test_mode = if args.len() > 1 { &args[1] } else { "query" };

    println!("Initializing contract connection from .env file...");

    // Initialize the contract instance using settings from .env
    let contract = initialize_contract_from_env().await?;
    println!("Contract initialized successfully.");

    match test_mode {
        "query" => test_query_invoices(&contract).await?,
        "create" => test_batch_create_invoices(&contract).await?,
        "batch" => test_create_token_batch(&contract).await?,
        "confirm" => test_confirm_token_batch(&contract).await?,
        "purchase" => test_purchase_shares(&contract).await?,
        "all" => {
            // Run all tests in sequence (be careful - real transactions!)
            test_query_invoices(&contract).await?;
            test_batch_create_invoices(&contract).await?;
            test_create_token_batch(&contract).await?;
            test_confirm_token_batch(&contract).await?;
            test_purchase_shares(&contract).await?;
        }
        _ => println!("Unknown test mode: {}. Available modes: query, create, batch, confirm, purchase, all", test_mode),
    }

    println!("\nScript finished.");
    Ok(())
}

// Test function for querying invoices
async fn test_query_invoices(contract: &impl pharos_interact::ContractQuerier) -> Result<()> {
    println!("\n--- Testing queryInvoices ---");

    let query_params = QueryParamsDto {
        payer: None, // Query any payer
        is_cleared: None,
        payee: None,                                       // Query any payee
        invoice_number: Some("INV1745657293".to_string()), // Query for the specific invoice number INV1745560448
        is_valid: None,
    };

    println!("Query parameters: {:?}", query_params);

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
                invoice_data.invoice_number, invoice_data.payee, invoice_data.payer, invoice_data.amount, invoice_data.is_valid, invoice_data.is_cleared, invoice_data.token_batch
            );
        }
    }

    Ok(())
}

// Test function for creating invoices
async fn test_batch_create_invoices(contract: &impl pharos_interact::ContractWriter) -> Result<()> {
    println!("\n--- Testing batchCreateInvoices ---");
    println!("WARNING: This will create REAL invoices on the blockchain");

    // Example payee/payer addresses - REPLACE WITH REAL ADDRESSES FOR TESTING
    // 0x95459aed5538bfa47a194d3a0bbbe7a472b5dcd0
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

    println!("Attempting to create  invoice(s): {:?}", invoices);

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
async fn test_create_token_batch(contract: &impl pharos_interact::ContractWriter) -> Result<()> {
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
async fn test_confirm_token_batch(contract: &impl pharos_interact::ContractWriter) -> Result<()> {
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
async fn test_purchase_shares(contract: &impl pharos_interact::ContractWriter) -> Result<()> {
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
