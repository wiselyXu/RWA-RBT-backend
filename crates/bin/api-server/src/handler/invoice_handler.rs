use std::sync::Arc;
use salvo::{Request, Response, Depot, handler};
use salvo::http::StatusCode;
use salvo::oapi::extract::{JsonBody, PathParam};
use salvo_oapi::extract::QueryParam;
use service::invoice::InvoiceService;
use common::domain::dto::{PurchaseInvoiceDto, InvoiceRedisDto, HoldingDto, InterestDetailDto};
use crate::response::{res_json, res_json_custom, res_json_data};

pub struct InvoiceHandler {
    invoice_service: Arc<InvoiceService>,
}

impl InvoiceHandler {
    pub fn new(invoice_service: Arc<InvoiceService>) -> Self {
        Self { invoice_service }
    }
    
    // 获取可购买的票据列表
    #[handler]
    pub async fn get_available_invoices(&self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self.invoice_service.get_available_invoices().await {
            Ok(invoices) => {
                res.render(res_json_data(invoices));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(res_json_custom::<()>(500, &format!("获取票据列表失败: {}", e)));
            }
        }
    }
    
    // 购买票据
    #[handler]
    pub async fn purchase_invoice(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 从depot中获取用户ID（假设认证中间件已经注入）
        let user_id = match depot.get::<String>("user_address") {
            Some(user_id) => user_id.to_string(),
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(res_json_custom::<()>(401, "未授权访问"));
                return;
            }
        };
        
        // 获取购买请求参数
        let purchase_req = match req.parse_json::<PurchaseInvoiceDto>().await {
            Ok(req) => req,
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(res_json_custom::<()>(400, &format!("无效的请求参数: {}", e)));
                return;
            }
        };
        
        // 执行购买
        match self.invoice_service.purchase_invoice(&user_id, purchase_req).await {
            Ok(holding_id) => {
                res.render(res_json_data(holding_id));
            },
            Err(e) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(res_json_custom::<()>(400, &format!("购买票据失败: {}", e)));
            }
        }
    }
    
    // 获取用户持仓列表
    #[handler]
    pub async fn get_user_holdings(&self, _req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 从depot中获取用户ID
        let user_id = match depot.get::<String>("user_address") {
            Some(user_id) => user_id.to_string(),
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(res_json_custom::<()>(401, "未授权访问"));
                return;
            }
        };
        
        match self.invoice_service.get_user_holdings(&user_id).await {
            Ok(holdings) => {
                res.render(res_json_data(holdings));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(res_json_custom::<()>(500, &format!("获取持仓列表失败: {}", e)));
            }
        }
    }
    
    // 获取持仓利息明细
    #[handler]
    pub async fn get_holding_interest_details(&self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 从depot中获取用户ID
        let user_id = match depot.get::<String>("user_address") {
            Some(user_id) => user_id.to_string(),
            None => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(res_json_custom::<()>(401, "未授权访问"));
                return;
            }
        };
        
        // 获取持仓ID
        let holding_id = match req.param::<String>("holdingId") {
            Some(id) => id,
            None => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(res_json_custom::<()>(400, "缺少持仓ID参数"));
                return;
            }
        };
        
        match self.invoice_service.get_holding_interest_details(&user_id, &holding_id).await {
            Ok(details) => {
                res.render(res_json_data(details));
            },
            Err(e) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(res_json_custom::<()>(500, &format!("获取利息明细失败: {}", e)));
            }
        }
    }
}
