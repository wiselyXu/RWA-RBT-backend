use salvo::Router;

use crate::controller::{
    common_controller, enterprise_controller, interest_controller, invoice_controller, purchase_controller, token_controller, transaction_controller, user_controller,
};

pub fn init_user_router() -> Router {
    let router = Router::with_path("/user");
    // 账户相关路由
    router
        // Web3 认证相关路由
        .push(Router::with_path("/challenge").post(user_controller::challenge))
        .push(Router::with_path("/login").post(user_controller::login))
        // 绑定企业路由 (需要认证)
        .push(
            Router::with_path("/bind-enterprise")
                .hoop(common_controller::auth_token)
                .post(user_controller::bind_enterprise),
        )
        // 获取用户绑定的企业信息路由 (需要认证)
        .push(
            Router::with_path("/enterprise-info")
                .hoop(common_controller::auth_token)
                .get(user_controller::get_enterprise_info),
        )
}

pub fn init_enterprise_router() -> Router {
    // Base path for enterprise routes
    Router::with_path("/enterprise")
        .push(Router::with_path("/list").get(enterprise_controller::list_enterprises))
        .push(Router::with_path("/detail").get(enterprise_controller::get_enterprise_by_id))
        .push(Router::with_path("/del").delete(enterprise_controller::delete_enterprise))
        .push(
            Router::with_path("/create")
                // .hoop(common_controller::auth_token)
                .post(enterprise_controller::create_enterprise),
        )
}

pub fn init_invoice_router() -> Router {
    // 创建公共路由（无需认证）
    let public_routes = Router::new()
        .push(Router::with_path("/list").get(invoice_controller::list_invoices))
        .push(Router::with_path("/detail").get(invoice_controller::query_invoice_data));
    
    // 创建需要认证的路由
    let auth_routes = Router::new()
        .hoop(common_controller::auth_token) // 复用认证中间件
        .push(Router::with_path("/del").delete(invoice_controller::delete_invoice))
        .push(Router::with_path("/create").post(invoice_controller::create_invoice))
        .push(Router::with_path("/holding/interest-details").get(invoice_controller::get_holding_interest_details))
        .push(Router::with_path("/verify").post(invoice_controller::verify_invoice))
        .push(Router::with_path("/issue").post(invoice_controller::issue_invoices))
        .push(Router::with_path("/batches").get(invoice_controller::list_user_invoice_batches))
        .push(Router::with_path("/batch/:id").get(invoice_controller::get_invoice_batch_by_id))
        .push(Router::with_path("/batch/:id/invoices").get(invoice_controller::get_invoices_by_batch_id));
    
    // 合并路由
    Router::with_path("/invoice")
        .push(public_routes)
        .push(auth_routes)
}

pub fn init_purchase_router() -> Router {
    Router::with_path("/purchase")
        .push(Router::with_path("/available").get(purchase_controller::list_available_invoices))
        .push(
            Router::new()
                .hoop(common_controller::auth_token)
                .push(Router::with_path("/purchase").post(purchase_controller::purchase_invoice))
                .push(Router::with_path("/holdings").get(purchase_controller::list_my_holdings)),
        )
}

pub fn init_admin_router() -> Router {
    Router::with_path("/admin")
        // TODO: Add proper admin authentication middleware
        .hoop(common_controller::auth_token) // Temporarily reuse standard auth, should be replaced with admin-specific auth
        .push(Router::with_path("/calc-interest").get(invoice_controller::trigger_daily_interest_calculation))
        .push(Router::with_path("/process-maturity").get(invoice_controller::trigger_maturity_payments))
}

// 新增交易相关路由
pub fn init_transaction_router() -> Router {
    Router::with_path("/transaction")
        .hoop(common_controller::auth_token) // 所有交易查询接口都需要认证
        .push(Router::with_path("/list").get(transaction_controller::list_user_transactions))
        .push(Router::with_path("/by-holding").get(transaction_controller::list_holding_transactions))
        .push(Router::with_path("/by-type").get(transaction_controller::list_transactions_by_type))
}

// 新增利息相关路由
pub fn init_interest_router() -> Router {
    Router::with_path("/interest")
        .hoop(common_controller::auth_token) // 所有利息查询接口都需要认证
        .push(Router::with_path("/list").get(interest_controller::list_user_interest_accruals))
        .push(Router::with_path("/by-holding").get(interest_controller::list_holding_interest_accruals))
}

// 新增 Token 相关路由
pub fn init_token_router() -> Router {
    Router::with_path("/token")
        .push(Router::with_path("/markets").get(token_controller::list_token_markets))
        .push(Router::with_path("/batches").get(token_controller::list_token_batches))
        .push(
            Router::new()
                .hoop(common_controller::auth_token) // Token routes requiring authentication
                .push(Router::with_path("/create").post(token_controller::create_token_batch))
                .push(Router::with_path("/purchase").post(token_controller::purchase_tokens))
                .push(Router::with_path("/holdings").get(token_controller::get_user_token_holdings))
                .push(Router::with_path("/from_invoice_batch").post(token_controller::create_token_batch_from_invoice_batch))
        )
}
