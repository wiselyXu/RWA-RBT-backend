
// server/crates/lib/service/src/entity/enterprise_dto
use mongodb::bson::{DateTime, oid::ObjectId};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enterprise {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub wallet_address: String, // Blockchain address for identification/signing
    pub status: EnterpriseStatus,
    pub kyc_details_ipfs_hash: Option<String>, // Link to KYC documents on IPFS
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize,ToSchema)]
pub enum EnterpriseStatus {
    PendingVerification,
    Verified,
    Suspended,
}

// Helper methods if needed
impl Enterprise {
    pub fn new(name: String, wallet_address: String) -> Self {
        let now = DateTime::now();
        Self {
            id: None,
            name,
            wallet_address,
            status: EnterpriseStatus::PendingVerification,
            kyc_details_ipfs_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize,ToSchema)]
pub struct EnterpriseDto {
    pub id: String,
    pub name: String,
    pub wallet_address: String, // Blockchain address for identification/signing
    pub status: EnterpriseStatus,
    pub kyc_details_ipfs_hash: Option<String>, // Link to KYC documents on IPFS
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl EnterpriseDto {
    pub fn from(data:Enterprise)-> EnterpriseDto {
        EnterpriseDto{
            id: data.id.unwrap().to_string(),
            name: data.name,
            wallet_address: data.wallet_address,
            status: data.status,
            kyc_details_ipfs_hash: data.kyc_details_ipfs_hash,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
    }
}