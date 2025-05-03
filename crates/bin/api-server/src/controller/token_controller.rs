use salvo::{oapi::{ToSchema, extract::JsonBody, extract::PathParam, extract::QueryParam}, prelude::*};
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;
use log::{debug, info, error};

use common::domain::entity::{
    CreateTokenBatchRequest, PurchaseTokenRequest, QueryTokenMarketRequest, QueryUserTokenHoldingsRequest,
    TokenBatchStatus, TokenBatchResponse, TokenMarketResponse, TokenHoldingResponse
};
use service::service::TokenService;

use crate::utils::res::{Res, res_json_err, res_json_ok, res_json_custom, res_bad_request};
use crate::controller::Claims;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenBatchIdResponse {
    batch_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenHoldingIdResponse {
    holding_id: String,
}

/// 创建代币批次（必须是债权人或管理员）
/// 创建代币批次（必须是债权人或管理员）
#[salvo::oapi::endpoint(
    tags("代币管理"),
    status_codes(200, 400, 403, 500),
    request_body = CreateTokenBatchRequest,
    responses(
        (status_code = 200, description = "代币批次创建成功", body = TokenBatchIdResponse),
        (status_code = 400, description = "无效的请求参数"),
        (status_code = 403, description = "无权限创建代币批次"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn create_token_batch(req: JsonBody<CreateTokenBatchRequest>, depot: &mut Depot) -> Res<TokenBatchIdResponse> {
    info!("Creating token batch: {:?}", req);
    
    // 获取Token服务
    let token_service = depot.obtain::<Arc<TokenService>>()
        .expect("TokenService not found in depot");
    
    // 获取认证用户信息
    let claims = match depot.get::<Claims>("claims") {
        Ok(claims) => claims,
        Err(_) => {
            error!("Failed to get claims from depot");
            return Err(res_json_custom(403, "认证失败"));
        }
    };
    
    // 验证用户是债权人或管理员
    if !claims.is_admin() && !claims.is_creditor() {
        return Err(res_json_custom(403, "只有债权人或管理员可以创建代币批次"));
    }

    match token_service.create_token_batch(req.into_inner()).await {
        Ok(batch_id) => {
            let response = TokenBatchIdResponse { batch_id };
            Ok(res_json_ok(Some(response)))
        },
        Err(e) => {
            error!("Failed to create token batch: {}", e);
            Err(res_json_err(&format!("创建代币批次失败: {}", e)))
        }
    }
}

/// 查询代币批次列表（可选过滤条件）
#[salvo::oapi::endpoint(
    tags("代币管理"),
    status_codes(200, 500),
    parameters(
        ("status" = Option<String>, Query, description = "代币批次状态过滤"),
        ("creditor_id" = Option<String>, Query, description = "债权人ID过滤"),
        ("stablecoin_symbol" = Option<String>, Query, description = "稳定币符号过滤"),
        ("page" = Option<i64>, Query, description = "页码"),
        ("page_size" = Option<i64>, Query, description = "每页数量")
    ),
    responses(
        (status_code = 200, description = "代币批次列表", body = Vec<TokenBatchResponse>),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_token_batches(
    status: QueryParam<Option<String>>,
    creditor_id: QueryParam<Option<String>>,
    stablecoin_symbol: QueryParam<Option<String>>,
    page: QueryParam<Option<i64>>,
    page_size: QueryParam<Option<i64>>,
    depot: &mut Depot
) -> Res<Vec<TokenBatchResponse>> {
    // 获取Token服务
    let token_service = depot.obtain::<Arc<TokenService>>()
        .expect("TokenService not found in depot");
    
    // 解析状态参数
    let status_enum = if let Some(status_str) = status.into_inner() {
        match status_str.as_str() {
            "Pending" => Some(TokenBatchStatus::Pending),
            "Available" => Some(TokenBatchStatus::Available),
            "Funding" => Some(TokenBatchStatus::Funding),
            "Funded" => Some(TokenBatchStatus::Funded),
            "Cancelled" => Some(TokenBatchStatus::Cancelled),
            "Completed" => Some(TokenBatchStatus::Completed),
            "Expired" => Some(TokenBatchStatus::Expired),
            _ => None,
        }
    } else {
        None
    };
    
    info!("Listing token batches with filters: status={:?}, creditor_id={:?}, stablecoin={:?}, page={:?}, page_size={:?}", 
        status_enum, creditor_id, stablecoin_symbol, page, page_size);
    
    match token_service.list_token_batches(
        status_enum,
        creditor_id.into_inner(),
        stablecoin_symbol.into_inner(),
        page.into_inner(),
        page_size.into_inner()
    ).await {
        Ok(token_batches) => {
            Ok(res_json_ok(Some(token_batches)))
        },
        Err(e) => {
            error!("Failed to list token batches: {}", e);
            Err(res_json_err(&format!("查询代币批次列表失败: {}", e)))
        }
    }
}

/// 查询代币市场列表（可选过滤条件）
#[salvo::oapi::endpoint(
    tags("代币管理"),
    status_codes(200, 500),
    parameters(
        ("stablecoin_symbol" = Option<String>, Query, description = "稳定币符号过滤"),
        ("page" = Option<i64>, Query, description = "页码"),
        ("page_size" = Option<i64>, Query, description = "每页数量")
    ),
    responses(
        (status_code = 200, description = "代币市场列表", body = Vec<TokenMarketResponse>),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_token_markets(
    stablecoin_symbol: QueryParam<Option<String>>,
    page: QueryParam<Option<i64>>,
    page_size: QueryParam<Option<i64>>,
    depot: &mut Depot
) -> Res<Vec<TokenMarketResponse>> {
    // 获取Token服务
    let token_service = depot.obtain::<Arc<TokenService>>()
        .expect("TokenService not found in depot");
    
    // 构建请求参数
    let request = QueryTokenMarketRequest {
        stablecoin_symbol: stablecoin_symbol.into_inner(),
        page: page.into_inner(),
        page_size: page_size.into_inner(),
    };
    
    info!("Listing token markets with query: {:?}", request);
    
    match token_service.list_token_markets(request).await {
        Ok(token_markets) => {
            Ok(res_json_ok(Some(token_markets)))
        },
        Err(e) => {
            error!("Failed to list token markets: {}", e);
            Err(res_json_err(&format!("查询代币市场列表失败: {}", e)))
        }
    }
}

/// 购买代币
#[salvo::oapi::endpoint(
    tags("代币管理"),
    status_codes(200, 400, 401, 500),
    request_body = PurchaseTokenRequest,
    responses(
        (status_code = 200, description = "代币购买成功", body = TokenHoldingIdResponse),
        (status_code = 400, description = "无效的请求参数或代币不可用"),
        (status_code = 401, description = "未认证用户"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn purchase_tokens(req: JsonBody<PurchaseTokenRequest>, depot: &mut Depot) -> Res<TokenHoldingIdResponse> {
    // 获取Token服务
    let token_service = depot.obtain::<Arc<TokenService>>()
        .expect("TokenService not found in depot");
    
    // 获取认证用户信息
    let user_address = match depot.get::<String>("user_address") {
        Ok(address) => address,
        Err(_) => {
            error!("User address not found in depot");
            return Err(res_json_custom(401, "用户未认证"));
        }
    };
    
    info!("Purchasing tokens: user={}, request={:?}", user_address, req);
    
    match token_service.purchase_tokens(user_address.to_string(), req.into_inner()).await {
        Ok(holding_id) => {
            let response = TokenHoldingIdResponse { holding_id };
            Ok(res_json_ok(Some(response)))
        },
        Err(e) => {
            error!("Failed to purchase tokens: {}", e);
            Err(res_json_err(&format!("购买代币失败: {}", e)))
        }
    }
}

/// 查询用户代币持有情况
#[salvo::oapi::endpoint(
    tags("代币管理"),
    status_codes(200, 401, 500),
    responses(
        (status_code = 200, description = "用户代币持有列表", body = Vec<TokenHoldingResponse>),
        (status_code = 401, description = "未认证用户"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn get_user_token_holdings(depot: &mut Depot) -> Res<Vec<TokenHoldingResponse>> {
    // 获取Token服务
    let token_service = depot.obtain::<Arc<TokenService>>()
        .expect("TokenService not found in depot");
    
    // 获取认证用户信息
    let user_id = match depot.get::<String>("user_id") {
        Ok(id) => id,
        Err(_) => {
            error!("User ID not found in depot");
            return Err(res_json_custom(401, "用户未认证"));
        }
    };
    
    let request = QueryUserTokenHoldingsRequest { 
        user_id: user_id.to_string() 
    };
    
    info!("Getting token holdings for user: {}", user_id);
    
    match token_service.get_user_token_holdings(request).await {
        Ok(holdings) => {
            Ok(res_json_ok(Some(holdings)))
        },
        Err(e) => {
            error!("Failed to get token holdings: {}", e);
            Err(res_json_err(&format!("查询代币持有列表失败: {}", e)))
        }
    }
} 