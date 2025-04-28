use crate::utils::res::{Res, res_bad_request, res_json_err, res_json_ok, res_not_found};
use mongodb::{Database, bson::oid::ObjectId};
use salvo::oapi::{ToSchema, extract::JsonBody, extract::PathParam, extract::QueryParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use service::repository::EnterpriseRepository;
use service::repository::enterprise_repository::UpdateEnterpriseData;
use std::sync::Arc;

use common::domain::entity::EnterpriseStatus;
use common::domain::entity::enterprise::EnterpriseDto;

// --- Request DTOs ---
#[derive(Deserialize, ToSchema, Debug)]
#[salvo(schema(example = json!({ "name": "Acme Corp", "walletAddress": "0x..."})))]
pub struct CreateEnterpriseRequest {
    pub name: String,
    #[serde(rename = "walletAddress")]
    pub wallet_address: String,
}

#[derive(Deserialize, ToSchema, Debug, Default)] // Default for partial updates
#[salvo(schema(example = json!({ "name": "Updated Name", "status": "Verified"})))]
pub struct UpdateEnterpriseRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "walletAddress", skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EnterpriseStatus>,
    #[serde(rename = "kycDetailsIpfsHash", skip_serializing_if = "Option::is_none")]
    pub kyc_details_ipfs_hash: Option<String>,
}

// --- Response DTO ---
// We can reuse the entity directly or create a specific response DTO
// For simplicity, reusing Enterprise entity here.
// Note: ObjectId and DateTime might not serialize to simple strings by default depending on features.
// Consider creating a dedicated EnterpriseResponse struct if needed.

// --- Handlers ---

/// 创建企业实体
#[salvo::oapi::endpoint(
    tags("企业"),
    status_codes(200, 400, 500),
    request_body = CreateEnterpriseRequest,
    responses(
        (status_code = 200, description = "Enterprise created successfully.", body = EnterpriseDto),
        (status_code = 400, description = "Invalid request data."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn create_enterprise(req: JsonBody<CreateEnterpriseRequest>, depot: &mut Depot) -> Res<EnterpriseDto> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = EnterpriseRepository::new(&mongodb);
    log::warn!("enterprise database connection established");
    match repo.create(&req.name, &req.wallet_address).await {
        Ok(enterprise) => {
            let data = EnterpriseDto::from(enterprise);
            Ok(res_json_ok(Some(data)))
        }

        Err(e) => {
            log::error!("Failed to create enterprise: {}", e);
            Err(res_json_err("Failed to create enterprise"))
        }
    }
}

/// 根据企业ID查询详情
#[salvo::oapi::endpoint(
    tags("企业"),
    status_codes(200, 400, 404, 500),
    parameters(
        ("id" = String, Query, description = "Enterprise MongoDB ObjectId")
    ),
    responses(
        (status_code = 200, description = "Enterprise found.", body = EnterpriseDto),
        (status_code = 400, description = "Invalid ID format or missing ID."),
        (status_code = 404, description = "Enterprise not found."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn get_enterprise_by_id(id: QueryParam<String>, depot: &mut Depot) -> Res<EnterpriseDto> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = EnterpriseRepository::new(&mongodb);

    let id_str = id.into_inner();

    log::warn!("get_enterprise_by_id, id: {}", &id_str);
    let oid = match ObjectId::parse_str(&id_str) {
        Ok(oid) => oid,
        Err(_) => return Err(res_bad_request("Invalid ObjectId format")),
    };

    match repo.find_by_id(oid).await {
        Ok(Some(enterprise)) => {
            let data = EnterpriseDto::from(enterprise);
            Ok(res_json_ok(Some(data)))
        }
        Ok(None) => Err(res_not_found("Enterprise not found")),
        Err(e) => {
            log::error!("Failed to get enterprise by ID: {}", e);
            Err(res_json_err("Failed to get enterprise"))
        }
    }
}

/// 查询所有企业
#[salvo::oapi::endpoint(
    tags("企业"),
    status_codes(200, 500),
    responses(
        (status_code = 200, description = "List of enterprises.", body = Vec<EnterpriseDto>),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn list_enterprises(depot: &mut Depot) -> Res<Vec<EnterpriseDto>> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = EnterpriseRepository::new(&mongodb);

    match repo.find_all().await {
        Ok(list) => {
            let data: Vec<EnterpriseDto> = list
                .into_iter()
                .map(EnterpriseDto::from) // 这里直接传递 From Trait 的实现
                .collect(); // 收集为 Vec<EnterpriseDto>

            // 返回转换后的 Vec<EnterpriseDto>
            Ok(res_json_ok(Some(data)))
        }
        Err(e) => {
            log::error!("Failed to list enterprises: {}", e);
            Err(res_json_err("Failed to list enterprises"))
        }
    }
}

/// 更新企业信息
#[salvo::oapi::endpoint(
    tags("企业"),
    status_codes(200, 400, 404, 500),
    parameters(
        ("id" = String, Path, description = "Enterprise MongoDB ObjectId")
    ),
    request_body = UpdateEnterpriseRequest,
    responses(
        (status_code = 200, description = "Enterprise updated successfully."),
        (status_code = 400, description = "Invalid ID format or request data."),
        (status_code = 404, description = "Enterprise not found."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn update_enterprise(id: PathParam<String>, req: JsonBody<UpdateEnterpriseRequest>, depot: &mut Depot) -> Res<()> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = EnterpriseRepository::new(&mongodb);

    let oid = match ObjectId::parse_str(&id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => return Err(res_bad_request("Invalid ObjectId format")),
    };

    // Map request DTO to repository update struct
    let update_data = UpdateEnterpriseData {
        name: req.name.clone(),
        wallet_address: req.wallet_address.clone(),
        status: req.status.clone(),
        kyc_details_ipfs_hash: req.kyc_details_ipfs_hash.clone(),
    };

    match repo.update(oid, update_data).await {
        Ok(update_result) => {
            if update_result.matched_count == 0 {
                Err(res_not_found("Enterprise not found"))
            } else {
                Ok(res_json_ok(None))
            }
        }
        Err(e) => {
            log::error!("Failed to update enterprise: {}", e);
            Err(res_json_err("Failed to update enterprise"))
        }
    }
}

/// 删除企业
#[salvo::oapi::endpoint(
    tags("企业"),
    status_codes(200, 400, 404, 500),
    parameters(
        ("id" = String, Query, description = "Enterprise MongoDB ObjectId")
    ),
    responses(
        (status_code = 200, description = "Enterprise deleted successfully."),
        (status_code = 400, description = "Invalid ID format."),
        (status_code = 404, description = "Enterprise not found."),
        (status_code = 500, description = "Internal server error."),
    )
)]
pub async fn delete_enterprise(id: QueryParam<String>, depot: &mut Depot) -> Res<()> {
    let mongodb = depot.obtain::<Arc<Database>>().expect("Database connection not found").clone();
    let repo = EnterpriseRepository::new(&mongodb);

    let oid = match ObjectId::parse_str(&id.into_inner()) {
        Ok(oid) => oid,
        Err(_) => return Err(res_bad_request("Invalid ObjectId format")),
    };

    match repo.delete(oid).await {
        Ok(delete_result) => {
            if delete_result.deleted_count == 0 {
                Err(res_not_found("Enterprise not found"))
            } else {
                Ok(res_json_ok(None))
            }
        }
        Err(e) => {
            log::error!("Failed to delete enterprise: {}", e);
            Err(res_json_err("Failed to delete enterprise"))
        }
    }
}
