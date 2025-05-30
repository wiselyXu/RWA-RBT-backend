use crate::utils::res::{Res, res_bad_request, res_json_err, res_json_ok, res_not_found};
use common::domain::dto::invoice_dto::InvoiceDataDto;
use common::domain::dto::query_invoice_dto::QueryParamsDto;
use common::domain::entity::enterprise::EnterpriseDto;
use common::domain::entity::invoice::InvoiceDto;
use common::domain::entity::{Invoice, InvoiceStatus};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use mongodb::{
    Database,
    bson::{DateTime, Decimal128, oid::ObjectId},
};
use pharos_interact::{ContractQuerier, InvoiceContract};
use salvo::{
    oapi::{ToSchema, extract::JsonBody, extract::PathParam, extract::QueryParam},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use service::repository::InvoiceRepository;
use service::repository::invoice_repository::UpdateInvoiceData;
use std::convert::From;
use std::str::FromStr;
use std::sync::Arc;

// --- Handlers ---
/// 创建一个票据 (Standard endpoint for creating invoice directly in DB)
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 500),
    request_body = InvoiceDataDto,
    responses(
        (status_code = 200, description = "Invoice created successfully.", body = InvoiceDto),
        (status_code = 400, description = "Invalid request data."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn create_invoice(req: JsonBody<InvoiceDataDto>, depot: &mut Depot) -> Res<InvoiceDto> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = InvoiceRepository::new(&mongodb);

    // 1. Get authenticated user address from depot (inserted by auth_token middleware)
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            log::error!("Authenticated user address not found or wrong type in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };

    let data = req.into_inner();
    log::warn!("create_invoice:{:?}", data.clone());
    match repo.create_from_blockchain(&data).await {
        Ok(invoice) => {
            // Convert the created Invoice entity to InvoiceDto for the response
            let data = InvoiceDto::from(&invoice);
            Ok(res_json_ok(Some(data)))
        }
        Err(e) => {
            log::error!("Failed to create invoice: {}", e);
            Err(res_json_err("Failed to create invoice"))
        }
    }
}

/// 查询所有票据
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 500),
    responses(
        (status_code = 200, description = "List of invoices.", body = Vec<InvoiceDto>),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn list_invoices(depot: &mut Depot) -> Res<Vec<InvoiceDto>> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = InvoiceRepository::new(&mongodb);

    match repo.find_all().await {
        Ok(list) => {
            let data: Vec<InvoiceDto> = list
                .iter()
                .map(InvoiceDto::from) // 这里直接传递 From Trait 的实现
                .collect(); // 收集为 Vec<EnterpriseDto>
            // 返回转换后的 Vec<EnterpriseDto>
            Ok(res_json_ok(Some(data)))
        }
        Err(e) => {
            log::error!("Failed to list invoices: {}", e);
            Err(res_json_err("Failed to list invoices"))
        }
    }
}

/// 删除票据
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 404, 500),
    parameters(
        ("id" = String, Query, description = "Invoice MongoDB ObjectId")
    ),
    responses(
        (status_code = 200, description = "Invoice deleted successfully."),
        (status_code = 400, description = "Invalid ID format."),
        (status_code = 404, description = "Invoice not found."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn delete_invoice(invoice_number: QueryParam<String>, depot: &mut Depot) -> Res<()> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = InvoiceRepository::new(&mongodb);

    let oid = match ObjectId::parse_str(&invoice_number.into_inner()) {
        Ok(oid) => oid,
        Err(_) => return Err(res_bad_request("Invalid ObjectId format")),
    };

    match repo.delete(oid).await {
        Ok(delete_result) => {
            if delete_result.deleted_count == 0 {
                Err(res_not_found("Invoice not found"))
            } else {
                Ok(res_json_ok(None))
            }
        }
        Err(e) => {
            log::error!("Failed to delete invoice: {}", e);
            Err(res_json_err("Failed to delete invoice"))
        }
    }
}

/// 查询与我相关的票据（仅查询数据库）
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 404, 500),
    responses(
        (status_code = 200, description = "Invoice data found.", body = Vec<InvoiceDto>),
        (status_code = 400, description = "Invalid request parameters (e.g., missing invoiceNumber)."),
        (status_code = 404, description = "Invoice not found in DB or on chain."),
        (status_code = 500, description = "Internal server error or blockchain query failed."),
    )
)]
pub async fn query_my_invoice(depot: &mut Depot) -> Res<Vec<InvoiceDto>> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = InvoiceRepository::new(&mongodb);

    // 2. Retrieve the authenticated user's address from the Depot using obtain
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            log::error!("Authenticated user address not found or wrong type in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };

    // Use the retrieved user_address to query invoices
    let res = repo.find_by_user(user_address).await;

    match res {
        Ok(data) => {
            let data_list = data.iter().map(InvoiceDto::from).collect::<Vec<InvoiceDto>>();
            Ok(res_json_ok(Option::from(data_list)))
        }
        Err(e) => {
            // Log the specific database error but return a generic message to the user
            log::error!("Failed to query my invoice for user {}: {}", user_address, e);
            Err(res_json_err("Failed to query my invoice"))
        }
    }
}

