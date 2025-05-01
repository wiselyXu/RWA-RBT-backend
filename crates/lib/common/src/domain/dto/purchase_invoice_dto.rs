use serde::{Deserialize, Serialize};
use salvo::oapi::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PurchaseInvoiceDto {
    pub invoice_id: String,
    pub purchase_amount: f64,
}
