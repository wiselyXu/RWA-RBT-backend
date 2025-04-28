// Define the modules within the entity directory
pub mod enterprise;
pub mod user;
pub mod invoice;
pub mod invoice_batch;
pub mod rbt_holding;
pub mod repayment;
pub mod settlement_nft;
// Optional: Re-export entities for easier access
// pub use user::Entity as User;
// pub use login_log::Entity as LoginLog; 

// Re-export entity types for easier access
pub use enterprise::{Enterprise, EnterpriseStatus};
pub use user::{User, UserRole};
pub use invoice::{Invoice, InvoiceStatus};
pub use invoice_batch::{InvoiceBatch, InvoiceBatchStatus};
pub use rbt_holding::RbtHolding;
pub use repayment::Repayment;
pub use settlement_nft::SettlementNft;
