use salvo::{Depot, FlowCtrl, Request, Response, handler, prelude::StatusCode};
use crate::utils::res::res_json_custom;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use configs::CFG; // Assuming your JWT secret is in CFG
use crate::controller::Claims; // Import the Claims struct

#[handler]
pub async fn auth_token(req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl, depot: &mut Depot) {
    // Check if the request contains an Authorization header
    let auth_header = req.headers().get("Authorization").and_then(|v| v.to_str().ok());
    if let Some(auth_str) = auth_header {
        if auth_str.starts_with("Bearer ") {
            let token = &auth_str[7..]; // Remove "Bearer " prefix

            // Retrieve the secret key from configuration
            // Ensure CFG.jwt.secret is properly configured and accessible
            let secret = &CFG.jwt.secret;
            let decoding_key = DecodingKey::from_secret(secret.as_ref());
            // Use HS256 Algorithm
            let mut validation = Validation::new(Algorithm::HS256);
            // Optionally add leeway for clock skew
            // validation.leeway = 60; 
            // validation.validate_exp = true; // Default is true

            match decode::<Claims>(&token, &decoding_key, &validation) {
                Ok(token_data) => {
                    // Token is valid, extract the user_address (subject)
                    let user_address = token_data.claims.sub;
                    // Inject the user_address into the depot
                    depot.insert("user_address", user_address);
                    // Continue to the next handler
                    // ctrl.call_next(req, depot, res).await; // call_next is implicitly called if not skipped
                }
                Err(e) => {
                    log::error!("JWT validation failed: {}", e);
                    ctrl.skip_rest();
                    res.render(res_json_custom::<()>(401, "Invalid or expired token"));
                }
            }
        } else {
            // Header format is incorrect (not Bearer)
            ctrl.skip_rest();
            res.render(res_json_custom::<()>(401, "Invalid Authorization header format"));
        }
    } else {
        // Authorization header is missing
        ctrl.skip_rest();
        res.render(res_json_custom::<()>(401, "Authorization header missing"));
    }
}

#[handler]
pub async fn catcher_err(req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl) {
    // 记录请求基本信息
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let client_ip = req.remote_addr().to_string();

    // 仅处理错误状态码
    if let Some(status_code) = res.status_code {
        match status_code {
            StatusCode::NOT_FOUND => handle_not_found(req, res, ctrl).await,
            StatusCode::INTERNAL_SERVER_ERROR => handle_server_error(res, ctrl).await,
            _ => handle_other_errors(req, res, status_code, ctrl).await,
        }
    } else {
        // 记录未处理的成功请求（可选）
        log::warn!("请求成功但未记录 | 方法: {} | 路径: {} | 客户端IP: {}", method, path, client_ip);
    }
}

async fn handle_not_found(req: &Request, res: &mut Response, ctrl: &mut FlowCtrl) {
    ctrl.skip_rest();

    // 收集请求参数
    let params = req.params().iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(", ");

    if !req.uri().path().contains("actuator") {
        // 记录详细错误信息
        log::error!(
            "未找到接口 | 路径: {} | 方法: {} | 参数: {} | 客户端IP: {}",
            req.uri().path(),
            req.method(),
            params,
            req.remote_addr()
        );
    }
    res.render(res_json_custom::<()>(404, "请求的资源不存在"));
}

async fn handle_server_error(res: &mut Response, ctrl: &mut FlowCtrl) {
    ctrl.skip_rest();
    log::error!("服务器内部错误: {:?}", res.to_string());

    res.render(res_json_custom::<()>(500, "服务器内部错误，请稍后重试"));
}

async fn handle_other_errors(req: &Request, res: &mut Response, code: StatusCode, ctrl: &mut FlowCtrl) {
    ctrl.skip_rest();
    let status_code = code.as_u16();

    // 收集请求参数
    let params = req.params().iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join(", ");

    // 记录详细错误信息
    log::error!(
        "其他错误, 路径: {} | 方法: {} | 参数: {} | 客户端IP: {},错误码: {}",
        req.uri().path(),
        req.method(),
        params,
        req.remote_addr(),
        code
    );
    res.render(res_json_custom::<()>(status_code as i32, format!("请求处理失败 (状态码: {})", status_code).as_str()));
}
