use crate::utils::res::{Res, res_bad_request, res_json_err, res_json_ok, res_not_found, res_json_custom};
use chrono::NaiveDate;
use common::domain::dto::interest_detail_dto::InterestDetailDto;
use common::domain::dto::invoice_dto::{CreateInvoiceDto, InvoiceDataDto};
use common::domain::dto::query_invoice_dto::QueryParamsDto;
use common::domain::entity::Invoice;
use common::domain::entity::enterprise::EnterpriseDto;
use common::domain::entity::invoice::InvoiceDto;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;
use log::{error, warn};
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
use service::invoice::InvoiceService;
use service::repository::InvoiceRepository;
use service::repository::invoice_repository::UpdateInvoiceData;
use std::convert::From;
use std::str::FromStr;
use std::sync::Arc;
use std::fs::read;
use salvo::http::StatusCode;
use salvo::oapi::oapi;
use service::{EnterpriseRepository, UserRepository};
use crate::controller::EnterpriseInfoResponse;
use service::error::ServiceError;
use common::domain::entity::invoice_status::InvoiceStatus;
use service::repository::invoice_batch_repository::InvoiceBatchRepository;

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
pub async fn create_invoice(req: &mut Request, depot: &mut Depot) -> Res<InvoiceDto> {
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

    // 2. Manually parse the body and handle potential deserialization errors
    let data = match req.parse_body::<CreateInvoiceDto>().await {
        Ok(dto) => dto,
        Err(e) => {
            log::error!("Failed to deserialize InvoiceDataDto: {}", e);
            return Err(res_bad_request(&format!("Invalid request body: {}", e)));
        }
    };

    match repo.create_from_blockchain(&data).await {
        Ok(invoice) => {
            log::info!("Successfully created invoice {} for user {}", invoice.invoice_number, user_address);
            let response_dto = InvoiceDto::from(&invoice);
            Ok(res_json_ok(Some(response_dto)))
        }
        Err(e) => {
            log::error!("Failed to create invoice in repository for user {}: {}", user_address, e);
            Err(res_json_err("Failed to save invoice data"))
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

/// 查询持仓利息明细
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 401, 404, 500),
    parameters(
        ("holding_id" = String, Query, description = "Holding ID to get interest details for")
    ),
    responses(
        (status_code = 200, description = "Interest details found.", body = Vec<InterestDetailDto>),
        (status_code = 400, description = "Invalid request parameters."),
        (status_code = 401, description = "User not authenticated."),
        (status_code = 404, description = "Holding not found or not owned by user."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn get_holding_interest_details(holding_id: QueryParam<String>, depot: &mut Depot) -> Res<Vec<InterestDetailDto>> {
    // Get the authenticated user address
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            log::error!("Authenticated user address not found in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };

    // Get the Invoice Service
    let invoice_service = depot.obtain::<Arc<InvoiceService>>().expect("InvoiceService not found in depot");

    let holding_id_str = holding_id.into_inner();

    log::info!("Fetching interest details for holding {} owned by user {}", holding_id_str, user_address);

    match invoice_service.get_holding_interest_details(user_address, &holding_id_str).await {
        Ok(details) => Ok(res_json_ok(Some(details))),
        Err(e) => {
            log::error!("Failed to get interest details for holding {}: {}", holding_id_str, e);
            Err(res_json_err(&format!("Failed to retrieve interest details: {}", e)))
        }
    }
}

/// 管理员触发每日利息计算
#[salvo::oapi::endpoint(
    tags("管理员"),
    status_codes(200, 400, 401, 500),
    parameters(
        ("date" = String, Query, description = "Date to calculate interest for (YYYY-MM-DD)")
    ),
    responses(
        (status_code = 200, description = "Interest calculation triggered successfully.", body = u32),
        (status_code = 400, description = "Invalid date format."),
        (status_code = 401, description = "Not authorized."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn trigger_daily_interest_calculation(date: QueryParam<String>, depot: &mut Depot) -> Res<u32> {
    // TODO: Add admin authentication check

    let invoice_service = depot.obtain::<Arc<InvoiceService>>().expect("InvoiceService not found in depot");

    // Parse the date
    let date_str = date.into_inner();
    let calculation_date = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(e) => {
            log::error!("Invalid date format: {}", e);
            return Err(res_bad_request("Invalid date format. Please use YYYY-MM-DD."));
        }
    };

    log::info!("Triggering daily interest calculation for date: {}", calculation_date);

    match invoice_service.calculate_daily_interest_for_date(calculation_date).await {
        Ok(count) => {
            log::info!("Successfully calculated interest for {} holdings", count);
            Ok(res_json_ok(Some(count)))
        }
        Err(e) => {
            log::error!("Failed to calculate daily interest: {}", e);
            Err(res_json_err(&format!("Failed to calculate daily interest: {}", e)))
        }
    }
}

/// 管理员触发到期票据还款处理
#[salvo::oapi::endpoint(
    tags("管理员"),
    status_codes(200, 400, 401, 500),
    parameters(
        ("date" = String, Query, description = "Date to process maturity payments for (YYYY-MM-DD)")
    ),
    responses(
        (status_code = 200, description = "Maturity payment processing triggered successfully.", body = u32),
        (status_code = 400, description = "Invalid date format."),
        (status_code = 401, description = "Not authorized."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn trigger_maturity_payments(date: QueryParam<String>, depot: &mut Depot) -> Res<u32> {
    // TODO: Add admin authentication check

    let invoice_service = depot.obtain::<Arc<InvoiceService>>().expect("InvoiceService not found in depot");

    // Parse the date
    let date_str = date.into_inner();
    let payment_date = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(e) => {
            log::error!("Invalid date format: {}", e);
            return Err(res_bad_request("Invalid date format. Please use YYYY-MM-DD."));
        }
    };

    log::info!("Triggering maturity payments for date: {}", payment_date);

    match invoice_service.process_maturity_payments_for_date(payment_date).await {
        Ok(count) => {
            log::info!("Successfully processed maturity payments for {} holdings", count);
            Ok(res_json_ok(Some(count)))
        }
        Err(e) => {
            log::error!("Failed to process maturity payments: {}", e);
            Err(res_json_err(&format!("Failed to process maturity payments: {}", e)))
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
                    match repo.find_by_invoice_number(&invoice_data_dto.invoice_number.clone()).await {
                        // 查找成功，并且找到了票据
                        Ok(Some(existing_invoice)) => {
                            log::info!(
                                "Invoice {} already exists in DB with id {:?}, skipping creation.",
                                existing_invoice.invoice_number,
                                existing_invoice.id
                            );
                            // 可选：如果需要，您可以在这里将找到的现有票据 DTO 添加到 saved_invoice_dtos
                            // saved_invoice_dtos.push(InvoiceDto::from(&existing_invoice));
                        }
                        // 查找成功，但没有找到票据
                        Ok(None) => {
                            // 票据不存在，可以安全地尝试创建
                            log::debug!("Invoice {:?} not found in DB, attempting to create from blockchain data.", invoice_data_dto);

                            let create_dto = CreateInvoiceDto {
                                payee: invoice_data_dto.payee.clone(),
                                payer: invoice_data_dto.payer.clone(),
                                amount: invoice_data_dto.amount.clone(),
                                invoice_ipfs_hash: invoice_data_dto.invoice_ipfs_hash.clone(),
                                contract_ipfs_hash: invoice_data_dto.contract_ipfs_hash.clone(),
                                due_date: invoice_data_dto.due_date.clone(),
                                currency: invoice_data_dto.currency.clone(),
                            };
                            match repo.create_from_blockchain(&create_dto).await {
                                Ok(saved_invoice) => {
                                    log::info!("Successfully saved new invoice {} from blockchain to DB.", saved_invoice.invoice_number);
                                    saved_invoice_dtos.push(InvoiceDto::from(&saved_invoice));
                                }
                                Err(e) => {
                                    // 创建过程中发生错误
                                    log::error!("Failed to save new invoice {:?} from blockchain to DB: {}", invoice_data_dto, e);
                                }
                            }
                        }
                        // 查找过程中发生错误
                        Err(e) => {
                            log::error!("Failed to check existence for invoice {:?}: {}. Skipping creation.", invoice_data_dto, e);
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

// 票据上链接口参数
#[derive(Deserialize, Serialize, Debug, ToSchema)]
struct VerifyInvoiceParams {
    pub id: String,
}

// 批量发行接口参数
#[derive(Deserialize, Serialize, Debug, ToSchema)]
struct IssueInvoicesParams {
    pub invoice_ids: Vec<String>,
}

/// 票据上链(将票据状态更新为已上链)
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 404, 500),
    request_body = VerifyInvoiceParams,
    responses(
        (status_code = 200, description = "Invoice verified successfully.", body = InvoiceDto),
        (status_code = 400, description = "Invalid invoice ID."),
        (status_code = 404, description = "Invoice not found."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn verify_invoice(req: &mut Request, depot: &mut Depot) -> Res<InvoiceDto> {
    // 1. 获取已认证用户的地址（由auth_token中间件插入）
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            log::error!("Authenticated user address not found or wrong type in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };

    // 2. 解析请求参数
    let params = match req.parse_json::<VerifyInvoiceParams>().await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to parse verify invoice parameters: {:?}", e);
            return Err(res_bad_request("Invalid request parameters"));
        }
    };

    // 3. 获取依赖
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    // 从depot获取已初始化的服务实例
    let invoice_service = depot.obtain::<Arc<InvoiceService>>().expect("InvoiceService not found in depot");

    // 4. 调用服务将票据状态更新为已上链
    match invoice_service.verify_invoice(&params.id).await {
        Ok(invoice) => {
            log::info!("Successfully verified invoice {} by user {}", params.id, user_address);
            // 返回成功响应
            let dto = InvoiceDto::from(&invoice);
            Ok(res_json_ok(Some(dto)))
        }
        Err(e) => {
            log::error!("Failed to verify invoice {}: {}", params.id, e);
            match e {
                ServiceError::NotFound(_) => Err(res_not_found(&format!("Invoice not found: {}", params.id))),
                ServiceError::InternalError(msg) => Err(res_bad_request(&msg)),
                ServiceError::MongoDbError(msg) => Err(res_json_err(&format!("Database error: {}", msg))),
                _ => Err(res_json_err(&format!("Failed to verify invoice: {}", e)))
            }
        }
    }
}

/// 票据批量发行到市场(将票据状态从已上链更新为在售)
#[salvo::oapi::endpoint(
    tags("票据"),
    status_codes(200, 400, 500),
    request_body = IssueInvoicesParams,
    responses(
        (status_code = 200, description = "Invoices issued successfully.", body = String),
        (status_code = 400, description = "Invalid request parameters or consistency checks failed."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn issue_invoices(req: &mut Request, depot: &mut Depot) -> Res<String> {
    // 1. 获取已认证用户的地址（由auth_token中间件插入）
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            log::error!("Authenticated user address not found or wrong type in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };

    // 2. 解析请求参数
    let params = match req.parse_json::<IssueInvoicesParams>().await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to parse issue invoices parameters: {:?}", e);
            return Err(res_bad_request("Invalid request parameters"));
        }
    };

    // 验证参数有效性
    if params.invoice_ids.is_empty() {
        return Err(res_bad_request("No invoices selected for issuance"));
    }

    // 3. 获取依赖
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    // 从depot获取已初始化的服务实例
    let invoice_service = depot.obtain::<Arc<InvoiceService>>().expect("InvoiceService not found in depot");

    // 4. 调用服务批量更新票据状态为在售(OnSale)
    match invoice_service.issue_invoices(&params.invoice_ids).await {
        Ok(count) => {
            log::info!("Successfully issued {} invoices by user {}", count, user_address);
            let message = format!("Successfully issued {} invoices", count);
            Ok(res_json_ok(Some(message)))
        }
        Err(e) => {
            log::error!("Failed to issue invoices: {}", e);
            // 区分验证错误和其他错误
            match e {
                ServiceError::InvoiceNotIssue(msg) if msg.contains("same payee, payer and currency") => {
                    // 这是一个一致性验证错误
                    Err(res_json_err(&format!("Consistency check failed: {}", msg)))
                }
                ServiceError::InternalError(msg) if msg.contains("No valid verified invoices") => {
                    // 没有找到有效的已上链票据
                    Err(res_bad_request("No valid verified invoices found"))
                }
                _ => Err(res_json_err(&format!("Failed to issue invoices: {}", e)))
            }
        }
    }
}

/// 获取当前用户的发票批次列表
#[salvo::oapi::endpoint(
    tags("发票批次"),
    status_codes(200, 401, 500),
    responses(
        (status_code = 200, description = "用户发票批次列表", body = Vec<InvoiceBatchDto>),
        (status_code = 401, description = "用户未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_user_invoice_batches(depot: &mut Depot) -> Res<Vec<InvoiceBatchDto>> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let invoice_batch_repo = InvoiceBatchRepository::new(&mongodb);
    let enterprise_repo = EnterpriseRepository::new(&mongodb);
    
    // 获取认证用户信息
    let user_id = match depot.get::<String>("user_id") {
        Ok(id) => id,
        Err(_) => {
            error!("User ID not found in depot");
            return Err(res_json_custom(401, "用户未认证"));
        }
    };
    
    // 获取用户绑定的企业ID
    let enterprise_id = match depot.get::<String>("enterprise_id") {
        Ok(id) => {
            match ObjectId::parse_str(id) {
                Ok(oid) => oid,
                Err(_) => {
                    error!("Invalid enterprise ID format");
                    return Err(res_json_err("企业ID格式无效"));
                }
            }
        },
        Err(_) => {
            error!("Enterprise ID not found in depot");
            return Err(res_json_custom(403, "用户未绑定企业"));
        }
    };
    
    // 查询用户作为债权人的发票批次
    match invoice_batch_repo.find_by_creditor(enterprise_id).await {
        Ok(batches) => {
            let mut batch_dtos = Vec::new();
            
            for batch in batches {
                // 确保批次有ID
                let batch_id = match batch.id {
                    Some(id) => id,
                    None => continue, // 如果没有ID，跳过这个批次
                };
                
                // 获取债权人和债务人信息
                let creditor = match enterprise_repo.find_by_id(batch.creditor_id).await {
                    Ok(Some(e)) => e,
                    _ => continue, // 如果找不到债权人，跳过这个批次
                };
                
                let debtor = match enterprise_repo.find_by_id(batch.debtor_id).await {
                    Ok(Some(e)) => e,
                    _ => continue, // 如果找不到债务人，跳过这个批次
                };
                
                // 获取批次中的发票数量
                let invoice_repo = InvoiceRepository::new(&mongodb);
                let invoice_count = match invoice_repo.find_by_batch_id(batch_id).await {
                    Ok(invoices) => invoices.len(),
                    Err(_) => 0,
                };
                
                // 计算批次总金额
                let mut total_amount: u64 = 0;
                if let Ok(invoices) = invoice_repo.find_by_batch_id(batch_id).await {
                    for invoice in &invoices {
                        total_amount += invoice.amount;
                    }
                }
                
                let batch_dto = InvoiceBatchDto {
                    id: batch_id.to_string(),
                    creditor_name: creditor.name.clone(),
                    debtor_name: debtor.name.clone(),
                    accepted_currency: batch.accepted_currency.clone(),
                    status: format!("{:?}", batch.status),
                    created_at: batch.created_at.to_string(),
                    invoice_count: invoice_count as u32,
                    total_amount,
                    token_batch_id: batch.token_batch_id.map(|id| id.to_string()),
                };
                
                batch_dtos.push(batch_dto);
            }
            
            Ok(res_json_ok(Some(batch_dtos)))
        },
        Err(e) => {
            error!("Failed to get user invoice batches: {}", e);
            Err(res_json_err(&format!("获取发票批次列表失败: {}", e)))
        }
    }
}

/// 获取发票批次详情
#[salvo::oapi::endpoint(
    tags("发票批次"),
    status_codes(200, 400, 404, 500),
    parameters(
        ("id" = String, Path, description = "批次ID")
    ),
    responses(
        (status_code = 200, description = "批次详情", body = InvoiceBatchDto),
        (status_code = 400, description = "无效的批次ID"),
        (status_code = 404, description = "批次不存在"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn get_invoice_batch_by_id(id: PathParam<String>, depot: &mut Depot) -> Res<InvoiceBatchDto> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let invoice_batch_repo = InvoiceBatchRepository::new(&mongodb);
    let enterprise_repo = EnterpriseRepository::new(&mongodb);
    
    // 解析批次ID
    let batch_id = match ObjectId::parse_str(&id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => {
            error!("Invalid batch ID format");
            return Err(res_bad_request("批次ID格式无效"));
        }
    };
    
    // 获取批次信息
    match invoice_batch_repo.find_by_id(batch_id).await {
        Ok(Some(batch)) => {
            // 确保批次有ID
            let batch_id_obj = match batch.id {
                Some(id) => id,
                None => {
                    error!("Batch ID is missing");
                    return Err(res_json_err("批次ID缺失"));
                }
            };
            
            // 获取债权人和债务人信息
            let creditor = match enterprise_repo.find_by_id(batch.creditor_id).await {
                Ok(Some(e)) => e,
                _ => {
                    error!("Creditor not found for batch {}", batch_id);
                    return Err(res_json_err("找不到批次的债权人信息"));
                }
            };
            
            let debtor = match enterprise_repo.find_by_id(batch.debtor_id).await {
                Ok(Some(e)) => e,
                _ => {
                    error!("Debtor not found for batch {}", batch_id);
                    return Err(res_json_err("找不到批次的债务人信息"));
                }
            };
            
            // 获取批次中的发票数量
            let invoice_repo = InvoiceRepository::new(&mongodb);
            let invoice_count = match invoice_repo.find_by_batch_id(batch_id).await {
                Ok(invoices) => invoices.len(),
                Err(_) => 0,
            };
            
            // 计算批次总金额
            let mut total_amount: u64 = 0;
            if let Ok(invoices) = invoice_repo.find_by_batch_id(batch_id).await {
                for invoice in &invoices {
                    total_amount += invoice.amount;
                }
            }
            
            let batch_dto = InvoiceBatchDto {
                id: batch_id_obj.to_string(),
                creditor_name: creditor.name.clone(),
                debtor_name: debtor.name.clone(),
                accepted_currency: batch.accepted_currency.clone(),
                status: format!("{:?}", batch.status),
                created_at: batch.created_at.to_string(),
                invoice_count: invoice_count as u32,
                total_amount,
                token_batch_id: batch.token_batch_id.map(|id| id.to_string()),
            };
            
            Ok(res_json_ok(Some(batch_dto)))
        },
        Ok(None) => {
            error!("Invoice batch not found: {}", batch_id);
            Err(res_not_found("找不到指定批次"))
        },
        Err(e) => {
            error!("Failed to get invoice batch: {}", e);
            Err(res_json_err(&format!("获取批次详情失败: {}", e)))
        }
    }
}

/// 获取批次中的发票列表
#[salvo::oapi::endpoint(
    tags("发票批次"),
    status_codes(200, 400, 404, 500),
    parameters(
        ("id" = String, Path, description = "批次ID")
    ),
    responses(
        (status_code = 200, description = "批次中的发票列表", body = Vec<InvoiceDto>),
        (status_code = 400, description = "无效的批次ID"),
        (status_code = 404, description = "批次不存在"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn get_invoices_by_batch_id(id: PathParam<String>, depot: &mut Depot) -> Res<Vec<InvoiceDto>> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let invoice_repo = InvoiceRepository::new(&mongodb);
    
    // 解析批次ID
    let batch_id = match ObjectId::parse_str(&id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => {
            error!("Invalid batch ID format");
            return Err(res_bad_request("批次ID格式无效"));
        }
    };
    
    // 获取批次中的发票
    match invoice_repo.find_by_batch_id(batch_id).await {
        Ok(invoices) => {
            let invoice_dtos = invoices.iter().map(InvoiceDto::from).collect::<Vec<InvoiceDto>>();
            Ok(res_json_ok(Some(invoice_dtos)))
        },
        Err(e) => {
            error!("Failed to get invoices by batch: {}", e);
            Err(res_json_err(&format!("获取批次发票列表失败: {}", e)))
        }
    }
}

// 定义InvoiceBatchDto
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InvoiceBatchDto {
    pub id: String,
    pub creditor_name: String,
    pub debtor_name: String,
    pub accepted_currency: String,
    pub status: String,
    pub created_at: String,
    pub invoice_count: u32,
    pub total_amount: u64,
    pub token_batch_id: Option<String>,
}

