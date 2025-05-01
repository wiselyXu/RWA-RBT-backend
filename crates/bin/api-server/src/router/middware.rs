use std::time::Instant;

use log::info;
use salvo::{Depot, FlowCtrl, Request, Response, handler};

use crate::utils::res::res_json_custom;
use salvo::{prelude::*, http::StatusCode};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use configs::CFG;
use log::{error, warn, debug};
#[handler]
pub async fn route_logger(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    // 记录开始时间
    let start = Instant::now();

    // 保存请求路径，因为在after_send中无法访问req
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    // 继续处理请求
    ctrl.call_next(req, depot, res).await;

    // 计算处理时间
    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    // 添加响应头
    res.headers_mut().insert("X-Response-Time", format!("{:.3}ms", duration_ms).parse().unwrap());

    // 输出日志
    log::warn!(
        target: "response_time",
        "{} {} - {:.3}ms",
        method,
        path,
        duration_ms
    );
}


// JWT声明结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // 用户地址
    pub exp: usize,      // 过期时间（Unix时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<usize>,      // 签发时间（可选）
}

