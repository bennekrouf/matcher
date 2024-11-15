use super::matcher;
use super::MatcherService;
use tonic::{Request, Response, Status};
use tracing::info;

use super::matcher::matcher_server::Matcher;
use super::matcher::{MatchRequest, MatchResponse};

#[tonic::async_trait]
impl Matcher for MatcherService {
    async fn match_query(
        &self,
        request: Request<MatchRequest>,
    ) -> Result<Response<MatchResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Received match request - query: {}, language: {}, show_all_matches: {}",
            req.query, req.language, req.show_all_matches
        );

        let results = self
            .engine
            .find_matches(
                &req.query,
                &req.language,
                if req.show_all_matches { 5 } else { 1 },
            )
            .await
            .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;

        let matches = results
            .iter()
            .map(Self::convert_match_to_response)
            .collect();

        Ok(Response::new(matcher::MatchResponse { matches }))
    }
}
