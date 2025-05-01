use salvo::oapi::{ToSchema, extract::JsonBody};
use salvo::prelude::*;
use std::sync::Arc;
use mongodb::Database;
use crate::utils::res::{Res, res_json_err, res_json_ok};

use common::domain::entity::UserInvoiceHolding;
use service::service::PurchaseService;
use log::{error, info};
use serde::{Deserialize, Serialize};
use common::domain::dto::holding_dto::HoldingDto;
use common::domain::dto::invoice_redis_dto::InvoiceRedisDto;
use common::domain::dto::purchase_invoice_dto::PurchaseInvoiceDto;
// --- API Handlers ---

/// 获取可购买的票据列表
#[salvo::oapi::endpoint(
    tags("购买"),
    status_codes(200, 500),
    responses(
        (status_code = 200, description = "成功获取可购买票据列表", body = Vec<InvoiceRedisDto>),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_available_invoices(depot: &mut Depot) -> Res<Vec<InvoiceRedisDto>> {
    let purchase_service = depot.obtain::<Arc<PurchaseService>>()
        .expect("PurchaseService not found in depot");
    
    match purchase_service.get_available_invoices().await {
        Ok(invoices) => {
            Ok(res_json_ok(Some(invoices)))
        }
        Err(e) => {
            error!("Failed to get available invoices: {}", e);
            Err(res_json_err("获取可购买票据失败"))
        }
    }
}

/// 购买票据
#[salvo::oapi::endpoint(
    tags("购买"),
    status_codes(200, 400, 401, 500),
    request_body = PurchaseInvoiceDto,
    responses(
        (status_code = 200, description = "购买成功", body = HoldingDto),
        (status_code = 400, description = "无效的请求数据"),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn purchase_invoice(req: JsonBody<PurchaseInvoiceDto>, depot: &mut Depot) -> Res<HoldingDto> {
    // 1. Get authenticated user address from depot (inserted by auth_token middleware)
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("Authenticated user address not found in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };
    
    let purchase_service = depot.obtain::<Arc<PurchaseService>>()
        .expect("PurchaseService not found in depot");
    
    let purchase_data = req.into_inner();
    
    info!("User {} is purchasing invoice: {}, amount: {}", 
          user_address, purchase_data.invoice_id, purchase_data.purchase_amount);
    
    match purchase_service.purchase_invoice(user_address, &purchase_data).await {
        Ok(holding) => {
            // 转换为DTO
            match convert_to_holding_dto(holding).await {
                Ok(holding_dto) => Ok(res_json_ok(Some(holding_dto))),
                Err(e) => {
                    error!("Failed to convert holding to DTO after purchase: {}", e);
                    Err(res_json_err("购买成功但转换数据失败"))
                }
            }
        }
        Err(e) => {
            error!("Purchase failed for user {}: {}", user_address, e);
            Err(res_json_err(&format!("购买失败: {}", e)))
        }
    }
}

/// 查询我的持仓列表
#[salvo::oapi::endpoint(
    tags("购买"),
    status_codes(200, 401, 500),
    responses(
        (status_code = 200, description = "成功获取持仓列表", body = Vec<HoldingDto>),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_my_holdings(depot: &mut Depot) -> Res<Vec<HoldingDto>> {
    // 1. Get authenticated user address from depot
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("Authenticated user address not found in depot: {:?}", e);
            return Err(res_json_err("User not authenticated"));
        }
    };
    
    let purchase_service = depot.obtain::<Arc<PurchaseService>>()
        .expect("PurchaseService not found in depot");
    
    info!("Fetching holdings for user {}", user_address);
    
    match purchase_service.get_user_holdings(user_address).await {
        Ok(holdings) => {
            // 转换为DTO列表
            let mut holding_dtos = Vec::new();
            
            for holding in holdings {
                match convert_to_holding_dto(holding).await {
                    Ok(dto) => holding_dtos.push(dto),
                    Err(e) => {
                        error!("Failed to convert holding to DTO: {}", e);
                        // 继续处理其他持仓
                    }
                }
            }
            
            Ok(res_json_ok(Some(holding_dtos)))
        }
        Err(e) => {
            error!("Failed to get holdings for user {}: {}", user_address, e);
            Err(res_json_err("获取持仓列表失败"))
        }
    }
}

// --- Helper Functions ---

// 将实体转换为DTO
async fn convert_to_holding_dto(holding: UserInvoiceHolding) -> Result<HoldingDto, anyhow::Error> {
    // 在实际实现中，这里需要查询发票信息以获取标题、票据号和到期日等
    // 简化演示，使用占位符
    
    let holding_dto = HoldingDto {
        holding_id: holding.holding_id,
        user_id: holding.user_id,
        invoice_id: holding.invoice_id.to_hex(),
        invoice_number: "PLACEHOLDER".to_string(), // 实际应查询获取
        title: "PLACEHOLDER".to_string(),          // 实际应查询获取
        purchase_date: holding.purchase_date,
        current_balance: holding.current_balance.to_string(),
        total_accrued_interest: holding.total_accrued_interest.to_string(),
        annual_rate: 0.0,                          // 实际应查询获取
        maturity_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), // 实际应查询获取
        status: holding.holding_status,
    };
    
    Ok(holding_dto)
}
