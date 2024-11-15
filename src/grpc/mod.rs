pub mod service;
pub mod start_grpc_server;

use crate::matcher_service::MatcherEngine;
use crate::vector_db::search_result::MatchResult;
use std::sync::Arc;

pub mod matcher {
    tonic::include_proto!("matcher");
}

pub struct MatcherService {
    engine: Arc<MatcherEngine>,
}

impl MatcherService {
    pub fn new(engine: Arc<MatcherEngine>) -> Self {
        Self { engine }
    }

    fn convert_match_to_response(match_result: &MatchResult) -> matcher::EndpointMatch {
        match match_result {
            MatchResult::Complete(result) => matcher::EndpointMatch {
                endpoint_id: result.endpoint_id.clone(),
                similarity: result.similarity as f64,
                parameters: result.parameters.clone(),
                is_partial_match: false,
                missing_parameters: Vec::new(),
                pattern: result.pattern.clone(),
                text: result.text.clone(),
                description: result.description.clone(),
            },
            MatchResult::Partial {
                result,
                missing_params,
            } => matcher::EndpointMatch {
                endpoint_id: result.endpoint_id.clone(),
                similarity: result.similarity as f64,
                parameters: result.parameters.clone(),
                is_partial_match: true,
                missing_parameters: missing_params.clone(),
                pattern: result.pattern.clone(),
                text: result.text.clone(),
                description: result.description.clone(),
            },
        }
    }
}
