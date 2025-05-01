use salvo::Router;

use crate::controller::{common_controller, enterprise_controller, invoice_controller, user_controller, purchase_controller, transaction_controller, interest_controller};


pub fn init_user_router() -> Router {
    let router = Router::with_path("/user");
    // 账户相关路由
    router
        // Web3 认证相关路由
        .push(Router::with_path("/challenge").post(user_controller::challenge))
        .push(Router::with_path("/login").post(user_controller::login))
        // 绑定企业路由 (需要认证)
        .hoop(common_controller::auth_token)
        .push(
            Router::with_path("/bind-enterprise")
                .hoop(common_controller::auth_token)
                .post(user_controller::bind_enterprise),
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
                .hoop(common_controller::auth_token)
                .post(enterprise_controller::create_enterprise),
        )
}

pub fn init_invoice_router() -> Router {
    Router::with_path("/invoice")
        .push(Router::with_path("/list").get(invoice_controller::list_invoices))
        .push(Router::with_path("/detail").get(invoice_controller::query_invoice_data))
        .push(Router::with_path("/del").hoop(common_controller::auth_token).delete(invoice_controller::delete_invoice))
        .push(Router::with_path("/create").hoop(common_controller::auth_token).post(invoice_controller::create_invoice))
        .push(
            Router::with_path("/holding/interest-details")
                .hoop(common_controller::auth_token)
                .get(invoice_controller::get_holding_interest_details)
        )
}

pub fn init_purchase_router() -> Router {
    Router::with_path("/purchase")
        .push(Router::with_path("/available").get(purchase_controller::list_available_invoices))
        .push(
            Router::new()
                .hoop(common_controller::auth_token)
                .push(Router::with_path("/purchase").post(purchase_controller::purchase_invoice))
                .push(Router::with_path("/holdings").get(purchase_controller::list_my_holdings))
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