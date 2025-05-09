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
    // 如果连接字符串中**包含**了数据库名 (如 /pharos_rwa)，通常可以直接用 client.database("您的数据库名")
    // 驱动会知道如何使用解析出的认证信息等连接到服务器并操作指定的数据库。
    // 或者，更安全的做法是获取解析出的默认数据库名：
    let db_name = client_options.default_database.as_deref().ok_or_else(|| {
        error!("Database name not specified in connection string, which is required.");
        mongodb::error::Error::custom("Database name not specified in connection string")
    })?;


    // Create the client
    let client = Client::with_options(client_options.clone())?;

    // Get a handle to the database using the correctly parsed name
    // 现在传递的是从 ClientOptions 中获取的，或者硬编码的正确数据库名
    let db = client.database(db_name); // 使用正确解析出的数据库名

    // Ping the database to test the connection
    match db.run_command(mongodb::bson::doc! {"ping": 1}).await {
        Ok(_) => info!("Successfully connected to MongoDB and selected database '{}'", db_name),
        Err(e) => {
            error!("Failed to ping database '{}': {}", db_name, e);
        }
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