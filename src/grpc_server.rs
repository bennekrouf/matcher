
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::info;
use crate::process_search_results;
use crate::config::Config;
use crate::db::VectorDB;

// Import the generated proto code
pub mod matcher {
    tonic::include_proto!("matcher");
}

pub struct MatcherService {
    config: Arc<Config>,
    db: Arc<VectorDB>,
}

#[tonic::async_trait]
impl matcher::matcher_server::Matcher for MatcherService {
    async fn match_query(
        &self,
        request: Request<matcher::MatchRequest>,
    ) -> Result<Response<matcher::MatchResponse>, Status> {
        let req = request.into_inner();

        // Search for similar endpoints
        let results = self.db.search_similar(&req.query, &req.language, if req.show_all_matches { 5 } else { 1 })
            .await
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        // Convert search results to response format
        let matches: Vec<matcher::EndpointMatch> = results
            .iter()
            .map(|result| matcher::EndpointMatch {
                endpoint_id: result.endpoint_id.clone(),
                similarity: result.similarity as f64,
                parameters: result.parameters.clone(),
            })
            .collect();

        // Process matches through Iggy if needed
        if !matches.is_empty() {
            if let Err(e) = process_search_results(results).await {
                eprintln!("Failed to process results through Iggy: {}", e);
                // Don't return error, continue with response
            }
        }

        Ok(Response::new(matcher::MatchResponse { matches }))
    }
}

pub async fn start_grpc_server(
    config: Arc<Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50030".parse()?;

    // Initialize VectorDB
    let db = Arc::new(
        VectorDB::new("data/mydb", Some((*config).clone()), false)
            .await
            .expect("Failed to initialize VectorDB")
    );

    let matcher_service = MatcherService {
        config,
        db,
    };

    // Get the file descriptor set
    let descriptor_set = include_bytes!(concat!(env!("OUT_DIR"), "/matcher_descriptor.bin"));

    // Build the reflection service
    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(descriptor_set)
        .build_v1()?;

    info!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(matcher::matcher_server::MatcherServer::new(matcher_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    info!("gRPC server has been shut down");
    Ok(())
}

