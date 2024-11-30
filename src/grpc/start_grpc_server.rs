use crate::database::vector_db::VectorDB;
use crate::{
    config::Config,
    health::{health::health_server::HealthServer, HealthService},
};

use super::matcher_service::matcher::matcher_server::MatcherServer;
use super::matcher_service::MatcherService;

use std::sync::Arc;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::{error, info};

pub async fn start_grpc_server(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50030".parse()?;
    info!("Connecting to VectorDB");

    // Simply connect to existing database without initialization
    let db = match VectorDB::new("data/mydb", None, false).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to connect to VectorDB: {}. Make sure to initialize the database first using --reload flag", e);
            return Err(e.into());
        }
    };

    let matcher_service = MatcherService {
        config,
        db: db.clone(),
    };
    let health_service = HealthService::new(db);

    // Get the file descriptor set
    let descriptor_set = include_bytes!(concat!(env!("OUT_DIR"), "/matcher_descriptor.bin"));

    // Build the reflection service
    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(descriptor_set)
        .build_v1()?;

    info!("Starting gRPC server on {}", addr);
    Server::builder()
        .add_service(MatcherServer::new(matcher_service))
        .add_service(HealthServer::new(health_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    info!("gRPC server has been shut down");
    Ok(())
}
