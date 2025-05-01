use anyhow::{Result, Context, anyhow};
use mongodb::{Database, bson::{self, Decimal128, doc}};
use std::sync::Arc;
use log::{info, error, warn};
use common::domain::dto::invoice_dto::InvoiceRedisDto;
use common::domain::dto::purchase_invoice_dto::PurchaseInvoiceDto;
use common::domain::entity::{UserInvoiceHolding, Transaction, TransactionType};
use crate::repository::{UserRepository, InvoiceRepository, UserInvoiceHoldingRepository, TransactionRepository};
use crate::cache::InvoiceRedisService;

pub struct PurchaseService {
    db: Arc<Database>,
    redis_service: Arc<InvoiceRedisService>,
    user_repo: UserRepository,
    invoice_repo: InvoiceRepository,
    holding_repo: UserInvoiceHoldingRepository,
    transaction_repo: TransactionRepository,
}

impl PurchaseService {
    pub fn new(db: Arc<Database>, redis_service: Arc<InvoiceRedisService>) -> Self {
        Self {
            user_repo: UserRepository::new(&db),
            invoice_repo: InvoiceRepository::new(&db),
            holding_repo: UserInvoiceHoldingRepository::new(&db),
            transaction_repo: TransactionRepository::new(&db),
            db,
            redis_service,
        }
    }
    
    /// 用户购买票据
    pub async fn purchase_invoice(&self, user_address: &str, purchase_data: PurchaseInvoiceDto) -> Result<UserInvoiceHolding> {
        info!("Processing invoice purchase for user: {}, invoice: {}", user_address, purchase_data.invoice_id);
        
        // 1. 检查用户是否存在
        let user = self.user_repo.find_by_wallet_address(user_address).await?
            .ok_or_else(|| anyhow!("用户不存在"))?;
            
        // 2. 从Redis获取票据信息
        let invoice_redis = self.redis_service.get_invoice(&purchase_data.invoice_id).await?
            .ok_or_else(|| anyhow!("票据不存在或已下架"))?;
            
        // 3. 验证票据是否可购买
        if !invoice_redis.is_available_for_purchase() {
            return Err(anyhow!("票据不可购买或已售罄"));
        }
        
        // 4. 验证购买份数
        if purchase_data.shares == 0 || purchase_data.shares > invoice_redis.available_shares {
            return Err(anyhow!("购买份数无效，超出可用份数"));
        }
        
        // 5. 计算购买金额
        let purchase_amount = (purchase_data.shares as f64 * invoice_redis.share_price).to_string();
        let purchase_decimal = Decimal128::from_str_exact(&purchase_amount)?;
        
        // 6. 查找数据库中的票据记录
        let invoice_mongo = self.invoice_repo.find_by_number(&invoice_redis.invoice_number).await?
            .ok_or_else(|| anyhow!("数据库中不存在该票据记录"))?;
            
        // 7. 创建持仓记录
        let holding = UserInvoiceHolding::new(
            user_address.to_string(),
            invoice_mongo.id.unwrap(),
            purchase_decimal.clone(),
        );
        
        let created_holding = self.holding_repo.create(holding).await?;
        
        // 8. 创建购买交易记录
        let transaction = Transaction::new_purchase(
            user_address.to_string(),
            invoice_mongo.id.unwrap(),
            created_holding.holding_id.clone(),
            purchase_decimal,
        );
        
        self.transaction_repo.create(transaction).await?;
        
        // 9. 更新Redis中的票据可用份数
        self.redis_service.update_invoice_shares(&purchase_data.invoice_id, purchase_data.shares).await?;
        
        info!("Successfully created holding {} for user {}", created_holding.holding_id, user_address);
        Ok(created_holding)
    }
    
    /// 获取用户的所有票据持仓
    pub async fn get_user_holdings(&self, user_address: &str) -> Result<Vec<UserInvoiceHolding>> {
        info!("Fetching holdings for user: {}", user_address);
        
        let holdings = self.holding_repo.find_by_user_id(user_address).await?;
        
        info!("Found {} holdings for user {}", holdings.len(), user_address);
        Ok(holdings)
    }
    
    /// 获取可购买的票据列表
    pub async fn get_available_invoices(&self) -> Result<Vec<InvoiceRedisDto>> {
        info!("Fetching available invoices for purchase");
        
        let invoices = self.redis_service.get_available_invoices().await?;
        
        info!("Found {} available invoices", invoices.len());
        Ok(invoices)
    }
}
