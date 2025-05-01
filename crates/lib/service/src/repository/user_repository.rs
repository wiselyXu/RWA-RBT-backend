use mongodb::{bson, bson::{doc, oid::ObjectId, DateTime, Decimal128}, Collection, Database, ClientSession};

use log::{info, error};
use common::domain::entity::{User, UserRole};
use mongodb::options::UpdateOptions;
use futures::stream::TryStreamExt; // For cursor iteration
use regex;
use crate::error::ServiceError;

pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<User>("users"),
        }
    }

    // Find user by wallet address (case-insensitive)
    pub async fn find_by_wallet_address(&self, wallet_address: &str) -> Result<Option<User>, mongodb::error::Error> {
        let filter = doc! { 
            "wallet_address": bson::Regex { 
                pattern: format!("^{}$", regex::escape(wallet_address)), 
                options: "i".to_string()
            }
        };
        self.collection.find_one(filter).await
    }

    // Find user by wallet address within a transaction session (case-insensitive)
    pub async fn find_by_wallet_address_session(&self, wallet_address: &str, session: &mut ClientSession) -> Result<Option<User>, ServiceError> {
        let filter = doc! { 
            "wallet_address": bson::Regex { 
                pattern: format!("^{}$", regex::escape(wallet_address)), 
                options: "i".to_string()
            }
        };
        // Use find_one() with session argument
        self.collection.find_one(filter).session(session).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))
    }

    // Atomically update user balance within a transaction session
    // `amount_to_add` can be negative to deduct balance.
    pub async fn update_balance_session(&self, user_wallet_address: &str, amount_to_add: Decimal128, session: &mut ClientSession) -> Result<bool, ServiceError> {
        let filter = doc! { 
            "wallet_address": bson::Regex { 
                pattern: format!("^{}$", regex::escape(user_wallet_address)), 
                options: "i".to_string()
            }
            // Optionally add a condition to prevent balance going below zero if needed
            // "$expr": { "$gte": [ { "$add": ["$balance", amount_to_add] }, Decimal128::from(0i64) ] } 
        };
        let update = doc! { 
            "$inc": { "balance": amount_to_add },
            "$set": { "updated_at": DateTime::now() } 
        };

        // Use update_one() with session argument
        let result = self.collection.update_one(filter, update).session(session).await
            .map_err(|e| ServiceError::MongoDbError(e.to_string()))?;

        Ok(result.modified_count > 0)
    }

    // Create a new user
    pub async fn create_user(&self, user: User) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(user).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }
    
    // Update user login timestamp
    pub async fn update_login_timestamp(&self, user_id: ObjectId) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "_id": user_id };
        let update = doc! { "$set": { "login_timestamp": DateTime::now(), "updated_at": DateTime::now() } };
        self.collection.update_one(filter, update).await?;
        Ok(())
    }

    // Find or create user, and update login time
    pub async fn process_login(&self, wallet_address: &str) -> Result<User, mongodb::error::Error> {
        match self.find_by_wallet_address(wallet_address).await? {
            Some(mut user) => {
                // User found, update login time
                self.update_login_timestamp(user.id.unwrap()).await?;
                // Update the timestamp in the returned object as well
                user.update_login_time(); 
                Ok(user)
            }
            None => {
                // User not found, create a new one
                // Assign a default role, e.g., Investor. Adjust as needed.
                let new_user = User::new(wallet_address.to_lowercase(),"".to_string(), common::domain::entity::UserRole::Investor);
                let created_user_id = self.create_user(new_user.clone()).await?;
                let mut created_user = new_user;
                created_user.id = Some(created_user_id); // Set the ID after creation
                Ok(created_user)
            }
        }
    }

    // Bind a user to an enterprise
    pub async fn bind_enterprise(&self, user_wallet_address: &str, enterprise_id: ObjectId) -> Result<bool, mongodb::error::Error> {
        let filter = doc! { 
            "wallet_address": bson::Regex { 
                pattern: format!("^{}$", regex::escape(user_wallet_address)), 
                options: "i".to_string()
            }
        };
        let update = doc! { 
            "$set": { 
                "enterprise_id": enterprise_id,
                "updated_at": DateTime::now()
                // Optionally update role here if binding implies a role change
                // "role": bson::to_bson(&common::domain::entity::UserRole::EnterpriseAdmin).unwrap()
            }
        };

        let result = self.collection.update_one(filter, update).await?;
        
        // Return true if a document was modified, false otherwise
        Ok(result.modified_count > 0)
    }
} 