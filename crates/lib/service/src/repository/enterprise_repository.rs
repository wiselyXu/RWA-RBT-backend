use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime},
    options::{FindOptions, UpdateOptions}, // Import necessary options
    results::{DeleteResult, UpdateResult},  // Import result types
    Collection, Database,
};

use serde::Serialize;
use common::domain::entity::{Enterprise, EnterpriseStatus};
// Needed for generic update

pub struct EnterpriseRepository {
    collection: Collection<Enterprise>,
}

// --- Struct for partial updates ---
#[derive(Serialize, Default, Debug)]
pub struct UpdateEnterpriseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EnterpriseStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kyc_details_ipfs_hash: Option<String>,
    // We won't allow updating id, created_at, updated_at directly here
    // updated_at will be set automatically
}

impl EnterpriseRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Enterprise>("enterprises"),
        }
    }

    // Find enterprise by ID
    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<Enterprise>, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        self.collection.find_one(filter).await
    }

    // Find all enterprises (consider adding pagination later)
    pub async fn find_all(&self) -> Result<Vec<Enterprise>, mongodb::error::Error> {
        let filter = doc! {}; // Use empty document instead of None
        let mut cursor = self.collection.find(filter).await?; // Pass options wrapped in Some()
        let mut results = Vec::new();
        while let Some(result) = cursor.try_next().await? {
            results.push(result);
        }
        Ok(results)
    }

    // Find enterprise by wallet address
    pub async fn find_by_wallet(&self, wallet_address: &str) -> Result<Option<Enterprise>, mongodb::error::Error> {
        let filter = doc! { "wallet_address": wallet_address.to_lowercase() };
        self.collection.find_one(filter).await
    }

    // Find enterprise by wallet address (case-insensitive)
    pub async fn find_by_wallet_address(&self, wallet_address: &str) -> Result<Option<Enterprise>, mongodb::error::Error> {
        // Create a case-insensitive regex query
        let filter = doc! { 
            "wallet_address": bson::Regex { 
                pattern: format!("^{}$", regex::escape(wallet_address)), 
                options: "i".to_string() // "i" for case-insensitive
            }
        };
        self.collection.find_one(filter).await
    }

    // Create new enterprise
    pub async fn create(&self, name: &str, wallet_address: &str) -> Result<Enterprise, mongodb::error::Error> {
        let enterprise = Enterprise::new(name.to_string(), wallet_address.to_lowercase());
        
        let result = self.collection.insert_one(&enterprise).await?;
        
        let mut created_enterprise = enterprise;
        created_enterprise.id = result.inserted_id.as_object_id();
        
        Ok(created_enterprise)
    }

    // Generic Update enterprise data
    pub async fn update(&self, id: ObjectId, data: UpdateEnterpriseData) -> Result<UpdateResult, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        
        // Convert the update data struct to BSON, skipping None fields automatically due to serde attributes
        let mut update_doc = bson::to_document(&data)?;
        
        // Add updated_at timestamp
        update_doc.insert("updated_at", DateTime::now());

        let update = doc! { "$set": update_doc };
        
        self.collection.update_one(filter, update).await
    }
    
    // Delete enterprise by ID
    pub async fn delete(&self, id: ObjectId) -> Result<DeleteResult, mongodb::error::Error> {
        let filter = doc! { "_id": id };
        self.collection.delete_one(filter).await
    }

    // Update enterprise status
    pub async fn update_status(&self, id: ObjectId, status: EnterpriseStatus) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        let update = doc! { "$set": { "status": bson::to_bson(&status)?, "updated_at": now } };
        
        self.collection.update_one(filter, update).await
    }

    // Update enterprise KYC details IPFS hash
    pub async fn update_kyc_hash(&self, id: ObjectId, hash: &str) -> Result<UpdateResult, mongodb::error::Error> {
        let now = DateTime::now();
        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "kyc_details_ipfs_hash": hash,
                "updated_at": now
            }
        };
        
        self.collection.update_one(filter, update).await
    }
} 