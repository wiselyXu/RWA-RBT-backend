#![allow(warnings)]
mod router;
mod controller;
mod utils;

use common::config::logger;
use configs::CFG;
use log::{info, error};
use salvo::cors::Cors;
use salvo::prelude::*;
use salvo::oapi::OpenApi;
use service::{db::init_mongodb};

use std::sync::Arc;
use pharos_interact::initialize_contract_from_env;
use anyhow::Context;
use service::cache::init_redis_client;

#[tokio::main]
async fn main() {
    // Initialize logging
    log4rs::init_file("config/log4rs.yaml", Default::default()).context("Failed to initialize log4rs").expect("Failed to initialize log4rs");

    let db_config = CFG.database.clone();
    let redis_config = CFG.redis.clone();
    let server_config = CFG.server.clone();
    info!("Configuration loaded successfully.");

    // Initialize MongoDB connection (async)
    let mongodb = match init_mongodb(&db_config).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to initialize MongoDB connection: {}", e);
            // Depending on requirements, panic or exit gracefully
            panic!("MongoDB connection failed!");
        }
    };

    // Initialize Redis Client (sync)
    let redis_client = match init_redis_client(&redis_config) {
        Ok(client) => Arc::new(client),
        Err(e) => {
            error!("Failed to initialize Redis client: {}", e);
             // Depending on requirements, panic or exit gracefully
            panic!("Redis connection failed!");
        }
    };

    // Initialize blockchain contract connection (async)
    let contract = match initialize_contract_from_env().await {
        Ok(contract) => {
            info!("Blockchain contract connection initialized successfully");
            Some(Arc::new(contract))
        },
        Err(e) => {
            error!("Failed to initialize blockchain contract connection: {}", e);
            // Don't panic, just continue without contract capability
            None
        }
    };

    info!("Starting Pharos API server");



    // Initialize services and create the main router
    let service = router::init_service(mongodb, redis_client, contract) ;// Returns Router


    // Setup server address
    let address = format!("{}:{}", server_config.ip, server_config.port);
    let listener = TcpListener::new(&address).bind().await;

    info!("Server listening on http://{:?}", address);
    // info!("Swagger UI available at http://{:?}/swagger-ui", address);

    // Start Server with the configured router
    Server::new(listener).serve(service).await;
}
