use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Debug, Clone, Default, Serialize, Deserialize,ToSchema)]
#[salvo(schema(example = json!({
    "payee": "",
    "payer": "",
    "invoiceNumber": "INV001",
    "isCleared": false,
    "isValid": true
})))]
pub struct QueryParamsDto {
    pub payee: Option<String>, // Address as String
    pub payer: Option<String>, // Address as String
    pub invoice_number: Option<String>, // Example: Add other relevant fields if needed
    pub is_cleared: Option<bool>,
    pub is_valid: Option<bool>,
}