pub mod common_controller;
pub mod swagger_controller;
pub mod user_controller;
pub mod enterprise_controller;
pub mod invoice_controller;
pub mod purchase_controller;
pub mod transaction_controller;
pub mod interest_controller;

use serde::{Deserialize, Serialize};

/// Defines the structure of the JWT claims (payload).
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (typically the user identifier, e.g., wallet address)
    pub sub: String, 
    /// Expiration time (Unix timestamp)
    pub exp: usize,  
    // You can add other custom claims here if needed, e.g.:
    // pub role: String,
}

