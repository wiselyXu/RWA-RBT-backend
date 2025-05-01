use salvo::{async_trait, prelude::*};
use salvo::http::{header::AUTHORIZATION, Request, Response, StatusCode};
use salvo::jwt_auth::{AuthError, Extractible, JwtAuth};
use serde::{Deserialize, Serialize};

use crate::setting::SETTING;
use crate::utils::res::ResObj;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub address: String,
    pub exp: i64,
}

#[async_trait]
impl Extractible for Claims {
    type Error = AuthError;
    async fn extract(req: &mut Request) -> Result<Self, AuthError> {
        Err(AuthError::Unauthorized)
    }

    async fn extract_with_req(req: &mut Request) -> Option<Self> {
        let token = match req.headers().get(AUTHORIZATION) {
            Some(header) => {
                 match header.to_str() {
                     Ok(s) => s.trim_start_matches("Bearer ").trim(),
                     Err(_) => return None,
                 }
            },
            None => return None,
        };
        
        if token.is_empty() {
            return None;
        }

        let setting = SETTING.get().unwrap();
        JwtAuth::<Claims>::decode_token(token, &setting.jwt_secret)
    }
}

pub fn jwt_auth() -> JwtAuth<Claims> {
    let secret = match SETTING.get() {
        Some(s) => s.jwt_secret.clone(),
        None => panic!("JWT Settings not initialized!"),
    };
    JwtAuth::<Claims>::new(secret)
        .invalid_token_handler(|_req, _depot, ctrl, err| async move {
            ctrl.respond(Response::with_status_code(StatusCode::UNAUTHORIZED).with_json(ResObj::<()>::err(StatusCode::UNAUTHORIZED.as_u16(), &err.to_string())));
        })
        .token_expiring_handler(|_req, _depot, ctrl, _claims| async move {
             ctrl.skip_rest();
        })
} 