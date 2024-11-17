use crate::config::Config;
use crate::database::vector_db::VectorDB;

use crate::preprocessing::preprocess_query::preprocess_query;
use matcher::EndpointMatch;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

pub mod matcher {
    tonic::include_proto!("matcher");
}

pub struct MatcherService {
    #[allow(dead_code)]
    pub config: Arc<Config>,
    pub db: Arc<VectorDB>,
}

#[tonic::async_trait]
impl matcher::matcher_server::Matcher for MatcherService {
    async fn match_query(
        &self,
        request: Request<matcher::MatchRequest>,
    ) -> Result<Response<matcher::MatchResponse>, Status> {
        let req = request.into_inner();
        let processed = preprocess_query(&req.query, &req.language);

        info!(
            "Received match request - query: {}, language: {}, show_all_matches: {}",
            req.query, req.language, req.show_all_matches
        );

        let (results, best_similarity) = match self
            .db
            .search_similar(
                &processed.cleaned_text,
                &req.language,
                if req.show_all_matches { 5 } else { 1 },
                &self.config,
            )
            .await
        {
            Ok((results, similarity)) => {
                if results.is_empty() {
                    warn!("No matches found for query: {}", req.query);
                }
                (results, similarity)
            }
            Err(e) => {
                error!("Search failed: {}", e);
                return Err(Status::internal(format!("Search failed: {}", e)));
            }
        };

        let matches: Vec<matcher::EndpointMatch> = results
            .iter()
            .map(|result| {
                // Find corresponding endpoint configuration
                let endpoint = self
                    .config
                    .endpoints
                    .iter()
                    .find(|e| e.id == result.endpoint_id)
                    .expect("Endpoint not found in config");

                // Analyze parameters
                let param_analysis = endpoint.analyze_parameters(&result.parameters);

                EndpointMatch {
                    endpoint_id: result.endpoint_id.clone(),
                    similarity: (1.0 - result.similarity) as f64,
                    parameters: result.parameters.clone(),
                    is_negated: processed.is_negated,
                    missing_required: param_analysis
                        .missing_required
                        .iter()
                        .map(|p| matcher::ParameterInfo {
                            name: p.name.clone(),
                            description: p.description.clone(),
                            required: true,
                        })
                        .collect(),
                    missing_optional: param_analysis
                        .missing_optional
                        .iter()
                        .map(|p| matcher::ParameterInfo {
                            name: p.name.clone(),
                            description: p.description.clone(),
                            required: false,
                        })
                        .collect(),
                }
            })
            .collect();

        info!(
            "Returning response with {} matches, best similarity: {}",
            matches.len(),
            best_similarity
        );
        let has_matches = !matches.is_empty();
        let score = best_similarity as f64;
        Ok(Response::new(matcher::MatchResponse {
            matches,
            score,
            has_matches,
        }))
    }
}
