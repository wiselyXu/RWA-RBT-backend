use crate::utils::res::{Res, res_bad_request, res_json_err, res_json_ok, res_not_found};
use mongodb::{Database, bson::oid::ObjectId};
use salvo::{
    oapi::{ToSchema, extract::QueryParam},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use service::repository::DailyInterestAccrualRepository;
use common::domain::entity::DailyInterestAccrual;
use std::sync::Arc;
use log::{error, info};

/// 每日利息记录DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DailyInterestDto {
    pub id: String,
    pub user_id: String,
    pub invoice_id: String,
    pub holding_id: String,
    pub accrual_date: String,
    pub daily_interest_amount: String,
    pub calculated_at: String,
}

impl From<&DailyInterestAccrual> for DailyInterestDto {
    fn from(accrual: &DailyInterestAccrual) -> Self {
        Self {
            id: accrual.id.map_or("".to_string(), |id| id.to_string()),
            user_id: accrual.user_id.clone(),
            invoice_id: accrual.invoice_id.to_string(),
            holding_id: accrual.holding_id.clone(),
            accrual_date: accrual.accrual_date.to_string(),
            daily_interest_amount: accrual.daily_interest_amount.to_string(),
            calculated_at: accrual.calculated_at.to_string(),
        }
    }
}

/// 查询用户的所有日利息记录
#[salvo::oapi::endpoint(
    tags("利息"),
    status_codes(200, 401, 500),
    responses(
        (status_code = 200, description = "用户日利息记录列表", body = Vec<DailyInterestDto>),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_user_interest_accruals(depot: &mut Depot) -> Res<Vec<DailyInterestDto>> {
    // 获取认证用户地址
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("认证用户地址未找到: {:?}", e);
            return Err(res_json_err("用户未认证"));
        }
    };
    
    let mongodb = depot.obtain::<Arc<Database>>().expect("数据库连接未找到").clone();
    let repo = DailyInterestAccrualRepository::new(&mongodb);
    
    info!("查询用户 {} 的日利息记录", user_address);
    
    match repo.find_by_user_id(user_address).await {
        Ok(accruals) => {
            let dtos = accruals.iter().map(DailyInterestDto::from).collect();
            Ok(res_json_ok(Some(dtos)))
        }
        Err(e) => {
            error!("查询用户日利息记录失败: {}", e);
            Err(res_json_err("查询日利息记录失败"))
        }
    }
}

/// 查询特定持仓的日利息记录
#[salvo::oapi::endpoint(
    tags("利息"),
    status_codes(200, 400, 401, 500),
    parameters(
        ("holding_id" = String, Query, description = "持仓ID")
    ),
    responses(
        (status_code = 200, description = "持仓日利息记录列表", body = Vec<DailyInterestDto>),
        (status_code = 400, description = "无效的请求参数"),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_holding_interest_accruals(holding_id: QueryParam<String>, depot: &mut Depot) -> Res<Vec<DailyInterestDto>> {
    // 获取认证用户地址
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("认证用户地址未找到: {:?}", e);
            return Err(res_json_err("用户未认证"));
        }
    };
    
    let mongodb = depot.obtain::<Arc<Database>>().expect("数据库连接未找到").clone();
    let repo = DailyInterestAccrualRepository::new(&mongodb);
    
    let holding_id_str = holding_id.into_inner();
    
    info!("查询持仓 {} 的日利息记录", holding_id_str);
    
    match repo.find_by_user_id_and_holding_id(user_address, &holding_id_str).await {
        Ok(accruals) => {
            let dtos = accruals.iter().map(DailyInterestDto::from).collect();
            Ok(res_json_ok(Some(dtos)))
        }
        Err(e) => {
            error!("查询持仓日利息记录失败: {}", e);
            Err(res_json_err("查询日利息记录失败"))
        }
    }
} 