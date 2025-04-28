use ethers::types::{Address, U256};
use pharos_invoice_interact::{initialize_contract_from_env, InvoiceDataInput}; // Import from lib.rs
use std::str::FromStr;
use rand::Rng; // Use rand::Rng for random numbers

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing contract connection from .env file...");

    // Initialize the contract instance using settings from .env
    let contract = initialize_contract_from_env().await?;
    println!("Contract initialized successfully.");

    // --- Create Test Invoices ---
    println!("Attempting to create batch test invoices...");

    // Helper function to create sample data (similar to tests in lib.rs)
    fn create_main_sample_invoice(num: u32) -> InvoiceDataInput {
        let payee = Address::from_str("0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap(); // Placeholder
        let payer = Address::from_str("0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB").unwrap(); // Placeholder

        // 假设一个月之后的时间戳
        let future_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600 * 24 * 30; // 1 月 from now

        InvoiceDataInput {
            invoice_number: format!("MAIN-TEST-{}", num),
            payee,
            payer,
            amount: U256::from(100 * num),
            ipfs_hash: format!("QmMainTestHash{}", num),
            timestamp: U256::zero(), // Will be overwritten by contract
            due_date: U256::from(future_timestamp),
            is_valid: false, // Will be overwritten by contract
        }
    }

    // Create a couple of invoices
    let invoice1 = create_main_sample_invoice(rand::thread_rng().gen::<u16>() as u32); // Use random suffix
    let invoice2 = create_main_sample_invoice(rand::thread_rng().gen::<u16>() as u32);
    println!("invoice1: {:?}", invoice1.clone());
    println!("invoice2: {:?}", invoice2.clone());
    let invoices_to_create = vec![invoice1.clone(), invoice2.clone()];

    println!("Invoices to create: [{}, {}]", invoice1.invoice_number, invoice2.invoice_number);

    match contract.batch_create_invoices(invoices_to_create).await {
        Ok(Some(receipt)) => {
            println!("Batch creation transaction successful: {:?}", receipt.transaction_hash);
            // You might want to check receipt.status here as well
            if receipt.status != Some(1.into()) {
                 println!("Warning: Transaction succeeded but status is not 1 (Success). Status: {:?}", receipt.status);
            }
        }
        Ok(None) => {
            eprintln!("Batch creation transaction was dropped from mempool or not mined yet.");
            // Decide if you want to proceed or exit here
        }
        Err(e) => {
            eprintln!("Error during batch_create_invoices: {}", e);
            // Decide if you want to proceed or exit here
            return Err(e.into()); // Exit if creation failed
        }
    }

    // --- Wait for Transaction ---
    println!("Waiting 10 seconds for transaction to be potentially mined and state updated...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // --- Verification Step 1: Get Specific Invoice ---
    println!("Attempting to verify invoice: {}", invoice1.invoice_number);
    match contract.get_invoice(invoice1.invoice_number.clone(), true).await {
        Ok(data) => {
            println!("Successfully fetched invoice {} data:", invoice1.invoice_number);
            println!("  Invoice Number: {}", data.invoice_number);
            println!("  Payee: {}", data.payee);
            println!("  Payer: {}", data.payer);
            println!("  Amount: {}", data.amount);
            println!("  IPFS Hash: {}", data.ipfs_hash);
            println!("  Timestamp: {}", data.timestamp);
            println!("  Due Date: {}", data.due_date);
            println!("  Is Valid: {}", data.is_valid);
            // Add assertions if needed, e.g.,
            assert_eq!(data.invoice_number, invoice1.invoice_number, "Verification failed: Invoice number mismatch");
            assert!(data.is_valid, "Verification failed: Invoice should be valid");
        }
        Err(e) => {
            eprintln!("Error fetching invoice {}: {}", invoice1.invoice_number, e);
            // Depending on the error, this might indicate the transaction didn't succeed or wasn't processed yet.
        }
    }

    // --- Verification Step 2: Get User Invoices (Optional) ---
    // Using the payee address from the created invoices
    let user_address = invoice1.payee;
    println!("Attempting to get all invoices for user: {}", user_address);

    match contract.get_user_invoices(user_address).await {
        Ok(invoice_numbers) => {
            println!("Successfully fetched invoices for user {}:", user_address);
            if invoice_numbers.is_empty() {
                println!("  No invoices found for this user.");
            } else {
                println!("  Found {} invoices:", invoice_numbers.len());
                for num in &invoice_numbers { // Iterate over reference
                    println!("  - {}", num);
                }
                // Check if the created invoice numbers are present
                 if invoice_numbers.contains(&invoice1.invoice_number) {
                     println!("  Verification successful: Found {} in the list.", invoice1.invoice_number);
                 } else {
                     println!("  Verification warning: {} not found in the user's list.", invoice1.invoice_number);
                 }
                 if invoice_numbers.contains(&invoice2.invoice_number) {
                     println!("  Verification successful: Found {} in the list.", invoice2.invoice_number);
                 } else {
                      println!("  Verification warning: {} not found in the user's list.", invoice2.invoice_number);
                 }
            }
        }
        Err(e) => {
            eprintln!("Error fetching invoices for user {}: {}", user_address, e);
        }
    }

    println!("Script finished.");
    Ok(())
}
