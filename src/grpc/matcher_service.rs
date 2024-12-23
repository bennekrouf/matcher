use crate::config::Config;
use crate::database::vector_db::VectorDB;
use crate::interaction::handlers::{
    handle_confirmation, handle_initial_query, handle_parameter_value,
};
use crate::interaction::state::InteractionState;
use crate::preprocessing::preprocess_query::preprocess_query;
use futures::StreamExt;
use matcher::MatchResponse;
use matcher::{
    interactive_request::Request as InteractiveRequestType, EndpointMatch, InteractiveRequest,
    InteractiveResponse,
};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, warn};
pub mod matcher {
    tonic::include_proto!("matcher");
}

use crate::grpc::matcher_service::matcher::interactive_response::Response as InteractiveResponseType;
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
                EndpointMatch {
                    endpoint_id: result.endpoint_id.clone(),
                    similarity: result.similarity as f64, // Remove the 1.0 - conversion
                    parameters: result.parameters.clone(),
                    is_negated: processed.is_negated,
                    missing_required: result
                        .parameter_analysis
                        .missing_required
                        .iter()
                        .map(|p| matcher::ParameterInfo {
                            name: p.name.clone(),
                            description: p.description.clone(),
                            required: true,
                        })
                        .collect(),
                    missing_optional: result
                        .parameter_analysis
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

        let score = best_similarity as f64; // This should match the similarities now
        let has_matches = !matches.is_empty();

        Ok(Response::new(matcher::MatchResponse {
            matches,
            score,
            has_matches,
        }))
    }

    type InteractiveMatchStream =
        Pin<Box<dyn Stream<Item = Result<InteractiveResponse, Status>> + Send + 'static>>;

    async fn interactive_match(
        &self,
        request: Request<Streaming<InteractiveRequest>>,
    ) -> Result<Response<Self::InteractiveMatchStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let db = self.db.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut state: Option<InteractionState> = None;

            while let Some(req) = in_stream.next().await {
                match req {
                    Ok(interactive_req) => match interactive_req.request {
                        Some(InteractiveRequestType::InitialQuery(initial_query)) => {
                            state = handle_initial_query(
                                &initial_query.query,
                                &initial_query.language,
                                &db,
                                &config,
                                &tx,
                            )
                            .await;
                            println!("State after initial query: {:?}", state); // Debug log
                        }
                        Some(InteractiveRequestType::ConfirmationResponse(confirmation)) => {
                            if let Some(current_state) = state {
                                state =
                                    handle_confirmation(confirmation.confirmed, current_state, &tx)
                                        .await;
                                println!("State after confirmation: {:?}", state);
                                // Debug log
                            }
                        }
                        Some(InteractiveRequestType::ParameterValue(param_value)) => {
                            if let Some(current_state) = state.take() {
                                state =
                                    handle_parameter_value(param_value, current_state, &tx).await;

                                // Handle completed state
                                if let Some(InteractionState::Completed { endpoint_match }) = &state
                                {
                                    // Now we're actually using the endpoint_match field
                                    println!(
                                        "✅ Executing completed action for endpoint: {}",
                                        endpoint_match.endpoint_id
                                    );

                                    match execute_completed_action(endpoint_match).await {
                                        Ok(_) => {
                                            let response = MatchResponse {
                                                matches: vec![endpoint_match.clone()],
                                                score: 1.0,
                                                has_matches: true,
                                            };

                                            if let Err(e) = tx
                                                .send(Ok(InteractiveResponse {
                                                    response: Some(
                                                        InteractiveResponseType::MatchResult(
                                                            response,
                                                        ),
                                                    ),
                                                }))
                                                .await
                                            {
                                                error!("Failed to send completion response: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to execute completed action: {}", e);
                                            if let Err(send_err) = tx
                                                .send(Err(Status::internal(format!(
                                                    "Action execution failed: {}",
                                                    e
                                                ))))
                                                .await
                                            {
                                                error!(
                                                    "Failed to send error response: {}",
                                                    send_err
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            error!("Received empty request");
                        }
                    },
                    Err(e) => {
                        error!("Error receiving request: {}", e);
                        let _ = tx.send(Err(Status::internal("Stream error"))).await;
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

//use crate::grpc::matcher_service::matcher::EndpointMatch;
use crate::messaging::get_authenticated_iggy_client::get_authenticated_iggy_client;
use crate::messaging::send_structured_message::send_structured_message;
use anyhow::{anyhow, Result as AnyhowResult};
//use tracing::{error, info};

async fn execute_completed_action(endpoint_match: &EndpointMatch) -> AnyhowResult<()> {
    info!(
        "Processing completed match for endpoint: {}",
        endpoint_match.endpoint_id
    );

    let client = get_authenticated_iggy_client().await?;

    // Convert parameters map to vector of values
    let message_params: Vec<String> = endpoint_match.parameters.values().cloned().collect();

    // Send message to Iggy
    send_structured_message(
        &client,
        "gibro",        // stream name
        "notification", // topic name
        &endpoint_match.endpoint_id,
        message_params,
    )
    .await
    .map_err(|e| {
        error!(
            "Failed to send message for endpoint {}: {}",
            endpoint_match.endpoint_id, e
        );
        anyhow!("Message sending failed: {}", e)
    })?;

    info!(
        "Sent notification for endpoint: {} with similarity: {}",
        endpoint_match.endpoint_id, endpoint_match.similarity
    );

    info!("Completed processing match");
    Ok(())
}
