use crate::utils::res::{Res, res_bad_request, res_json_err, res_json_ok, res_not_found};
use mongodb::{Database, bson::oid::ObjectId};
use salvo::{
    oapi::{ToSchema, extract::QueryParam},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use service::repository::TransactionRepository;
use common::domain::entity::Transaction;
use std::sync::Arc;
use log::{error, info};

/// 交易明细DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransactionDto {
    pub id: String,
    pub user_id: String,
    pub invoice_id: String,
    pub holding_id: String,
    pub transaction_type: String,
    pub amount: String,
    pub transaction_date: String,
    pub status: String,
}

impl From<&Transaction> for TransactionDto {
    fn from(tx: &Transaction) -> Self {
        Self {
            id: tx.id.map_or("".to_string(), |id| id.to_string()),
            user_id: tx.user_id.clone(),
            invoice_id: tx.invoice_id.to_string(),
            holding_id: tx.holding_id.clone(),
            transaction_type: format!("{:?}", tx.transaction_type),
            amount: tx.amount.to_string(),
            transaction_date: tx.transaction_date.to_string(),
            status: tx.status.clone(),
        }
    }
}

/// 查询用户的所有交易记录
#[salvo::oapi::endpoint(
    tags("交易"),
    status_codes(200, 401, 500),
    responses(
        (status_code = 200, description = "用户交易记录列表", body = Vec<TransactionDto>),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_user_transactions(depot: &mut Depot) -> Res<Vec<TransactionDto>> {
    // 获取认证用户地址
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("认证用户地址未找到: {:?}", e);
            return Err(res_json_err("用户未认证"));
        }
    };
    
    let mongodb = depot.obtain::<Arc<Database>>().expect("数据库连接未找到").clone();
    let repo = TransactionRepository::new(&mongodb);
    
    info!("查询用户 {} 的交易记录", user_address);
    
    match repo.find_by_user_id(user_address).await {
        Ok(transactions) => {
            let dtos = transactions.iter().map(TransactionDto::from).collect();
            Ok(res_json_ok(Some(dtos)))
        }
        Err(e) => {
            error!("查询用户交易记录失败: {}", e);
            Err(res_json_err("查询交易记录失败"))
        }
    }
}

/// 查询特定持仓的交易记录
#[salvo::oapi::endpoint(
    tags("交易"),
    status_codes(200, 400, 401, 500),
    parameters(
        ("holding_id" = String, Query, description = "持仓ID")
    ),
    responses(
        (status_code = 200, description = "持仓交易记录列表", body = Vec<TransactionDto>),
        (status_code = 400, description = "无效的请求参数"),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_holding_transactions(holding_id: QueryParam<String>, depot: &mut Depot) -> Res<Vec<TransactionDto>> {
    // 获取认证用户地址
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("认证用户地址未找到: {:?}", e);
            return Err(res_json_err("用户未认证"));
        }
    };
    
    let mongodb = depot.obtain::<Arc<Database>>().expect("数据库连接未找到").clone();
    let repo = TransactionRepository::new(&mongodb);
    
    let holding_id_str = holding_id.into_inner();
    
    info!("查询持仓 {} 的交易记录", holding_id_str);
    
    match repo.find_by_holding_id(&holding_id_str).await {
        Ok(transactions) => {
            // 过滤只属于当前用户的交易
            let filtered_transactions: Vec<_> = transactions
                .into_iter()
                .filter(|tx| tx.user_id == user_address)
                .collect();
            
            let dtos = filtered_transactions.iter().map(TransactionDto::from).collect();
            Ok(res_json_ok(Some(dtos)))
        }
        Err(e) => {
            error!("查询持仓交易记录失败: {}", e);
            Err(res_json_err("查询交易记录失败"))
        }
    }
}

/// 按交易类型查询用户交易记录
#[salvo::oapi::endpoint(
    tags("交易"),
    status_codes(200, 400, 401, 500),
    parameters(
        ("transaction_type" = String, Query, description = "交易类型(Purchase/InterestAccrual/MaturityPayment/Withdrawal)")
    ),
    responses(
        (status_code = 200, description = "按类型筛选的交易记录列表", body = Vec<TransactionDto>),
        (status_code = 400, description = "无效的请求参数"),
        (status_code = 401, description = "未认证"),
        (status_code = 500, description = "服务器内部错误"),
    )
)]
pub async fn list_transactions_by_type(transaction_type: QueryParam<String>, depot: &mut Depot) -> Res<Vec<TransactionDto>> {
    // 获取认证用户地址
    let user_address = match depot.get::<String>("user_address") {
        Ok(address_ref) => address_ref.as_str(),
        Err(e) => {
            error!("认证用户地址未找到: {:?}", e);
            return Err(res_json_err("用户未认证"));
        }
    };
    
    let mongodb = depot.obtain::<Arc<Database>>().expect("数据库连接未找到").clone();
    let repo = TransactionRepository::new(&mongodb);
    
    let tx_type = transaction_type.into_inner();
    
    // 验证交易类型
    if !["Purchase", "InterestAccrual", "MaturityPayment", "Withdrawal"].contains(&tx_type.as_str()) {
        return Err(res_bad_request("无效的交易类型"));
    }
    
    info!("查询用户 {} 的 {} 类型交易记录", user_address, tx_type);
    
    match repo.find_by_user_id_and_type(user_address, &tx_type).await {
        Ok(transactions) => {
            let dtos = transactions.iter().map(TransactionDto::from).collect();
            Ok(res_json_ok(Some(dtos)))
        }
        Err(e) => {
            error!("查询交易记录失败: {}", e);
            Err(res_json_err("查询交易记录失败"))
        }
    }
} 