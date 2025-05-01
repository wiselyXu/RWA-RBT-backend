use crate::{
    controller::{common_controller, swagger_controller},
    router::middware::route_logger,
};

use configs::{cfgs::Redis as RedisConfig, CFG};
use mongodb::Database; // Changed from sea_orm::DatabaseConnection
use redis::Client as RedisClient;
use salvo::Handler;
use salvo::cors::Cors;
use salvo::http::Method;
use salvo::{
    Router,
    Service,
    async_trait,
    catcher::Catcher,
    handler, // Import async_trait and handler macros
    logging::Logger,
    oapi::{OpenApi, Operation, swagger_ui::SwaggerUi},
    prelude::{CatchPanic, Depot, FlowCtrl, Request, Response, SessionHandler},
    serve_static::StaticDir,
    session::CookieStore,
};
use service::invoice::InvoiceService; // Import InvoiceService
use std::{env, sync::Arc};
use pharos_interact::{InvoiceContract, ContractQuerier, ContractWriter}; // Import for contract interaction
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;

pub mod middware;
pub mod router;
use router::{init_user_router, init_enterprise_router, init_invoice_router}; // Import all routers

// --- Injection Middleware Struct ---
#[derive(Clone)] // Clone is needed for the handler
struct InjectConnections {
    mongodb: Arc<Database>, // Changed from db_conn: Arc<DatabaseConnection>
    redis_client: Arc<RedisClient>,
    contract: Option<Arc<InvoiceContract<SignerMiddleware<Provider<Http>, LocalWallet>>>>, // Contract connection
    invoice_service: Arc<InvoiceService>, // Add InvoiceService
}

#[async_trait]
impl Handler for InjectConnections {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        depot.inject(self.mongodb.clone()); // Updated
        depot.inject(self.redis_client.clone());
        depot.inject(self.invoice_service.clone()); // Inject InvoiceService
        
        // Inject contract connection if available
        if let Some(contract) = &self.contract {
            depot.inject(contract.clone());
        }
        
        // Indicate that the next handler should be called
        ctrl.call_next(req, depot, res).await;
    }
}

// init_router remains mostly the same, but doesn't add inject_connections middleware here
pub fn init_router() -> Router {
    let current_dir = env::current_dir().unwrap();
    log::warn!("Current working directory: {:?}", current_dir);
    let static_router = Router::with_path("/<**path>").get(StaticDir::new(current_dir.join("/static")).defaults("index.html").auto_list(true));

    // Base router without connection injection yet
    let router = Router::new().hoop(Logger::new()).hoop(CatchPanic::new()).push(static_router);

    // Business routes under /rwa prefix
    let api_router = Router::with_path(&CFG.server.api_prefix) // Use configured prefix
        .push(init_user_router()) // Existing user/auth routes
        .push(init_enterprise_router()) // Add enterprise routes
        .push(init_invoice_router()); // Keep non-RWA invoice routes if needed


    let router = router.push(api_router);

    // Swagger UI and docs setup
    let session_handler = SessionHandler::builder(CookieStore::new(), b"salvo-adminsalvo-adminalvo-adminsalvo-admin2023salvo-admin2023salvo-admin2023")
        .build()
        .unwrap();

    // OpenAPI Documentation
    let doc = OpenApi::new("Pharos-RWA", "0.1.1").merge_router(&router);

    let router = router.push(
        Router::new()
            .hoop(session_handler)
            .push(
                Router::new()
                    .hoop(swagger_controller::auth_token)
                    .push(doc.into_router("/api-doc/openapi.json"))
                    .push(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui")),
            )
            .push(Router::with_path("/swaggerLogin").post(swagger_controller::swagger_login)),
    );
    router
}

// Modify init_service to create and inject InvoiceService
pub fn init_service(
    mongodb: Arc<Database>, 
    redis_client: Arc<RedisClient>,
    contract: Option<Arc<InvoiceContract<SignerMiddleware<Provider<Http>, LocalWallet>>>>,
) -> Service {
    let router = init_router();

    // Create InvoiceService instance here
    // Note: InvoiceService::new expects Database, not Arc<Database>. Need to adjust service or clone DB?
    // Assuming InvoiceService::new can take Arc<Database> or we adjust it later.
    // Also InvoiceService::new expects RedisClient, not Arc<RedisClient>.
    let invoice_service = Arc::new(InvoiceService::new((*mongodb).clone(), (*redis_client).clone()));

    // Setup scheduled tasks (ensure this runs only once)
    // Might be better to do this in main.rs after service creation
    // service::invoice::setup_scheduled_tasks(invoice_service.clone()); 

    // Create the injector instance
    let injector = InjectConnections {
        mongodb, 
        redis_client,
        contract,
        invoice_service, // Inject the created service
    };

    // Apply CORS, then injection, then catcher, then router
    Service::new(router)
        .hoop(injector) // Use the injector instance
        .catcher(Catcher::default().hoop(common_controller::catcher_err))
}
