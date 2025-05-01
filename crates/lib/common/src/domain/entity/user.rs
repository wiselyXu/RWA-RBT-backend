use std::str::FromStr;
use mongodb::bson::{DateTime, oid::ObjectId, Decimal128};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String, // Role of the user in the platform
    pub wallet_address: String, // Store the address (e.g., "0x...")
    pub enterprise_id: Option<ObjectId>, // Link to the enterprise this user represents, if any
    pub role: UserRole, // Role of the user in the platform
    pub balance: Decimal128, // User's token balance (e.g., USDC)
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub login_timestamp: DateTime, // Keep track of the last login
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Investor,
    EnterpriseAdmin,
    PlatformAdmin,
}

// Helper methods
impl User {
    pub fn new(wallet_address: String,name:String, role: UserRole) -> Self {
        let now = DateTime::now();
        let zero_balance = Decimal128::from_str("0.0").unwrap();
        Self {
            id: None,
            name,
            wallet_address,
            enterprise_id: None,
            role,
            balance: zero_balance,
            created_at: now,
            updated_at: now,
            login_timestamp: now,
        }
    }
    
    pub fn update_login_time(&mut self) {
        self.login_timestamp = DateTime::now();
        self.updated_at = DateTime::now();
    }
}


