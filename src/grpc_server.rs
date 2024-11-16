use crate::config::Config;
use crate::database::VectorDB;
use crate::process_search_results;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::{debug, error, info, warn};

pub mod matcher {
    tonic::include_proto!("matcher");
}

pub struct MatcherService {
    #[allow(dead_code)]
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
        info!(
            "Received match request - query: {}, language: {}, show_all_matches: {}",
            req.query, req.language, req.show_all_matches
        );

        let results = match self
            .db
            .search_similar(
                &req.query,
                &req.language,
                if req.show_all_matches { 5 } else { 1 },
            )
            .await
        {
            Ok(results) => {
                if results.is_empty() {
                    warn!("No matches found for query: {}", req.query);
                } else {
                    debug!("Found {} matches", results.len());
                    for (i, result) in results.iter().enumerate() {
                        debug!(
                            "Match {}: id={}, similarity={:.4}",
                            i + 1,
                            result.endpoint_id,
                            result.similarity
                        );
                    }
                }
                results
            }
            Err(e) => {
                error!("Search failed: {}", e);
                return Err(Status::internal(format!("Search failed: {}", e)));
            }
        };

        let matches: Vec<matcher::EndpointMatch> = results
            .iter()
            .map(|result| {
                let match_data = matcher::EndpointMatch {
                    endpoint_id: result.endpoint_id.clone(),
                    similarity: result.similarity as f64,
                    parameters: result.parameters.clone(),
                };
                debug!(
                    "Created match response: id={}, similarity={:.4}",
                    match_data.endpoint_id, match_data.similarity
                );
                match_data
            })
            .collect();

        // Process only the best match through Iggy if available
        if let Some(best_result) = results.first() {
            info!("Processing best match through Iggy");
            if let Err(e) = process_search_results(vec![best_result.clone()]).await {
                error!("Failed to process best result through Iggy: {}", e);
            }
        }

        info!("Returning response with {} matches", matches.len());
        Ok(Response::new(matcher::MatchResponse { matches }))
    }
}

pub async fn start_grpc_server(config: Arc<Config>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50030".parse()?;
    info!("Initializing VectorDB");

    // Initialize VectorDB
    let db = match VectorDB::new("data/mydb", Some((*config).clone()), false).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to initialize VectorDB: {}", e);
            return Err(e.into());
        }
    };

    let matcher_service = MatcherService { config, db };

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
