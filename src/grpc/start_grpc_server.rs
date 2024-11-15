use super::MatcherService;
use crate::config::Config;
//use crate::matcher_service::matcher::matcher_server::MatcherServer;
use crate::matcher_service::MatcherEngine;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;

use super::matcher::matcher_server::MatcherServer;
pub mod matcher {
    tonic::include_proto!("matcher");
}
//
pub async fn start_grpc_server(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50030".parse()?;

    let engine = Arc::new(MatcherEngine::new(config).await?);
    let matcher_service = MatcherService::new(engine);

    let descriptor_set = include_bytes!(concat!(env!("OUT_DIR"), "/matcher_descriptor.bin"));
    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(descriptor_set)
        .build_v1()?;

    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(MatcherServer::new(matcher_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    info!("gRPC server has been shut down");
    Ok(())
}
