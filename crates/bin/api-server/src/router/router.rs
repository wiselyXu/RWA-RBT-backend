use salvo::Router;

use crate::controller::{common_controller, enterprise_controller, invoice_controller, user_controller, purchase_controller};


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
}

pub fn init_purchase_router() -> Router {
    Router::with_path("/rwa")
        .push(Router::with_path("/invoices/available").get(purchase_controller::list_available_invoices))
        .push(
            Router::new()
                .hoop(common_controller::auth_token)
                .push(Router::with_path("/invoices/purchase").post(purchase_controller::purchase_invoice))
                .push(Router::with_path("/holdings").get(purchase_controller::list_my_holdings))
            
        )
}