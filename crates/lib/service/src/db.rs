use mongodb::{Client, Database, options::ClientOptions};
use log::{info, error};
use mongodb::options::Credential;
use configs::cfgs::Database as DbConfig;
use common::domain::entity::{Repayment, User};

// MongoDB client initialization
pub async fn init_mongodb(db_config: &DbConfig) -> Result<Database, mongodb::error::Error> {
    info!("Connecting to MongoDB at: {}", db_config.url);
    
    // Parse connection string
    let mut client_options = ClientOptions::parse(&db_config.url).await?;
    
    // Optionally set additional options
    client_options.app_name = Some("pharos-rwa".to_string());
    // Create the client
    let client = Client::with_options(client_options)?;
    
    // Get a handle to the database
    let db_name = db_config.url.split('/').last().unwrap_or("pharos_rwa");
    let db = client.database(db_name);
    
    // Ping the database to test the connection
    match client.list_database_names().await {
        Ok(_) => info!("Successfully connected to MongoDB"),
        Err(e) => error!("Failed to connect to MongoDB: {}", e),
    }
    
    Ok(db)
}

// Helper function to get collection names
pub async fn get_collection_names(db: &Database) -> Result<Vec<String>, mongodb::error::Error> {
    let names = db.list_collection_names().await?;
    Ok(names)
}

// Example function to create indexes - you'd call this during app initialization
pub async fn create_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    use mongodb::bson::doc;
    use mongodb::options::IndexOptions;
    use mongodb::IndexModel;
    
    // Example: Create a unique index on wallet_address in the users collection
    let options = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! { "wallet_address": 1 })
        .options(options)
        .build();
        
    db.collection::<User>("users")
        .create_index(index)
        .await?;
        
    // Example: Create a unique index on transaction_hash in the repayments collection
    let options = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! { "transaction_hash": 1 })
        .options(options)
        .build();
        
    db.collection::<Repayment>("repayments")
        .create_index(index)
        .await?;
    Ok(())
} 