/// 查询票据数据 (优先查数据库，没有再查链上)
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 404, 500),
    // Use QueryParam for invoice_number as it's a filter
    parameters(
        ("invoice_number" = String, Query, description = "Invoice number to query")
    ),
    responses(
        (status_code = 200, description = "Invoice data found.", body = Vec<InvoiceDto>),
        (status_code = 400, description = "Invalid request parameters (e.g., missing invoiceNumber)."),
        (status_code = 404, description = "Invoice not found in DB or on chain."),
        (status_code = 500, description = "Internal server error or blockchain query failed."),
    )
)]
pub async fn query_invoice_data(invoice_number: QueryParam<String>, depot: &mut Depot) -> Res<Vec<InvoiceDto>> {
    let invoice_number_query = invoice_number.into_inner();
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = InvoiceRepository::new(&mongodb);

    // 1. Query the database first
    match repo.find_by_invoice_number(&invoice_number_query).await {
        Ok(Some(db_invoice)) => {
            log::info!("Found invoice {} in database.", invoice_number_query);
            // Convert to DTO and return as Vec
            let data = vec![InvoiceDto::from(&db_invoice)];
            Ok(res_json_ok(Some(data)))
        }
        Ok(None) => {
            log::info!("Invoice {} not found in database. Querying blockchain...", invoice_number_query);
            // 2. If not in DB, query the blockchain
            query_and_save_from_blockchain(&invoice_number_query, depot, &repo).await
        }
        Err(e) => {
            log::error!("Database query failed for invoice {}: {}", invoice_number_query, e);
            Err(res_json_err("Database query failed"))
        }
    }
}

// Helper function to query blockchain and save to DB
async fn query_and_save_from_blockchain(invoice_number: &str, depot: &mut Depot, repo: &InvoiceRepository) -> Res<Vec<InvoiceDto>> {
    // Try to get contract connection
    let contract_opt = depot.obtain::<Arc<InvoiceContract<SignerMiddleware<Provider<Http>, LocalWallet>>>>();

    if contract_opt.is_err() {
        log::warn!("Blockchain contract connection not available.");
        // Return Not Found as we couldn't check the canonical source
        return Err(res_not_found("Invoice not found and blockchain connection unavailable"));
    }
    let contract = contract_opt.unwrap();

    // Prepare blockchain query parameters
    let mut params = QueryParamsDto {
        invoice_number: Some(invoice_number.to_string()),
        // Set other params to None or default as needed for specific query
        payee: None,
        payer: None,
        is_cleared: None,
        is_valid: None,
    };
    if invoice_number.eq("all") {
        params.invoice_number = None;
    }

    log::info!("Querying blockchain for invoice number: {} with params: {:?}", invoice_number, params);

    // Call the blockchain contract
    match contract.query_invoices(params).await {
        Ok(blockchain_invoices_data) => {
            if blockchain_invoices_data.is_empty() {
                log::warn!("Invoice {} not found on blockchain.", invoice_number);
                Err(res_not_found("Invoice not found on blockchain"))
            } else {
                log::info!(
                    "Found {} invoice(s) on blockchain for number {}. Saving to DB...",
                    blockchain_invoices_data.len(),
                    invoice_number
                );
                let mut saved_invoice_dtos = Vec::new();

                // Save each found invoice to DB
                for invoice_data_dto in blockchain_invoices_data {
                    // 首先，尝试根据 invoice_number 查找票据
                    match repo.find_by_invoice_number(&invoice_data_dto.invoice_number).await {
                        // 查找成功，并且找到了票据
                        Ok(Some(existing_invoice)) => {
                            log::info!(
                                "Invoice {} already exists in DB with id {:?}, skipping creation.",
                                existing_invoice.invoice_number,
                                existing_invoice.id
                            );
                            // 可选：如果需要，您可以在这里将找到的现有票据 DTO 添加到 saved_invoice_dtos
                            saved_invoice_dtos.push(InvoiceDto::from(&existing_invoice));
                        }
                        // 查找成功，但没有找到票据
                        Ok(None) => {
                            // 票据不存在，可以安全地尝试创建
                            log::debug!("Invoice {} not found in DB, attempting to create from blockchain data.", invoice_data_dto.invoice_number);
                            match repo.create_from_blockchain(&invoice_data_dto).await {
                                Ok(saved_invoice) => {
                                    log::info!("Successfully saved new invoice {} from blockchain to DB.", saved_invoice.invoice_number);
                                    saved_invoice_dtos.push(InvoiceDto::from(&saved_invoice));
                                }
                                Err(e) => {
                                    // 创建过程中发生错误
                                    log::error!("Failed to save new invoice {} from blockchain to DB: {}", invoice_data_dto.invoice_number, e);
                                }
                            }
                        }
                        // 查找过程中发生错误
                        Err(e) => {
                            log::error!("Failed to check existence for invoice {}: {}. Skipping creation.", invoice_data_dto.invoice_number, e);
                            // 如果查找失败，也跳过创建，避免潜在的重复
                        }
                    }
                }

                if saved_invoice_dtos.is_empty() {
                    // This case might happen if all saves failed
                    Err(res_json_err("Found invoice on chain but failed to save to database"))
                } else {
                    // Return the successfully saved invoices as DTOs
                    Ok(res_json_ok(Some(saved_invoice_dtos)))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to query blockchain for invoice {}: {}", invoice_number, e);
            Err(res_json_err(&format!("Blockchain query failed: {}", e)))
        }
    }
}
