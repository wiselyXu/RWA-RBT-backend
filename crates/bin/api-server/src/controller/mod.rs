pub mod common_controller;
pub mod enterprise_controller;
pub mod interest_controller;
pub mod invoice_controller;
pub mod purchase_controller;
pub mod transaction_controller;
pub mod user_controller;
pub mod swagger_controller;
pub mod token_controller;


pub use common_controller::*;
pub use enterprise_controller::*;
pub use interest_controller::*;
pub use invoice_controller::*;
pub use purchase_controller::*;
pub use transaction_controller::*;
pub use user_controller::*;
pub use swagger_controller::*;
pub use token_controller::*;

use serde::{Deserialize, Serialize};

/// Defines the structure of the JWT claims (payload).
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (typically the user identifier, e.g., wallet address)
    pub sub: String, 
    /// Expiration time (Unix timestamp)
    pub exp: usize,  
    /// User ID (MongoDB ObjectId)
    pub user_id: String,
    /// User role
    pub role: String,
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
    
    pub fn is_creditor(&self) -> bool {
        self.role == "creditor"
    }
    
    pub fn is_investor(&self) -> bool {
        self.role == "investor"
    }
}

